mod cli;
mod context;
mod device;
mod io;
mod logging;
mod process;

// THIRD PARTY CRATES
use clap::Parser;
use shared::{DisplayInfo, External};
use tokio::sync::{broadcast, mpsc::channel};
use tracing::{debug, info, trace};

// VENDOR CRATES
// LOCAL CRATES
use crate::context::ScanMode;
use crate::io::{IOManager, Internal};
use crate::process::{ProcessContext, ProcessType, process_peaks};
use crate::{cli::Cli, context::Context, device::DevMsg};

#[tokio::main]
#[tracing::instrument]
async fn main() {
    // Parse command line arguments
    let args = Cli::parse();

    // Initialize logging first
    if let Err(e) = logging::init_logging(&args) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Starting SDR Scanner");
    info!("Configuration: rate={}, fft_size={}, scan_mode={}", 
          args.rate, args.fft_size, args.scan_mode);
    info!("Frequency ranges: {:?}", args.ranges);

    //// Inter-Thread Communication Channels
    // Messages for the device to action.
    let (dev_tx, dev_rx) = channel::<DevMsg>(8);

    // Messages from the device for processing.
    let (process_tx, mut process_rx) = channel(512);

    // Messages from other threads that need to be actioned in the main loop.
    let (internal_tx, mut internal_rx) = channel::<Internal>(512);

    // Create broadcast channel for external messages (non-realtime)
    let (external_tx, _external_rx) = broadcast::channel(1024);

    // Create watch channel for realtime freq_block data
    let (realtime_tx, realtime_rx) = tokio::sync::watch::channel(shared::FreqBlock::new());

    // Initialize variables into the top-level context object.
    let mut ctx = Context::new(&args, dev_tx.clone()).unwrap();
    info!("Context initialized successfully");

    // Create IO manager
    let io_manager = IOManager::new();
    let client_count = io_manager.client_count();

    //// START DEDICATED THREADS
    // Dedicated OS thread to handle the SDR device.
    info!("Starting device thread");
    device::start(
        &args,
        ctx.scan.current(),
        dev_rx,
        internal_tx.clone(),
        process_tx,
        realtime_tx,
        client_count,
    );

    // Start IO manager
    let initial_display_info = External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate(), ctx.process.squelch));
    info!("Starting IO manager...");
    io_manager.start(external_tx.clone(), realtime_rx, internal_tx.clone(), initial_display_info).await;
    info!("IO manager started");

    // Main Loop with proper yielding
    info!("Entering main processing loop");
    loop {
        tokio::select! {
            // Handle messages from clients (fast, non-blocking)
            Some(msg) = internal_rx.recv() => {
                match msg {
                    Internal::DeviceFreqUpdated => {
                        debug!("Device frequency updated to {}", ctx.scan.current());
                        // Clear pending process messages
                        while process_rx.try_recv().is_ok() {}

                        let display_info = External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate(), ctx.process.squelch));
                        let _ = external_tx.send(display_info);
                        ctx.process.start()
                    },
                    Internal::BlockMetadata((block_metadata, peaks)) => {
                        trace!("Received block metadata with {} entries", block_metadata.len());
                        ctx.current.remove_processed_peaks(peaks);
                        ctx.storage.update_metadata(block_metadata);
                        let metadata_msg = External::Metadata(ctx.storage.metadata.clone());
                        let _ = external_tx.send(metadata_msg);
                    },
                    Internal::Squelch(squelch) => {
                        ctx.process.squelch = squelch;
                        let display_info = External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate(), ctx.process.squelch));
                        let _ = external_tx.send(display_info);
                    }
                }
            },

            // Handle IQ & Freq blocks with proper yielding
            Some((iq_block, freq_block)) = process_rx.recv(), if ctx.process.is_processing() => {
                // freq_block.squelch(ctx.process.squelch);

                // Process data (lightweight operations)
                let peak_detection_criteria_met = ctx.current.update(iq_block, freq_block, &ctx.process);

                // YIELD POINT: Let client tasks run
                tokio::task::yield_now().await;

                if peak_detection_criteria_met {
                    // Do peak detection (CPU-intensive but necessary)
                    ctx.current.detect_peaks(&ctx.process, ctx.scan.current());

                    // YIELD POINT: Let client tasks run after peak detection
                    tokio::task::yield_now().await;

                    // Send peaks and continue processing
                    let peaks_msg = External::Peaks(ctx.current.peaks.clone());
                    let _ = external_tx.send(peaks_msg);

                    if ctx.current.peaks.is_empty() ||
                       (ctx.scan.mode == ScanMode::SweepThenProcess &&
                        (ctx.scan.cycles() < ctx.process.scan_cycles_required)) {
                        ctx.next();
                        continue;
                    }
                }

                // Handle metadata processing (already async)
                // TODO: Sliding window for peaks on context
                if ctx.current.collected_iq.len() > ctx.process.num_required_for_metadata {
                    let current = ctx.scan.current();
                    let rate = ctx.scan.rate();
                    let iq_blocks = std::mem::take(&mut ctx.current.collected_iq);
                    let internal_tx = internal_tx.clone();
                    ctx.current.reduce_peaks(&ctx.process);
                    // println!("REDUCE {:?}", ctx.current.peaks);
                    // let peaks_to_process = ctx.current.peaks_to_process();
                    // println!("TO PROCESS {:?}", peaks_to_process);

                    let peaks_to_process = std::mem::take(&mut ctx.current.peaks);

                    tokio::task::spawn(async move {
                        let process_ctx = ProcessContext::new(
                            current,
                            rate,
                            ProcessType::PreProcess
                        );

                        // Save off the peaks to report which peaks have been processed.
                        let processed_peaks = peaks_to_process.clone();

                        // Move CPU-intensive process_peaks to blocking thread
                        let metadata = tokio::task::spawn_blocking(move || {
                            process_peaks(process_ctx, iq_blocks, peaks_to_process)
                        }).await.unwrap();

                        // Send the metadata result back to main loop
                        let _ = internal_tx.send(Internal::BlockMetadata((metadata, processed_peaks))).await;
                    });
                    ctx.next();
                }
            },
        }
    }
}

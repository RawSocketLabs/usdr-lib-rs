mod cli;
mod context;
mod device;
mod io;
mod process;

// THIRD PARTY CRATES
use clap::Parser;
use shared::{DisplayInfo, External};
use tokio::sync::{broadcast, mpsc::channel};

// VENDOR CRATES
// LOCAL CRATES
use crate::context::ScanMode;
use crate::io::{Internal, IOManager};
use crate::process::{ProcessContext, ProcessType, process_peaks};
use crate::{cli::Cli, context::Context, device::DevMsg};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Cli::parse();

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

    // Create IO manager
    let io_manager = IOManager::new();
    let client_count = io_manager.client_count();

    //// START DEDICATED THREADS
    // Dedicated OS thread to handle the SDR device.
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
    let initial_display_info = External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate()));
    eprintln!("Starting IO manager...");
    io_manager.start(external_tx.clone(), realtime_rx, internal_tx.clone(), initial_display_info).await;
    eprintln!("IO manager started");

    // Main Loop with proper yielding
    loop {
        tokio::select! {
            // Handle messages from clients (fast, non-blocking)
            Some(msg) = internal_rx.recv() => {
                match msg {
                    Internal::DeviceFreqUpdated => {
                        // Clear pending process messages
                        while process_rx.try_recv().is_ok() {}
                        
                        let display_info = External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate()));
                        let _ = external_tx.send(display_info);
                        ctx.process.start()
                    },
                    Internal::BlockMetadata(block_metadata) => {
                        ctx.storage.update_metadata(block_metadata);
                        let metadata_msg = External::Metadata(ctx.storage.metadata.clone());
                        let _ = external_tx.send(metadata_msg);
                    }
                }
            },

            // Handle IQ & Freq blocks with proper yielding
            Some((iq_block, freq_block)) = process_rx.recv(), if ctx.process.is_processing() => {
                // Process data (lightweight operations)
                ctx.current.update(iq_block, freq_block);
                
                // YIELD POINT: Let client tasks run
                tokio::task::yield_now().await;
                
                if ctx.current.peak_detection_criteria_met(&ctx.process) {
                    // Do peak detection (CPU-intensive but necessary)
                    ctx.current.detect_peaks(&ctx.process);
                    
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
                if ctx.current.collected_iq.len() > ctx.process.num_required_for_metadata {
                    let peaks = std::mem::take(&mut ctx.current.peaks);
                    let iq_blocks = std::mem::take(&mut ctx.current.collected_iq);

                    let process_ctx = ProcessContext::new(
                        ctx.scan.current() as u32, 
                        ctx.scan.rate(), 
                        ProcessType::PreProcess
                    );
                    let _metadata_tx = internal_tx.clone();
                    
                    // Move CPU-intensive process_peaks to blocking thread
                    let metadata = tokio::task::spawn_blocking(move || {
                        process_peaks(process_ctx, iq_blocks, peaks)
                    }).await.unwrap();
                    
                    // Send the metadata result back to main loop
                    let _ = internal_tx.send(Internal::BlockMetadata(metadata)).await;

                    ctx.next();
                }
            },
        }
    }
}

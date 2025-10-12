mod cli;
mod context;
mod device;
mod io;
mod process;

// THIRD PARTY CRATES
use clap::Parser;
use shared::{DisplayInfo, DmrMetadata, External};
use tokio::sync::{broadcast, mpsc::channel};

// VENDOR CRATES
// LOCAL CRATES
use crate::context::ScanMode;
use crate::io::{Client, Internal};
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

    // Clients connected messages
    let (client_tx, mut client_rx) = channel::<Client>(4);

    // Structured messages from the program destin to external applications.
    let (out_tx, mut out_rx) = broadcast::channel::<External>(512);

    // Dedicated channel to send metadata
    let (metadata_tx, mut metadata_rx) = channel::<Vec<DmrMetadata>>(32);
    ////

    // Initialize variables into the top-level context object.
    let mut ctx = Context::new(&args, dev_tx.clone()).unwrap();

    //// START DEDICATED THREADS
    // Dedicated OS thread to handle the SDR device.
    device::start(
        &args,
        ctx.scan.current(),
        dev_rx,
        internal_tx.clone(),
        process_tx,
    );

    // Dedicated OS thread to handle new external applications (in/out)
    io::start(client_tx, internal_tx.clone()).await;
    ////

    //let main_loop_internal_tx_clone = internal_tx.clone();
    // Main Loop
    loop {
        tokio::select! {
            biased;

            // TODO: Make the client sends actually queue the messages to be sent
            // On a tick rate send all messages to the clients might be a way to drain messages back to dedicated thread for sending.
            // Have a realtime_send potentially if needed for visualizing.
            // TODO: Simplify the channels being utilized -- in_tx probably just becomes what io_tx perhaps better called client_tx and client_rx
            // TODO: in_rx is probably just a dev_rx channel as it is currently utilized.
            // TODO: Need to come up with a tick rate to accept new input messages for all clients and handle their message (ctx.tick()? just a timeout on a normal message... tokio has a solution to this.)

            // TICK RAGE MESSAGES

            // Handle Internal Messages.
            Some(msg) = internal_rx.recv() => {
                match msg {
                    Internal::DeviceFreqUpdated => {
                        // Dump the pending messages from the IQ & Freq channel.
                        while process_rx.try_recv().is_ok() {}
                        ctx.clients.send(&External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate())));
                        ctx.process.start()
                    },
                    Internal::BlockMetadata(block_metadata) => {
                        ctx.storage.update_metadata(block_metadata);
                        ctx.clients.send(&External::Metadata(ctx.storage.metadata.clone()))
                    }
                    _ => unimplemented!(),
                }
            },

            // Handle Client Messages
            Some(client) = client_rx.recv() => {
                ctx.clients.push(client)
            }

            // Handle IQ & Freq blocks being sent from the SDR.
            Some((iq_block, freq_block)) = process_rx.recv(), if ctx.process.is_processing() => {
                // Update any connected clients with the new IQ & Freq blocks.
                ctx.clients.send(&External::Realtime(freq_block.clone()));
                ctx.clients.send(&External::Display(DisplayInfo::new(ctx.scan.current(), ctx.scan.rate())));

                // Add the IQ block and update the average Freq block for the current context.
                ctx.current.update(iq_block, freq_block);

                // If no peaks have been detected and enough blocks have been collected to detect
                //  peaks, determine if there are peaks within the spectrum.
                if ctx.current.peak_detection_criteria_met(&ctx.process) {
                    ctx.current.detect_peaks(&ctx.process);

                    // TODO: Tidy this up..
                    // If no peaks were detected, then we clean up the current context and move on
                    ctx.clients.send(&External::Peaks(ctx.current.peaks.clone()));
                    if ctx.current.peaks.is_empty() || (ctx.scan.mode == ScanMode::SweepThenProcess && (ctx.scan.cycles() < ctx.process.scan_cycles_required)) {
                        ctx.next();
                        continue;
                    }
                }

                // TODO: Probably a top level wrapper here..
                // If peaks have been detected and enough IQ blocks have been stored for the metadata
                //  extraction, this section of code will be run.
                if ctx.current.collected_iq.len() > ctx.process.num_required_for_metadata {
                    // Take the peaks and IQ blocks from the current context
                    let peaks = std::mem::take(&mut ctx.current.peaks);
                    let iq_blocks = std::mem::take(&mut ctx.current.collected_iq);

                    // Spawn a green thread to run as a background task when the scheduler has
                    // time. It handles reporting sending its output to the appropriate channel
                    // directly.
                    // TODO: This should be cleaner as well.

                    let process_ctx = ProcessContext::new(ctx.scan.current() as u32, ctx.scan.rate(), ProcessType::PreProcess);
                    let metadata_tx = internal_tx.clone();
                    tokio::task::spawn(async move {
                        let metadata = process_peaks(process_ctx, iq_blocks, peaks);
                        metadata_tx.send(Internal::BlockMetadata(metadata)).await.unwrap();
                    });

                    // Clean up the current context and move on
                    ctx.next();
                }
            },
        }
    }
}

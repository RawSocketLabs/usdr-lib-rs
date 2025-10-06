mod cli;
mod context;
mod device;
mod io;
mod process;

// THIRD PARTY CRATES
use clap::Parser;
use shared::{DisplayInfo, DmrMetadata};
use tokio::sync::{broadcast, mpsc::channel, watch};

// VENDOR CRATES
use sdr::FreqBlock;
// LOCAL CRATES
use crate::context::ScanMode;
use crate::io::{Input, Output};
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

    // Messages from external applications that need to be actioned in the main loop.
    let (in_tx, mut in_rx) = channel::<Input>(512);

    // Structured messages from the program destin to external applications.
    let (out_tx, out_rx) = broadcast::channel::<Output>(512);

    // Dedicated channel to send metadata
    let (metadata_tx, mut metadata_rx) = channel::<Vec<DmrMetadata>>(32);

    // Realtime messages from internal threads to external applications.
    let (realtime_tx, realtime_rx) = watch::channel(FreqBlock::new());
    ////

    // Initialize variables into the top-level context object.
    let mut ctx = Context::new(&args, dev_tx.clone()).unwrap();

    //// START DEDICATED THREADS
    // Dedicated OS thread to handle the SDR device.
    device::start(
        &args,
        ctx.scan.current(),
        dev_rx,
        in_tx.clone(),
        process_tx,
        realtime_tx,
    );

    // Dedicated OS thread to handle external applications (in/out)
    io::start(in_tx, out_tx.clone(), realtime_rx).await;
    ////

    // Main Loop
    loop {
        tokio::select! {
            biased;

            // Handle Input Messages.
            Some(input) = in_rx.recv() => {
                match input {
                    Input::DeviceFreqUpdated => {
                        while process_rx.try_recv().is_ok() {}
                            // TODO: Figure out what types should be returned from each thing...
                        out_tx.send(Output::Display(DisplayInfo {center_freq: ctx.scan.current(), rate: ctx.scan.rate() as usize})).unwrap();
                        ctx.process.start()
                    },
                    Input::ClientAtLeastOneConnected => {
                        dev_tx.clone().send(DevMsg::ClientsConnected(true)).await.unwrap();
                    }
                    _ => unimplemented!(),
                }
            },

            Some(metadata) = metadata_rx.recv() => {
                ctx.storage.update_metadata(metadata);
                out_tx.send(Output::Metadata(ctx.storage.metadata.clone())).unwrap();
            },

            // Handle IQ & Freq blocks being sent from the SDR.
            Some((iq_block, freq_block)) = process_rx.recv(), if ctx.process.is_processing() => {
                // Add the IQ block and update the average Freq block for the current context.
                ctx.current.update(iq_block, freq_block);

                // If no peaks have been detected and enough blocks have been collected to detect
                //  peaks, determine if there are peaks within the spectrum.
                if ctx.current.peak_detection_criteria_met(&ctx.process) {
                    ctx.current.detect_peaks(&ctx.process);

                    // TODO: Tidy this up..
                    // If no peaks were detected, then we clean up the current context and move on
                    out_tx.send(Output::Peaks(ctx.current.peaks.clone())).unwrap();
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
                    let process_ctx = ProcessContext::new(ctx.scan.current() as f32,ctx.scan.rate() as f32,ProcessType::PreProcess, out_tx.clone());
                    let metadata_out_tx = metadata_tx.clone();
                    tokio::task::spawn(async move {
                        let metadata = process_peaks(process_ctx, iq_blocks, peaks);
                        metadata_out_tx.send(metadata).await.unwrap();
                    });

                    // Clean up the current context and move on
                    ctx.next();
                }
            },
        }
    }
}

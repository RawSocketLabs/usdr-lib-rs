mod cli;
mod context;
mod device;
mod io;
mod process;

// THIRD PARTY CRATES
use clap::Parser;
use tokio::sync::{broadcast, mpsc::channel, watch};

// VENDOR CRATES
use sdr::FreqBlock;

// LOCAL CRATES
use crate::context::ScanMode;
use crate::io::{Input, Output};
use crate::process::{process_peaks, ProcessContext, ProcessType};
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
    let (out_tx, _) = broadcast::channel::<Output>(512);

    // Realtime messages from internal threads to external applications.
    let (realtime_tx, realtime_rx) = watch::channel(FreqBlock::new());
    ////

    // Initialize variables into the top-level context object.
    let mut ctx = Context::new(&args, dev_tx.clone()).unwrap();

    //// START DEDICATED THREADS
    // Dedicated OS thread to handle the SDR device.
    device::start(
        &args,
        ctx.scan_manager.current(),
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
                    while let Ok(_) = process_rx.try_recv() {}
                    ctx.process_blocks = true
                },
                    Input::ClientAtLeastOneConnected => {
                        dev_tx.clone().send(DevMsg::ClientsConnected(true)).await.unwrap();
                    }
                    _ => unimplemented!(),
                }
            },

            // Handle IQ & Freq blocks being sent from the SDR.
            Some((iq_block, freq_block)) = process_rx.recv(), if ctx.process_blocks => {
                // Add the IQ block and update the average Freq block for the current context.
                ctx.collected_iq.push(iq_block);
                ctx.update_average(freq_block);

                // If no peaks have been detected and enough blocks have been collected to detect
                // peaks determine if there are peaks within the spectrum.
                if ctx.peaks.is_empty() && ctx.collected_iq.len() >= ctx.blocks_required_for_average {
                    ctx.detect_peaks();

                    // If no peaks were detected, then we clean up the current context and move on
                    match (ctx.peaks.is_empty(), &ctx.mode, ctx.completed_required_scan_cycles()) {
                        (true, _, _) => {ctx.next(); continue;},
                        (false, mode, completed_required_scan_cycles) => {
                            out_tx.send(Output::Peaks(ctx.peaks.clone())).unwrap();
                            if mode == &ScanMode::SweepThenProcess && !completed_required_scan_cycles {
                                ctx.next();
                                continue;
                            }
                        },
                    }
                }

                // If peaks have been detected and enough IQ blocks have been stored for the metadata
                //  extraction, this section of code will be run.
                if ctx.collected_iq.len() > ctx.blocks_required_for_metadata {
                    // Take the peaks and IQ blocks from the current context
                    let peaks = std::mem::take(&mut ctx.peaks);
                    let iq_blocks = std::mem::take(&mut ctx.collected_iq);

                    // Spawn a green thread to run as a background task when the scheduler has
                    // time. It handles reporting sending its output to the appropriate channel
                    // directly.
                    let process_ctx = ProcessContext::new(0.0,0.0,ProcessType::PreProcess, out_tx.clone());
                    tokio::task::spawn(async move {process_peaks(process_ctx, iq_blocks, &peaks);});

                    // Clean up the current context and move on
                    ctx.next();
                }
            },
        }
    }
}

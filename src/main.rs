mod cli;
mod context;
mod ctrl;
mod device;
mod display;
mod process;
mod report;
mod scan;
mod tui;

// THIRD PARTY CRATES
use clap::Parser;
use tokio::sync::{mpsc::channel, watch};

// VENDOR CRATES
use sdr::FreqBlock;

// LOCAL CRATES
use crate::{cli::Cli, context::Context, device::DevMsg, init::init};

// TODO: MOVE THIS
pub enum CtrlMsg {
    DevFreqUpdated,
    ConnectToStream(usize),
}

pub enum OutMsg {
    Metadata,
    Peaks(Vec<Peak>),
    FreqBlock(FreqBlock),
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Cli::parse();

    //// Inter-Thread Communication Channels
    // Messages for the device to action.
    let (dev_tx, dev_rx) = channel::<DevMsg>(8);

    // Messages from the device to further process.
    let (process_tx, mut process_rx) = channel(512);

    // Messages from external applications that need to be actioned in the main loop.
    let (ctrl_tx, mut ctrl_rx) = channel::<CtrlMsg>(128);

    // Structred messages from the program destin to external applications.
    let (out_tx, out_rx) = channel::<OutMsg>(512);

    // Realtime messages from internal threads to external applications.
    let (realtime_tx, realtime_rx) = watch::channel(OutMsg::FreqBlock(FreqBlock::new()));
    ////

    // Initialize variables into the top level context object
    let mut ctx = Context::new(&args, dev_tx.clone()).unwrap();

    //// START DEDICATED THREADS
    // Dedicated OS thread to handle SDR device
    device::start(
        args.file,
        args.raw,
        args.rate,
        ctx.scan_manager.current(),
        args.fft_size,
    );

    // Dedicated OS thread to handle external applications (in/out)
    //ctrl::start();
    ////

    // Main Loop
    loop {
        tokio::select! {
            biased;

            // Handle External Ctrl Messages.
            Some(ctrl_msg) = ctrl_rx.recv() => {
                match ctrl_msg {
                    CtrlMsg::DevFreqUpdated => {
                    while let Ok(_) = process_rx.try_recv() {}
                    ctx.process_blocks = true
                },
                    _ => unimplemented!(),
                }
            },

            // Handle IQ & Freq blocks being sent from the SDR.
            Some((iq_block, freq_block)) = process_rx.recv(), if ctx.process_blocks => {
                // Add the IQ block and update the average Freq blok for the current context.
                ctx.collected_iq.push(iq_block);
                ctx.update_average(freq_block);

                // If no peaks have been detected and enough blocks have been collected to detect
                // peaks determine if there are peaks within the spectrum.
                if ctx.peaks.is_empty() && ctx.collected_iq.len() >= ctx.blocks_required_for_average {
                    ctx.detect_peaks();

                    // If no peaks were detected cleanup the current context and move on
                    match ctx.peaks.is_empty() {
                        true => {ctx.next(); continue;},
                        false => ctrl_tx.blocking_send(CtrlMsg::Peaks(ctx.peaks.clone())).unwrap(),
                    }
                }

                if ctx.mode == ScanMode::SweepThenProcess && !ctx.completed_required_scan_cycles() {
                    ctx.next();
                    continue;
                }

                // If peaks have been detected and enough IQ blocks have been stored for metadata
                // processing run this section of code
                if ctx.collected_iq.len() > ctx.blocks_required_for_metadata {
                    // Take the peaks and IQ blocks from the current context
                    let peaks = std::mem::take(&mut ctx.peaks);
                    let iq_blocks = std::mem::take(&mut ctx.collected_iq);

                    // Spawn a green thread to run as a background task when the scheduler has
                    // time. It handles reporting sending its output to the appropriate channel
                    // directly.
                    tokio::task::spawn(async {});

                    // Cleanup the current context and move on
                    ctx.next();
                }
            },
        }
    }
}

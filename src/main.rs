mod cli;
mod display;
mod init;
mod report;
mod sample;
mod scan;
mod tui;

// THIRD PARTY CRATES
use clap::Parser;

// LOCAL CRATES
use crate::{cli::Cli, init::init};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Cli::parse();

    // Initialize channels and parameters for sampling and scanning
    let (sample, scan, display) = init(&args);

    // spawn a dedicated thread for sampling from the sdr
    sample::sample(args.file, args.raw, args.rate, sample.channels, sample.params);

    // Spawn a task for scanning the spectrum for signals
    scan::scan(scan.channels, scan.params);

    // report the scan results
    display::display(args.tui, display).await;
}

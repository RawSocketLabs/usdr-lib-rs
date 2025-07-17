mod cli;
mod init;
mod report;
mod sample;
mod scan;

// THIRD PARTY CRATES
use clap::Parser;

// LOCAL CRATES
use crate::{cli::Cli, init::init, report::report};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Cli::parse();

    // Initialize channels and parameters for sampling and scanning
    let (sample, scan, scan_rx) = init(&args);

    // spawn a dedicated thread for sampling from the sdr
    sample::sample(args, sample.channels, sample.params);

    // Spawn a task for scanning the spectrum for signals
    scan::scan(scan.channels, scan.params);

    // report the scan results
    report(scan_rx).await;
}

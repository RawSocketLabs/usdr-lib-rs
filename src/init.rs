// STD LIB
use std::time::Duration;

// THIRD PARTY CRATES
use tokio::sync::{
    mpsc::{channel},
    watch,
};

// REMOTE CRATES
use sdr::FreqBlock;

// LOCAL CRATES
use crate::{
    cli::Cli, display::{DisplayArgs, DisplayChannels, DisplayParams}, process::{ProcessArgs, ProcessChannels, ProcessParams}, sample::{SampleArgs, SampleChannels, SampleParams}, scan::{ScanArgs, ScanChannels, ScanParams, ScanResults}
};

pub fn init(args: &Cli) -> (SampleArgs, ScanArgs, ProcessArgs, DisplayArgs) {
    let lag: usize = (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize; // a LAG rate that equates to twice the target bandwidth

    let (scan_freq_block_tx, scan_freq_block_rx) = channel::<FreqBlock>(500);
    let (display_freq_block_tx, display_freq_block_rx) = watch::channel::<FreqBlock>(FreqBlock::new());
    let (sample_freq_tx, sample_freq_rx) = channel::<u32>(1);
    let (display_freq_tx, display_freq_rx) = channel::<u32>(1);
    let (scan_tx, scan_rx) = channel::<ScanResults>(50);
    let (tui_scan_tx, tui_scan_rx) = channel::<ScanResults>(50);
    let (sample_flow_tx, sample_flow_rx) = channel::<bool>(1);
    let (iq_block_tx, iq_block_rx) = channel::<sdr::IQBlock>(1);


    let sample_channels = SampleChannels {
        freq_block_tx_mpsc: scan_freq_block_tx,
        freq_block_tx_watch: display_freq_block_tx,
        freq_rx: sample_freq_rx,
        flow_rx: sample_flow_rx,
        iq_block_tx
    };

    let sample_params = SampleParams {
        rate: args.rate,
        freq: args.start_freq,
        fft_size: args.fft_size,
    };

    let sample_args = SampleArgs {
        channels: sample_channels,
        params: sample_params,
    };

    let scan_channels = ScanChannels {
        freq_tx: vec![sample_freq_tx, display_freq_tx],
        freq_block_rx: scan_freq_block_rx,
        result_tx: scan_tx,
        tui_result_tx: tui_scan_tx,
        flow_tx: sample_flow_tx,
    };

    let scan_params = ScanParams {
        lag,
        bandwidth: args.bandwidth,
        rate: args.rate,
        range: args.start_freq..args.end_freq,
        sleep_time: Duration::from_millis(args.sleep_ms),
    };

    let scan_args = ScanArgs {
        channels: scan_channels,
        params: scan_params,
    };

    let process_channels = ProcessChannels {
        iq_block_rx,
        peaks_rx: scan_rx,
    };

    let process_args = ProcessArgs {
        channels: process_channels,
        params: ProcessParams {}
    };

    let display_channels = DisplayChannels {
        freq_block_rx: display_freq_block_rx,
        freq_rx: display_freq_rx,
        scan_rx: tui_scan_rx,
    };

    let display_params = DisplayParams {
        rate: args.rate,
        start_freq: args.start_freq,
    };

    let display_args = DisplayArgs {
        channels: display_channels,
        params: display_params,
    };

    (sample_args, scan_args, process_args, display_args)
}

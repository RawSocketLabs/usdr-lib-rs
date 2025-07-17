// STD LIB
use std::time::Duration;

// THIRD PARTY CRATES
use tokio::sync::mpsc::{Receiver, channel};

// REMOTE CRATES
use sdr::Spectrum;

// LOCAL CRATES
use crate::{
    cli::Cli,
    sample::{SampleChannels, SampleParams},
    scan::{ScanChannels, ScanParams, ScanResults},
};

pub struct SampleArgs {
    pub channels: SampleChannels,
    pub params: SampleParams,
}

pub struct ScanArgs {
    pub channels: ScanChannels,
    pub params: ScanParams,
}

pub fn init(args: &Cli) -> (SampleArgs, ScanArgs, Receiver<ScanResults>) {
    let lag: usize = (12500 / (args.rate / 2 / args.fft_size as u32)) as usize; // a LAG rate that equates to roughly 12.5 kHz blocks

    let (current_spectrum_tx, current_spectrum_rx) = channel::<Spectrum>(500);
    let (freq_tx, freq_rx) = channel::<u32>(1);
    let (scan_tx, scan_rx) = channel::<ScanResults>(50);

    let sample_channels = SampleChannels {
        spectrum_tx: current_spectrum_tx,
        freq_rx,
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
        freq_tx,
        spectrum_rx: current_spectrum_rx,
        result_tx: scan_tx,
    };

    let scan_params = ScanParams {
        lag,
        rate: args.rate,
        range: args.start_freq..args.end_freq,
        sleep_time: Duration::from_millis(args.sleep_ms),
    };

    let scan_args = ScanArgs {
        channels: scan_channels,
        params: scan_params,
    };

    (sample_args, scan_args, scan_rx)
}

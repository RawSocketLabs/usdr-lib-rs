// STD LIB
use std::time::Duration;

// THIRD PARTY CRATES
use tokio::sync::{
    mpsc::{channel},
    watch,
};

// REMOTE CRATES
use sdr::Spectrum;

// LOCAL CRATES
use crate::{
    cli::Cli,
    display::{DisplayArgs, DisplayChannels, DisplayParams},
    sample::{SampleArgs, SampleChannels, SampleParams},
    scan::{ScanArgs, ScanChannels, ScanParams, ScanResults},
};

pub fn init(args: &Cli) -> (SampleArgs, ScanArgs, DisplayArgs) {
    let lag: usize = (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize; // a LAG rate that equates to twice the target bandwidth

    // Create multiple channels for spectrum - send vec of channels to anywhere that needs them
    let (scan_spectrum_tx, scan_spectrum_rx) = channel::<Spectrum>(500);
    let (display_spectrum_tx, display_spectrum_rx) = watch::channel::<Spectrum>(Spectrum::new());
    let (sample_freq_tx, sample_freq_rx) = channel::<u32>(1);
    let (display_freq_tx, display_freq_rx) = channel::<u32>(1);
    let (scan_tx, scan_rx) = channel::<ScanResults>(50);
    let (sample_flow_tx, sample_flow_rx) = channel::<bool>(1);

    let sample_channels = SampleChannels {
        spectrum_tx_mpsc: scan_spectrum_tx,
        spectrum_tx_watch: display_spectrum_tx,
        freq_rx: sample_freq_rx,
        flow_rx: sample_flow_rx,
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
        spectrum_rx: scan_spectrum_rx,
        result_tx: scan_tx,
        flow_tx: sample_flow_tx,
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

    let display_channels = DisplayChannels {
        spectrum_rx: display_spectrum_rx,
        freq_rx: display_freq_rx,
        scan_rx,
    };

    let display_params = DisplayParams {
        rate: args.rate,
        start_freq: args.start_freq,
    };

    let display_args = DisplayArgs {
        channels: display_channels,
        params: display_params,
    };

    (sample_args, scan_args, display_args)
}

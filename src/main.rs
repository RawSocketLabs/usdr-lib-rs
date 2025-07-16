use std::time::Duration;

use clap::Parser;
use rtlsdr::RTLSDRDevice;
use sdr::{
    device::{Device, DeviceType, file::WavFile},
    types::Spectrum,
};
use tokio::sync::mpsc;

use crate::{
    scanner::{ScanResults, scan_frequency_range},
    sdrreader::initiate_sdr_reader,
};

mod scanner;
mod sdrreader;

#[derive(Parser, Debug)]
#[clap(name = "sdrscanner", about = "Scan a frequency range for signal peaks")]
struct Args {
    /// Start frequency in Hz
    #[clap(long, default_value = "400000000")]
    start_freq: u32,
    /// End frequency in Hz
    #[clap(long, default_value = "520000000")]
    end_freq: u32,
    /// Delay between switching frequencies in milliseconds
    #[clap(long, default_value = "0")]
    sleep_ms: u64,
    /// Sample Rate
    #[clap(long, default_value = "2000000")]
    sample_rate: u32,
    /// Number of FFT bins
    #[clap(long, default_value = "4096")]
    fft_size: usize,
    /// File path to IQ recording for playback
    #[clap(long, default_value = "")]
    file: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let lag: usize = (12500 / (args.sample_rate / 2 / args.fft_size as u32)) as usize; // a LAG rate that equates to roughly 12.5 kHz blocks

    let (current_spectrum_tx, current_spectrum_rx) = mpsc::channel::<Spectrum>(500);
    let (freq_tx, freq_rx) = mpsc::channel::<u32>(1);
    let (scan_tx, mut scan_rx) = mpsc::channel::<ScanResults>(50);

    let device = if args.file.is_empty() {
        DeviceType::Rtlsdr(Device::<RTLSDRDevice>::new(args.sample_rate))
    } else {
        DeviceType::WavFile(Device::<WavFile>::new(args.file))
    };

    // spawn a task for sdr reading
    tokio::spawn(async move {
        initiate_sdr_reader(
            current_spectrum_tx,
            freq_rx,
            args.sample_rate,
            args.fft_size,
            args.start_freq,
            device.into(), // How do we send a generic into a function without knowing the type in advance?  Attempted to use the enum approach.
        )
        .await
    });

    tokio::spawn(async move {
        scan_frequency_range(
            freq_tx,
            current_spectrum_rx,
            args.start_freq..args.end_freq,
            scan_tx,
            Duration::from_millis(args.sleep_ms),
            args.sample_rate,
            lag,
        )
        .await
    });

    tokio::select! {
        scan_res = scan_rx.recv() => {
                println!("RES: {:?}", scan_res);
            }
    }
}

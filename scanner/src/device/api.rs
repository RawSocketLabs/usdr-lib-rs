// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

// STD LIB
use sdr::dsp::{Hann, Window};
use sdr::sample::Freq;
use sdr::{Device, FreqBlock, IQBlock, RawFile, WavFile};
use std::thread;
// THIRD PARTY CRATES
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch,
};
use tracing::info;

// VENDOR CRATES
// LOCAL CRATE
use crate::io::Internal;
use crate::{
    cli::Cli,
    device::{DevChannels, DevMsg, SampleContext, traits::Sample},
};

/// Public API for starting a dedicated thread for sampling from a given device or file.
pub fn start(
    args: &Cli,
    freq: Freq,
    dev_rx: Receiver<DevMsg>,
    internal_tx: Sender<Internal>,
    process_tx: Sender<(IQBlock, FreqBlock)>,
    realtime_tx: watch::Sender<FreqBlock>,
    client_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    info!("Starting device thread with frequency: {} Hz", freq);
    let ctx = SampleContext::new(
        args.rate,
        freq,
        args.fft_size,
        Window::Hann(Hann::new(args.fft_size)),
    );
    let channels = DevChannels::new(dev_rx, internal_tx, process_tx, realtime_tx, client_count);
    let (file, rate, raw, throttle) = (
        args.file.clone(),
        args.rate,
        args.raw,
        !args.no_throttle,
    );

    #[cfg(feature = "usdr")]
    let fft_size = args.fft_size;

    thread::spawn(move || match file {
        None => {
            #[cfg(feature = "usdr")]
            {
                Device::new_auto(rate, fft_size as u32)
                    .expect("Failed to open an SDR device")
                    .sample(channels, ctx);
            }

            #[cfg(not(feature = "usdr"))]
            {
                Device::new_auto(rate)
                    .expect("Failed to open an SDR device")
                    .sample(channels, ctx);
            }
        }
        Some(path) => {
            info!(
                "Opening file device: {} (raw: {}, throttle: {})",
                path, raw, throttle
            );
            if raw {
                Device::<RawFile>::new(path, rate, throttle).sample(channels, ctx)
            } else {
                Device::<WavFile>::new(path, throttle).sample(channels, ctx)
            }
        }
    });
}

// STD LIB
use std::thread;
use std::time::Duration;

// THIRD PARTY CRATES
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch::Sender as WatchSender,
};

// VENDOR CRATES
use sdr::{Device, FreqBlock, Hann, IQBlock, Window};
use sdr::file::raw::RawFile;
use sdr::file::wav::WavFile;
use sdr::tuner::rtl::Rtl;
// LOCAL CRATE
use crate::io::Internal;
use crate::{
    cli::Cli,
    device::{DevChannels, DevMsg, SampleContext, traits::Sample},
};

/// Public API for starting a dedicated thread for sampling from a given device or file.
pub fn start(
    args: &Cli,
    freq: usize,
    dev_rx: Receiver<DevMsg>,
    internal_tx: Sender<Internal>,
    process_tx: Sender<(IQBlock, FreqBlock)>,
) {
    let ctx = SampleContext::new(
        args.rate,
        freq,
        args.fft_size,
        Window::Hann(Hann::new(args.fft_size)),
        Duration::from_millis(args.sleep_ms),
    );
    let channels = DevChannels::new(dev_rx, internal_tx, process_tx);
    let (file, rate, raw, throttle) = (args.file.clone(), args.rate, args.raw, !args.no_throttle);

    thread::spawn(move || match file {
        None => Device::<Rtl>::new(rate)
            .expect("Failed to open the RTL-SDR")
            .sample(channels, ctx),
        Some(path) => {
            if raw {
                Device::<RawFile>::new(path, rate, throttle).sample(channels, ctx)
            } else {
                Device::<WavFile>::new(path, throttle).sample(channels, ctx)
            }
        }
    });
}

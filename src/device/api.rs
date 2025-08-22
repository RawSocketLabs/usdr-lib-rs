// STD LIB
use std::thread;
use std::time::Duration;

// THIRD PARTY CRATES
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch::Sender as WatchSender,
};

// VENDOR CRATES
use sdr::device::file::WavFile;
use sdr::device::rtl::Rtl;
use sdr::{Device, device::rawfile::RawFile};
use sdr::{FreqBlock, IQBlock, Window};

// LOCAL CRATE
use crate::io::Input;
use crate::{
    cli::Cli,
    device::{DevChannels, DevMsg, SampleContext, traits::Sample},
};

/// Public API for starting a dedicated thread for sampling from a given device or file.
pub fn start(
    args: &Cli,
    freq: usize,
    dev_rx: Receiver<DevMsg>,
    input_tx: Sender<Input>,
    process_tx: Sender<(IQBlock, FreqBlock)>,
    realtime_tx: WatchSender<FreqBlock>,
) {
    let ctx = SampleContext::new(
        args.rate,
        freq,
        args.fft_size,
        Window::Hann(Vec::with_capacity(0)),
        Duration::from_millis(50),
    );
    let channels = DevChannels::new(dev_rx, input_tx, process_tx, realtime_tx);
    let (file, rate, raw) = (args.file.clone(), args.rate, args.raw);

    thread::spawn(move || match file {
        None => Device::<Rtl>::new(rate)
            .expect("Failed to open the RTL-SDR")
            .sample(channels, ctx),
        Some(path) => {
            if raw {
                Device::<RawFile>::new(path).sample(channels, ctx)
            } else {
                Device::<WavFile>::new(path).sample(channels, ctx)
            }
        }
    });
}

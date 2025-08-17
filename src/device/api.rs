// STD LIB
use std::thread;

// THIRD PARTY CRATES
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch::Sender as WatchSender,
};

// VENDOR CRATES
use sdr::device::file::WavFile;
use sdr::device::rtl::Rtl;
use sdr::{Device, device::rawfile::RawFile};
use sdr::{FreqBlock, IQBlock};

// LOCAL CRATE
use crate::{
    OutMsg,
    device::{DevMsg, SampleArgs, traits::Sample},
};

/// Public API for starting a dedicated thread for sampling from a given device or file.
pub fn start(
    file: Option<String>,
    raw: bool,
    rate: u32,
    center_freq: usize,
    fft_size: usize,
    out_tx: WatchSender<OutMsg>,
    dev_rx: Receiver<DevMsg>,
    process_tx: Sender<(IQBlock, FreqBlock)>,
) {
    let sample_args = SampleArgs {
        rate,
        center_freq,
        fft_size,
        out_watch,
        out_tx,
        dev_rx,
        process_tx,
    };

    thread::spawn(move || match file {
        None => Device::<Rtl>::new(rate)
            .expect("Failed to open the RTL-SDR")
            .sample(sample_args),
        Some(path) => {
            if raw {
                Device::<RawFile>::new(path).sample(sample_args)
            } else {
                Device::<WavFile>::new(path).sample(sample_args)
            }
        }
    });
}

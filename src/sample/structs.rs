use tokio::sync::mpsc::{Receiver, Sender};

use sdr::Spectrum;

/// Channels for sending and receiving data during sampling.
pub struct SampleChannels {
    pub spectrum_tx: Sender<Spectrum>,
    pub freq_rx: Receiver<u32>,
}

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleParams {
    pub rate: u32,
    pub freq: u32,
    pub fft_size: usize,
}

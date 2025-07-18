use tokio::sync::{mpsc, watch};

use sdr::Spectrum;

pub struct SampleArgs {
    pub channels: SampleChannels,
    pub params: SampleParams,
}

/// Channels for sending and receiving data during sampling.
pub struct SampleChannels {
    pub spectrum_tx_mpsc: mpsc::Sender<Spectrum>,
    pub spectrum_tx_watch: watch::Sender<Spectrum>,
    pub freq_rx: mpsc::Receiver<u32>,
    pub flow_rx: mpsc::Receiver<bool>,
}

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleParams {
    pub rate: u32,
    pub freq: u32,
    pub fft_size: usize,
}

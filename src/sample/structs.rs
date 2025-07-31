use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch,
};

use sdr::FreqBlock;

pub struct SampleArgs {
    pub channels: SampleChannels,
    pub params: SampleParams,
}

/// Channels for sending and receiving data during sampling.
pub struct SampleChannels {
    pub spectrum_tx_mpsc: Sender<FreqBlock>,
    pub spectrum_tx_watch: watch::Sender<FreqBlock>,
    pub freq_rx: Receiver<u32>,
    pub flow_rx: Receiver<bool>,
}

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleParams {
    pub rate: u32,
    pub freq: u32,
    pub fft_size: usize,
}

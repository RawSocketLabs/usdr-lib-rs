use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch,
};

use sdr::{FreqBlock, IQBlock};

pub struct SampleArgs {
    pub channels: SampleChannels,
    pub params: SampleParams,
}

/// Channels for sending and receiving data during sampling.
pub struct SampleChannels {
    /// Sender for frequency blocks to be processed by the scanner.
    pub freq_block_tx_mpsc: Sender<FreqBlock>,
    /// Watch sender for frequency blocks to be displayed.
    pub freq_block_tx_watch: watch::Sender<FreqBlock>,
    /// Receiver for frequency updates.
    pub freq_rx: Receiver<u32>,
    /// Receiver for flow control signals.  Flow control determines whether frequency blocks are sent to be processed.
    pub flow_rx: Receiver<bool>,
    /// Sender for IQ blocks to be processed.
    pub iq_block_tx: Sender<IQBlock>,

}

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleParams {
    pub rate: u32,
    pub freq: u32,
    pub fft_size: usize,
}

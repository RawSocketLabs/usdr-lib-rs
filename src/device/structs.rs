use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch::Receiver as WatchReceiver,
};

use sdr::{FreqBlock, IQBlock};

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleArgs {
    pub rate: u32,
    pub center_freq: usize,
    pub fft_size: usize,
    pub out_watch: WatchReceiver,
    pub out_tx: Sender<OutMsg>,
    pub dev_rx: Receiver<DevMsg>,
    pub process_tx: Sender<(IQBlock, FreqBlock)>,
}

use std::{ops::Range, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};

use sdr::Spectrum;

#[derive(Debug)]
pub struct ScanResults {
    pub center_freq: u32,
    pub peaks: Spectrum,
}

pub struct ScanChannels {
    pub freq_tx: Sender<u32>,
    pub spectrum_rx: Receiver<Spectrum>,
    pub result_tx: Sender<ScanResults>,
}

pub struct ScanParams {
    pub range: Range<u32>,
    pub sleep_time: Duration,
    pub rate: u32,
    pub lag: usize,
}

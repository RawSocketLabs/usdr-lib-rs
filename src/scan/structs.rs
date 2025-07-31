use sdr::FreqBlock;
use std::{ops::Range, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ScanArgs {
    pub channels: ScanChannels,
    pub params: ScanParams,
}

#[derive(Debug)]
pub struct ScanResults {
    pub center_freq: u32,
    pub peaks: FreqBlock,
}

pub struct ScanChannels {
    pub freq_tx: Vec<Sender<u32>>,
    pub spectrum_rx: Receiver<FreqBlock>,
    pub result_tx: Sender<ScanResults>,
    pub flow_tx: Sender<bool>,
}

pub struct ScanParams {
    pub range: Range<u32>,
    pub sleep_time: Duration,
    pub rate: u32,
    pub lag: usize,
}

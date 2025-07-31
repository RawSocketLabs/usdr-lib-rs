use tokio::sync::{mpsc::Receiver, watch};

use sdr::FreqBlock;

use crate::scan::ScanResults;

pub struct DisplayArgs {
    pub channels: DisplayChannels,
    pub params: DisplayParams,
}

pub struct DisplayChannels {
    pub spectrum_rx: watch::Receiver<FreqBlock>,
    pub freq_rx: Receiver<u32>,
    pub scan_rx: Receiver<ScanResults>,
}

pub struct DisplayParams {
    pub rate: u32,
    pub start_freq: u32,
}

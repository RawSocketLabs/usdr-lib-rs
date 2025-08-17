use sdr::FreqBlock;
use std::{ops::Range, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone)]
pub struct ScanResults {
    pub center_freq: u32,
    pub peaks: FreqBlock,
}

pub struct ScanParams {
    pub range: Range<u32>,
    pub sleep_time: Duration,
    pub rate: u32,
    pub lag: usize,
    pub bandwidth: u32,
}

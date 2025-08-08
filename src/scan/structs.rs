use sdr::FreqBlock;
use std::{ops::Range, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ScanArgs {
    pub channels: ScanChannels,
    pub params: ScanParams,
}

#[derive(Debug, Clone)]
pub struct ScanResults {
    pub center_freq: u32,
    pub peaks: FreqBlock,
}

pub struct ScanChannels {
    /// A vector of channels for sending frequency updates.
    pub freq_tx: Vec<Sender<u32>>,
    /// Receiver for frequency blocks after processing samples from input source.
    pub freq_block_rx: Receiver<FreqBlock>,
    /// Sender for scan results.
    pub result_tx: Sender<ScanResults>,
    /// Sender for scan results to the TUI.
    pub tui_result_tx: Sender<ScanResults>,
    /// Flow control sender to manage the flow of frequency blocks.
    pub flow_tx: Sender<bool>,
    
}

pub struct ScanParams {
    pub range: Range<u32>,
    pub sleep_time: Duration,
    pub rate: u32,
    pub lag: usize,
    pub bandwidth: u32,
}

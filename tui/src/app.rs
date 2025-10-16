// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use shared::{DisplayInfo, DmrMetadata, FreqBlock, Peaks};
use std::collections::{BTreeMap};
use tokio::sync::{mpsc, watch};

pub struct App {
    pub current_freq_block_rx: watch::Receiver<FreqBlock>,
    pub display_info_rx: mpsc::Receiver<DisplayInfo>,
    pub metadata_rx: mpsc::Receiver<BTreeMap<u32, DmrMetadata>>,
    pub squelch_tx: mpsc::Sender<f32>,
    pub sample_rate: u32,
    pub peaks_rx: mpsc::Receiver<Peaks>,
    pub x_bounds: [f64; 2],
    pub y_bounds: [f64; 2],
    pub current_freq_block: FreqBlock,
    pub current_metadata: BTreeMap<u32, DmrMetadata>,
    pub should_quit: bool,
    pub frequency: u32,
    pub current_peaks: Option<Peaks>,
    pub table_scroll_state: usize,
    pub squelch: f32,
}

impl App {
    pub fn new(
        current_freq_block_rx: watch::Receiver<FreqBlock>,
        center_freq_rx: mpsc::Receiver<DisplayInfo>,
        peaks_rx: mpsc::Receiver<Peaks>,
        metadata_rx: mpsc::Receiver<BTreeMap<u32, DmrMetadata>>,
        squelch_tx: mpsc::Sender<f32>,
        sample_rate: u32,
        start_freq: u32,
    ) -> Self {
        let frequency = start_freq;
        let half_span_mhz = (sample_rate / 2) / 1e6 as u32;
        let center_mhz = frequency / 1e6 as u32;
        Self {
            current_freq_block_rx,
            display_info_rx: center_freq_rx,
            sample_rate,
            peaks_rx,
            metadata_rx,
            squelch_tx,
            frequency,
            x_bounds: [
                center_mhz as f64 - half_span_mhz as f64,
                center_mhz as f64 + half_span_mhz as f64,
            ],
            y_bounds: [-60.0, 0.0],
            current_freq_block: FreqBlock::new(),
            current_metadata: BTreeMap::new(),
            should_quit: false,
            current_peaks: None,
            table_scroll_state: 0,
            squelch: -100.0
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn scroll_table_down(&mut self) {
        if self.current_metadata.len() > 0 {
            self.table_scroll_state = (self.table_scroll_state + 1)
                .min(self.current_metadata.len().saturating_sub(1));
        }
    }

    pub fn scroll_table_up(&mut self) {
        self.table_scroll_state = self.table_scroll_state.saturating_sub(1);
    }
}

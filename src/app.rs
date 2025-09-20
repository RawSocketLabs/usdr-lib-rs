use comms::{FreqBlock, FreqSample};
use tokio::sync::{mpsc, watch};

pub struct App {
    pub current_freq_block_rx: watch::Receiver<FreqBlock>,
    pub center_freq_rx: mpsc::Receiver<u32>,
    pub sample_rate: u32,
    pub peaks_rx: mpsc::Receiver<Vec<FreqSample>>,
    pub x_bounds: [f64; 2],
    pub y_bounds: [f64; 2],
    pub current_freq_block: FreqBlock,
    pub should_quit: bool,
    pub frequency: u32,
    pub current_peaks: Option<Vec<FreqSample>>,
}

impl App {
    pub fn new(
        current_freq_block_rx: watch::Receiver<FreqBlock>,
        center_freq_rx: mpsc::Receiver<u32>,
        peaks_rx: mpsc::Receiver<Vec<FreqSample>>,
        sample_rate: u32,
        start_freq: u32,
    ) -> Self {
        let frequency = start_freq;
        let half_span_mhz = (sample_rate / 2) / 1e6 as u32;
        let center_mhz = frequency / 1e6 as u32;
        Self {
            current_freq_block_rx,
            center_freq_rx,
            sample_rate,
            peaks_rx,
            frequency,
            x_bounds: [center_mhz as f64 - half_span_mhz as f64, center_mhz as f64 + half_span_mhz as f64],
            y_bounds: [-60.0, 0.0],
            current_freq_block: Vec::new(),
            should_quit: false,
            current_peaks: None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

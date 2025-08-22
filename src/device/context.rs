use std::sync::Arc;
use std::time::Duration;
use rustfft::{Fft, FftPlanner};
use sdr::{apply_hann_window, compute_freq_block, compute_hann_window, FreqBlock, IQBlock, Window};

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleContext {
    pub rate: u32,
    pub freq: u32,
    pub window: Window,
    pub fft_size: usize,
    pub fft: Arc<dyn Fft<f32>>,
    pub update_sleep_time: Duration,
    pub clients_connected: bool,
}

impl SampleContext {
    pub fn new(
        rate: u32,
        freq: usize,
        fft_size: usize,
        window: Window,
        update_sleep_time: Duration,
    ) -> Self {
        // Initialize the FFT
        let fft = FftPlanner::new().plan_fft_forward(fft_size);

        // Depending on the window type, initialize the window.
        let window = match window {
            Window::Hann(_) => Window::Hann(compute_hann_window(fft_size as u32)),
        };

        // Return the context.
        Self {
            fft,
            rate,
            window,
            fft_size,
            update_sleep_time,
            freq: freq as u32,
            clients_connected: false,
        }
    }

    pub fn convert_to_freq_block(&self, iq_block: IQBlock) -> Result<FreqBlock, ()> {
        compute_freq_block(self.rate, self.fft.as_ref(), self.freq, iq_block).map_err(|_| ())
    }

    pub fn apply_window(&self, iq_block: &mut IQBlock) {
        match &self.window {
            Window::Hann(window) => apply_hann_window(iq_block, window)
        }
    }
}
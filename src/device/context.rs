use std::sync::Arc;
use std::time::Duration;
use rustfft::{Fft, FftPlanner};
use sdr::Window;

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
}
// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use rustfft::{Fft, FftPlanner};
use sdr::dsp::Window;
use sdr::sample::Freq;
use std::sync::Arc;

/// Parameters for sampling, including sample rate, frequency, and FFT size.
pub struct SampleContext {
    pub rate: u32,
    pub freq: Freq,
    pub window: Window,
    pub fft_size: usize,
    pub fft: Arc<dyn Fft<f32>>,
}

impl SampleContext {
    pub fn new(rate: u32, freq: Freq, fft_size: usize, window: Window) -> Self {
        // Initialize the FFT
        let fft = FftPlanner::new().plan_fft_forward(fft_size);

        // Return the context.
        Self {
            fft,
            rate,
            window,
            fft_size,
            freq,
        }
    }
}

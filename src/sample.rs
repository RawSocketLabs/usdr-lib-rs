use std::f32::consts::PI;

use rustfft::FftPlanner;
use sdr::{
    common::{compute_spectrum, remove_dc_offset},
    device::{Device},
    traits::SdrControl,
    types::Spectrum,
};
use tokio::sync::mpsc;
pub trait Sample<T: SdrControl> {
    fn sample(
        &mut self,
        current_spectrum_tx: mpsc::Sender<Spectrum>,
        sdr_freq_rx: mpsc::Receiver<u32>,
        sample_rate: u32,
        fft_size: usize,
        start_freq: u32,
    );
}


// TODO: make sample take a struct to encapsulate params and clean up call site.
impl<T: SdrControl> Sample<T> for Device<T> {
    fn sample(
        &mut self,
        current_spectrum_tx: mpsc::Sender<Spectrum>,
        mut sdr_freq_rx: mpsc::Receiver<u32>,
        sample_rate: u32,
        fft_size: usize,
        start_freq: u32,
    ) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        // Precompute Hann window for FFT_SIZE
        let hann_window: Vec<f32> = (0..fft_size)
            .map(|n| {
                let theta = 2.0 * PI * (n as f32) / ((fft_size - 1) as f32);
                0.5 * (1.0 - theta.cos())
            })
            .collect();
        let mut current_frequency = start_freq;

        self.set_center_frequency(current_frequency)
            .expect("Failed to set initial center frequency");

        loop {
            // Update frequency if changed
            if let Ok(new_freq) = sdr_freq_rx.try_recv() {
                current_frequency = new_freq;
                let _ = self.set_center_frequency(current_frequency);
            }

            // Read IQ samples
            match self.read_raw_spectrum(fft_size) {
                Ok(iq_samples) => {
                    // Remove DC offset to avoid dip at center frequency
                    let mut iq_samples = iq_samples;
                    remove_dc_offset(&mut iq_samples);
                    // Apply Hann window to IQ samples
                    for (i, sample) in iq_samples.iter_mut().enumerate() {
                        let w = hann_window[i];
                        sample.re *= w;
                        sample.im *= w;
                    }
                    if let Ok(spectrum) =
                        compute_spectrum(sample_rate, &*fft, current_frequency, iq_samples)
                    {
                        current_spectrum_tx.try_send(spectrum).unwrap_or(());
                    }
                }
                Err(err_msg) => {
                    eprintln!("{}", err_msg);
                }
            }
        }
    }
}

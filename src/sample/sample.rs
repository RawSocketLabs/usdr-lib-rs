use std::f32::consts::PI;
use std::sync::Arc;

use rustfft::{Fft, FftPlanner};

use sdr::{Device, SdrControl, compute_spectrum, remove_dc_offset};

use crate::sample::structs::{SampleChannels, SampleParams};
use crate::sample::traits::Sample;

impl<T: SdrControl> Sample<T> for Device<T> {
    fn sample(&mut self, mut channels: SampleChannels, params: SampleParams) {
        let (fft, hann_window, mut current_freq) = init(self, &params);

        loop {
            update_frequency_on_change(self, &mut channels, &mut current_freq);
            send_current_spectrum(self, &channels, &params, &*fft, &hann_window, current_freq);
        }
    }
}

fn init<T: SdrControl>(
    device: &mut Device<T>,
    params: &SampleParams,
) -> (Arc<dyn Fft<f32>>, Vec<f32>, u32) {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(params.fft_size);
    // Precompute Hann window for FFT_SIZE
    let hann_window: Vec<f32> = (0..params.fft_size)
        .map(|n| {
            let theta = 2.0 * PI * (n as f32) / ((params.fft_size - 1) as f32);
            0.5 * (1.0 - theta.cos())
        })
        .collect();

    device
        .set_center_frequency(params.freq)
        .expect("Failed to set initial center frequency");

    (fft, hann_window, params.freq)
}

fn update_frequency_on_change<T: SdrControl>(
    device: &mut Device<T>,
    channels: &mut SampleChannels,
    current_freq: &mut u32,
) {
    if let Ok(new_freq) = channels.freq_rx.try_recv() {
        let _ = device.set_center_frequency(new_freq);
        *current_freq = new_freq;
    }
}

fn send_current_spectrum<T: SdrControl>(
    device: &mut Device<T>,
    channels: &SampleChannels,
    params: &SampleParams,
    fft: &dyn Fft<f32>,
    hann_window: &Vec<f32>,
    current_freq: u32,
) {
    if let Ok(mut iq_samples) = device.read_raw_spectrum(params.fft_size) {
        // Remove DC offset to avoid dip at center frequency
        remove_dc_offset(&mut iq_samples);

        // Apply Hann window to IQ samples
        for (i, sample) in iq_samples.iter_mut().enumerate() {
            let w = hann_window[i];
            sample.re *= w;
            sample.im *= w;
        }

        // Compute the spectrum and send it through the channel
        if let Ok(spectrum) = compute_spectrum(params.rate, fft, current_freq, iq_samples) {
            let _ = channels.spectrum_tx.try_send(spectrum);
        }
    }
}

use std::sync::Arc;

use rustfft::{Fft, FftPlanner};

use sdr::{
    Device, SdrControl, apply_hann_window, compute_freq_block, compute_hann_window,
    remove_dc_offset,
};

use crate::sample::structs::{SampleChannels, SampleParams};
use crate::sample::traits::Sample;

impl<T: SdrControl> Sample<T> for Device<T> {
    fn sample(&mut self, mut channels: SampleChannels, params: SampleParams) {
        let (fft, hann_window, mut current_freq) = init(self, &params);

        let mut flow = false;
        loop {
            match channels.flow_rx.try_recv() {
                Ok(f) => flow = f,
                Err(_) => (),
            }
            update_frequency_on_change(self, &mut channels, &mut current_freq);
            send_current_freq_block(
                self,
                &channels,
                &params,
                &*fft,
                &hann_window,
                current_freq,
                flow,
            );
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
    let hann_window = compute_hann_window(params.fft_size as u32);

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

fn send_current_freq_block<T: SdrControl>(
    device: &mut Device<T>,
    channels: &SampleChannels,
    params: &SampleParams,
    fft: &dyn Fft<f32>,
    hann_window: &Vec<f32>,
    current_freq: u32,
    flow: bool,
) {
    if let Ok(mut iq_block) = device.read_raw_iq(params.fft_size) {
        // ISSUE: Using try_send, we will drop a ton of iq_blocks - we need to guarantee they are sent/received in order.
        // Should we use an unbounded channel? We need a mechanism to ensure we only send/receive iq blocks from the target tuned freq.
        let _ = channels.iq_block_tx.try_send(iq_block.clone());
        // Remove DC offset to avoid dip at center frequency
        remove_dc_offset(&mut iq_block);

        apply_hann_window(&mut iq_block, hann_window);
        
        if let Ok(freq_block) = compute_freq_block(params.rate, fft, current_freq, iq_block) {
            if flow {
                let _ = channels.freq_block_tx_mpsc.try_send(freq_block.clone());
            }
            let _ = channels.freq_block_tx_watch.send(freq_block);
        }
    }
}

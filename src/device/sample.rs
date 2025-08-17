// STD LIB
use std::sync::Arc;
use std::time::Duration;

// THIRD PARTY CRATES
use rustfft::{Fft, FftPlanner};

// VENDOR CRATES
use sdr::{
    Device, IQBlock, SdrControl, apply_hann_window, compute_freq_block, compute_hann_window,
    remove_dc_offset,
};

// LOCAL CRATE
use crate::CtrlMsg;
use crate::device::structs::SampleArgs;
use crate::device::traits::Sample;

impl<T: SdrControl> Sample<T> for Device<T> {
    fn sample(&mut self, mut args: SampleArgs) {
        let (fft, hann_window, mut center_freq) = init(self, &args);

        loop {
            update_frequency_on_change(self, &mut args, &mut center_freq);
            send_current_freq_block(self, &args, &*fft, &hann_window, center_freq);
        }
    }
}

fn init<T: SdrControl>(
    args: &SampleArgs,
    device: &mut Device<T>,
) -> (Arc<dyn Fft<f32>>, Vec<f32>, u32) {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(args.fft_size);

    // Precompute Hann window for FFT_SIZE
    let hann_window = compute_hann_window(args.fft_size as u32);

    device
        .set_center_frequency(args.center_freq)
        .expect("Failed to set initial center frequency");

    (fft, hann_window, args.center_freq)
}

fn update_frequency_on_change<T: SdrControl>(
    device: &mut Device<T>,
    args: &mut SampleArgs,
    current_freq: &mut u32,
) {
    if let Ok(DevMsg::ChangeDevFreq(new_freq)) = args.dev_rx.try_recv() {
        // Change the device center frequency
        let _ = device.set_center_frequency(new_freq);
        *current_freq = new_freq;

        // Give the SDR time to actually change over to the new center freq
        std::thread::sleep(args.sleep_duration);

        // Send the message back up that we have switched frequencies
        args.ctrl_tx.blocking_send(CtrlMsg::DevFreqUpdated);
    }
}

fn send_current_freq_block<T: SdrControl>(
    device: &mut Device<T>,
    args: &SampleArgs,
    fft: &dyn Fft<f32>,
    hann_window: &Vec<f32>,
    center_freq: u32,
) {
    if let Ok(mut iq_block) = device.read_raw_iq(params.fft_size) {
        // Clone the raw iq for later use
        let iq_block_raw = iq_block.clone();

        // Remove DC offset to avoid dip at center frequency
        remove_dc_offset(&mut iq_block);
        apply_hann_window(&mut iq_block, hann_window);

        if let Ok(freq_block) = compute_freq_block(params.rate, fft, center_freq, iq_block) {
            if clients_connected {
                let _ = args.out_tx.send(freq_block.clone());
            }
            // NOTE: We should come back to this and determine if we need a tick interval and
            // whether or not this thread needs to be async in nature...
            let _ = args.process_tx(iq_block_raw, freq_block).try_send();
        }
    }
}

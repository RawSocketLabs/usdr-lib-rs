use std::time::Duration;
// VENDOR CRATES
use sdr::{Device, Freq, SdrControl};

// LOCAL CRATE
use crate::device::traits::Sample;
use crate::device::{DevChannels, DevMsg, SampleContext};
use crate::io::Internal;

impl<T: SdrControl> Sample<T> for Device<T> {
    fn sample(&mut self, mut channels: DevChannels, mut ctx: SampleContext) {
        // println!("Sampling thread started");
        // TODO: Properly handle errors here...
        self.set_center_frequency(ctx.freq).unwrap();
        // println!("Set center frequency to {}", ctx.freq);

        loop {
            handle_message(self, &mut channels, &mut ctx);
            send_freq_and_iq(self, &channels, &ctx);
        }
    }
}

fn handle_message<T: SdrControl>(
    device: &mut Device<T>,
    channels: &mut DevChannels,
    ctx: &mut SampleContext,
) {
    match channels.dev_rx.try_recv() {
        Ok(DevMsg::ChangeFreq(new_freq)) => change_dev_freq(device, channels, ctx, new_freq),
        _ => {} // Log this weird error case...
    }
}

fn change_dev_freq<T: SdrControl>(
    device: &mut Device<T>,
    channels: &mut DevChannels,
    ctx: &mut SampleContext,
    new_freq: Freq,
) {
    // Change the device center frequency
    // TODO: Properly handle errors here...
    ctx.freq = new_freq;
    let _ = device.set_center_frequency(ctx.freq);

    // Give the SDR time to actually change over to the new center freq
    std::thread::sleep(Duration::from_millis(100));

    // Send the message back up that we have switched frequencies
    // TODO: Properly handle errors here...
    let _update = channels.main_tx.blocking_send(Internal::DeviceFreqUpdated);
}

fn send_freq_and_iq<T: SdrControl>(
    device: &mut Device<T>,
    channels: &DevChannels,
    ctx: &SampleContext,
) {
    if let Ok(mut iq_block) = device.read_raw_iq(ctx.fft_size) {
        // println!("Read IQ block with {} samples", iq_block.len());
        // Clone the raw iq for later use
        let iq_block_raw = iq_block.clone();

        // Remove DC offset to avoid dip at the center frequency
        iq_block.remove_dc_offset();
        iq_block.apply_window(&ctx.window);

        if let Ok(freq_block) = iq_block.compute_freq_block(ctx.rate, &*ctx.fft, ctx.freq) {
            // println!("Computed freq_block with {} samples", freq_block.len());
            // Send realtime data via watch channel if display clients are connected
            let client_count = channels.client_count.load(std::sync::atomic::Ordering::Relaxed);
            // println!("Client count: {}", client_count);
            if client_count > 0 {
                // println!("Sending realtime data to {} clients", client_count);
                let _ = channels.realtime_tx.send(freq_block.clone());
            }
            
            // Always send to process channel for main loop processing
            let _ = channels.process_tx.try_send((iq_block_raw, freq_block));
        } else {
            // println!("Failed to compute freq_block");
        }
    } else {
        // println!("Failed to read IQ block");
    }
}

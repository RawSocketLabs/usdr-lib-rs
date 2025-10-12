use std::time::Duration;
// VENDOR CRATES
use sdr::{Device, SdrControl};

// LOCAL CRATE
use crate::device::traits::Sample;
use crate::device::{DevChannels, DevMsg, SampleContext};
use crate::io::Internal;

impl<T: SdrControl> Sample<T> for Device<T> {
    fn sample(&mut self, mut channels: DevChannels, mut ctx: SampleContext) {
        // TODO: Properly handle errors here...
        self.set_center_frequency(ctx.freq).unwrap();

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
    new_freq: usize,
) {
    // Change the device center frequency
    // TODO: Properly handle errors here...
    ctx.freq = new_freq as u32;
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
        // Clone the raw iq for later use
        let iq_block_raw = iq_block.clone();

        // Remove DC offset to avoid dip at the center frequency
        iq_block.remove_dc_offset();
        iq_block.apply_window(&ctx.window);

        if let Ok(freq_block) = iq_block.compute_freq_block(ctx.rate, &*ctx.fft, ctx.freq) {
            // TODO: Properly handle errors here...
            // NOTE: We should come back to this and determine if we need a tick interval and
            // whether this thread needs to be async in nature...
            let _ = channels.process_tx.try_send((iq_block_raw, freq_block));
        }
    }
}

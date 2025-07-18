use std::thread;

use sdr::Device;
use sdr::device::file::WavFile;
use sdr::device::rtl::Rtl;

use crate::sample::{SampleChannels, SampleParams, traits::Sample};

/// Public API for starting a dedicated thread for sampling from a given device or file.
pub fn sample(
    file: String,
    rate: u32,
    sample_channels: SampleChannels,
    sample_params: SampleParams,
) {
    thread::spawn(move || match file.is_empty() {
        true => Device::<Rtl>::new(rate)
            .expect("Failed to open the RTL-SDR")
            .sample(sample_channels, sample_params),
        false => Device::<WavFile>::new(file).sample(sample_channels, sample_params),
    });
}

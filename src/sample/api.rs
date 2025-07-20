use std::thread;

use sdr::{device::rawfile::RawFile, Device};
use sdr::device::file::WavFile;
use sdr::device::rtl::Rtl;

use crate::sample::{SampleChannels, SampleParams, traits::Sample};

/// Public API for starting a dedicated thread for sampling from a given device or file.

pub fn sample(
    file: String,
    raw: bool,
    rate: u32,
    sample_channels: SampleChannels,
    sample_params: SampleParams,
) {
    thread::spawn(move || match file.is_empty() {
        true => Device::<Rtl>::new(rate)
            .expect("Failed to open the RTL-SDR")
            .sample(sample_channels, sample_params),
        false => { 
            if raw {
                Device::<RawFile>::new(file).sample(sample_channels, sample_params) 
            } else {
                Device::<WavFile>::new(file).sample(sample_channels, sample_params)
            }
        }
    });
}

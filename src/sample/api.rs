use std::thread;

use sdr::device::file::WavFile;
use sdr::device::rtl::Rtl;
use sdr::{Device, device::rawfile::RawFile};

use crate::sample::{SampleChannels, SampleParams, traits::Sample};

/// Public API for starting a dedicated thread for sampling from a given device or file.

pub fn sample(
    file: Option<String>,
    raw: bool,
    rate: u32,
    sample_channels: SampleChannels,
    sample_params: SampleParams,
) {
    thread::spawn(move || match file {
        None => Device::<Rtl>::new(rate)
            .expect("Failed to open the RTL-SDR")
            .sample(sample_channels, sample_params),
        Some(path) => {
            if raw {
                Device::<RawFile>::new(path).sample(sample_channels, sample_params)
            } else {
                Device::<WavFile>::new(path).sample(sample_channels, sample_params)
            }
        }
    });
}

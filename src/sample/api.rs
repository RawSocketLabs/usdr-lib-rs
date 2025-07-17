use std::thread;

use sdr::Device;
use sdr::device::file::WavFile;
use sdr::device::rtl::Rtl;

use crate::cli::Cli;
use crate::sample::{SampleChannels, SampleParams, traits::Sample};

/// Public API for starting a dedicated thread for sampling from a given device or file.
pub fn sample(args: Cli, sample_channels: SampleChannels, sample_params: SampleParams) {
    thread::spawn(move || match args.file.is_empty() {
        true => Device::<Rtl>::new(args.rate).sample(sample_channels, sample_params),
        false => Device::<WavFile>::new(args.file).sample(sample_channels, sample_params),
    });
}

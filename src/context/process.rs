use sdr::FreqRange;
use std::str::FromStr;

use crate::cli::Cli;

pub(crate) const DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA: usize = 1;
pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE: usize = 50;
pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_METADATA: usize = 1_000;

pub struct ProcessParameters {
    process: bool,
    pub bandwidth: u32,
    pub lag: usize,
    pub fft_size: usize,
    pub scan_cycles_required: usize,
    pub num_required_for_average: usize,
    pub num_required_for_metadata: usize,
    freq_ranges_to_ignore: Vec<FreqRange>,
}

impl ProcessParameters {
    pub fn new(args: &Cli) -> Self {
        Self {
            process: true,
            fft_size: args.fft_size,
            bandwidth: args.bandwidth,
            lag: (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize,
            scan_cycles_required: args
                .scans_before_processing
                .unwrap_or(DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA),
            num_required_for_average: args.blocks_for_average.unwrap_or(DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE),
            num_required_for_metadata: args.blocks_for_metadata.unwrap_or(DEFAULT_BLOCKS_REQUIRED_FOR_METADATA),
            freq_ranges_to_ignore: args
                .freq_ranges_to_ignore
                .clone()
                .map(|ranges| {
                    ranges
                        .iter()
                        .map(|range| FreqRange::from_str(range).unwrap())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn is_processing(&self) -> bool {
        self.process
    }

    pub fn stop(&mut self) {
        self.process = false;
    }

    pub fn start(&mut self) {
        self.process = true;
    }
}

use sdr::{FreqRange, SAMPLES_PER_SYMBOL_4800};
use std::str::FromStr;
use sdr::dmr::BITS_PER_BURST;
use crate::cli::Cli;
use crate::process::AUDIO_RATE;

pub(crate) const DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA: usize = 1;
pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE: usize = 10;

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
        let min_blocks_required_for_burst_recovery = (BITS_PER_BURST * 2 * SAMPLES_PER_SYMBOL_4800) as f32 / (args.fft_size as f32 * (AUDIO_RATE as f32 / args.rate as f32));
        Self {
            process: true,
            fft_size: args.fft_size,
            bandwidth: args.bandwidth,
            lag: (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize,
            scan_cycles_required: args
                .scans_before_processing
                .unwrap_or(DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA),
            num_required_for_average: args.blocks_for_average.unwrap_or(DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE),
            num_required_for_metadata: args.blocks_for_metadata.unwrap_or(min_blocks_required_for_burst_recovery as usize),
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

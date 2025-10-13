use sdr::{FreqRange, SAMPLES_PER_SYMBOL_4800};
use std::str::FromStr;
use sdr::dmr::BITS_PER_BURST;
use crate::cli::Cli;
use crate::process::AUDIO_RATE;
use tracing::info;

pub(crate) const DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA: usize = 1;
pub(crate) const DEFAULT_OBSERVATION_TIME_MS: u32 = 20;
pub(crate) const DEFAULT_METADATA_TIME_MS: u32 = 100;

/// Convert milliseconds to number of FFT blocks based on sample rate and FFT size
fn ms_to_blocks(time_ms: u32, sample_rate: u32, fft_size: usize) -> usize {
    let blocks = (time_ms as f32 / 1000.0 * sample_rate as f32) / fft_size as f32;
    blocks.ceil() as usize
}

pub struct ProcessParameters {
    process: bool,
    pub bandwidth: u32,
    pub lag: usize,
    pub fft_size: usize,
    pub scan_cycles_required: usize,
    pub num_required_for_average: usize,
    pub num_required_for_metadata: usize,
    freq_ranges_to_ignore: Vec<FreqRange>,
    is_file: bool,
}

impl ProcessParameters {
    pub fn new(args: &Cli) -> Self {
        // Calculate block duration in milliseconds
        let block_duration_ms = (args.fft_size as f32 / args.rate as f32) * 1000.0;
        
        // Convert millisecond times to block counts
        let observation_blocks = ms_to_blocks(args.peak_detection_time_ms, args.rate, args.fft_size);

        // Calculate minimum blocks required for DMR burst recovery
        let blocks_per_burst = (BITS_PER_BURST * 2 * SAMPLES_PER_SYMBOL_4800) as f32 / (args.fft_size as f32 * (AUDIO_RATE as f32 / args.rate as f32));
        
        // Use the larger of requested metadata time or minimum DMR requirement
        let final_metadata_blocks = (blocks_per_burst * args.max_number_of_bursts as f32) as usize;
        
        // Log the conversions for transparency
        info!("Block duration: {:.3}ms", block_duration_ms);
        info!("Observation time: {}ms = {} blocks", args.peak_detection_time_ms, observation_blocks);
        info!("Metadata time: {}ms = {} blocks (min DMR: {} blocks)", 
              args.max_number_of_bursts, final_metadata_blocks, blocks_per_burst as usize);
        
        Self {
            process: true,
            fft_size: args.fft_size,
            bandwidth: args.bandwidth,
            lag: (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize,
            scan_cycles_required: args
                .scans_before_processing
                .unwrap_or(DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA),
            num_required_for_average: observation_blocks,
            num_required_for_metadata: final_metadata_blocks,
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
            is_file: args.file.is_some(),
        }
    }

    pub fn is_processing(&self) -> bool {
        self.process
    }

    pub fn stop(&mut self) {
        if !self.is_file {
            self.process = false;
        }
    }

    pub fn start(&mut self) {
        self.process = true;
    }
}

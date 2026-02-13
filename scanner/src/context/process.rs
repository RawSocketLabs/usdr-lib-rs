// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use crate::cli::Cli;
use tracing::info;

pub(crate) const DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA: usize = 1;

/// Convert milliseconds to number of FFT blocks based on sample rate and FFT size
fn ms_to_blocks(time_ms: u32, sample_rate: u32, fft_size: usize) -> usize {
    let blocks = (time_ms as f32 / 1000.0 * sample_rate as f32) / fft_size as f32;
    blocks.ceil() as usize
}

pub struct ProcessParameters {
    process: bool,
    pub bandwidth: u32,
    pub lag: usize,
    pub scan_cycles_required: usize,
    pub num_required_for_average: usize,
    pub num_required_for_evaluation: usize,
    pub num_required_for_metadata: usize,
    is_file: bool,
    pub squelch: f32,
}

impl ProcessParameters {
    pub fn new(args: &Cli) -> Self {
        // Calculate block duration in milliseconds
        let block_duration_ms = (args.fft_size as f32 / args.rate as f32) * 1000.0;

        // Convert millisecond times to block counts
        let observation_blocks =
            ms_to_blocks(args.peak_detection_time_ms, args.rate, args.fft_size);

        // Calculate minimum blocks required for DMR burst recovery
        let blocks_per_burst = (30f32 / block_duration_ms).ceil() as usize;

        let num_required_for_evaluation = blocks_per_burst * args.min_number_of_bursts;

        // Use the larger of requested metadata time or minimum DMR requirement
        let num_required_for_metadata = blocks_per_burst * args.max_number_of_bursts;

        // Log the conversions for transparency
        info!("Block duration: {:.3}ms", block_duration_ms);
        info!(
            "Observation time: {}ms = {} blocks",
            args.peak_detection_time_ms, observation_blocks
        );
        info!(
            "Evaluation time: {}ms = {} blocks",
            block_duration_ms * num_required_for_evaluation as f32,
            num_required_for_evaluation
        );
        info!(
            "Metadata time: {}ms = {} blocks",
            block_duration_ms * num_required_for_metadata as f32,
            num_required_for_metadata
        );

        Self {
            process: true,
            bandwidth: args.bandwidth,
            lag: (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize,
            scan_cycles_required: args
                .scans_before_processing
                .unwrap_or(DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA),
            num_required_for_average: observation_blocks,
            num_required_for_evaluation,
            num_required_for_metadata,
            is_file: args.file.is_some(),
            squelch: args.squelch,
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

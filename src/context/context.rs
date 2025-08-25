// STD LIB
use std::collections::HashMap;
use std::str::FromStr;

// THIRD PARTY CRATES
use tokio::sync::mpsc::Sender;
use tokio::sync::broadcast::Sender as BraodcastSender;
// TODO: WOULD BE NICE TO REMOVE THIS DEPENDENCY BY PUSHING INTO LIBSDR
use smoothed_z_score::PeaksDetector;

// VENDOR CRATES
use sdr::{FreqBlock, FreqSample, IQBlock, update_average_db, find_peak_in_freq_block, FreqRange};
// LOCAL CRATE
use crate::Cli;
use crate::context::scan::{ScanManager, ScanMode};
use crate::device::DevMsg;
use crate::io::Output;

pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE: usize = 50;
pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_METADATA: usize = 1_000;
pub(crate) const DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA: usize = 1;


pub struct StoredInfo {
    observations: HashMap<FreqRange, Vec<Observation>>
}

pub struct CurrentState {
    peaks: Vec<FreqSample>,
    collected_iq: Vec<IQBlock>,
    average_freq_block: FreqBlock,
}


pub struct ProcessParameters {
    scan_cycles_required: usize,
    freq_ranges_to_ignore: Vec<FreqRange>,
}

//pub struct Testing {
//    pub block: BlockParameters,
//    pub channels: ContextChannels,
//    pub current: CurrentState,
//    pub storage: StoredInfo,
//    pub process: ProcessParameters,
//    pub scan: ScanContext,
//}

/// Context object to hold state during runtime
pub(crate) struct Context {
    /// The current scan mode.
    pub(crate) mode: ScanMode,

    /// The current detected peaks that were not ignored.
    pub(crate) peaks: Vec<FreqSample>,

    /// Whether to process blocks of IQ and Freq data.
    pub(crate) process_blocks: bool,

    /// The collected IQ blocks for the current frequency.
    pub(crate) collected_iq: Vec<IQBlock>,

    /// The running average of the frequency blocks.
    pub(crate) average_freq_block: FreqBlock,

    /// The number of blocks required before leveraging the running average to detect peaks.
    pub(crate) blocks_required_for_average: usize,

    /// The number of blocks required before attempting to process the IQ data for signal metadata.
    pub(crate) blocks_required_for_metadata: usize,

    /// The scan manager tracks the current frequency and cycles through the scan ranges.
    pub(crate) scan_manager: ScanManager,

    /// Contains the observed peaks and their associated metadata, for the duration of the runtime.
    observed_peaks: HashMap<FreqRange, Vec<Observation>>,

    /// Contains information about peaks that should be ignored.
    freq_ranges_to_ignore: Vec<FreqRange>,

    /// The number of scan cycles required to process the IQ data for signal metadata.
    scan_cycles_required_for_metadata: usize,

    // TODO: MAKE THESE OPTIONS INTO A HELD STRUCT OR SOMETHING...
    bandwidth: u32,
    lag: usize,
    fft_size: usize,
}

impl Context {
    pub fn new(args: &Cli, dev_tx: Sender<DevMsg>, out_tx: BraodcastSender<Output>) -> Result<Self, ()> {
        // Validate the scan manager before moving it into the struct
        let scan_manager = ScanManager::new(&args, dev_tx, out_tx)?;

        Ok(Self {
            peaks: vec![],
            process_blocks: true,
            mode: ScanMode::SweepThenProcess,
            average_freq_block: FreqBlock::new(),
            collected_iq: Vec::with_capacity(DEFAULT_BLOCKS_REQUIRED_FOR_METADATA),
            blocks_required_for_average: DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE,
            blocks_required_for_metadata: DEFAULT_BLOCKS_REQUIRED_FOR_METADATA,
            scan_cycles_required_for_metadata: DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA,
            scan_manager,
            observed_peaks: HashMap::new(),
            freq_ranges_to_ignore: Vec::new(),
            lag: (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize,
            bandwidth: args.bandwidth,
            fft_size: args.fft_size,
        })
    }

    /// Advance the context.
    ///
    /// Ensure the current context is cleaned up before moving on to the next context.
    /// Calls on the scan manager to advance to the next frequency as well.
    pub fn next(&mut self) {
        // Ensure the context objects are cleaned up
        self.peaks.clear();
        self.collected_iq.clear();
        self.average_freq_block.clear();

        // Stop processing blocks and switch to the next frequency
        self.process_blocks = false;
        self.scan_manager.next()
    }

    pub fn update_average(&mut self, freq_block: FreqBlock) {
        if self.average_freq_block.is_empty() {
            self.average_freq_block = freq_block;
        } else {
            update_average_db(self.collected_iq.len() as f32, &mut self.average_freq_block, freq_block);
        }
    }

    pub fn detect_peaks(&mut self) {
        self.peaks = find_peak_in_freq_block(
            self.average_freq_block.clone(),
            self.bandwidth,
            PeaksDetector::new(self.lag, 5.0, 0.5),
        );

        let timestamp = chrono::Utc::now();
        for peak_sample in &self.peaks {
            //for key in self.observed_peaks.keys() {
            //    if key.includes(peak_sample) {
            //        let observations = self.observed_peaks.get_mut(&key).unwrap();
            //        observations.push(Observation {
            //            timestamp,
            //            sample: *peak_sample,
            //       });
            //    }
            //}
            // TODO: How do we do some math to figure out what the likely center of the peak is?
            // NOTE: Potentially give slack space on either side of the peak? - Make this a configurable context item.
            self.observed_peaks.insert(
                FreqRange::new(peak_sample.freq as usize, (peak_sample.freq + 1) as usize).unwrap(),
                vec![Observation {
                    timestamp,
                    sample: *peak_sample,
                }],
            );
        }
    }

    pub fn completed_required_scan_cycles(&self) -> bool {
        self.scan_manager.cycles_completed >= self.scan_cycles_required_for_metadata
    }
}

#[derive(Clone)]
pub struct Observation {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sample: FreqSample,
}

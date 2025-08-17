// STD LIB
use std::{collections::HashSet, time::Duration};

// THIRD PARTY CRATES
use tokio::sync::mpsc::Sender;

// VENDOR CRATES
use sdr::{FreqBlock, FreqSample, IQBlock};

use crate::Cli;
use crate::device::SampleParams;

pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE: usize = 50;
pub(crate) const DEFAULT_BLOCKS_REQUIRED_FOR_METADATA: usize = 1_000;
pub(crate) const DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA: usize = 1;

pub(crate) struct Context {
    pub(crate) mode: ScanMode,
    pub(crate) peaks: Vec<Peak>,
    pub(crate) process_blocks: bool,
    pub(crate) collected_iq: Vec<IQBlock>,
    pub(crate) average_freq_block: FreqBlock,
    pub(crate) blocks_required_for_average: usize,
    pub(crate) blocks_required_for_metadata: usize,
    pub(crate) scan_manager: ScanManager,

    observed_peaks: HashSet<Peak>,
    peaks_to_ignore: HashSet<Peak>,
    scan_cycles_required_for_metadata: usize,
    bandwidth: u32,
    lag: usize,
    fft_size: usize,
}

impl Context {
    pub fn new(args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        // Validate the scan manager before moving it into the struct
        let scan_manager = ScanManager::new(&args, dev_tx)?;

        Ok(Self {
            peaks: vec![],
            process_blocks: true,
            mode: ScanMode::SweepAndProcess,
            average_freq_block: FreqBlock::new(),
            collected_iq: Vec::with_capacity(DEFAULT_BLOCKS_REQUIRED_FOR_METADATA),
            blocks_required_for_average: DEFAULT_BLOCKS_REQUIRED_FOR_AVERAGE,
            blocks_required_for_metadata: DEFAULT_BLOCKS_REQUIRED_FOR_METADATA,
            scan_cycles_required_for_metadata: DEFAULT_SCAN_CYCLES_REQUIRED_FOR_METADATA,
            scan_manager,
            observed_peaks: HashSet::new(),
            peaks_to_ignore: HashSet::new(),
            lag: (args.bandwidth / (args.rate / 4 / args.fft_size as u32)) as usize,
            bandwidth: args.bandwidth,
            fft_size: args.fft_size,
        })
    }

    pub fn next(&mut self) {
        // Ensure the context objects are cleaned up
        self.peaks.clear();
        self.collected_iq.clear();
        self.average_freq_block.clear();

        // Stop proecssing blocks and switch to the next frequency
        self.process_blocks = false;
        self.scan_manager.next()
    }

    pub fn update_average(&mut self, freq_block: FreqBlock) {
        if self.average_freq_block.is_empty() {
            self.average_freq_block = freq_block;
        } else {
        }
    }

    pub fn detect_peaks(&mut self) -> &Vec<Peak> {
        // TODO:....
        &self.peaks
    }

    pub fn completed_required_scan_cycles(self) -> bool {
        self.scan_manager.cycles_completed >= self.scan_cycles_required_for_metadata
    }
}

pub enum ScanMode {
    SweepThenProcess,
    SweepAndProcess,
}

struct ScanManager {
    idx: usize,
    rate: u32,
    current: usize,
    step_size: usize,
    cycles_completed: usize,
    sleep_duration: Duration,
    ranges: Vec<FreqRange>,
    dev_tx: Sender<DevMsg>,
}

impl ScanManager {
    fn new(args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        match freq_ranges.len() {
            0 => Err(()),
            _ => Self {
                idx: 0,
                rate: args.rate,
                step_size: args.rate / 4,
                sleep_duration: Duration::from_millis(args.sleep_ms),
                cycles_completed: 0,
                current: ranges[0].start,
                ranges,
                dev_tx,
            },
        }
    }

    pub fn current(&self) -> usize {
        self.current
    }

    fn next(&mut self) {
        if self.ranges[self.idx].end <= self.current + self.step_size {
            self.current += self.step_size;
        } else {
            self.idx = (self.idx + 1) % self.ranges.len();
            if self.idx == 0 {
                self.cycles_completed += 1;
            }
            self.current = self.ranges[self.idx].start;
        }

        self.dev_tx
            .blocking_send(DevMsg::ChangeDevFreq(self.current))
            .unwrap();
    }
}

////// LIBSDR
pub struct FreqRange {
    start: usize,
    stop: usize,
}

impl FreqRange {
    pub fn new(start: usize, stop: usize) -> Result<Self, ()> {
        match start < stop {
            true => Ok(Self { start, stop }),
            false => Err(()),
        }
    }

    pub fn includes(&self, sample: FreqSample) -> bool {
        sample.freq >= self.start && sample.freq <= self.end
    }
}

pub struct Peak {
    freq: usize,
    //params: PeakParameters,
    observations: Vec<PeakMetadata>,
}

pub struct PeakMetadata {
    timestamp: u64,
    samples: Vec<FreqSample>,
}

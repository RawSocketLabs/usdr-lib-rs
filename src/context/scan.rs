use crate::cli::Cli;
use crate::device::DevMsg;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use sdr::FreqRange;
use tokio::sync::mpsc::Sender;

pub struct ScanContext {
    pub mode: ScanMode,
    manager: ScanManager,
}

impl ScanContext {
    pub fn new(mode: ScanMode, args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        Ok(Self {
            mode,
            manager: ScanManager::new(args, dev_tx)?
        })
    }

    pub fn current(&self) -> usize {
        self.manager.current()
    }

    pub fn rate(&self) -> u32 {
        self.manager.rate
    }

    pub fn next(&mut self) {
        self.manager.next()
    }

    pub fn cycles(&self) -> usize {
        self.manager.cycles_completed
    }
}


pub(crate) struct ScanManager {
    idx: usize,
    pub(crate) rate: u32,
    current: usize,
    step_size: usize,
    pub(crate) cycles_completed: usize,
    sleep_duration: Duration,
    ranges: Vec<FreqRange>,
    dev_tx: Sender<DevMsg>,
}

impl ScanManager {
    pub(crate) fn new(args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        let ranges: Vec<FreqRange> = args
            .ranges
            .iter()
            .map(|s| FreqRange::from_str(s).unwrap())
            .collect();
        match args.ranges.len() {
            0 => Err(()),
            _ => Ok(Self {
                idx: 0,
                rate: args.rate,
                step_size: (args.rate as usize) / 4,
                sleep_duration: Duration::from_millis(args.sleep_ms),
                cycles_completed: 0,
                current: ranges[0].start,
                ranges,
                dev_tx,
            }),
        }
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub(crate) fn next(&mut self) {
        // Increment the current center frequency
        if self.ranges[self.idx].stop >= self.current + self.step_size {
            self.current += self.step_size;
        } else {
            self.idx = (self.idx + 1) % self.ranges.len();
            self.current = self.ranges[self.idx].start;
            if self.idx == 0 {
                self.cycles_completed += 1;
            }
        }

        sleep(self.sleep_duration);
        
        // Send a message to the device
        // TODO: Handle errors properly
        self.dev_tx
            .try_send(DevMsg::ChangeFreq(self.current))
            .unwrap();

    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ScanMode {
    SweepThenProcess,
    SweepAndProcess,
}
impl FromStr for ScanMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SweepThenProcess" => Ok(ScanMode::SweepThenProcess),
            "SweepAndProcess" => Ok(ScanMode::SweepAndProcess),
            _ => Err(()),
        }
    }
}
// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use crate::cli::Cli;
use crate::device::DevMsg;
use sdr::sample::{Freq, FreqRange};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub struct ScanContext {
    pub mode: ScanMode,
    manager: ScanManager,
}

impl ScanContext {
    pub fn new(mode: ScanMode, args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        Ok(Self {
            mode,
            manager: ScanManager::new(args, dev_tx)?,
        })
    }

    pub fn current(&self) -> Freq {
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
    current: Freq,
    step_size: usize,
    pub(crate) cycles_completed: usize,
    sleep_duration: Duration,
    range_type: FreqRangeType,
    dev_tx: Sender<DevMsg>,
}

pub enum FreqRangeType {
    #[allow(dead_code)]
    Fixed(usize),
    Ranges(Vec<FreqRange>),
}

impl ScanManager {
    pub(crate) fn new(args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        let (range_type, current) = match args.file.is_some() {
            true => (
                FreqRangeType::Fixed(args.center_frequency as usize),
                args.center_frequency as usize,
            ),
            false => {
                let ranges: Vec<FreqRange> = args
                    .ranges
                    .iter()
                    .map(|s| FreqRange::from_str(s).unwrap())
                    .collect();
                if ranges.len() == 0 {
                    return Err(());
                }
                let current = ranges.first().unwrap().start;
                (FreqRangeType::Ranges(ranges), current)
            }
        };

        Ok(Self {
            idx: 0,
            rate: args.rate,
            step_size: (args.rate as usize) / 4,
            sleep_duration: Duration::from_millis(args.sleep_ms),
            cycles_completed: 0,
            current: current.into(),
            range_type,
            dev_tx,
        })
    }

    pub fn current(&self) -> Freq {
        self.current
    }

    pub(crate) fn next(&mut self) {
        match &self.range_type {
            FreqRangeType::Fixed(_) => self.cycles_completed += 1,
            FreqRangeType::Ranges(ranges) => {
                // Increment the current center frequency
                if ranges[self.idx].stop >= self.current.as_usize() + self.step_size {
                    *self.current += self.step_size as u32;
                } else {
                    self.idx = (self.idx + 1) % ranges.len();
                    self.current = ranges[self.idx].start.into();
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

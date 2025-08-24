use crate::cli::Cli;
use crate::device::DevMsg;
use std::str::FromStr;
use std::time::Duration;
use sdr::FreqRange;
use tokio::sync::mpsc::Sender;

pub struct ScanContext {
    mode: ScanMode,
    manager: ScanManager,
}


pub(crate) struct ScanManager {
    idx: usize,
    rate: u32,
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
        }

        // Update the cycle count
        if self.idx == 0 {
            self.cycles_completed += 1;
        }

        // Send a message to the device
        self.dev_tx
            .try_send(DevMsg::ChangeFreq(self.current))
            .unwrap();
    }
}

#[derive(PartialEq, Eq)]
pub enum ScanMode {
    SweepThenProcess,
    SweepAndProcess,
}

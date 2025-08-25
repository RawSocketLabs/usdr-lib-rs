use crate::cli::Cli;
use crate::device::DevMsg;
use std::str::FromStr;
use std::time::Duration;
use comms::DisplayInfo;
use sdr::FreqRange;
use tokio::sync::mpsc::Sender;
use tokio::sync::broadcast::Sender as BroadcastSender;
use crate::io::Output;

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
    out_tx: BroadcastSender<Output>,
}

impl ScanManager {
    pub(crate) fn new(args: &Cli, dev_tx: Sender<DevMsg>, out_tx: BroadcastSender<Output>) -> Result<Self, ()> {
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
                out_tx,
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
        
        self.out_tx.send(Output::Display(DisplayInfo { center_freq: self.current, rate: self.rate as usize})).unwrap();

        // Send a message to the device
        // TODO: Handle errors properly
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

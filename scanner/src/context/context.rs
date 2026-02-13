// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

// STD LIB
use std::str::FromStr;

// THIRD PARTY CRATES
use tokio::sync::mpsc::Sender;
// TODO: WOULD BE NICE TO REMOVE THIS DEPENDENCY BY PUSHING INTO LIBSDR

// VENDOR CRATES

// LOCAL CRATE
use crate::Cli;
use crate::context::{CurrentState, ProcessParameters, ScanContext, ScanMode, StoredInfo};
use crate::device::DevMsg;

/// Context object to hold state during runtime
pub(crate) struct Context {
    pub(crate) current: CurrentState,
    pub(crate) process: ProcessParameters,
    pub(crate) scan: ScanContext,
    pub(crate) storage: StoredInfo,
}

impl Context {
    pub fn new(args: &Cli, dev_tx: Sender<DevMsg>) -> Result<Self, ()> {
        Ok(Self {
            current: CurrentState::default(),
            process: ProcessParameters::new(args),
            scan: ScanContext::new(ScanMode::from_str(&args.scan_mode)?, args, dev_tx)?,
            storage: StoredInfo::default(),
        })
    }

    /// Advance the context.
    ///
    /// Ensure the current context is cleaned up before moving on to the next context.
    /// Calls on the scan manager to advance to the next frequency as well.
    pub fn next(&mut self) {
        self.process.stop();
        self.current.clear();
        self.scan.next();
    }

    //pub fn detect_peaks(&mut self) {
    //    self.current.peaks = find_peak_in_freq_block(
    //        self.current.average_freq_block.clone(),
    //        self.process.bandwidth,
    //        PeaksDetector::new(self.process.lag, 5.0, 0.5),
    //    );

    //    //let timestamp = chrono::Utc::now();
    //    //for peak_sample in &self.peaks {
    //    //    //for key in self.observed_peaks.keys() {
    //    //    //    if key.includes(peak_sample) {
    //    //    //        let observations = self.observed_peaks.get_mut(&key).unwrap();
    //    //    //        observations.push(Observation {
    //    //    //            timestamp,
    //    //    //            sample: *peak_sample,
    //    //    //       });
    //    //    //    }
    //    //    //}
    //    //    // TODO: How do we do some math to figure out what the likely center of the peak is?
    //    //    // NOTE: Potentially give slack space on either side of the peak? - Make this a configurable context item.
    //    //    self.observed_peaks.insert(
    //    //        FreqRange::new(peak_sample.freq as usize, (peak_sample.freq + 1) as usize).unwrap(),
    //    //        vec![Observation {
    //    //            timestamp,
    //    //            sample: *peak_sample,
    //    //        }],
    //    //    );
    //    //}
    //}
}

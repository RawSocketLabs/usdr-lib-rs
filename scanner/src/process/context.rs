// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use sdr::sample::Freq;

pub struct ProcessContext {
    pub(crate) center_freq: Freq,
    pub(crate) sample_rate: u32,
    // pub(crate) process_type: ProcessType,
}

impl ProcessContext {
    pub fn new(
        center_freq: Freq,
        sample_rate: u32,
        // process_type: ProcessType,
    ) -> Self {
        Self {
            center_freq,
            sample_rate,
            // process_type,
        }
    }
}

// pub enum ProcessType {
// RawIQ,
// PreProcess,
// Metadata,
// }

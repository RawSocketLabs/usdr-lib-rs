// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use crate::context::ProcessParameters;
use sdr::sample::{AverageFreqBlock, Freq, PeakParameters, Peaks};
use sdr::{FreqBlock, IQBlock};
use smoothed_z_score::PeaksDetector;

// // TODO: There is something here about how it should impact center freq tracking.
// pub struct ObservedPeaks {
//     center_freq: Vec<CenterFreq>,
// }
//
// #[derive(Debug, Default)]
// pub struct CenterFreq {
//     pub average_center_freq_sample: FreqSample,
//     pub observation_count: usize,
//     pub bandwidth: usize,
//     pub last_updated: u128,
// }
//
// impl CenterFreq {
//     // Take an initial peak sample and an expected signal bandwidth to generate a new center freq for tracking
//     pub fn new(initial_freq_sample: FreqSample, bandwidth: usize) -> Self {
//         Self {
//             average_center_freq_sample: initial_freq_sample,
//             observation_count: 1,
//             bandwidth,
//             last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u128,
//         }
//     }
//
//     // Update an existing center freq with a new sample
//     pub fn update(&mut self, current_freq: FreqSample) {
//         self.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u128;
//         self.observation_count += 1;
//         self.
//     }
//
//     // Check to see if the passed frequency should update an existing center freq or be processed as a new center freq
//     // TODO: This probably relates to a confidence interval based on how many observations have occured ect.
//     // TODO: There is probably an additional update step where this should store the last 5 samples and get metadata for it
//     // TODO: From there we can leverage the metadata and the current info to figure out if there are two close frequencies
//     // TODO: Start simple and just go within band portion.
//     pub fn meets_center_freq_threshold_criteria() -> bool {
//         true
//     }
// }

// #[derive(Debug, Default)]
// pub struct ProcessingFreq(u128, Freq);
//
// impl ProcessingFreq {
//     pub fn new(end_time: u128, freq: Freq) -> Self {
//         Self(end_time,freq)
//     }
//
//     pub fn expired(&self, current_time: u128) -> bool {
//         current_time >= self.0
//     }
//
//     pub fn freq(&self) -> Freq {
//         self.1
//     }
// }

#[derive(Debug, Default)]
pub struct CurrentState {
    // Peaks detected during the peak evaluation interval
    pub peaks: Peaks,

    // Peaks that are currently being processed (within band)
    pub processing_peaks: Peaks,

    // pub observed_peaks: Vec<CenterFreq>,
    pub collected_iq: Vec<IQBlock>,
    pub average_freq_block: AverageFreqBlock,
}

impl CurrentState {
    pub fn clear(&mut self) {
        self.peaks.clear();
        self.collected_iq.clear();
        self.average_freq_block.block.clear();
        self.average_freq_block.count = 0;
    }

    pub fn remove_processed_peaks(&mut self, processed_peaks: Peaks) {
        for processed_peak in processed_peaks {
            self.processing_peaks
                .retain(|peak| peak.sample.freq != processed_peak.sample.freq);
        }
    }

    pub fn update(
        &mut self,
        iq_block: IQBlock,
        freq_block: FreqBlock,
        params: &ProcessParameters,
    ) -> bool {
        self.collected_iq.push(iq_block);
        if self.collected_iq.len() % params.num_required_for_average != 0 {
            self.update_average(freq_block);
            false
        } else {
            true
        }
    }

    fn update_average(&mut self, freq_block: FreqBlock) {
        self.average_freq_block.update(freq_block);
    }

    pub fn detect_peaks(&mut self, params: &ProcessParameters, center_freq: Freq) {
        let peak_params = PeakParameters {
            bandwidth: params.bandwidth,
            detector: PeaksDetector::new(params.lag, 5.0, 0.5),
        };
        self.average_freq_block.block.squelch(params.squelch);
        self.peaks.extend(
            self.average_freq_block
                .get_peaks_with_params(peak_params, center_freq),
        );
        self.average_freq_block.block.clear();
        self.average_freq_block.count = 0;
    }

    pub fn reduce_peaks(&mut self, params: &ProcessParameters) {
        let half_bandwidth = params.bandwidth as i64 / 2;
        let mut unfiltered_peaks = std::mem::take(&mut self.peaks);
        unfiltered_peaks.sort_by(|a, b| b.sample.db.total_cmp(&a.sample.db));

        for unfilitered in unfiltered_peaks.into_iter() {
            let mut existing = false;
            for reduced in &self.peaks {
                if ((*unfilitered.sample.freq as i64) - (*reduced.sample.freq as i64)).abs()
                    <= half_bandwidth
                {
                    // A stronger peak already occupies this band; skip
                    existing = true;
                    break;
                }
            }
            if !existing {
                self.peaks.push(unfilitered);
            }
        }

        self.peaks.sort_by_key(|p| *p.sample.freq);
    }
}

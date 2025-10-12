use sdr::{AverageFreqBlock, FreqBlock, IQBlock, PeakParameters, Peaks};
use smoothed_z_score::PeaksDetector;
use crate::context::ProcessParameters;

#[derive(Debug, Default)]
pub struct CurrentState {
    pub peaks: Peaks,
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
    pub fn update(&mut self, iq_block: IQBlock, freq_block: FreqBlock) {
        self.collected_iq.push(iq_block);
        if self.peaks.is_empty() {
            self.update_average(freq_block);
        }
    }

    fn update_average(&mut self, freq_block: FreqBlock) {
       self.average_freq_block.update(freq_block);
    }

    pub fn detect_peaks(&mut self, params: &ProcessParameters) {
        let peak_params = PeakParameters {
            bandwidth: params.bandwidth,
            detector: PeaksDetector::new(params.lag, 5.0, 0.5),
        };

        self.peaks = self.average_freq_block.get_peaks_with_params(peak_params);
    }

    pub fn peak_detection_criteria_met(&self, params: &ProcessParameters) -> bool {
        self.peaks.is_empty() && self.collected_iq.len() >= params.num_required_for_average
    }

}

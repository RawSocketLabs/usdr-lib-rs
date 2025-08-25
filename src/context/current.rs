use sdr::{find_peak_in_freq_block, update_average_db, FreqBlock, FreqSample, IQBlock};
use smoothed_z_score::PeaksDetector;
use crate::context::ProcessParameters;

#[derive(Debug, Default)]
pub struct CurrentState {
    pub peaks: Vec<FreqSample>,
    pub collected_iq: Vec<IQBlock>,
    pub average_freq_block: FreqBlock,
}

impl CurrentState {
    pub fn clear(&mut self) {
        self.peaks.clear();
        self.collected_iq.clear();
        self.average_freq_block = FreqBlock::default();
    }
    pub fn update(&mut self, iq_block: IQBlock, freq_block: FreqBlock) {
        self.collected_iq.push(iq_block);
        if self.peaks.is_empty() {
            self.update_average(freq_block);
        }
    }

    fn update_average(&mut self, freq_block: FreqBlock) {
        if self.average_freq_block.is_empty() {
            self.average_freq_block = freq_block;
        } else {
            update_average_db(self.collected_iq.len() as f32, &mut self.average_freq_block, freq_block);
        }
    }

    pub fn detect_peaks(&mut self, params: &ProcessParameters) {
        self.peaks = find_peak_in_freq_block(
            std::mem::take(&mut self.average_freq_block),
            params.bandwidth,
            PeaksDetector::new(params.lag, 5.0, 0.5),
        );
    }

    pub fn peak_detection_criteria_met(&self, params: &ProcessParameters) -> bool {
        self.peaks.is_empty() && self.collected_iq.len() >= params.num_required_for_average
    }

}

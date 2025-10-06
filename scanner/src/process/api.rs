use shared::DmrMetadata;
// THIRD PARTY CRATES
use rayon::prelude::*;
// VENDOR CRATES
use crate::process::{ScanDmrMetadataExt, ProcessContext, SignalPreProcessor};
use sdr::{FreqSample, IQBlock, IQSample, Peaks};
use sdr::dmr::DmrProcessor;

pub fn process_peaks(ctx: ProcessContext, iq_blocks: Vec<IQBlock>, peaks: Peaks) -> Vec<DmrMetadata> {
    // TODO: There has to be a better way...
    let flat = IQBlock::from(iq_blocks.into_iter().flat_map(|block| block.inner()).collect::<Vec<IQSample>>());

    let blocks: Vec<(&FreqSample, IQBlock)> = peaks.iter().map(|peak| (&peak.sample, flat.clone())).collect();

    let metadata: Vec<DmrMetadata> = blocks
        .into_par_iter()
        .filter_map(|(peak, mut flat)| {
            flat.freq_shift(ctx.sample_rate, (peak.freq as i32 - ctx.center_freq as i32) as f32);

            let mut metadata = DmrMetadata::new(peak.freq, peak.db);

            let mut signal_pre_processor = SignalPreProcessor::new(flat, ctx.sample_rate);
            signal_pre_processor.run().unwrap();

            let mut dmr_processor = DmrProcessor::new();
            for sample in signal_pre_processor.get_processed_samples() {
                dmr_processor.push_sample(sample);
            }

            let mut recovered_bursts = false;
            dmr_processor.get_bursts().into_iter().for_each(|burst| {
                recovered_bursts = true;
                metadata.update_from_burst(burst);
            });

            if recovered_bursts {
                return Some(metadata)
            }
            None
        }).collect();

    metadata
}

// TODO: Send the data to the output channel

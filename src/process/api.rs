// THIRD PARTY CRATES
use rayon::prelude::*;
// VENDOR CRATES
use crate::process::{DmrMetadata, ProcessContext, SignalPreProcessor};
use sdr::{DmrProcessor, FreqSample, IQBlock, freq_shift_iq_block};

pub fn process_peaks(mut ctx: ProcessContext, iq_blocks: Vec<IQBlock>, peaks: &[FreqSample]) -> Vec<DmrMetadata> {
    let flat: IQBlock = iq_blocks.into_iter().flatten().collect();

    let blocks: Vec<(&FreqSample, IQBlock)> = peaks.iter().map(|p| (p, flat.clone())).collect();

    let metadata: Vec<DmrMetadata> = blocks
        .into_par_iter()
        .filter_map(|(peak, mut flat)| {
            freq_shift_iq_block(
                &mut flat,
                ctx.sample_rate,
                (peak.freq as i32 - ctx.center_freq as i32) as f32,
            );
            let mut metadata = DmrMetadata::new(peak.freq, peak.db);

            let mut signal_pre_processor = SignalPreProcessor::new(flat, ctx.sample_rate);
            signal_pre_processor.run().unwrap();

            let mut dmr_processor = DmrProcessor::new();
            for sample in signal_pre_processor.get_processed_samples() {
                dmr_processor.push_sample(sample);
            }

            // Find the first matching data burst that contains the needed information
            dmr_processor.get_bursts().into_iter().for_each(|burst| {
                metadata.update_from_burst(burst);
            });

            match metadata.we_care {
                true => Some(metadata),
                _ => None
            }

            // TODO: Metadata decode happens here as well... can be a decision (via injected context options)

            // Collect the data into a structured output message rather than a specific type...

        }).collect();

    println!("{:#?}", metadata);

    metadata
}

// TODO: Send the data to the output channel

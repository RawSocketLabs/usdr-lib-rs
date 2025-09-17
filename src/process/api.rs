// THIRD PARTY CRATES
use rayon::prelude::*;
use rust_dsdcc::DSDDecoder;
use rust_dsdcc::ffi::DSDDecodeMode;
// VENDOR CRATES
use sdr::{FreqSample, IQBlock, freq_shift_iq_block, DmrProcessor};

use crate::process::{ProcessContext, SignalMetadata, SignalPreProcessor};

pub fn process_peaks(ctx: ProcessContext, iq_blocks: Vec<IQBlock>, peaks: &Vec<FreqSample>) {
    let mut flat: IQBlock = iq_blocks.into_iter().flatten().collect();

    let mut blocks: Vec<(&FreqSample, IQBlock)> = peaks.iter().map(|p| (p, flat.clone())).collect();



    let _res: Vec<SignalMetadata> = blocks
        .into_par_iter()
        .map(|(peak, mut flat)| {
            freq_shift_iq_block(
                &mut flat,
                ctx.sample_rate,
                (peak.freq as i32 - ctx.center_freq as i32) as f32,
            );
            let mut signal_pre_processor = SignalPreProcessor::new(flat);
            let _ = signal_pre_processor.run().unwrap();

            let mut dmr_processor = DmrProcessor::new();
            for sample in signal_pre_processor.get_processed_samples() {
                dmr_processor.push_sample(sample);
            }

            for burst in dmr_processor.get_bursts() {
                println!("{:#?}", burst);
            }

            // TODO: Metadata decode happens here as well... can be a decision (via injected context options)

            // Collect the data into a structured output message rather than a specific type...
            SignalMetadata {
                timestamp: chrono::Utc::now().timestamp_millis(),
                peak: peak.clone(),
                processed_samples: signal_pre_processor.get_processed_samples(),
            }
        })
        .collect();

    // TODO: Send the data to the output channel
}

// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use shared::DmrMetadata;
// THIRD PARTY CRATES
use rayon::prelude::*;
use sdr::sample::{FreqSample, IQSample, Peaks};
use sdr::{DmrProcessor, IQBlock};
use tracing::trace;
// VENDOR CRATES
use crate::process::{ProcessContext, ScanDmrMetadataExt, SignalPreProcessor};

pub fn process_peaks(
    sample_rate: u32,
    center_freq: u32,
    iq_blocks: Vec<IQBlock>,
    peaks: Peaks,
) -> Vec<DmrMetadata> {
    // TODO: There has to be a better way...
    let flat = IQBlock::from(
        iq_blocks
            .into_iter()
            .flat_map(|block| block.inner())
            .collect::<Vec<IQSample>>(),
    );

    let blocks: Vec<(&FreqSample, IQBlock)> = peaks
        .iter()
        .map(|peak| (&peak.sample, flat.clone()))
        .collect();

    let metadata: Vec<DmrMetadata> = blocks
        .into_par_iter()
        .filter_map(|(peak, mut flat)| {
            flat.freq_shift(sample_rate, (*peak.freq as i32 - center_freq as i32) as f32);

            let mut metadata = DmrMetadata::new(peak.freq, peak.db);

            let mut signal_pre_processor = SignalPreProcessor::new(flat, sample_rate);
            signal_pre_processor.run().unwrap();

            let mut dmr_processor = DmrProcessor::new();
            for sample in signal_pre_processor.get_processed_samples() {
                dmr_processor.push_sample(sample);
            }

            let mut recovered_bursts = false;
            dmr_processor.get_bursts().into_iter().for_each(|burst| {
                trace!(
                    "{} MHz Recovered burst: {:?}",
                    peak.freq.as_f64() / 1e6,
                    burst
                );
                recovered_bursts = true;
                metadata.update_from_burst(burst);
            });

            if recovered_bursts {
                return Some(metadata);
            }
            None
        })
        .collect();

    metadata
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::process::process_peaks;
    use sdr::sample::{Freq, FreqSample, IQSample, Peak, Peaks};
    use sdr::{Device, IQBlock, SdrControl, WavFile};

    #[test]
    fn integration() {
        let mut device = Device::<WavFile>::new("resources/iq.wav", false);

        let data = device.read_raw_iq(device.dev.header.data_header.len as usize).unwrap();
        let data = data
            .chunks(4096)
            .collect::<Vec<&[IQSample]>>();

        let peaks: Peaks = vec![Peak::new(
            Freq::new(device.dev.header.auxi.center_freq),
            FreqSample::new(145190000, 1.0),
            0,
        )];
        
        let syncs_count = Arc::new(AtomicUsize::new(0));
        let messages_count = Arc::new(AtomicUsize::new(0));

        let handles = data.chunks(90).map(|chunk|
        {
            let chunk = chunk
                .iter()
                .map(|chunk| IQBlock::from(chunk.to_vec()))
                .collect::<Vec<IQBlock>>();
            let peaks = peaks.clone();
            let sample_rate = device.dev.header.info.sample_rate;
            let center_freq = device.dev.header.auxi.center_freq;
            let syncs_count = Arc::clone(&syncs_count);
            let messages_count = Arc::clone(&messages_count);
            std::thread::spawn(move || {
                let metadata = process_peaks(
                    sample_rate,
                    center_freq,
                    chunk,
                    peaks.clone(),
                );
                metadata.iter().for_each(|metadata| {
                    syncs_count.fetch_add(metadata.sync_count, Ordering::Relaxed);
                    messages_count.fetch_add(metadata.messages.len(), Ordering::Relaxed);
                })
            })
        }).collect::<Vec<_>>();

        handles.into_iter().for_each(|handle| {handle.join().unwrap()});

        assert_eq!(syncs_count.load(Ordering::Relaxed), 691);
        assert_eq!(messages_count.load(Ordering::Relaxed), 21);
    }
}

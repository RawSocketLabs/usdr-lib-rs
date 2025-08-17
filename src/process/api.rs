use std::thread;

use rust_dsdcc::{DSDDecoder, ffi::DSDDecodeMode};
use sdr::IQBlock;

use crate::process::{
    DmrProcessor, ProcessChannels, ProcessParams, ProcessedMetadata, freq_shift_num_complex,
    get_new_iq_block,
};

// scan for peaks
// save processed_iq
// demod processed_iq

pub fn process_peaks(mut process_channels: ProcessChannels, process_params: ProcessParams) {
    thread::spawn(move || {
        // TODO: Receiver process will spawn new tasks to process each peak with cloned IQ block vec
        let mut iq_blocks = Vec::<IQBlock>::with_capacity(4_000_000 / 4096);
        loop {
            //let _ = get_new_iq_block(&mut process_channels.iq_block_rx, &mut iq_blocks);
            while iq_blocks.len() < 4_000_000 / 4096 {
                let _ = get_new_iq_block(&mut process_channels.iq_block_rx, &mut iq_blocks);
            }
            let mut flat: IQBlock = iq_blocks.clone().into_iter().flatten().collect();
            flat.truncate(4_000_000);

            let peaks_result = process_channels.peaks_rx.try_recv();

            match peaks_result {
                Ok(scan_results) => {
                    if scan_results.peaks.len() > 0 {
                        // For some reason, we aren't getting enough IQ blocks when we get a new peak - temporary workaround.
                        eprintln!("Processing peaks");
                        let peaks = scan_results.peaks;
                        eprintln!("Received peaks: {}", peaks.len());

                        let center_freq = scan_results.center_freq;
                        for peak in peaks {
                            eprintln!(
                                "Processing peak at {} Hz. Center: {}",
                                peak.freq, center_freq
                            );
                            eprintln!("{}", (peak.freq as i32 - center_freq as i32));
                            let mut flat = flat.clone();
                            freq_shift_num_complex(
                                flat.as_mut_slice(),
                                2_000_000.0,
                                (peak.freq as i32 - center_freq as i32) as f32,
                            );
                            let mut processor = DmrProcessor::new(flat);
                            let _ = processor.run().unwrap();
                            let metadata = ProcessedMetadata {
                                peak,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                processed_samples: processor.get_processed_samples(),
                            };
                            eprintln!("{:?} Peak: {:?}", metadata.timestamp, metadata.peak);
                        }
                        iq_blocks.clear();
                        eprintln!("Cleared blocks");
                    } else {
                        iq_blocks.clear();
                        continue;
                    }
                }
                Err(_) => {
                    iq_blocks.clear();
                    continue;
                }
            }
        }
    });
}

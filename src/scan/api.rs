use std::time::Duration;

use smoothed_z_score::PeaksDetector;
use tokio::time::sleep;

use sdr::{FreqSample, detect_peaks};

use crate::scan::{ScanChannels, ScanParams, ScanResults};

pub fn scan(scan_channels: ScanChannels, scan_params: ScanParams) {
    tokio::spawn(async move { identify_peaks_in_range(scan_channels, scan_params).await });
}

async fn identify_peaks_in_range(mut channels: ScanChannels, params: ScanParams) {
    sleep(Duration::from_millis(1500)).await;

    loop {
        for center_freq in params.range.clone().step_by((params.rate / 4) as usize) {
            for channel in &channels.freq_tx {
                let _ = channel.try_send(center_freq);
                while let Ok(_) = channels.freq_block_rx.try_recv() {}
                channels.flow_tx.send(true).await.unwrap();
            }
            sleep(Duration::from_millis(100)).await;

            let mut count = 1.0;
            let mut average_freq_block = channels.freq_block_rx.recv().await.unwrap();

            loop {
                // Average blocks of frequency
                // take the first block from the ring_buffer
                // for each other block that has been given (up to 50) do the logic below
                let current = channels.freq_block_rx.recv().await.unwrap();
                let zipped: Vec<(FreqSample, FreqSample)> =
                    average_freq_block.into_iter().zip(current).collect();
                average_freq_block = zipped
                    .into_iter()
                    .map(|(mut avg, cur)| {
                        avg.db += (cur.db - avg.db) / count;
                        avg
                    })
                    .collect();
                count += 1.0;
                if count > 50.0 {
                    channels.flow_tx.send(false).await.unwrap();
                    break;
                }
            }

            // Get peaks
            let peaks = detect_peaks(
                average_freq_block,
                params.bandwidth,
                PeaksDetector::new(params.lag, 5.0, 0.5),
            );

            // Build Vec of IQ blocks while we wait to see if we get a peak detected, if peaks were not detected, send empty scan results and immediately move on
            // If peak(s) detected, wait for N seconds to allow processor to collect more IQ blocks, then send scan results

            let scan_results = ScanResults {
                center_freq: center_freq,
                peaks,
            };
            let _ = channels.result_tx.send(scan_results.clone()).await.unwrap();
            let _ = channels.tui_result_tx.send(scan_results).await.unwrap();
            sleep(params.sleep_time).await;
        }
    }
}

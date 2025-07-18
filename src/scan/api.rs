use std::time::Duration;

use smoothed_z_score::PeaksDetector;
use tokio::time::sleep;

use sdr::{detect_peaks, Sample};

use crate::scan::{ScanChannels, ScanParams, ScanResults};

pub fn scan(scan_channels: ScanChannels, scan_params: ScanParams) {
    tokio::spawn(async move { identify_peaks_in_range(scan_channels, scan_params).await });
}

async fn identify_peaks_in_range(mut channels: ScanChannels, params: ScanParams) {
    sleep(Duration::from_millis(1500)).await;

    loop {
        for i in params.range.clone().step_by((params.rate / 4) as usize) {
            for channel in &channels.freq_tx {
                let _ = channel.try_send(i);
                while let Ok(_) = channels.spectrum_rx.try_recv() {}
                channels.flow_tx.send(true).await.unwrap();
            }
            sleep(Duration::from_millis(100)).await;

            let mut count = 1.0;
            let mut average_spectrum = channels.spectrum_rx.recv().await.unwrap();

            loop {
                let current = channels.spectrum_rx.recv().await.unwrap();
                let zipped: Vec<(Sample, Sample)> =
                    average_spectrum.into_iter().zip(current).collect();
                average_spectrum = zipped
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
            // eprintln!("{} ({} - {}) || {:?}", i, i-1000000, i+1000000, if !average_spectrum.is_empty() {average_spectrum[0]} else {Sample::new(0, 0.0)});
            let peaks = detect_peaks(average_spectrum, PeaksDetector::new(params.lag, 5.0, 0.5));
            let scan_results = ScanResults {
                center_freq: i,
                peaks,
            };
            let _ = channels.result_tx.send(scan_results).await.unwrap();
            sleep(params.sleep_time).await;
        }
    }
}

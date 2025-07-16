use std::{ops::Range, time::Duration};

use sdr::{common::detect_peaks, types::{Sample, Spectrum}};
use smoothed_z_score::PeaksDetector;
use tokio::{sync::mpsc, time::sleep};

#[derive(Debug)]
pub struct ScanResults {
    pub center_freq: u32,
    pub peaks: Spectrum,
}

pub async fn scan_frequency_range(
    scan_freq_tx: mpsc::Sender<u32>,
    mut current_spectrum_rx: mpsc::Receiver<Spectrum>,
    range: Range<u32>,
    scan_res_tx: mpsc::Sender<ScanResults>,
    sleep_time: Duration,
    sample_rate: u32,
    lag: usize,
) {
    sleep(Duration::from_millis(3000)).await;

    loop {
        for i in range.clone().step_by((sample_rate / 4) as usize) {
            _ = scan_freq_tx.send(i);
            sleep(Duration::from_millis(100)).await;

            let mut count = 1.0;
            let mut average_spectrum = current_spectrum_rx.recv().await.unwrap();

            loop {
                let current = current_spectrum_rx.recv().await.unwrap();
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
                    while let Ok(_) = current_spectrum_rx.try_recv() {}
                    break;
                }
            }
            let peaks = detect_peaks(average_spectrum, PeaksDetector::new(lag, 3.0, 0.5));
            let scan_results = ScanResults {
                center_freq: i,
                peaks,
            };

            let _ = scan_res_tx.send(scan_results).await.unwrap();
            sleep(sleep_time).await;
        }
    }
}
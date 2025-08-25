mod tui;

use crate::tui::App;
use bincode::error::DecodeError;
use bincode::{Decode, Encode};
use comms::{ConnectionType, External};
use sdr::{FreqBlock, FreqSample};
use std::os::unix::net::UnixStream;
use tokio::select;
use tokio::sync::mpsc::channel;
use tokio::sync::watch::channel as watch_channel;

#[tokio::main]
async fn main() {
    let (current_freq_block_tx, current_freq_block_rx) = watch_channel(FreqBlock::new());
    let (peaks_tx, peaks_rx) = channel(1);
    let (center_freq_tx, center_freq_rx) = channel(1);

    std::thread::spawn(move || {
        let terminal = ratatui::init();
        let app = App::new(
            current_freq_block_rx,
            center_freq_rx,
            peaks_rx,
            2_000_000,
            445_500_000,
        );

        let _ = app.run(terminal);
        ratatui::restore();
    });

    let config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    let mut stream = UnixStream::connect("/tmp/sdrscanner").unwrap();
    bincode::encode_into_std_write(
        External::Connection(ConnectionType::Display),
        &mut stream,
        config,
    )
    .unwrap();

    loop {
        if let Ok(message) = bincode::decode_from_std_read(&mut stream, config) {
            match message {
                External::Realtime(freq_block) => {
                    current_freq_block_tx.send(freq_block).unwrap();
                },
                External::Peaks(peaks) => {
                    peaks_tx.send(peaks).await.unwrap();
                },
                External::Display(display_info) => {
                    center_freq_tx.send(display_info.center_freq as u32).await.unwrap();
                }
                _ => {}
            }
        }
    }
}

use sdr::FreqBlock;
use tokio::select;
use tokio::sync::mpsc::channel;
use tokio::sync::watch::channel as watch_channel;
use crate::tui::App;

mod tui;

#[tokio::main]
async fn main() {
    // Setup channels
    // realtime freq blocks
    let (current_freq_block_tx, current_freq_block_rx) = watch_channel(FreqBlock::new());
    // peaks
    let (peaks_tx, peaks_rx) = channel(512);
    // center freq
    let (center_freq_tx, center_freq_rx) = channel(512);

    // Start tui thread
    std::thread::spawn(move || {
        let terminal = ratatui::init();
        let app = App::new(
            current_freq_block_rx,
            center_freq_rx,
            peaks_rx,
            2_000_000,
            1_000_000,
        );

        let _ = app.run(terminal);
        ratatui::restore();
    });

    loop {

    }

    // // Handle data receipt from sdrscanner
    // loop {
    //     select!(
    //         // Read from socket/tcp, depending on message type, dispatch to appropriate channel
    //     )
    // }
}

// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

mod app;
mod event;
mod tui;
mod ui;
mod update;

use crate::app::App;
use crate::event::{Event, EventHandler};
use crate::tui::Tui;
use crate::update::{handle_key_event, receive_new_data};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use shared::{ConnectionType, DmrMetadata, External, FreqBlock};
use std::collections::BTreeMap;
use std::os::unix::net::UnixStream;
use tokio::sync::mpsc::channel;
use tokio::sync::watch::channel as watch_channel;

#[tokio::main]
async fn main() {
    let (current_freq_block_tx, current_freq_block_rx) = watch_channel(FreqBlock::new());
    let (peaks_tx, peaks_rx) = channel(1);
    let (center_freq_tx, center_freq_rx) = channel(1);
    let (metadata_tx, metadata_rx) = channel::<BTreeMap<u32, DmrMetadata>>(1);
    let (squelch_tx, mut squelch_rx) = channel::<f32>(1);

    std::thread::spawn(move || {
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal = Terminal::new(backend).unwrap();
        let events = EventHandler::new(30);
        let mut tui = Tui::new(terminal, events);
        tui.enter().unwrap();

        let mut app = App::new(
            current_freq_block_rx,
            center_freq_rx,
            peaks_rx,
            metadata_rx,
            squelch_tx,
            2_000_000,
            445_500_000,
        );

        while !app.should_quit {
            let _ = tui.draw(&mut app);

            match tui.events.next().unwrap() {
                Event::Tick => receive_new_data(&mut app),
                Event::Key(key_event) => handle_key_event(&mut app, key_event),
            };
        }
        tui.exit().unwrap();
    });

    let config = bincode::config::standard()
        .with_big_endian()
        .with_fixed_int_encoding();
    // eprintln!("TUI connecting to scanner...");
    let mut stream = UnixStream::connect("/tmp/sdrscanner").unwrap();
    // eprintln!("TUI connected to scanner");
    // eprintln!("TUI sending connection message...");
    let serialized = bincode::encode_to_vec(
        External::Connection(ConnectionType::Display),
        config,
    ).unwrap();
    
    // Write length prefix
    use std::io::Write;
    stream.write_all(&(serialized.len() as u32).to_be_bytes()).unwrap();
    // Write data
    stream.write_all(&serialized).unwrap();
    stream.flush().unwrap();
    // eprintln!("TUI sent connection message, starting message loop...");

    loop {
        // Read length prefix
        use std::io::Read;
        let mut len_buf = [0u8; 4];
        if let Some(squelch) = squelch_rx.try_recv().ok() {
            let serialized = bincode::encode_to_vec(External::Squelch(squelch), config).unwrap();
            stream.write_all(&(serialized.len() as u32).to_be_bytes()).unwrap();
            stream.write_all(&serialized).unwrap();
            stream.flush().unwrap();
        }
        if stream.read_exact(&mut len_buf).is_ok() {
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut buffer = vec![0u8; len];
            if stream.read_exact(&mut buffer).is_ok() {
                if let Ok((message, _)) = bincode::decode_from_slice(&buffer, config) {
                    // eprintln!("TUI received message: {:?}", message);
                    match message {
                        External::Realtime(freq_block) => {
                            // eprintln!("TUI sending realtime data to display");
                            current_freq_block_tx.send(freq_block).unwrap();
                        }
                        External::Peaks(peaks) => {
                            peaks_tx.try_send(peaks).unwrap_or_else(|_| {
                                // eprintln!("Failed to send peaks to TUI - channel full");
                            });
                        }
                        External::Display(display_info) => {
                            center_freq_tx.try_send(display_info).unwrap_or_else(|_| {
                                // eprintln!("Failed to send display info to TUI - channel full");
                            });
                        }
                        External::Metadata(metadata) => {
                            metadata_tx.try_send(metadata).unwrap_or_else(|_| {
                                // eprintln!("Failed to send metadata to TUI - channel full");
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

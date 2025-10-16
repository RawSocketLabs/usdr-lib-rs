// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn handle_key_event(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        }
       KeyCode::Up => {
            app.y_bounds[0] += if app.y_bounds[0] + 10.0 < app.y_bounds[1] {5.0} else {0.0};
        }
       KeyCode::Down => {
            app.y_bounds[0] -= if app.y_bounds[0] - 5.0 > -200.0 {5.0} else {0.0};
        }
       KeyCode::Right => {
            app.y_bounds[1] += if app.y_bounds[1] + 5.0 < 200.0 {5.0} else {0.0};
        }
       KeyCode::Left => {
            app.y_bounds[1] -= if app.y_bounds[1] - 10.0 > app.y_bounds[0] {5.0} else {0.0};
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.scroll_table_down();
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            app.scroll_table_up();
        }
        KeyCode::Char('a') => {
            app.squelch_tx.try_send((app.squelch - 1.0).max(-100.0)).unwrap();
        }
        KeyCode::Char('d')=> {
            app.squelch_tx.try_send((app.squelch + 1.0).min(100.0)).unwrap();
        }
        KeyCode::Char('A') => {
            app.squelch_tx.try_send((app.squelch - 10.0).max(-100.0)).unwrap();
        }
        KeyCode::Char('D')=> {
            app.squelch_tx.try_send((app.squelch + 10.0).min(100.0)).unwrap();
        }
        _ => {}
    };
}

pub fn receive_new_data(app: &mut App) {
    let new_freq_block = app.current_freq_block_rx.borrow().to_vec();
    if new_freq_block.len() != app.current_freq_block.len() || !new_freq_block.is_empty() {
        // eprintln!("TUI received new freq_block with {} samples", new_freq_block.len());
        app.current_freq_block = new_freq_block.into();
    }

    if let Ok(display_info) = app.display_info_rx.try_recv() {
        // eprintln!("TUI received display info: center_freq={}, rate={}", display_info.center_freq, display_info.rate);
        app.frequency = *display_info.center_freq;
        app.sample_rate = display_info.rate as u32;
        app.squelch = display_info.squelch;
    }

    if let Ok(results) = app.peaks_rx.try_recv() {
        app.current_peaks = Some(results);
    }

    if let Ok(metadata) = app.metadata_rx.try_recv() {
        app.current_metadata = metadata;
    }
    let center_frequency = app.frequency as f64 / 1e6;
    let half_span_mhz = app.sample_rate as f64 / 2.0 / 1e6;
    app.x_bounds = [
        center_frequency - half_span_mhz,
        center_frequency + half_span_mhz,
    ];
}
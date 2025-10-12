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
        _ => {}
    };
}

pub fn receive_new_data(app: &mut App) {
    app.current_freq_block = app.current_freq_block_rx.borrow().to_vec();

    if let Ok(display_info) = app.display_info_rx.try_recv() {
        app.frequency = display_info.center_freq as u32;
        app.sample_rate = display_info.rate as u32;
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
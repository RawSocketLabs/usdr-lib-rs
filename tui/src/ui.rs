use crate::app::App;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Cell, Chart, Dataset, GraphType, Row, Table};
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use shared::Message;
use std::collections::BTreeSet;

pub fn render_fft_chart(app: &mut App, frame: &mut Frame, area: Rect) {
    let freq_block_vec = app
        .current_freq_block
        .iter()
        .map(|f| (f.freq as f64 / 1e6, f.db as f64))
        .collect::<Vec<(f64, f64)>>();

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&freq_block_vec);

    let mut datasets = vec![dataset];

    let peaks_vec = match app.current_peaks.as_ref() {
        Some(peaks) => {
            let mut result = Vec::new();
            for peak in peaks {
                for i in (peak.sample.db as i32)..=app.y_bounds[1] as i32 {
                    result.push((peak.sample.freq as f64 / 1e6, i as f64));
                }
            }
            result
        }
        None => Vec::new(),
    };

    if !peaks_vec.is_empty() {
        let peaks_dataset = Dataset::default()
            .name("Peaks")
            .marker(Marker::Braille)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Red))
            .data(&peaks_vec);
        datasets.push(peaks_dataset);
    }

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("FFT Display (Press q to exit)")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Frequency (MHz)")
                .style(Style::default().fg(Color::Gray))
                .bounds(app.x_bounds),
        )
        .y_axis(
            Axis::default()
                .title("Magnitude (dB)")
                .style(Style::default().fg(Color::Gray))
                .bounds(app.y_bounds)
                .labels({
                    let y_min = app.y_bounds[0].floor() as i32;
                    let y_max = app.y_bounds[1].ceil() as i32;
                    let start = (y_min / 10) * 10;
                    let end = ((y_max + 9) / 10) * 10;
                    let mut lab = Vec::new();
                    for val in (start..=end).step_by(10) {
                        lab.push(Span::raw(format!("{}", val)));
                    }
                    lab
                }),
        );
    frame.render_widget(chart, area);

    // TODO: Should this be its own function?
    let x_min = app.x_bounds[0];
    let x_max = app.x_bounds[1];
    let width = area.width as usize;
    let mut spans = Vec::with_capacity(width);

    for _col in 0..width {
        spans.push(Span::raw(" "));
    }

    for i in 0..=10 {
        let frac = i as f64 / 10.0;
        let value = x_min + frac * (x_max - x_min);
        let label = format!("{:.2}", value);
        let col = (width.saturating_sub(label.len()) as f64 * frac).round() as usize;
        for (j, ch) in label.chars().enumerate() {
            if col + j < spans.len() {
                spans[col + j] = Span::raw(ch.to_string());
            }
        }
    }

    let line = Line::from(spans);
    let label_row = Rect {
        x: area.x + 1,
        y: area.y + area.height - 1,
        width: area.width - 2,
        height: 1,
    };

    frame.render_widget(
        Paragraph::new(vec![line]).style(Style::default().fg(Color::Gray)),
        label_row,
    );
}

pub fn render_metadata_table(app: &mut App, frame: &mut Frame, area: Rect) {
    let header = [
        "Freq",
        "Color Codes",
        "Slot Data Types",
        "FIDS",
        "Talkgroups",
        "Sources",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .height(1);

    let rows = app
        .current_metadata
        .iter()
        .map(|(freq, metadata)| {
            Row::new([
                Cell::from(format!("{}", freq)),
                Cell::from(format!(
                    "{:?}",
                    BTreeSet::from_iter(metadata.color_codes.iter())
                )),
                Cell::from(format!(
                    "{:?}",
                    BTreeSet::from_iter(metadata.slot_data_types.iter())
                )),
                Cell::from(format!(
                    "{:?}",
                    BTreeSet::from_iter(metadata.messages.iter().filter_map(|message| {
                        match message {
                            Message::GroupVoice(m) => Some(format!("{:?}", m.fid)),
                            Message::CSBK(m) => Some(format!("{:?}", m.fid)),
                            _ => None,
                        }
                    }))
                )),
                Cell::from(format!(
                    "{:?}",
                    BTreeSet::from_iter(metadata.messages.iter().filter_map(|message| {
                        match message {
                            Message::GroupVoice(m) => Some(format!("{:?}", m.group)),
                            _ => None,
                        }
                    }))
                )),
                Cell::from(format!(
                    "{:?}",
                    BTreeSet::from_iter(metadata.messages.iter().filter_map(|message| {
                        match message {
                            Message::GroupVoice(m) => Some(format!("{:?}", m.source)),
                            _ => None,
                        }
                    }))
                )),
            ])
        })
        .collect::<Vec<Row>>();

    let table = Table::new(rows, [10, 20, 100, 50, 50, 50])
        .header(header)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(table, area);
}


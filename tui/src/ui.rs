// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::symbols::Marker;
use ratatui::widgets::{Axis, Cell, Chart, Dataset, GraphType, Row, Table, TableState, Scrollbar, ScrollbarState, ScrollbarOrientation};
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use shared::Message;
use std::collections::BTreeSet;
use chrono::{DateTime, Utc};
use ratatui::text::Text;

const INFO_TEXT: &str = "(Esc) or (q) quit | (↑ ↓ ← →) adjust FFT display | (w) scroll up | (s) scroll down | (a/A) decrease squelch | (d/D) increase squelch";
const DARK_DARK_GRAY: Color = Color::Rgb(10, 10, 10);
const ROW_BACKGROUND_1: Color = Color::Rgb(30, 30, 30);
const ROW_BACKGROUND_2: Color = Color::Rgb(20, 20, 20);
const HEADERS: [&str; 7] = [
    "Last Seen (UTC)",
    "Freq",
    "Color Codes",
    "Slot Data Types",
    "FIDS",
    "Talkgroups",
    "Sources",
];

const HEADER_LENGTHS: [u16; 7] = [
    HEADERS[0].len() as u16,
    HEADERS[1].len() as u16,
    HEADERS[2].len() as u16,
    HEADERS[3].len() as u16,
    HEADERS[4].len() as u16,
    HEADERS[5].len() as u16,
    HEADERS[6].len() as u16,
];

pub fn render_fft_chart(app: &mut App, frame: &mut Frame, area: Rect) {
    let background = Block::default().style(Style::default().bg(DARK_DARK_GRAY));
    frame.render_widget(background, area);

    let freq_block_vec = app
        .current_freq_block
        .iter()
        .map(|f| (f.freq.as_f64() / 1e6, f.db as f64))
        .collect::<Vec<(f64, f64)>>();

    let dataset = Dataset::default()
        .name("FFT")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::LightMagenta))
        .data(&freq_block_vec);

    let mut datasets = vec![dataset];

    let squelch_vec = match app.current_freq_block.len() {
        0 => &[(0., app.squelch as f64), (0., app.squelch as f64)],
        len => &[(*app.current_freq_block[0].freq as f64 / 1e6, app.squelch as f64), (*app.current_freq_block[len - 1].freq as f64 / 1e6, app.squelch as f64)]
    };

    let squelch_line = Dataset::default()
        .name("Squelch")
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Yellow))
        .data(squelch_vec);

    datasets.push(squelch_line);

    let peaks_vec = match app.current_peaks.as_ref() {
        Some(peaks) => {
            let mut result = Vec::new();
            for peak in peaks {
                for i in (peak.sample.db as i32)..=app.y_bounds[1] as i32 {
                    result.push((peak.sample.freq.as_f64() / 1e6, i as f64));
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
                .title("FFT Display")
                .borders(Borders::ALL)
                .title_style(Style::default().fg(Color::Yellow)),
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
        )
        .style(Style::default().bg(DARK_DARK_GRAY));
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
    let background = Block::default().style(Style::default().bg(DARK_DARK_GRAY));
    frame.render_widget(background, area);


    let header = HEADERS.into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .height(1)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .bg(DARK_DARK_GRAY),
        );

    let table_scroll_state = app.table_scroll_state;
    let metadata_len = app.current_metadata.len();

    let widths = calculate_column_widths(app, area);
    let rows = generate_table_rows(app);

    let table = Table::new(
        rows,
        widths
            .iter()
            .map(|width| ratatui::layout::Constraint::Length(*width))
            .collect::<Vec<_>>(),
    )
        .header(header)
        .block(Block::default().borders(Borders::ALL));

    let mut table_state = TableState::default().with_selected(Some(table_scroll_state));
    frame.render_stateful_widget(table, area, &mut table_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(metadata_len)
        .position(table_scroll_state);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin { horizontal: 0, vertical: 1 }),
        &mut scrollbar_state,
    );
}

pub fn render_footer(frame: &mut Frame, area: Rect) {
    let info_footer = Paragraph::new(Text::from(INFO_TEXT))
        .style(
            Style::new()
                .bg(DARK_DARK_GRAY),
        )
        .centered()
        .block(
            Block::bordered()
                .border_style(Style::new().fg(Color::Gray)),
        );
    frame.render_widget(info_footer, area);
}

fn generate_table_rows(app: &'_ mut App) -> Vec<Row<'_>> {
    let rows = app
        .current_metadata
        .iter()
        .enumerate()
        .map(|(index, (freq, metadata))| {
            let fids: Vec<String> = metadata
                .messages
                .iter()
                .filter_map(|message| match message {
                    Message::GroupVoice(m) => Some(format!("{:?}", m.fid)),
                    Message::CSBK(m) => Some(format!("{:?}", m.fid)),
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            let talkgroups: Vec<String> = metadata
                .messages
                .iter()
                .filter_map(|message| match message {
                    Message::GroupVoice(m) => Some(format!("{:?}", m.group)),
                    _ => None,
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            let sources: Vec<String> = metadata
                .messages
                .iter()
                .filter_map(|message| match message {
                    Message::GroupVoice(m) => Some(format!("{:?}", m.source)),
                    _ => None,
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            let max_items = [
                metadata.color_codes.len(),
                metadata.slot_data_types.len(),
                fids.len(),
                talkgroups.len(),
                sources.len(),
            ]
                .into_iter()
                .max()
                .unwrap_or(1)
                .max(1);

            let row_style = if index % 2 == 0 {
                Style::default().bg(ROW_BACKGROUND_1)
            } else {
                Style::default().bg(ROW_BACKGROUND_2)
            };

            let dtg: DateTime<Utc> = metadata.observation_time.into();

            Row::new([
                Cell::from(format!("\n{}", dtg.format("%Y-%m-%d %H:%M:%S").to_string())),
                Cell::from(format!("\n{:.03} MHz\n", *freq as f32 / 1e6)),
                Cell::from(
                    format!("\n{}", metadata
                        .color_codes
                        .iter()
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .map(|cc| format!("{:?}", cc))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    )),
                Cell::from(
                    format!("\n{}", metadata
                        .slot_data_types
                        .iter()
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .map(|sdt| format!("{:?}", sdt))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    )),
                Cell::from(format!("\n{}", fids.join("\n"))),
                Cell::from(format!("\n{}", talkgroups.join("\n"))),
                Cell::from(format!("\n{}", sources.join("\n"))),
            ])
                .height(max_items as u16 + 2)
                .style(row_style)
        })
        .collect::<Vec<Row>>();
    rows
}

fn calculate_column_widths(app: &mut App, area: Rect) -> Vec<u16> {
    let mut max_widths = [0u16; 7];

    for (i, width) in HEADER_LENGTHS.iter().enumerate() {
        max_widths[i] = *width as u16;
    }

    for (freq, metadata) in app.current_metadata.iter() {
        let last_seen_dt: DateTime<Utc> = metadata.observation_time.into();
        let last_seen_length = last_seen_dt.format("%Y-%m-%d %H:%M:%S").to_string().len();
        max_widths[0] = max_widths[0].max(last_seen_length as u16);

        let freq_width = format!("{:.03} MHz", *freq as f32 / 1e6).len() as u16;
        max_widths[1] = max_widths[1].max(freq_width);


        let cc_width = metadata
            .color_codes
            .iter()
            .map(|cc| format!("{:?}", cc).len())
            .max()
            .unwrap_or(0) as u16;
        max_widths[2] = max_widths[2].max(cc_width);

        let sdt_width = metadata
            .slot_data_types
            .iter()
            .map(|sdt| format!("{:?}", sdt).len())
            .max()
            .unwrap_or(0) as u16;
        max_widths[3] = max_widths[3].max(sdt_width);

        let fids_width = metadata
            .messages
            .iter()
            .filter_map(|message| match message {
                Message::GroupVoice(m) => Some(format!("{:?}", m.fid).len()),
                Message::CSBK(m) => Some(format!("{:?}", m.fid).len()),
            })
            .max()
            .unwrap_or(0) as u16;
        max_widths[4] = max_widths[4].max(fids_width);

        let tg_width = metadata
            .messages
            .iter()
            .filter_map(|message| match message {
                Message::GroupVoice(m) => Some(format!("{:?}", m.group).len()),
                _ => None,
            })
            .max()
            .unwrap_or(0) as u16;
        max_widths[5] = max_widths[5].max(tg_width);

        let src_width = metadata
            .messages
            .iter()
            .filter_map(|message| match message {
                Message::GroupVoice(m) => Some(format!("{:?}", m.source).len()),
                _ => None,
            })
            .max()
            .unwrap_or(0) as u16;
        max_widths[6] = max_widths[6].max(src_width);
    }

    for width in max_widths.iter_mut() {
        *width = (*width + 2).max(10);
    }

    let total_width: u16 = max_widths.iter().sum();
    let available_width = area.width.saturating_sub(3); // Account for borders and scrollbar

    let widths = if total_width > available_width {
        let scale = available_width as f32 / total_width as f32;
        max_widths
            .iter()
            .map(|w| ((*w as f32 * scale) as u16).max(8))
            .collect::<Vec<_>>()
    } else {
        max_widths.to_vec()
    };
    widths
}

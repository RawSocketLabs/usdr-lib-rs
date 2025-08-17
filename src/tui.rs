use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols,
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};
use sdr::FreqBlock;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

use crate::scan::ScanResults;

pub struct App {
    current_freq_block_rx: watch::Receiver<FreqBlock>,
    frequency_rx: mpsc::Receiver<u32>,
    sample_rate: u32,
    scan_rx: mpsc::Receiver<ScanResults>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    current_freq_block: FreqBlock,
    should_quit: bool,
    frequency: u32,
    latest_scan_results: Option<ScanResults>,
}

impl App {
    fn render_fft_chart(&self, frame: &mut Frame<'_>, area: Rect) {
        let freq_block_vec = self
            .current_freq_block
            .iter()
            .map(|f| ((f.freq as f64 / 1e6), f.db as f64))
            .collect::<Vec<(f64, f64)>>();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&freq_block_vec);

        let mut datasets = vec![dataset];

        let peaks_vec = match self.latest_scan_results.as_ref() {
            Some(v) => v
                .peaks
                .iter()
                .map(|sample| ((sample.freq as f64 / 1e6), sample.db as f64))
                .collect::<Vec<(f64, f64)>>(),
            None => Vec::new(),
        };

        if !peaks_vec.is_empty() {
            let peaks_dataset = Dataset::default()
                .name("Peaks")
                .marker(Marker::Braille)
                .graph_type(GraphType::Bar)
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
                    .bounds(self.x_bounds)
                    .labels({
                        let mut labels = Vec::new();
                        labels.push(Span::raw(""));
                        labels
                    }),
            )
            .y_axis(
                Axis::default()
                    .title("Magnitude (dB)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.y_bounds)
                    .labels({
                        let y_min = self.y_bounds[0].floor() as i32;
                        let y_max = self.y_bounds[1].ceil() as i32;
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
    }

    pub fn new(
        current_freq_block_rx: watch::Receiver<FreqBlock>,
        frequency_rx: mpsc::Receiver<u32>,
        scan_rx: mpsc::Receiver<ScanResults>,
        sample_rate: u32,
        start_freq: u32,
    ) -> Self {
        let frequency = start_freq;
        let half_span_mhz = (sample_rate / 2) / 1e6 as u32;
        let center_mhz = frequency / 1e6 as u32;
        Self {
            current_freq_block_rx,
            frequency_rx,
            sample_rate,
            scan_rx,
            frequency,
            x_bounds: [
                center_mhz as f64 - half_span_mhz as f64,
                center_mhz as f64 + half_span_mhz as f64,
            ],
            y_bounds: [-60.0, 0.0],
            current_freq_block: Vec::new(),
            should_quit: false,
            latest_scan_results: None,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(30);
        let mut last_tick = Instant::now();
        while !self.should_quit {
            self.current_freq_block = self.current_freq_block_rx.borrow().to_vec();

            while let Ok(f) = self.frequency_rx.try_recv() {
                self.frequency = f;
            }

            while let Ok(results) = self.scan_rx.try_recv() {
                self.latest_scan_results = Some(results);
            }

            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key);
                }
            }

            terminal.draw(|frame| {
                self.draw(frame);
            })?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            thread::sleep(timeout);
            last_tick = Instant::now();
        }
        terminal.clear()?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Up => {
                // Shift y-axis range up by 1 dB
                self.y_bounds[0] += 5.0;
            }
            KeyCode::Down => {
                // Shift y-axis range down by 1 dB
                self.y_bounds[0] -= 5.0;
            }
            KeyCode::Right => {
                self.y_bounds[1] += 5.0;
            }
            KeyCode::Left => {
                self.y_bounds[1] -= 5.0;
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let center_frequency = self.frequency as f64 / 1e6;
        let half_span_mhz = self.sample_rate as f64 / 2.0 / 1e6;
        self.x_bounds = [
            center_frequency - half_span_mhz,
            center_frequency + half_span_mhz,
        ];

        let areas = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(frame.area());
        let fft_area = areas[0];

        self.render_fft_chart(frame, fft_area);

        {
            let x_min = self.x_bounds[0];
            let x_max = self.x_bounds[1];
            let width = fft_area.width as usize;
            let mut spans = Vec::with_capacity(width);
            for _col in 0..width {
                spans.push(Span::raw(" "));
            }
            for i in 0..=10 {
                let frac = i as f64 / 10.0;
                let value = x_min + frac * (x_max - x_min);
                let label = format!("{:.2}", value);
                let col = ((width.saturating_sub(label.len())) as f64 * frac).round() as usize;
                for (j, ch) in label.chars().enumerate() {
                    if col + j < spans.len() {
                        spans[col + j] = Span::raw(ch.to_string());
                    }
                }
            }
            let line = Line::from(spans);
            let label_row = Rect {
                x: fft_area.x + 1,
                y: fft_area.y + fft_area.height - 1,
                width: fft_area.width - 2,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(vec![line]).style(Style::default().fg(Color::Gray)),
                label_row,
            );
        }

        // Render latest scan results in the bottom area
        let results_area = areas[1];
        // Prepare lines from the latest ScanResults
        let mut result_lines = Vec::new();
        if let Some(ref scan) = self.latest_scan_results {
            // Show center frequency
            result_lines.push(Line::from(vec![Span::raw(format!(
                "Center Frequency: {:.3} MHz",
                scan.center_freq as f64 / 1000000f64
            ))]));
            // Show peaks
            if scan.peaks.is_empty() {
                result_lines.push(Line::from(vec![Span::raw("No peaks detected")]));
            } else {
                result_lines.push(Line::from(vec![Span::raw("Peaks:")]));
                for &sample in &scan.peaks {
                    result_lines.push(Line::from(vec![Span::raw(format!(
                        "  {:.3} MHz: {:.2} dB",
                        sample.freq as f32 / 1e6,
                        sample.db
                    ))]));
                }
            }
        } else {
            result_lines.push(Line::from(vec![Span::raw("No scan results yet")]));
        }
        let results_block = Paragraph::new(result_lines)
            .block(Block::default().title("Scan Results").borders(Borders::ALL));
        frame.render_widget(results_block, results_area);
    }
}

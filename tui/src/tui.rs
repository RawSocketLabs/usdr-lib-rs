// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use std::{io, panic};
use std::io::Error;
use ratatui::crossterm::{execute, terminal};
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Constraint, Layout, Rect};
use crate::app::App;
use crate::event::EventHandler;
use crate::ui;

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stderr>>;

pub struct Tui {
    terminal: CrosstermTerminal,
    pub events: EventHandler,

}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self) -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }


    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self, app: &mut App) -> Result<(), Error> {
        self.terminal.draw(|frame| {
            let areas = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(frame.area());
            let fft_area = areas[0];
            let metadata_area = areas[1];
            let footer_area = Rect::new(0, metadata_area.y + metadata_area.height - 3, frame.area().width, 4);

            ui::render_fft_chart(app, frame, fft_area);
            ui::render_metadata_table(app, frame, metadata_area);
            ui::render_footer(frame, footer_area);
        })?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<(), Error> {
        terminal::disable_raw_mode()?;
        execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<(), Error> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
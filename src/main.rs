mod app;
mod cpu;
mod disassemble;
mod ui;

use anyhow::Result;

use crossterm::event::{
    self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::prelude::*;

use std::io;
use std::time::Duration;

use crate::app::App;
use crate::ui::ui;

fn main() -> Result<()> {
    // Begin by setting up the terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Obtain a rendering handle for the terminal.
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Construct and run app.
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    // Return the terminal to its normal operating state.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // We don't want to propagate a potential error from running the error earlier, or we might not
    // properly return the terminal to its normal operating state.
    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if poll(Duration::from_nanos(1))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(char) => {
                            // HACK: This is a super quick and dirty way to exit the application.
                            if char == 'c' && key.modifiers == KeyModifiers::CONTROL {
                                return Ok(());
                            }
                            app.command_buffer.push(char);
                        }
                        KeyCode::Backspace => {
                            app.command_buffer.pop();
                        }
                        KeyCode::Enter => {
                            app.execute_command();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.running {
            app.step();
        }
    }
}

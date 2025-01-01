use std::{error::Error, io};

use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;
use crate::app::App;
use crate::ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let _ = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Ok(true) = handle_events(app) {
            return Ok(true);
        }
    }
}

fn handle_events(app: &mut App) -> io::Result<bool> {
    match event::read()? {
        Event::Key(key) => {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => app.on_click(mouse.column, mouse.row),
            _ => {}
        },
        _ => {}
    }
    return Ok(false);
}

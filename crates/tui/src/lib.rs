use crossterm::{
    event::{self},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{io, time::Duration};
use std::sync::mpsc::Receiver;
use scribe_parser::ScribeParser;

pub mod app;
pub mod render;

use render::ui;
use render::input;

pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        event::DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: app::App,
    rx: Receiver<String>,
    parser: &mut ScribeParser,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, &mut app))?;
        app.tick();

        if event::poll(Duration::from_millis(16))? {
            let ev = event::read()?;
            input::handle_event(terminal, &mut app, ev);
        }

        if app.paused {
            continue;
        }

        while let Ok(raw_json) = rx.try_recv() {
            if app.verbose
                && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw_json)
                    && let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                        app.push_log(pretty);
                    }
            let mut events = parser.parse_packet(&raw_json);
            for event in &events {
                app.handle_event(event.clone());
            }
            events.clear();
        }
    }
    Ok(())
}

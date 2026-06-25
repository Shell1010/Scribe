use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal, Frame,
};
use std::{io, time::Duration};

// We will bring in the App struct we discussed earlier
use crate::app::App; 

pub mod app;

/// Hijacks the terminal for our UI
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restores the terminal to normal when we quit
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
// In lib.rs

fn ui(f: &mut Frame, _app: &App) {
    // Create a block with borders
    let block = Block::default()
        .title(" Scribe: MMO Packet Sniffer ")
        .borders(Borders::ALL);

    // Create a paragraph of text inside the block
    let p = Paragraph::new("Listening for packets on port 5594...\nPress 'q' to exit.")
        .block(block);

    // Render it to the entire terminal screen
    f.render_widget(p, f.size());
}


use std::sync::mpsc::Receiver;
use scribe_parser::ScribeParser; 

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
    rx: Receiver<String>,
    parser: &mut ScribeParser,
) -> io::Result<()> {
    
    while !app.should_quit {
        // 1. Draw the UI based on the current state
        terminal.draw(|f| ui(f, &app))?;

        // 2. Poll for keyboard events (timeout after 16ms to maintain ~60 FPS)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    _ => {}
                }
            }
        }

        // 3. Process ALL pending network packets that arrived in the last 16ms
        while let Ok(raw_json) = rx.try_recv() {
            // Parse the JSON into your ScribeEvent enum
            let events = parser.parse_packet(&raw_json);
            
            for event in events {
                // Pass the event to your app state to update the spreadsheet
                app.handle_event(event);
            }
        }
    }
    
    Ok(())
}
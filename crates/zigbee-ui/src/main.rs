mod app;
mod ui;
mod event;

use app::App;
use event::{Event, EventHandler};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;
    
    // Create event handler
    let event_handler = EventHandler::new(Duration::from_millis(100));
    
    // Run app
    let res = run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        match event_handler.next().await? {
            Event::Tick => {
                app.on_tick().await?;
            }
            Event::Key(key_event) => {
                if !app.on_key(key_event).await? {
                    return Ok(());
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }
}
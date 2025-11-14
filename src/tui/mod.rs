mod app;
mod events;
mod ui;
mod widgets;

pub use app::App;
pub use events::EventHandler;

use crate::Result;
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

pub async fn run(api_url: String) -> Result<()> {
    enable_raw_mode().map_err(|e| crate::Error::Io(e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| crate::Error::Io(e))?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::Error::Io(e))?;
    
    let mut app = App::new(api_url).await?;
    let mut event_handler = EventHandler::new();
    
    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;
    
    disable_raw_mode().map_err(|e| crate::Error::Io(e))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).map_err(|e| crate::Error::Io(e))?;
    terminal.show_cursor().map_err(|e| crate::Error::Io(e))?;
    
    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))
            .map_err(|e| crate::Error::Io(e))?;
        
        if event_handler.handle_events(app).await? {
            break;
        }
        
        app.update().await?;
    }
    
    Ok(())
}
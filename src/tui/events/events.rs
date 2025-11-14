use crate::tui::app::{App, Tab, LogLevel};
use crate::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn handle_events(&mut self, app: &mut App) -> Result<bool> {
        if event::poll(Duration::from_millis(100))
            .map_err(|e| crate::Error::Io(e))? 
        {
            match event::read().map_err(|e| crate::Error::Io(e))? {
                Event::Key(key) => {
                    return Ok(self.handle_key_event(key, app));
                }
                Event::Resize(_, _) => {
                }
                _ => {}
            }
        }
        
        Ok(false)
    }
    
    fn handle_key_event(&self, key: KeyEvent, app: &mut App) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.quit();
                return true;
            }
            
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.quit();
                return true;
            }
            
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_chain();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_chain();
            }
            
            KeyCode::Tab => {
                app.next_tab();
            }
            KeyCode::Char('1') => {
                app.current_tab = Tab::Dashboard;
            }
            KeyCode::Char('2') => {
                app.current_tab = Tab::ChainDetails;
            }
            KeyCode::Char('3') => {
                app.current_tab = Tab::Logs;
            }
            
            KeyCode::Char('r') | KeyCode::F(5) => {
                app.add_log(
                    LogLevel::Info,
                    "Manual refresh triggered"
                );
                app.last_update = std::time::Instant::now() 
                    - std::time::Duration::from_secs(10);
            }
            
            _ => {}
        }
        
        false
    }
}
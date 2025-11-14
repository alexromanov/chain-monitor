pub mod chains;
pub mod api;
pub mod error;
pub mod tui;

pub use error::{Error, Result};
pub use api::{AppState, start_server};
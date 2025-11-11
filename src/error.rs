use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Chain error: {0}")]
    Chain(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Messaging error: {0}")]
    Messaging(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
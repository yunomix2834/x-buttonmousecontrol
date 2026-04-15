use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Port error: {0}")]
    Port(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Thread error: {0}")]
    Thread(String),
}
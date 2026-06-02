use thiserror::Error;

#[derive(Debug, Error)]
pub enum HyperspaceError {
    #[error("dimension not found: {0}")]
    DimensionNotFound(String),
    #[error("object not found: {0}")]
    ObjectNotFound(String),
    #[error("filesystem error: {0}")]
    Filesystem(String),
    #[error("ai runtime error: {0}")]
    AiRuntime(String),
}

pub type Result<T> = std::result::Result<T, HyperspaceError>;

#[derive(Debug, thiserror::Error)]
pub enum DotmanError {
    #[error("Source file not found: {0}")]
    SourceFileNotFound(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Command '{command}' failed: {message}")]
    CommandError { command: String, message: String },
    #[error("Path error: {0}")]
    PathError(String),
}

impl From<String> for DotmanError {
    fn from(err_msg: String) -> Self {
        DotmanError::PathError(err_msg)
    }
}

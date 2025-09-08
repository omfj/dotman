#[derive(Debug, thiserror::Error)]
pub enum DotmanError {
    #[error("Source file not found: {0}")]
    SourceFileNotFound(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Command '{command}' failed: {message}")]
    CommandError { command: String, message: String },
    #[error("General error: {0}")]
    GeneralError(#[from] anyhow::Error),
}

impl DotmanError {
    pub fn message(&self) -> String {
        self.to_string()
    }
}

impl From<String> for DotmanError {
    fn from(err_msg: String) -> Self {
        DotmanError::IoError(std::io::Error::other(err_msg))
    }
}

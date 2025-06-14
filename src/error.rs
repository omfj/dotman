pub enum DotmanError {
    SourceFileNotFound(String),
    IoError(std::io::Error),
}

impl DotmanError {
    pub fn message(&self) -> String {
        match self {
            DotmanError::SourceFileNotFound(source) => {
                format!("Source file not found: {}", source)
            }
            DotmanError::IoError(err) => format!("I/O error: {}", err),
        }
    }
}

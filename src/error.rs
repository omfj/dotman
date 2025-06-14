#[derive(Debug)]
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

impl From<std::io::Error> for DotmanError {
    fn from(err: std::io::Error) -> Self {
        DotmanError::IoError(err)
    }
}

impl From<String> for DotmanError {
    fn from(err_msg: String) -> Self {
        DotmanError::IoError(std::io::Error::other(err_msg))
    }
}

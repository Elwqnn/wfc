use std::fmt;

/// Errors that can occur during WFC operations.
#[derive(Debug, Clone)]
pub enum Error {
    /// Failed to load an image file.
    ImageLoad(String),
    /// Failed to save an image file.
    ImageSave(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ImageLoad(msg) => write!(f, "image load error: {}", msg),
            Error::ImageSave(msg) => write!(f, "image save error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

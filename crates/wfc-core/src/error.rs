use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    ImageLoad(String),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    /// A cell was collapsed or a backtrack recovered.
    Progressed,
    Complete,
    Contradiction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    Complete,
    Contradiction,
}

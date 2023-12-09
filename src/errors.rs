use std::fmt::{Debug, Display};

/// Wrapper around errors that can potentially be produced by aspasia
#[non_exhaustive]
pub enum Error {
    /// Error opening or creating a file
    FileIoError(std::io::Error),
    /// Error trying to recognise file type
    UnknownFileTypeError,
}

impl std::error::Error for Error {}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")?;
        if let Some(source) = std::error::Error::source(&self) {
            writeln!(f, "Caused by:\n\t{source}")?;
        }
        Ok(())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileIoError(err) => write!(
                f,
                "file i/o error occurred while trying to read or write from a file: {err:?}"
            ),
            Error::UnknownFileTypeError => {
                write!(f, "could not detect file type automatically")
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::FileIoError(value)
    }
}

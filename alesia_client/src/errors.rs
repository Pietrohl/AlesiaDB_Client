// this should be redone after dropping rusqlite

use std::fmt::{self, Display};

#[derive(Debug)]
pub struct AlesiaError(pub Box<dyn std::error::Error + Sync + Send>);

#[derive(Debug)]
pub enum Error {
    RusqliteError(rusqlite::Error),
    ConfigError(AlesiaError),
    IoError(AlesiaError),
    InvalidQuery(AlesiaError),
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::RusqliteError(error)
    }
}

impl From<&str> for AlesiaError {
    fn from(error: &str) -> Self {
        AlesiaError(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            error,
        )))
    }
}

impl Display for AlesiaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error {
    pub fn config(e: AlesiaError) -> Self {
        Self::ConfigError(e)
    }
    pub fn io(e: AlesiaError) -> Self {
        Self::IoError(e)
    }
    pub fn invalid_query(e: AlesiaError) -> Self {
        Self::InvalidQuery(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RusqliteError(e) => Display::fmt(e, f),
            Self::ConfigError(e) => Display::fmt(e, f),
            Self::IoError(e) => Display::fmt(e, f),
            Self::InvalidQuery(e) => Display::fmt(e, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RusqliteError(e) => Some(e),
            Self::ConfigError(e) => Some(&*e.0),
            Self::IoError(e) => Some(&*e.0),
            Self::InvalidQuery(e) => Some(&*e.0),
        }
    }
}





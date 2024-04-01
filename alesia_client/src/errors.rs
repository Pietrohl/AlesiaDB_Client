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

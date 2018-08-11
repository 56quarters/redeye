//
//
//

//!

use chrono::format;
use serde_json::error::Error as SerdeError;
use std::io;

pub type RedeyeResult<T> = Result<T, RedeyeError>;

#[derive(Fail, Debug)]
pub enum RedeyeError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    SerializationError(#[cause] SerdeError),

    #[fail(display = "{}", _0)]
    TimestampParseError(#[cause] format::ParseError),

    #[fail(display = "Could not parse: {}", _0)]
    ParseError(String),

    #[fail(display = "An unknown error occurred")]
    Unknown,
}

impl From<io::Error> for RedeyeError {
    fn from(e: io::Error) -> Self {
        RedeyeError::IoError(e)
    }
}

impl From<SerdeError> for RedeyeError {
    fn from(e: SerdeError) -> Self {
        RedeyeError::SerializationError(e)
    }
}

impl From<format::ParseError> for RedeyeError {
    fn from(e: format::ParseError) -> Self {
        RedeyeError::TimestampParseError(e)
    }
}

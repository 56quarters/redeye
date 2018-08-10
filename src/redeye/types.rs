//
//
//

//!

use serde_json::error::Error as SerdeError;
use std::io;
use tokio::timer;

pub type RedeyeResult<T> = Result<T, RedeyeError>;

#[derive(Fail, Debug)]
pub enum RedeyeError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    TimerError(#[cause] timer::Error),

    #[fail(display = "{}", _0)]
    SerializationError(#[cause] SerdeError),

    #[fail(display = "Could not parse: {}", _0)]
    ParseError(String),

    #[fail(display = "Receiver closed the channel")]
    Disconnected,

    #[fail(display = "An unknown error occurred")]
    Unknown,
}

impl From<io::Error> for RedeyeError {
    fn from(e: io::Error) -> Self {
        RedeyeError::IoError(e)
    }
}

impl From<timer::Error> for RedeyeError {
    fn from(e: timer::Error) -> Self {
        RedeyeError::TimerError(e)
    }
}

impl From<SerdeError> for RedeyeError {
    fn from(e: SerdeError) -> Self {
        RedeyeError::SerializationError(e)
    }
}

impl RedeyeError {
    pub fn is_transient(&self) -> bool {
        !self.is_fatal()
    }

    pub fn is_fatal(&self) -> bool {
        match self {
            &RedeyeError::TimerError(ref err) => err.is_shutdown(),
            &RedeyeError::Disconnected => true,
            _ => false,
        }
    }
}

//
//
//

//!

use std::io;
use tokio::timer;

type RedeyeResult<T> = Result<T, RedeyeError>;

#[derive(Fail, Debug)]
pub enum RedeyeError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    TimerError(#[cause] timer::Error),

    #[fail(display = "Receiver closed the channel")]
    Disconnected,

    #[fail(display = "An unknown error occurred")]
    Unknown,
}

impl RedeyeError {
    pub fn is_transient(&self) -> bool {
        !self.is_fatal()
    }

    pub fn is_fatal(&self) -> bool {
        match self {
            &RedeyeError::TimerError(ref err) => err.is_shutdown(),
            &RedeyeError::Disconnected => true,
            _ => false
        }
    }
}

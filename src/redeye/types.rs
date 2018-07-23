//
//
//

//!

use std::io;

#[derive(Fail, Debug)]
pub enum RedeyeError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Receiver closed the channel")]
    Disconnected,
    #[fail(display = "An unknown error occurred")]
    Unknown,
}

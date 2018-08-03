//
//
//
//

//!

extern crate futures;
extern crate redeye;
extern crate tokio;

use redeye::input::StdinBufReader;
use redeye::types::RedeyeError;
use std::io;
use tokio::io as tio;
use tokio::prelude::*;
use tokio::runtime::Runtime;

fn new_stdin_task<R>(reader: R) -> impl Future<Item = (), Error = ()>
where
    R: AsyncRead + io::BufRead,
{
    tio::lines(reader)
        .map_err(|err| RedeyeError::from(err))
        .for_each(move |msg| {
            println!("Line: {}", msg);
            Ok(())
        })
        .map_err(|e| {
            handle_redeye_error(e);
        })
}

fn handle_redeye_error(err: RedeyeError) {
    if err.is_fatal() {
        panic!("ERROR: Unrecoverable error: {}", err);
    } else {
        eprintln!("WARNING: {}", err);
    }
}

fn main() {
    // stdin -> buffer
    let lines = new_stdin_task(StdinBufReader::default());

    let mut runtime = Runtime::new().unwrap();
    runtime.spawn(lines);
    runtime.shutdown_on_idle().wait().unwrap();
}

//
//
//
//

//!

extern crate futures;
extern crate redeye;
extern crate tokio;

use redeye::input::StdinBufReader;
use redeye::parser::{CommonLogLineParser, LogEvent, LogLineParser};
use redeye::types::RedeyeError;
use std::io;
use tokio::io as tio;
use tokio::prelude::*;

fn new_stdin_task<R, P>(reader: R, parser: P) -> impl Future<Item = (), Error = ()>
where
    R: AsyncRead + io::BufRead,
    P: LogLineParser,
{
    tio::lines(reader)
        .map_err(|err| RedeyeError::from(err))
        .for_each(move |msg| {
            //println!("parsed: {:?}", parser.parse(&msg));
            let _ = parser.parse(&msg);
            Ok(())
        }).map_err(|e| {
            handle_redeye_error(e);
        })
}

fn handle_redeye_error(err: RedeyeError) {
    eprintln!("WARNING: {}", err);
}

fn main() {
    let parser = CommonLogLineParser::new();
    let lines = new_stdin_task(StdinBufReader::default(), parser);

    tokio::run(lines);
}

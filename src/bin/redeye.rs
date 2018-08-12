//
//
//
//

//!

extern crate clap;
extern crate futures;
extern crate redeye;
extern crate serde_json;
extern crate tokio;

use redeye::input::StdinBufReader;
use redeye::parser::{CommonLogLineParser, LogLineParser};
use redeye::types::RedeyeError;
use std::io::BufRead;
use tokio::io::{lines, stdout};
use tokio::prelude::*;

fn new_parser_task<R, P, W>(reader: R, parser: P, mut writer: W) -> impl Future<Item = (), Error = ()>
where
    R: AsyncRead + BufRead,
    P: LogLineParser,
    W: AsyncWrite,
{
    lines(reader)
        .map_err(RedeyeError::from)
        .and_then(move |line| parser.parse(&line))
        .and_then(|event| serde_json::to_string(&event).map_err(RedeyeError::from))
        .for_each(move |json| {
            writeln!(writer, "{}", json);
            Ok(())
        }).map_err(|e| {
            handle_redeye_error(e);
        })
}

fn handle_redeye_error(err: RedeyeError) {
    eprintln!("redeye: WARNING: {}", err);
}

fn main() {
    let reader = StdinBufReader::default();
    let parser = CommonLogLineParser::new();
    let writer = stdout();
    let lines = new_parser_task(reader, parser, writer);

    tokio::run(lines);
}

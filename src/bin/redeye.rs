//
//
//
//

//!

#[macro_use]
extern crate clap;
extern crate redeye;
extern crate serde_json;
extern crate tokio;

use clap::{App, Arg, ArgMatches};
use redeye::input::StdinBufReader;
use redeye::parser::{CommonLogLineParser, LogLineParser};
use redeye::types::RedeyeError;
use std::env;
use std::io::BufRead;
use std::process;
use tokio::io::{lines, stdout};
use tokio::prelude::*;

const MAX_TERM_WIDTH: usize = 72;

fn parse_cli_opts<'a>(args: Vec<String>) -> ArgMatches<'a> {
    App::new("Redeye log converter")
        .version(crate_version!())
        .set_term_width(MAX_TERM_WIDTH)
        .about(
            "\nRedeye converts Apache httpd style access to JSON understood \
             by Logstash. Access log entries are read line by line from stdin, \
             converted to Logstash JSON, and emitted on stdout. Currently \
             Common and Combined access log formats are supported. For more \
             information about these formats, see \n\n\
             https://httpd.apache.org/docs/current/logs.html#accesslog",
        ).arg(
            Arg::with_name("common-format")
                .long("common-format")
                .help(
                    "Parse log entries assuming the Common log format. Entries \
                     that don't match this format will be discarded and a warning \
                     will be printed to stderr.",
                ).conflicts_with_all(&["combined-format"]),
        ).arg(
            Arg::with_name("combined-format")
                .long("combined-format")
                .help(
                    "Parse log entries assuming the Combined log format. Entries \
                     that don't match this format will be discarded and a warning \
                     will be printed to stderr.",
                ).conflicts_with_all(&["combined-format"]),
        ).get_matches_from(args)
}

fn new_parser_task<R, P, W>(
    reader: R,
    parser: P,
    mut writer: W,
) -> impl Future<Item = (), Error = ()>
where
    R: AsyncRead + BufRead,
    P: LogLineParser,
    W: AsyncWrite,
{
    lines(reader)
        .map_err(RedeyeError::from)
        .and_then(move |line| parser.parse(&line))
        .and_then(|event| serde_json::to_string(&event).map_err(RedeyeError::from))
        .for_each(move |json| writeln!(writer, "{}", json).map_err(RedeyeError::from))
        .map_err(|e| {
            handle_redeye_error(e);
        })
}

fn handle_redeye_error(err: RedeyeError) {
    eprintln!("redeye: WARNING: {}", err);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = parse_cli_opts(args);

    let parser = if matches.is_present("common-format") {
        CommonLogLineParser::new()
    } else if matches.is_present("combined-format") {
        unimplemented!();
    } else {
        eprintln!("redeye: ERROR: Log input format must be specified");
        process::exit(1);
    };

    let reader = StdinBufReader::default();
    let writer = stdout();
    let lines = new_parser_task(reader, parser, writer);

    tokio::run(lines);
}

// Redeye - Parse Apache-style access logs into Logstash JSON
//
// Copyright 2018 TSH Labs
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

//! Redeye - Parse Apache-style access logs into Logstash JSON

use clap::{crate_version, value_t, App, Arg, ArgMatches};
use redeye::io::{StdinBufReader, StdoutBufWriter};
use redeye::parser::{CombinedLogLineParser, CommonLogLineParser, LogLineParser};
use redeye::types::RedeyeError;
use std::env;
use std::io::BufRead;
use std::process;
use tokio::io::{lines, stdin, stdout};
use tokio::prelude::*;

const MAX_TERM_WIDTH: usize = 72;

fn parse_cli_opts<'a>(args: Vec<String>) -> ArgMatches<'a> {
    App::new("Redeye log converter")
        .version(crate_version!())
        .set_term_width(MAX_TERM_WIDTH)
        .about(
            "\nRedeye converts NCSA or Apache HTTPd style access logs to JSON \
             understood by Logstash. Access log entries are read line by line \
             from stdin, converted to Logstash JSON, and emitted on stdout. \
             Currently Common and Combined access log formats are supported. \
             For more information about these formats, see \n\n\
             https://httpd.apache.org/docs/current/logs.html#accesslog",
        )
        .arg(
            Arg::with_name("common-format")
                .long("common-format")
                .help(
                    "Parse log entries assuming the Common log format. Entries \
                     that don't match this format will be discarded and a warning \
                     will be printed to stderr.",
                )
                .conflicts_with_all(&["combined-format"]),
        )
        .arg(
            Arg::with_name("combined-format")
                .long("combined-format")
                .help(
                    "Parse log entries assuming the Combined log format. Entries \
                     that don't match this format will be discarded and a warning \
                     will be printed to stderr.",
                )
                .conflicts_with_all(&["common-format"]),
        )
        .arg(
            Arg::with_name("output-buffer")
                .long("output-buffer")
                .default_value("1024")
                .help("How large a buffer to use when writing output, in bytes."),
        )
        .arg(
            Arg::with_name("input-buffer")
                .long("input-buffer")
                .default_value("1024")
                .help("How large a buffer to use when reading input, in bytes."),
        )
        .get_matches_from(args)
}

fn new_parser_task<R, W>(
    reader: R,
    parser: Box<dyn LogLineParser + Send + Sync>,
    mut writer: W,
) -> impl Future<Item = (), Error = ()>
where
    R: AsyncRead + BufRead,
    W: AsyncWrite,
{
    lines(reader)
        .map_err(RedeyeError::from)
        .for_each(move |line| {
            let _ = parser
                .parse(&line)
                .and_then(|event| serde_json::to_string(&event).map_err(RedeyeError::from))
                .and_then(|json| writeln!(writer, "{}", json).map_err(RedeyeError::from))
                .map_err(handle_redeye_error);
            Ok(())
        })
        .map_err(handle_redeye_error)
}

fn handle_redeye_error(err: RedeyeError) {
    let display = match err {
        RedeyeError::IoError(e) => format!("I/O error: {}", e),
        RedeyeError::SerializationError(e) => format!("Serialization error: {}", e),
        RedeyeError::TimestampParseError(e) => format!("Invalid timestamp: {}", e),
        RedeyeError::ParseError(e) => format!("Invalid log line: {}", e),
    };

    eprintln!("redeye: WARNING: {}", display);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let matches = parse_cli_opts(args);

    let parser: Box<dyn LogLineParser + Send + Sync> = if matches.is_present("common-format") {
        Box::new(CommonLogLineParser::new())
    } else if matches.is_present("combined-format") {
        Box::new(CombinedLogLineParser::new())
    } else {
        eprintln!("redeye: error: Log input format must be specified");
        process::exit(1);
    };

    let reader = {
        let input_buf = value_t!(matches, "input-buffer", usize).unwrap_or_else(|e| e.exit());
        StdinBufReader::with_capacity(input_buf, stdin())
    };

    let writer = {
        let output_buf = value_t!(matches, "output-buffer", usize).unwrap_or_else(|e| e.exit());
        StdoutBufWriter::with_capacity(output_buf, stdout())
    };

    let lines = new_parser_task(reader, parser, writer);
    tokio::run(lines);
}

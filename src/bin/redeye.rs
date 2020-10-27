// Redeye - Parse Apache-style access logs into Logstash JSON
//
// Copyright 2018 Nick Pillitteri
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

use clap::Clap;
use redeye::parser::{CombinedLogLineParser, CommonLogLineParser, LogLineParser};
use redeye::types::RedeyeError;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};
use std::process;

/// Redeye converts NCSA or Apache HTTPd style access logs to JSON understood by
/// Logstash. Access log entries are read line by line from stdin, converted to
/// Logstash JSON, and emitted on stdout. Currently Common and Combined access
/// log formats are supported. For more information about these formats, see
/// https://httpd.apache.org/docs/current/logs.html#accesslog",
#[derive(Clap, Debug)]
#[clap(name = "redeye")]
struct RedeyeOptions {
    /// parse log entries assuming the Common log format. Entries
    /// that don't match this format will be discarded and a warning
    /// will be printed to stderr.
    #[clap(long)]
    common_format: bool,

    /// parse log entries assuming the Combined log format. Entries
    /// that don't match this format will be discarded and a warning
    /// will be printed to stderr.
    #[clap(long)]
    combined_format: bool,

    /// how large a buffer to use when writing output, in bytes.
    #[clap(long, default_value = "1024")]
    output_buffer: usize,

    /// how large a buffer to use when reading input, in bytes.
    #[clap(long, default_value = "1024")]
    input_buffer: usize,
}

fn handle_redeye_error(err: RedeyeError) {
    let display = match err {
        RedeyeError::IoError(e) => format!("I/O error: {}", e),
        RedeyeError::SerializationError(e) => format!("Serialization error: {}", e),
        RedeyeError::TimestampParseError(e) => format!("Invalid timestamp: {}", e),
        RedeyeError::ParseError(e) => format!("Invalid log line: {}", e),
    };

    eprintln!("redeye: warning: {}", display);
}

fn main() {
    let opts = RedeyeOptions::parse();

    let parser: Box<dyn LogLineParser + Send + Sync> = if opts.common_format {
        Box::new(CommonLogLineParser::new())
    } else if opts.combined_format {
        Box::new(CombinedLogLineParser::new())
    } else {
        eprintln!("redeye: error: Log input format must be specified");
        process::exit(1);
    };

    let reader = BufReader::with_capacity(opts.input_buffer, stdin());
    let mut writer = BufWriter::with_capacity(opts.output_buffer, stdout());

    for line in reader.lines() {
        let _r = line
            .map_err(RedeyeError::from)
            .and_then(|log| parser.parse(&log))
            .and_then(|event| serde_json::to_string(&event).map_err(RedeyeError::from))
            .and_then(|json| writeln!(writer, "{}", json).map_err(RedeyeError::from))
            .map_err(handle_redeye_error);
    }
}

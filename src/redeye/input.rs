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

//!

use std::io::{self, BufRead, BufReader, Read};
use tokio::io::{stdin, AsyncRead, Stdin};

/// `AsyncRead` implementation for standard input that supports
/// buffering and can be used for line-by-line reading of input.
pub struct StdinBufReader {
    reader: BufReader<Stdin>,
}

impl StdinBufReader {
    pub fn new(reader: Stdin) -> Self {
        StdinBufReader {
            reader: BufReader::new(reader),
        }
    }

    pub fn with_capacity(cap: usize, reader: Stdin) -> Self {
        StdinBufReader {
            reader: BufReader::with_capacity(cap, reader),
        }
    }
}

impl Default for StdinBufReader {
    fn default() -> Self {
        Self::new(stdin())
    }
}

impl Read for StdinBufReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.reader.read(buf)
    }
}

impl AsyncRead for StdinBufReader {}

impl BufRead for StdinBufReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

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

//! Adapters to enable async line-by-line input and output.

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use tokio::io::{stdin, stdout, AsyncRead, AsyncWrite};
use tokio::prelude::Poll;

const DEFAULT_BUF_INPUT_BYTES: usize = 1024;
const DEFAULT_BUF_OUTPUT_BYTES: usize = 1024;

/// `AsyncRead` implementation for standard input that supports
/// buffering and can be used for line-by-line reading of input.
pub struct StdinBufReader {
    reader: Box<BufRead + Send + Sync>,
}

impl StdinBufReader {
    pub fn new<R>(reader: R) -> Self
    where
        R: Read + Sync + Send + 'static,
    {
        Self::with_capacity(DEFAULT_BUF_INPUT_BYTES, reader)
    }

    pub fn with_capacity<R>(cap: usize, reader: R) -> Self
    where
        R: Read + Sync + Send + 'static,
    {
        StdinBufReader {
            reader: Box::new(BufReader::with_capacity(cap, reader)),
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

/// `AsyncWrite` implementation for standard output that supports buffering.
pub struct StdoutBufWriter {
    writer: Box<AsyncWrite + Send + Sync>,
}

impl StdoutBufWriter {
    pub fn new<W>(writer: W) -> Self
    where
        W: AsyncWrite + Send + Sync + 'static,
    {
        Self::with_capacity(DEFAULT_BUF_OUTPUT_BYTES, writer)
    }

    pub fn with_capacity<W>(cap: usize, writer: W) -> Self
    where
        W: AsyncWrite + Send + Sync + 'static,
    {
        StdoutBufWriter {
            writer: Box::new(BufWriter::with_capacity(cap, writer)),
        }
    }
}

impl Default for StdoutBufWriter {
    fn default() -> Self {
        Self::new(stdout())
    }
}

impl Write for StdoutBufWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.writer.flush()
    }
}

impl AsyncWrite for StdoutBufWriter {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.writer.shutdown()
    }
}

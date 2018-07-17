use std::io::{self, BufReader};
use tokio::io::{stdin, AsyncRead, Stdin};

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

impl io::Read for StdinBufReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.reader.read(buf)
    }
}

impl AsyncRead for StdinBufReader {}

impl io::BufRead for StdinBufReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

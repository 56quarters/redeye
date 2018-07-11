//
//

//!

pub struct LineBuffer {
    buf: Vec<String>,
}

impl LineBuffer {
    pub fn new() -> Self {
        LineBuffer { buf: Vec::new() }
    }

    pub fn push(&mut self, line: String) -> usize {
        self.buf.push(line);
        self.buf.len()
    }

    pub fn flush(&mut self) -> () {
        self.buf.clear();
    }
}

//
//

//!

use std::cell::RefCell;
use std::sync::Mutex;
use std::mem;

#[derive(Debug)]
pub struct LineBuffer {
    buf: Mutex<RefCell<Vec<String>>>,
}

impl LineBuffer {
    pub fn new() -> Self {
        LineBuffer {
            buf: Mutex::new(RefCell::new(Vec::new())),
        }
    }

    pub fn push(&self, line: String) {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        buf.push(line);
    }

    pub fn flush(&self) -> Vec<String> {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        mem::replace(&mut buf, Vec::new())
    }
}

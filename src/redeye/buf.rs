//
//

//!

use std::cell::RefCell;
use std::sync::Mutex;

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

    pub fn push(&self, line: String) -> usize {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        buf.push(line);
        buf.len()
    }

    pub fn flush(&self) {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        println!("Flushing {} entries...", buf.len());
        buf.clear();
    }
}

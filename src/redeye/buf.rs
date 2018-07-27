//
//

//!

use std::cell::RefCell;
use std::fmt::Debug;
use std::mem;
use std::sync::Mutex;

#[derive(Debug)]
pub struct LogBuffer<T>
where
    T: Debug,
{
    buf: Mutex<RefCell<Vec<T>>>,
}

impl<T> LogBuffer<T>
where
    T: Debug,
{
    pub fn new() -> Self {
        LogBuffer {
            buf: Mutex::new(RefCell::new(Vec::new())),
        }
    }

    pub fn push(&self, line: T) {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        buf.push(line);
    }

    pub fn flush(&self) -> Vec<T> {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        mem::replace(&mut buf, Vec::new())
    }
}

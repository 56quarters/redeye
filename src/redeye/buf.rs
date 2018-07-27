//
//

//!

use std::cell::RefCell;
use std::fmt::Debug;
use std::mem;
use std::sync::Mutex;

const DEFAULT_FLUSH_SIZE: usize = 10_000;
const DEFAULT_BUFFER_SIZE: usize = 128;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum NeedFlush {
    Yes,
    No,
}

#[derive(Debug)]
pub struct LogBuffer<T>
where
    T: Debug,
{
    flush_sz: usize,
    buffer_sz: usize,
    buf: Mutex<RefCell<Vec<T>>>,
}

impl<T> LogBuffer<T>
where
    T: Debug,
{
    pub fn new(flush_sz: usize) -> Self {
        Self::with_buf_size(flush_sz, DEFAULT_BUFFER_SIZE)
    }

    pub fn with_buf_size(flush_sz: usize, buffer_sz: usize) -> Self {
        LogBuffer {
            flush_sz,
            buffer_sz,
            buf: Mutex::new(RefCell::new(Vec::with_capacity(buffer_sz))),
        }
    }

    pub fn push(&self, val: T) -> NeedFlush {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        buf.push(val);

        if buf.len() >= self.flush_sz {
            NeedFlush::Yes
        } else {
            NeedFlush::No
        }
    }

    pub fn flush(&self) -> Vec<T> {
        let cell = self.buf.lock().unwrap();
        let mut buf = cell.borrow_mut();
        mem::replace(&mut buf, Vec::with_capacity(self.buffer_sz))
    }
}

impl<T> Default for LogBuffer<T>
where
    T: Debug,
{
    fn default() -> Self {
        Self::new(DEFAULT_FLUSH_SIZE)
    }
}

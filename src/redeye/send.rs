//
//
//

//!

use futures::sync::mpsc;
use std::cell::RefCell;
use std::io;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tokio::prelude::*;

#[derive(Fail, Debug)]
pub enum RedeyeError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Receiver closed the channel")]
    Disconnected,
    #[fail(display = "An unknown error occurred")]
    Unknown,
}

pub struct BackPressureSender<T>
where
    T: Debug + Clone,
{
    tx: Arc<Mutex<RefCell<mpsc::Sender<T>>>>,
}

impl<T> BackPressureSender<T>
where
    T: Debug + Clone,
{
    pub fn new(tx: mpsc::Sender<T>) -> Self {
        BackPressureSender {
            tx: Arc::new(Mutex::new(RefCell::new(tx))),
        }
    }

    pub fn send(&self, val: T) -> SenderFuture<T> {
        SenderFuture::new(self.tx.clone(), val)
    }
}

pub struct SenderFuture<T>
where
    T: Debug + Clone,
{
    tx: Arc<Mutex<RefCell<mpsc::Sender<T>>>>,
    val: T,
}

impl<T> SenderFuture<T>
where
    T: Debug + Clone,
{
    fn new(tx: Arc<Mutex<RefCell<mpsc::Sender<T>>>>, val: T) -> Self {
        SenderFuture { tx, val }
    }
}

impl<T> Future for SenderFuture<T>
where
    T: Debug + Clone,
{
    type Item = ();
    type Error = RedeyeError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let cell = self.tx.lock().unwrap();
        let mut tx = cell.borrow_mut();

        match tx.poll_ready() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            Err(_) => return Err(RedeyeError::Disconnected),
            _ => {}
        };

        let val = self.val.clone();
        match tx.try_send(val) {
            Ok(_) => Ok(Async::Ready(())),
            Err(e) => {
                if e.is_full() {
                    Ok(Async::NotReady)
                } else {
                    Err(RedeyeError::Disconnected)
                }
            }
        }
    }
}

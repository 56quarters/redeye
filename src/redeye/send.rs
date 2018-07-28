//
//
//

//!

use futures::sync::mpsc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tokio::prelude::*;
use types::RedeyeError;

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
        SenderFuture::new(Arc::clone(&self.tx), val)
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

        // Use the .poll_ready() / .start_send() combination so that we can avoid
        // cloning the value we're sending when the send will obviously fail due to
        // the channel being full. Note that we also don't care about the value we
        // tried to send if the send fails since it's just a copy.
        match tx.poll_ready() {
            Err(_) => Err(RedeyeError::Disconnected),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Ok(Async::Ready(_)) => match tx.start_send(self.val.clone()) {
                Err(_) => Err(RedeyeError::Disconnected),
                Ok(AsyncSink::NotReady(_)) => Ok(Async::NotReady),
                Ok(AsyncSink::Ready) => Ok(Async::Ready(())),
            },
        }
    }
}

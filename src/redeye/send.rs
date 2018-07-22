extern crate futures;
extern crate tokio;

use futures::sync::mpsc;
use tokio::prelude::*;

#[derive(Debug)]
pub enum SendError {
    Disconnected,
}

pub struct BackPressureSender<T> where T: Clone {
    tx: mpsc::Sender<T>
}

impl<T> BackPressureSender<T> where T: Clone {
    pub fn new(tx: mpsc::Sender<T>) -> Self {
        BackPressureSender { tx }
    }

    pub fn send(&self, val: T) -> SenderFuture<T> {
        SenderFuture::new(val, self.tx.clone())
    }
}

pub struct SenderFuture<T> where T: Clone {
    val: T,
    tx: mpsc::Sender<T>
}

impl<T> SenderFuture<T> where T: Clone {
    fn new(val: T, tx: mpsc::Sender<T>) -> Self {
        SenderFuture { val, tx }
    }
}

impl<T> Future for SenderFuture<T> where T: Clone {
    type Item = ();
    type Error = SendError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let ready = self.tx.poll_ready();
        match ready {
            Ok(Async::NotReady) => {
                println!("Not ready!");
                return Ok(Async::NotReady)
            }
            Err(_) => {
                println!("Disconnected!");
                return Err(SendError::Disconnected)
            }
            _ => {}
        }

        let msg = self.val.clone();
        match self.tx.try_send(msg) {
            Ok(_) => {
                println!("Send OK!");
                Ok(Async::Ready(()))

            }
            Err(e) => {
                if e.is_full() {
                    println!("Full!");
                    Ok(Async::NotReady)
                } else {
                    println!("Disconnected 2!!");
                    Err(SendError::Disconnected)
                }
            }
        }
    }
}

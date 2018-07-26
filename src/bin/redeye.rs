//
//
//
//

//!

extern crate futures;
extern crate redeye;
extern crate tokio;

use futures::sync::mpsc;
use redeye::buf::LineBuffer;
use redeye::input::StdinBufReader;
use redeye::send::BackPressureSender;
use redeye::types::RedeyeError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::{Delay, Interval};

#[allow(unused)]
fn delayed_receiver(rx: mpsc::Receiver<String>, delay: u64) -> impl Future<Item = (), Error = ()> {
    Delay::new(Instant::now() + Duration::from_secs(delay))
        .map_err(|err| {
            println!("Delay error: {:?}", err);
        })
        .and_then(|_| {
            rx.for_each(|msg| {
                println!("Message: {}", msg);
                Ok(())
            }).map_err(|err| {
                println!("Message error: {:?}", err);
            })
        })
}

fn main() {
    let (tx, rx) = mpsc::channel(1);
    let sender = BackPressureSender::new(tx);
    let stdin = StdinBufReader::new(io::stdin());

    let lines = io::lines(stdin)
        .map_err(|err| RedeyeError::IoError(err))
        .for_each(move |line| sender.send(line))
        .map_err(|err| {
            println!("Line error: {:?}", err);
        });

    let buf = Arc::new(LineBuffer::new());
    let buf_flush = Arc::clone(&buf);

    let start = Instant::now() + Duration::from_secs(1);
    let period = Interval::new(start, Duration::from_secs(1))
        .map_err(|err| RedeyeError::TimerError(err))
        .for_each(move |_instant| {
            buf_flush.flush();
            Ok(())
        })
        .map_err(|err| {
            println!("Period error: {:?}", err);
        });

    let buf_send = Arc::clone(&buf);

    let receiver = rx.for_each(move |msg| {
        buf_send.push(msg);
        Ok(())
    });

    let mut runtime = Runtime::new().unwrap();
    runtime.spawn(period);
    runtime.spawn(lines);
    runtime.spawn(receiver);
    runtime.shutdown_on_idle().wait().unwrap();
}

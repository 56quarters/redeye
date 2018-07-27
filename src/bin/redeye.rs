//
//
//
//

//!

extern crate futures;
extern crate redeye;
extern crate tokio;

use futures::sync::mpsc;
use redeye::buf::{LogBuffer, NeedFlush};
use redeye::enrich::LineParser;
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

enum FlushReason {
    Full,
    Timer,
}

fn main() {
    let buf_send = Arc::new(LogBuffer::default());
    let buf_flush = Arc::clone(&buf_send);
    let parser = Arc::new(LineParser::default());

    let (tx_flush, rx_flush) = mpsc::channel(1);
    let sender_flush = BackPressureSender::new(tx_flush);

    let lines = io::lines(StdinBufReader::default())
        .map_err(|err| RedeyeError::from(err))
        .and_then(move |line| parser.parse_line(line))
        .for_each(move |msg| {
            buf_send.push(msg);
            Ok(())
        })
        .map_err(|err| {
            eprintln!("Line error: {:?}", err);
        });

    let (tx_backend, rx_backend) = mpsc::channel(1);
    let sender_backend = BackPressureSender::new(tx_backend);
    let start = Instant::now() + Duration::from_secs(1);

    let period = Interval::new(start, Duration::from_millis(1000))
        .map(|_instant| ())
        .map_err(|err| {
            eprintln!("Period error: {:?}", err);
        })
        .select(rx_flush)
        .for_each(move |_instant| {
            sender_backend.send(buf_flush.flush()).map_err(|err| {
                eprintln!("Flush error: {:?}", err);
            })
        });

    let backend = rx_backend
        .for_each(|batch| {
            eprintln!("Received {} entries in backend", batch.len());
            Ok(())
        })
        .map_err(|err| {
            eprintln!("Backend error: {:?}", err);
        });

    let mut runtime = Runtime::new().unwrap();
    runtime.spawn(period);
    runtime.spawn(lines);
    runtime.spawn(backend);
    runtime.shutdown_on_idle().wait().unwrap();
}

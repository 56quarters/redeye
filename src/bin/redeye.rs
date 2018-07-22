extern crate futures;
extern crate redeye;
extern crate tokio;

use futures::sync::mpsc;
use redeye::input::StdinBufReader;
use redeye::send::BackPressureSender;
use std::time::{Duration, Instant};
use tokio::io;
use tokio::prelude::*;
use tokio::timer::Interval;

fn main() {
    let (tx, rx) = mpsc::channel(1);
    let sender = BackPressureSender::new(tx);
    let stdin = StdinBufReader::new(io::stdin());

    let lines = io::lines(stdin)
        .for_each(move |line| {
            println!("sending...");
            sender
                .send(line)
                .map_err(|_err| std::io::Error::from(std::io::ErrorKind::Other))
        })
        .map_err(|err| {
            println!("Line error: {:?}", err);
        });

    let start = Instant::now() + Duration::from_secs(1);
    let period = Interval::new(start, Duration::from_secs(1))
        .for_each(|instant| {
            println!("Period: {:?}", instant);
            Ok(())
        })
        .map_err(|err| {
            println!("Period error: {:?}", err);
        });

    let receiver =
        rx.for_each(|msg| {
            println!("Message: {}", msg);
            Ok(())
        }).map_err(|err| {
            println!("Message error: {:?}", err);
        });

    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(period);
    runtime.spawn(lines);
    runtime.spawn(receiver);
    runtime.shutdown_on_idle().wait().unwrap();
}

//
//
//
//

//!

extern crate futures;
extern crate redeye;
extern crate tokio;

use futures::sync::mpsc;
use redeye::buf::LogBuffer;
use redeye::input::StdinBufReader;
use redeye::parser::{LineParser, LogMessage};
use redeye::send::BackPressureSender;
use redeye::types::RedeyeError;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io as tio;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::Interval;

fn new_stdin_task<R>(
    reader: R,
    parser: Arc<LineParser>,
    buffer: Arc<LogBuffer<LogMessage>>,
) -> impl Future<Item = (), Error = ()>
where
    R: AsyncRead + io::BufRead,
{
    tio::lines(reader)
        .map_err(|err| RedeyeError::from(err))
        .and_then(move |line| parser.parse_line(line))
        .for_each(move |msg| {
            buffer.push(msg);
            Ok(())
        })
        .map_err(|e| {
            handle_redeye_error(e);
        })
}

fn new_period_sender(
    sender: BackPressureSender<Vec<LogMessage>>,
    buffer: Arc<LogBuffer<LogMessage>>,
) -> impl Future<Item = (), Error = ()> {
    Interval::new(Instant::now(), Duration::from_millis(1000))
        .map_err(|err| RedeyeError::from(err))
        .for_each(move |_instant| sender.send(buffer.flush()))
        .map_err(|e| {
            handle_redeye_error(e);
        })
}

fn new_backend_task<T>(rx: mpsc::Receiver<Vec<T>>) -> impl Future<Item = (), Error = ()> {
    rx.for_each(|batch| {
        eprintln!("Received {} entries in backend", batch.len());
        Ok(())
    })
}

fn handle_redeye_error(err: RedeyeError) {
    if err.is_fatal() {
        panic!("ERROR: Unrecoverable error: {}", err);
    } else {
        eprintln!("WARNING: {}", err);
    }
}

fn main() {
    // Log buffer and parser for storing things as they come in from stdin
    let buf_send = Arc::new(LogBuffer::default());
    let buf_flush = Arc::clone(&buf_send);
    let parser = Arc::new(LineParser::default());

    // Channel + sender between periodic flushes and the backend that processes batches
    let (tx_backend, rx_backend) = mpsc::channel(1);
    let sender_backend = BackPressureSender::new(tx_backend);

    // stdin -> buffer
    let lines = new_stdin_task(StdinBufReader::default(), parser, buf_send);

    // buffer -> backend
    let period = new_period_sender(sender_backend, buf_flush);

    // backend -> ???
    let backend = new_backend_task(rx_backend);

    let mut runtime = Runtime::new().unwrap();
    runtime.spawn(period);
    runtime.spawn(lines);
    runtime.spawn(backend);
    runtime.shutdown_on_idle().wait().unwrap();
}

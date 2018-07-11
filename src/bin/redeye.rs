extern crate redeye;
extern crate tokio;

use redeye::input::StdinBufReader;
use std::time::{Duration, Instant};
use tokio::io;
use tokio::prelude::*;
use tokio::timer::Interval;

/*
fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener
        .incoming()
        .for_each(|socket| {
            println!("accepted socket; addr={:?}", socket.peer_addr().unwrap());

            let connection = io::write_all(socket, "hello world\n")
                .then(|res| {
                    println!("wrote message; success={:?}", res.is_ok());
                    Ok(())
                });

            // Spawn a new task that processes the socket:
            tokio::spawn(connection);

            Ok(())
        })
        .map_err(|err| {
            // All tasks must have an `Error` type of `()`. This forces error
            // handling and helps avoid silencing failures.
            //
            // In our example, we are only going to log the error to STDOUT.
            println!("accept error = {:?}", err);
        });

    println!("server running on localhost:6142");
    tokio::run(server);
}
 */

fn main() {
    let stdin = StdinBufReader::new(io::stdin());
    let lines = io::lines(stdin)
        .for_each(|line| {
            println!("Line: {}", line);
            Ok(())
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

    //tokio::run(lines);
    tokio::run(period);
}

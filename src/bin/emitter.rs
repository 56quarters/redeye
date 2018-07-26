//
//
//

//!

use std::env;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut args = env::args();

    let out = if let Some(v) = args.nth(1) {
        v
    } else {
        println!("Need a string to print!");
        exit(1);
    };

    let delay = if let Some(v) = args.nth(0) {
        v.parse::<u64>().unwrap()
    } else {
        println!("Need a delay in milliseconds!");
        exit(1);
    };

    loop {
        println!("{}", out);
        sleep(Duration::from_millis(delay));
    }
}

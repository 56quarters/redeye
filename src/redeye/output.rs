//
//
//

//!

use std::fmt::Display;
use std::io::Stdout;

pub struct LinePrinter {}

impl LinePrinter {
    pub fn new(out: Stdout) -> Self {
        unimplemented!();
    }

    pub fn print<D>(&self, item: D)
    where
        D: Display,
    {

    }
}

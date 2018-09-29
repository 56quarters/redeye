#![feature(test)]
extern crate redeye;
extern crate test;

use redeye::parser::{CombinedLogLineParser, CommonLogLineParser, LogLineParser};
use test::Bencher;

#[bench]
fn bench_common_log_line_parser(b: &mut Bencher) {
    let parser = CommonLogLineParser::new();
    b.iter(|| {
        parser
            .parse(concat!(
                "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] ",
                "\"GET /index.html HTTP/1.0\" 200 2326"
            ))
            .unwrap()
    });
}

#[bench]
fn bench_combined_log_line_parser(b: &mut Bencher) {
    let parser = CombinedLogLineParser::new();
    b.iter(|| {
        parser
            .parse(concat!(
                "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] ",
                "\"GET /index.html HTTP/1.0\" 200 2326 ",
                "\"http://www.example.com/start.html\" ",
                "\"Mozilla/4.08 [en] (Win98; I ;Nav)\""
            ))
            .unwrap()
    });
}

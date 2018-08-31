# Redeye

[![Build Status](https://travis-ci.org/tshlabs/redeye.svg?branch=master)](https://travis-ci.org/tshlabs/redeye)
[![crates.io](https://img.shields.io/crates/v/redeye.svg)](https://crates.io/crates/redeye/)

Parse Apache-style access logs into Logstash JSON.

## About

Redeye reads NCSA or Apache-style access logs from stdin and writes Logstash
compatible JSON to stdout. This allows applications to continue to emit access
logs as they always have while getting the benefits of structured logging in
tools like [Kibana](https://www.elastic.co/products/kibana).

Redeye supports the Common Log Format as well as the Combined Log Format. More
information about these formats is available in the
[Apache Docs](https://httpd.apache.org/docs/current/logs.html#accesslog).

An example of Common Log Format would be:

```
127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326
```

While an example of Combined Log Format would be:

```
127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08 [en] (Win98; I ;Nav)"
```

## Usage

Some examples of how Redeye can be used to parse log files into structured JSON
are given below.

### Parsing a File

Since Redeye parses log lines from standard input, you can parse a file using something
like the following shell command. Note that this example uses the `jq` tool in order to
format the JSON nicely.

```
$ cat <<EOF > logs.txt
127.0.0.1 - - [02/Oct/2018:13:55:36 -0400] "GET /index.html HTTP/1.0" 200 2326
127.0.0.1 - - [02/Oct/2018:13:55:37 -0400] "GET /favicon.ico HTTP/1.0" 200 56
127.0.0.1 - - [02/Oct/2018:13:55:38 -0400] "GET /header.png HTTP/1.0" 304 4051
EOF
```

This creates a file with a few log entries named `logs.txt`. Next, we parse these entries.

```
$ redeye --common-format < log.txt | jq .
{
  "remote_host": "127.0.0.1",
  "requested_uri": "/index.html",
  "content_length": 2326,
  "requested_url": "GET /index.html HTTP/1.0",
  "@timestamp": "2018-10-02T13:55:36-04:00",
  "protocol": "HTTP/1.0",
  "message": "127.0.0.1 - - [02/Oct/2018:13:55:36 -0400] \"GET /index.html HTTP/1.0\" 200 2326",
  "method": "GET",
  "@version": "1",
  "status_code": 200
}
{
  "protocol": "HTTP/1.0",
  "message": "127.0.0.1 - - [02/Oct/2018:13:55:37 -0400] \"GET /favicon.ico HTTP/1.0\" 200 56",
  "@timestamp": "2018-10-02T13:55:37-04:00",
  "method": "GET",
  "requested_url": "GET /favicon.ico HTTP/1.0",
  "requested_uri": "/favicon.ico",
  "@version": "1",
  "content_length": 56,
  "remote_host": "127.0.0.1",
  "status_code": 200
}
{
  "@version": "1",
  "remote_host": "127.0.0.1",
  "content_length": 4051,
  "message": "127.0.0.1 - - [02/Oct/2018:13:55:38 -0400] \"GET /header.png HTTP/1.0\" 304 4051",
  "@timestamp": "2018-10-02T13:55:38-04:00",
  "method": "GET",
  "protocol": "HTTP/1.0",
  "status_code": 304,
  "requested_uri": "/header.png",
  "requested_url": "GET /header.png HTTP/1.0"
}
```

### Parsing Server Output

TBD

### Tailing a File

TBD

## Install

Redeye is written in Rust and can be built with the Rust build tool `cargo`.
It is also available as a Docker image. Instructions for each of these methods
are below.

### From Source - glibc

First, install a Rust toolchain with [rustup](https://rustup.rs/).

Next make sure you have the required non-Rust dependencies. 

* `build-essential` - C compiler toolchain

Then, checkout and build the project:

```
git clone https://github.com/tshlabs/redeye.git && cd redeye
cargo build --release
```

Your binary should be at `target/release/redeye`.

### From Source - musl libc

First, install a Rust toolchain with [rustup](https://rustup.rs/).

Then, add a musl target:

```
rustup target add x86_64-unknown-linux-musl
```

Next make sure you have the required non-Rust dependencies. 

* `build-essential` - C compiler toolchain
* `musl-tools` - musl libc implementation

Then, checkout and build the project:

```
git clone https://github.com/tshlabs/redeye.git && cd redeye
cargo build --release --target=x86_64-unknown-linux-musl
```

Your binary should be at `target/x86_64-unknown-linux-musl/release/redeye`.

### Docker

TBD

## Documentation

The library documentation is available at https://docs.rs/redeye/

## Source

The source code is available on GitHub at https://github.com/tshlabs/redeye

## Changesx86_64-unknown-linux-musl

Release notes for Cadence can be found in the [CHANGES.md](CHANGES.md) file.

## Development

Redeye uses Cargo for performing various development tasks.

To build Redeye:

```
$ cargo build
```

To run tests:

```
$ cargo test
```

or:

```
$ cargo test -- --ignored
```

To run benchmarks:

```
$ cargo bench
```

To build documentation:

```
$ cargo doc
```

## License

Redeye is available under the terms of the [GPL, version 3](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.

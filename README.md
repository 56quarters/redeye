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

Some examples of how Redeye can be used to parse log files into structured
JSON are given below.

### Parsing a File

Since Redeye parses log lines from standard input, you can parse a file using
something like the following shell command.

```
cat <<EOF > logs.txt
127.0.0.1 - - [02/Oct/2018:13:55:36 -0400] "GET /index.html HTTP/1.1" 200 2326
127.0.0.1 - - [02/Oct/2018:13:55:37 -0400] "GET /favicon.ico HTTP/1.1" 200 56
127.0.0.1 - - [02/Oct/2018:13:55:38 -0400] "GET /header.png HTTP/1.1" 304 4051
EOF
```

This creates a file with a few log entries named `logs.txt`. Next, we parse
these entries. Note that this example uses the `jq` tool in order to format
the JSON nicely.

```
redeye --common-format < logs.txt | jq -S .
{
  "@timestamp": "2018-10-02T13:55:36-04:00",
  "@version": "1",
  "content_length": 2326,
  "message": "127.0.0.1 - - [02/Oct/2018:13:55:36 -0400] \"GET /index.html HTTP/1.1\" 200 2326",
  "method": "GET",
  "protocol": "HTTP/1.1",
  "remote_host": "127.0.0.1",
  "requested_uri": "/index.html",
  "requested_url": "GET /index.html HTTP/1.1",
  "status_code": 200
}
{
  "@timestamp": "2018-10-02T13:55:37-04:00",
  "@version": "1",
  "content_length": 56,
  "message": "127.0.0.1 - - [02/Oct/2018:13:55:37 -0400] \"GET /favicon.ico HTTP/1.1\" 200 56",
  "method": "GET",
  "protocol": "HTTP/1.1",
  "remote_host": "127.0.0.1",
  "requested_uri": "/favicon.ico",
  "requested_url": "GET /favicon.ico HTTP/1.1",
  "status_code": 200
}
{
  "@timestamp": "2018-10-02T13:55:38-04:00",
  "@version": "1",
  "content_length": 4051,
  "message": "127.0.0.1 - - [02/Oct/2018:13:55:38 -0400] \"GET /header.png HTTP/1.1\" 304 4051",
  "method": "GET",
  "protocol": "HTTP/1.1",
  "remote_host": "127.0.0.1",
  "requested_uri": "/header.png",
  "requested_url": "GET /header.png HTTP/1.1",
  "status_code": 304
}
```

### Parsing Server Output

Redeye comes with a simple HTTP server written in Python (version 3.4+) that
emits access logs in Common Log Format over `stdout`. You can run this server
for an example of how Redeye might work parsing its output.

From the root of the Redeye codebase, run

```
python util/server.py | ./path/to/redeye --common-format | jq -S .
```

In another terminal, run the following command a few times.

```
curl 'http://localhost:8000/'
```

If you don't see any output from the Python server and Redeye, try running
the `curl` command a few more times. Redeye buffers input for efficiency
and so it might take several requests before it emits any output.

### Tailing a File

You can also parse log lines as they are written to a file using Redeye and
standard UNIX tools. An example using the same Python server from above
is given below. 

First, start the Python HTTP server to serve requests and write access logs
to a file.

```
python util/server.py > access.log
```

Next in another terminal, start tailing the contents of that file and pipe
them to Redeye.

```
tail -f access.log | ./path/to/redeye --common-format | jq -S .
```

In yet another terminal, make a few requests with `curl` to see this in
action.

```
curl 'http://localhost:8000/'
```

Again, be aware that there's a fair amount of buffering going on here so
you may need to make a few requests before you see any output.

## Install

Redeye is written in Rust and can be built or installed with the Rust tool
`cargo`. It is also available as a Docker image. Instructions for each of
these methods are below.

### From Cargo (Rust Package Manager)

First, install a Rust toolchain with [rustup](https://rustup.rs/).

Next, run the following command to download and install Redeye.

```
cargo install --force redeye
```

This will install Redeye alongside other Rust binaries. You'll want to make
sure that binaries installed this way are on your `$PATH`.

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

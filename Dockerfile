FROM rust:slim AS build
WORKDIR /usr/src/redeye
RUN apt-get update \
    && apt-get install -y musl-tools \
    && rustup target add x86_64-unknown-linux-musl
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl \
    && strip --strip-debug target/x86_64-unknown-linux-musl/release/redeye

FROM scratch
COPY --from=build /usr/src/redeye/target/x86_64-unknown-linux-musl/release/redeye /bin/redeye
CMD ["/bin/redeye", "--help"]

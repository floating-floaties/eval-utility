FROM rust:latest as build

RUN USER=root cargo new --lib /sandbox
WORKDIR /sandbox

ARG BIN_NAME=eval_utility

COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release
RUN rm -rf ./src
RUN ls -la ./target/release/deps
RUN rm ./target/release/deps/$BIN_NAME*

COPY ./src ./src
COPY ./examples ./examples

RUN cargo test --release

# Run examples
RUN cargo run --example example

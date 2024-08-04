# syntax=docker/dockerfile:1
FROM rust:1.80-slim-bullseye AS builder

COPY . /websocket/
WORKDIR /websocket
RUN apt update
RUN apt install -y build-essential pkg-config libssl-dev
RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder /websocket/target/release/server /usr/local/bin/

CMD ["server"]
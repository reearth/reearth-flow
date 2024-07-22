FROM rust:1.79-slim-bookworm as builder

ARG NAME=reearth-flow
WORKDIR /usr/src/${NAME}

ENV CARGO_TARGET_DIR=/tmp/target \
    DEBIAN_FRONTEND=noninteractive \
    LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8

RUN apt-get update && apt-get install -y \
    ca-certificates \
    locales \
    git \
    build-essential \
    libssl-dev \
    libxml2-dev \
    gcc \
    g++ \
    cmake \
    make \
    libc-dev \
    pkg-config \
    clang \
    strace  \
    libdbus-1-dev \
    && echo "ja_JP UTF-8" > /etc/locale.gen \
    && locale-gen \
    && echo "install rust tools"

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY examples ./examples
COPY tests ./tests
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    libxml2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /tmp/target/release/reearth-flow /app

ENTRYPOINT [ "/app/reearth-flow" ]

FROM rust:1.80-slim-bookworm as builder

ARG NAME=reearth-flow
WORKDIR /usr/src/${NAME}

ENV CARGO_TARGET_DIR=/tmp/target \
    DEBIAN_FRONTEND=noninteractive \
    LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8

RUN apt-get update && apt-get install -y --no-install-recommends \
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

RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    libxml2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    flow
USER flow

COPY --from=builder /tmp/target/release/reearth-flow /bin

ENTRYPOINT [ "/bin/reearth-flow" ]

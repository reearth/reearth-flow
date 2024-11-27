# syntax=docker/dockerfile:1
FROM rust:1.82-slim-bookworm AS builder

ARG NAME=websocket-app
WORKDIR /usr/src/${NAME}
ENV DEBIAN_FRONTEND=noninteractive \
    LC_CTYPE=en_US.utf8 \
    LANG=en_US.utf8 \
    RUST_LOG="info,tower_http=debug"

RUN --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && apt-get install -y \
    ca-certificates \
    locales \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    && echo "en_US UTF-8" > /etc/locale.gen \
    && locale-gen

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/${NAME}/target \
    cargo build --release --features gcs-storage && \
    mv target/release/main /tmp/websocket-app

FROM debian:bookworm-slim

RUN --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder --chmod=0755 /tmp/websocket-app /app/websocket-app
COPY conf/conf.yaml /app/conf/conf.yaml

ENV RUST_LOG="info,tower_http=debug" \
    HOST="0.0.0.0" \
    APP_ENV="production"

CMD ["/app/websocket-app"]

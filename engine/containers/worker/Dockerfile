# syntax=docker/dockerfile:1
FROM rust:1.85-slim-bookworm AS builder

ARG NAME=reearth-flow
WORKDIR /usr/src/${NAME}

ENV DEBIAN_FRONTEND=noninteractive \
    LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8

RUN  \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && apt-get install -y \
    ca-certificates \
    locales \
    git \
    build-essential \
    gcc \
    g++ \
    cmake \
    make \
    libc-dev \
    # Required libraries
    libssl-dev \
    libxml2-dev \
    pkg-config \
    clang \
    libdbus-1-dev

RUN echo "ja_JP UTF-8" > /etc/locale.gen && locale-gen

RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/${NAME}/target \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=runtime,target=runtime \
    --mount=type=bind,source=cli,target=cli \
    --mount=type=bind,source=plateau-gis-quality-checker,target=plateau-gis-quality-checker \
    --mount=type=bind,source=worker,target=worker \
    cargo build --package reearth-flow-worker --bin reearth-flow-worker --release && mv target/release/reearth-flow-worker /tmp/reearth-flow-worker

FROM debian:bookworm-slim

RUN \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && \
    apt-get install -y \
    ca-certificates \
    libxml2 && \
    rm -rf /var/lib/apt/lists/*

COPY --chmod=0755 --from=builder /tmp/reearth-flow-worker /bin

CMD [ "/bin/reearth-flow-worker" ]

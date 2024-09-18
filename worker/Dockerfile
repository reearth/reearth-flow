# syntax=docker/dockerfile:1
FROM rust:1.81-slim-bookworm AS builder

ARG NAME=reearth-flow
WORKDIR /usr/src/${NAME}

ENV DEBIAN_FRONTEND=noninteractive \
    LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8

RUN \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && apt-get install -y git \
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
    libdbus-1-dev

RUN echo "ja_JP UTF-8" > /etc/locale.gen && locale-gen

RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/${NAME}/target \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=bind,source=examples,target=examples \
    --mount=type=bind,source=tests,target=tests \
    cargo build --release && mv target/release/reearth-flow /tmp/reearth-flow


FROM debian:bookworm-slim AS final

RUN \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    apt-get update && apt-get install -y openssl \
    libxml2 \
    ca-certificates

COPY --chmod=0755 --from=builder /tmp/reearth-flow /bin

CMD [ "/bin/reearth-flow" ]

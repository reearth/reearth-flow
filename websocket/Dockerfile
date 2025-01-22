FROM rust:1.81 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release --no-default-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/main /usr/local/bin/websocket

WORKDIR /usr/local/bin
COPY .env .
EXPOSE 8000

CMD ["websocket"]
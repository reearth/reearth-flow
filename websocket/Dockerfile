FROM rust:1.81 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/main /usr/local/bin/websocket
COPY --from=builder /usr/src/app/src/conf.yaml /usr/local/bin/conf.yaml

WORKDIR /usr/local/bin
EXPOSE 8080

CMD ["websocket"]
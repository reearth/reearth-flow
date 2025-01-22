FROM rust:1.81 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release --no-default-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/main /usr/local/bin/websocket
COPY .env /.env

ENV REEARTH_FLOW_REDIS_URL=${REEARTH_FLOW_REDIS_URL}
ENV REEARTH_FLOW_REDIS_TTL=${REEARTH_FLOW_REDIS_TTL}
ENV REEARTH_FLOW_GCS_BUCKET_NAME=${REEARTH_FLOW_GCS_BUCKET_NAME}
ENV REEARTH_FLOW_GCS_ENDPOINT=${REEARTH_FLOW_GCS_ENDPOINT}
ENV REEARTH_FLOW_AUTH_URL=${REEARTH_FLOW_AUTH_URL}
ENV REEARTH_FLOW_AUTH_TIMEOUT_MS=${REEARTH_FLOW_AUTH_TIMEOUT_MS}
ENV REEARTH_FLOW_APP_ENV=${REEARTH_FLOW_APP_ENV}
ENV REEARTH_FLOW_ORIGINS=${REEARTH_FLOW_ORIGINS}

WORKDIR /usr/local/bin
EXPOSE 8000

CMD ["websocket"]
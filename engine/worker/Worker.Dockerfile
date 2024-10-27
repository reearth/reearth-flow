# Build stage
FROM rust:1.82 AS builder

# Copy only the Cargo files first to leverage Docker cache
WORKDIR /usr/src/worker
COPY ./worker/Cargo.toml ./

# Copy the rest of the source code
COPY . /usr/src

RUN cargo build --release

# Runtime stage
FROM rust:1.82-slim

# Install libxml2
RUN apt-get update && \
    apt-get install -y --no-install-recommends libxml2 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/target/release/reearth-flow-worker .

ENTRYPOINT ["reearth-flow-worker"]

## Build and Run
# docker build -f Worker.Dockerfile -t reearth-flow-worker:latest ../
# docker run --rm --name worker_app_container reearth-flow-worker:latest
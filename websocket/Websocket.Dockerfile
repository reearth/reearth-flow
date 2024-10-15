# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM rust:slim

WORKDIR /usr/local/bin

# Install OpenSSL
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/infra .

# Expose the port the app runs on
EXPOSE 8000

# Run the binary
CMD ["infra"]

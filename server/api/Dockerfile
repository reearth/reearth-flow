FROM golang:1.24-alpine AS build
ARG TAG=release
ARG VERSION

RUN apk add --update --no-cache \
    git \
    ca-certificates \
    build-base \
    protoc \
    protobuf-dev \
    curl \
    rust \
    cargo \
    openssl-dev \
    pkgconfig \
    libgcc

WORKDIR /reearth-flow

COPY server/websocket/Cargo.toml server/websocket/Cargo.lock /reearth-flow/server/
COPY server/websocket/src/ /reearth-flow/server/src/
COPY server/websocket/benches/ /reearth-flow/server/benches/
COPY server/websocket/build.rs /reearth-flow/server/
COPY server/websocket/proto/ /reearth-flow/server/proto/

WORKDIR /reearth-flow/server
RUN cargo build --release
WORKDIR /reearth-flow

RUN mkdir -p /usr/local/lib && \
    ln -s /reearth-flow/server/target/release/libwebsocket.so /usr/local/lib/libwebsocket.so && \
    ldconfig /usr/local/lib

RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.28 \
    && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.2

COPY server/api/go.mod server/api/go.sum server/api/main.go /reearth-flow/
RUN go mod download

COPY server/api/cmd/ /reearth-flow/cmd/
COPY server/api/pkg/ /reearth-flow/pkg/
COPY server/api/internal/ /reearth-flow/internal/
COPY server/api/proto/ /reearth-flow/proto/

RUN protoc --go_out=. --go_opt=paths=source_relative \
    --go-grpc_out=. --go-grpc_opt=paths=source_relative \
    proto/*.proto

ENV LD_LIBRARY_PATH=/reearth-flow/server/target/release
RUN CGO_ENABLED=1 RUSTFLAGS="-C target-feature=-crt-static" go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath ./cmd/reearth-flow

FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates \
    libc6-compat \
    libgcc \
    libstdc++

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /reearth-flow/reearth-flow /reearth-flow/reearth-flow
COPY --from=build /reearth-flow/server/target/release/libwebsocket.so /reearth-flow/target/release/
COPY ui/ /reearth-flow/ui/

WORKDIR /reearth-flow

ENV LD_LIBRARY_PATH=/reearth-flow/target/release

CMD ["./reearth-flow"]

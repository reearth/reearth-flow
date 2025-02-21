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
    cargo

WORKDIR /reearth-flow

RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.28 \
    && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.2

COPY go.mod go.sum main.go /reearth-flow/
RUN go mod download

COPY cmd/ /reearth-flow/cmd/
COPY pkg/ /reearth-flow/pkg/
COPY internal/ /reearth-flow/internal/
COPY proto/ /reearth-flow/proto/

COPY server/Cargo.toml server/Cargo.lock /reearth-flow/server/
COPY server/src/ /reearth-flow/server/src/
WORKDIR /reearth-flow/server
RUN cargo build --release

WORKDIR /reearth-flow

RUN protoc --go_out=. --go_opt=paths=source_relative \
    --go-grpc_out=. --go-grpc_opt=paths=source_relative \
    proto/*.proto

RUN CGO_ENABLED=1 RUSTFLAGS="-C target-feature=-crt-static" go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath ./cmd/reearth-flow

FROM alpine:latest

RUN apk add --no-cache ca-certificates libc6-compat

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /reearth-flow/reearth-flow /reearth-flow/reearth-flow
COPY --from=build /reearth-flow/server/target/release/libwebsocket.so /reearth-flow/target/release/
COPY ui* /reearth-flow/ui/

WORKDIR /reearth-flow

ENV LD_LIBRARY_PATH=/reearth-flow/target/release

CMD ["./reearth-flow"]

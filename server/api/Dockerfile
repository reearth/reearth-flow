FROM golang:1.24-alpine AS build
ARG TAG=release
ARG VERSION

RUN apk add --update --no-cache \
    git \
    ca-certificates \
    build-base \
    curl \
    pkgconfig \
    thrift

WORKDIR /reearth-flow

COPY server/api/go.mod server/api/go.sum server/api/main.go /reearth-flow/
RUN go mod download

COPY server/api/cmd/ /reearth-flow/cmd/
COPY server/api/pkg/ /reearth-flow/pkg/
COPY server/api/internal/ /reearth-flow/internal/

RUN go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath ./cmd/reearth-flow

FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates \
    libc6-compat

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /reearth-flow/reearth-flow /reearth-flow/reearth-flow
COPY ui* /reearth-flow/ui/

WORKDIR /reearth-flow

CMD ["./reearth-flow"]

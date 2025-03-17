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

COPY server/subscriber/go.mod server/subscriber/go.sum server/subscriber/main.go /reearth-flow/
RUN go mod download

COPY server/subscriber/cmd/ /reearth-flow/cmd/
COPY server/subscriber/pkg/ /reearth-flow/pkg/
COPY server/subscriber/internal/ /reearth-flow/internal/

RUN go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath -o /reearth-flow/reearth-flow-subscriber .

FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates \
    libc6-compat

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /reearth-flow/reearth-flow-subscriber /reearth-flow/reearth-flow-subscriber

WORKDIR /reearth-flow

CMD ["./reearth-flow-subscriber"]

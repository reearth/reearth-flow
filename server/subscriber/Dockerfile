FROM golang:1.24-alpine AS build
ARG TAG=release
ARG VERSION

RUN apk add --update --no-cache git ca-certificates build-base

WORKDIR /reearth-flow

COPY go.mod go.sum cmd/reearth-flow-subscriber/main.go /reearth-flow/
RUN go mod download

COPY cmd/ /reearth-flow/cmd/
COPY pkg/ /reearth-flow/pkg/
COPY internal/ /reearth-flow/internal/

RUN CGO_ENABLED=0 go build \
    -tags "${TAG}" \
    -ldflags="-X main.version=${VERSION} -s -w -buildid=" \
    -trimpath \
    -o /reearth-flow/reearth-flow-subscriber \
    ./cmd/reearth-flow-subscriber

FROM alpine:latest

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /reearth-flow/reearth-flow-subscriber /reearth-flow/reearth-flow-subscriber

WORKDIR /reearth-flow

CMD ["./reearth-flow-subscriber"]

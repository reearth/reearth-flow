# syntax=docker/dockerfile:1
FROM golang:1.22-alpine AS build
ARG TAG=release
ARG VERSION

RUN apk add --update --no-cache git ca-certificates build-base

COPY go.mod go.sum main.go /api/
WORKDIR /api
RUN go mod download

COPY cmd/ /api/cmd/
COPY pkg/ /api/pkg/
COPY internal/ /api/internal/

RUN CGO_ENABLED=0 go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath ./cmd/reearth-flow

FROM scratch

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /api /api

WORKDIR /api

CMD [ "./reearth-flow" ]

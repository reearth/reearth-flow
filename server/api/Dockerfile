FROM golang:1.24-alpine AS build
ARG TAG=release
ARG VERSION

RUN apk add --update --no-cache git ca-certificates build-base

WORKDIR /reearth-flow

COPY go.mod go.sum main.go /reearth-flow/
RUN go mod download

COPY cmd/ /reearth-flow/cmd/
COPY pkg/ /reearth-flow/pkg/
COPY internal/ /reearth-flow/internal/

RUN CGO_ENABLED=0 go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath ./cmd/reearth-flow

FROM scratch

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /reearth-flow/reearth-flow /reearth-flow/reearth-flow
COPY ui* /reearth-flow/ui/

WORKDIR /reearth-flow

CMD ["./reearth-flow"]

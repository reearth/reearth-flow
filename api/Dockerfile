FROM golang:1.22-alpine AS build
ARG TAG=release
ARG VERSION

RUN apk add --update --no-cache git ca-certificates build-base

COPY go.mod go.sum main.go /reearth-flow/
WORKDIR /reearth-flow
RUN go mod download

COPY cmd/ /reearth-flow/cmd/
COPY pkg/ /reearth-flow/pkg/
COPY internal/ /reearth-flow/internal/

RUN CGO_ENABLED=0 go build -tags "${TAG}" "-ldflags=-X main.version=${VERSION} -s -w -buildid=" -trimpath ./cmd/reearth-flow

FROM scratch

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /reearth/reearth-flow /reearth/reearth-flow
COPY ui* /reearth/ui/

WORKDIR /reearth-flow

CMD [ "./reearth-flow" ]

# syntax=docker/dockerfile:1

# Multi-stage build for the Go Y-WebSocket server. CGO is disabled so the binary
# is fully static and runs on distroless/static (no libc needed).
#
# PORT: the server binds 0.0.0.0:{REEARTH_FLOW_WS_PORT} and does NOT read Cloud
# Run's $PORT. The image default is 8000, but the Terraform module
# (eukarya-inc/infrastructure) sets REEARTH_FLOW_WS_PORT=8080 and the Cloud Run
# container_port to 8080 in production, so the deployed service listens on 8080.
FROM golang:1.25 AS builder

WORKDIR /src

# ygo is pinned to a released tag in go.mod (require github.com/reearth/ygo
# v1.22.0) with no local-path `replace`, so it resolves from the module proxy and
# the `go mod download` cache layer below is the right first step.
COPY go.mod go.sum ./
RUN go mod download

COPY . .

# Static build: CGO off, target linux. Trim paths + strip symbols for a smaller,
# reproducible artifact.
ARG VERSION=dev
ARG TARGETOS=linux
ARG TARGETARCH=amd64
RUN CGO_ENABLED=0 GOOS=${TARGETOS} GOARCH=${TARGETARCH} \
    go build \
      -trimpath \
      -ldflags="-s -w -X main.version=${VERSION}" \
      -o /out/websocket \
      ./cmd/websocket

# ── runtime ──────────────────────────────────────────────────────────────────
# distroless/static: no shell, no package manager, nonroot by default — minimal
# attack surface. Pinned to debian12.
FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /out/websocket /websocket

# Image default; the Terraform module overrides REEARTH_FLOW_WS_PORT (and the
# Cloud Run container_port) to 8080 in production. The binary binds
# REEARTH_FLOW_WS_PORT, NOT Cloud Run $PORT.
ENV REEARTH_FLOW_WS_PORT=8000
EXPOSE 8000

ENTRYPOINT ["/websocket"]

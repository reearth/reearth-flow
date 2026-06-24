# syntax=docker/dockerfile:1

# Multi-stage build for the Go Y-WebSocket server. Replaces the Rust
# server/websocket/Dockerfile.websocket. CGO is disabled so the binary is fully
# static and runs on distroless/static (no libc needed — the Rust image used
# cc-debian12 only because of its dynamic libssl link, finding G "deploy
# artifacts").
#
# PORT (finding G): the Rust server binds 0.0.0.0:{REEARTH_FLOW_WS_PORT}, default
# 8000, and EXPOSE 8000 — it does NOT read Cloud Run's $PORT (8080). The Go
# service replicates that: it listens on REEARTH_FLOW_WS_PORT (default 8000). We
# pin the env default here and EXPOSE 8000 so the container port matches the
# deployed Rust contract. The Cloud Run deploy step sets --port=8000 explicitly
# (see deploy/build_deploy_websocket.go.yml.draft).
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
# attack surface (finding I dependency-hygiene). Pinned to debian12.
FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /out/websocket /websocket

# Replicate the Rust deployed port contract (finding G): bind 8000, not $PORT.
ENV REEARTH_FLOW_WS_PORT=8000
EXPOSE 8000

ENTRYPOINT ["/websocket"]

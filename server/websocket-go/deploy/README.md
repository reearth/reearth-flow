# websocket-go deploy

The Go Y-WebSocket server deploys to the `reearth-flow-websocket-go` Cloud Run
service. The load balancer routes the `ws.` host to it in every environment.

`.github/workflows/build_deploy_websocket_go.yml` runs from `ci.yml` on pushes to
`main` and `release`.

## Files

| File | What it is |
|---|---|
| `../Dockerfile` | Multi-stage `golang:1.26` → `gcr.io/distroless/static-debian12` (CGO off, static). Binds `REEARTH_FLOW_WS_PORT`, **not** Cloud Run `$PORT`. The binary default is 8000. Terraform sets `REEARTH_FLOW_WS_PORT=8080` and `container_port=8080`, so it runs on **8080**. |
| `.github/workflows/build_deploy_websocket_go.yml` | Deploy: `go build` → `../Dockerfile` → push image `reearth/reearth-flow-websocket-go` → `gcloud run deploy reearth-flow-websocket-go` **image-only**. The Terraform module owns env, secrets, and container port. |
| `.github/workflows/ci_websocket_go.yml` | Go CI: gofmt, `go vet`, golangci-lint, `go test -race`, govulncheck. |
| `../docs/build-hygiene.md` | Go 1.26 linter/toolchain requirements and govulncheck triage. |
| `../docs/rust-test-coverage-map.md` | Historical Rust acceptance-test → Go-test coverage map, with open gaps. |

## Required GitHub Actions secrets

The deploy workflow uses `secrets: inherit` and reads `GC_SA_EMAIL`,
`GC_WORKLOAD_IDENTITY_PROVIDER`, `GC_REGION`, `DOCKERHUB_USERNAME`,
`DOCKERHUB_TOKEN`, and:

| Secret | Value |
|---|---|
| `WEBSOCKET_GO_IMAGE_GC` | GCP Artifact Registry path for the Go image (`.../reearth/reearth-flow-websocket-go`), matching the `image` the Terraform module points the service at. |

## Service config is owned by Terraform

The deploy is **image-only** (`gcloud run deploy --image …:nightly`). The
`reearth_flow_websocket` Terraform module in `eukarya-inc/infrastructure` sets
everything else: env vars, the Redis URL and API-secret bindings, and the
container port. The module `ignore_changes` the image, so `terraform apply` never
fights this deploy. `gcloud run deploy --image` rolls a new revision and keeps
that config, so this workflow must not re-set env, port, or secrets.

Cross-service invariant (owned by Terraform via `api_secret_id`): the Go service's
`REEARTH_FLOW_API_SECRET` must equal the API server's
`REEARTH_FLOW_WEBSOCKET_API_SECRET`, or the API's document HTTP calls return 401.

Every connection is verified against the flow API's `/auth/verify` at
`REEARTH_FLOW_THRIFT_AUTH_URL`. There is no switch to turn this off.

## Phase-2 `{projectId}/` GCS layout

Set `REEARTH_FLOW_GCS_PHASE2=true` on the service (new revision) to enable it. The
adapter dual-reads new-prefix → legacy-root and lazily backfills the legacy base
on first write. Backfill wiring at startup and activation is a follow-up before
relying on it broadly. Phase-2 changes the keyspace, so it was gated on no Rust
server reading or writing the same bucket.

## Workflow versions

The server saves a room's document as a version (an entry in the editor's
Version panel) only when users changed it, once per change across instances,
and records who did. See `internal/versions`.

| Env var | Meaning |
|---|---|
| `REEARTH_FLOW_AUTO_VERSION_EVERY` | How long a room's first unsaved edit waits before the room is saved as an `auto` version, measured across instances (default `15m`; `0` turns this trigger off). Before Flow saved versions itself, it meant "at most once per interval per instance". |
| `REEARTH_FLOW_KEEP_SNAPSHOTS` | Versions kept per room in each of two classes: `auto`, and every other label together (default `50`, at least `1`). Before, it bounded `auto` versions only. |

Versions are also saved when the last instance lets a room go, and on request:

- `POST /api/document/{id}/versions` with `{"label": "…"}` (behind
  `X-API-Secret`, label required, at most 256 bytes) answers
  `{"saved": true, "snapshotId": n}`, or `{"saved": false}` when no user edit
  is waiting. The API calls it on Ctrl+S, debug runs and deployments, with a
  fixed label for each.

After every version the server tells the API at
`POST {REEARTH_FLOW_THRIFT_AUTH_URL}/auth/versions` with `X-API-Secret` set to
`REEARTH_FLOW_API_SECRET`. The body and the rules the API must follow are on
`versions.CallbackPath`. In short: require the secret; notices are unordered,
retried and can repeat a version with grown attribution, so apply one only when
its `lastEditedAt` is later than what the project holds.

Change records live in the coordination store: a Redis hash per room
(`flow:version:changes:{room}`, 30-day TTL) or `ws_version_change` rows in
Postgres (swept after 30 days).


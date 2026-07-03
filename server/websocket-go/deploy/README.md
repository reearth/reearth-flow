# websocket-go deploy & blue-green runbook

The Go Y-WebSocket server deploys to its **own** Cloud Run service,
`reearth-flow-websocket-go`, running **in parallel** with the Rust
`reearth-flow-websocket` during the migration. Both share the same GCS bucket and
Redis, so a document is consistent whichever server handles it (coexistence).

The deploy pipeline is **active** — `.github/workflows/build_deploy_websocket_go.yml`,
dispatched from `ci.yml` on pushes to `main`/`release` (same gating as the Rust
deploy). It is **additive**: the Rust `build_deploy_websocket.yml` and service are
untouched. Deploying the Go service does **not** move any user traffic — traffic
only reaches it once the load balancer is pointed at it, which is the human-run
cutover below.

## Files

| File | What it is |
|---|---|
| `../Dockerfile` | Multi-stage `golang:1.25` → `gcr.io/distroless/static-debian12` (CGO off, static). Binds `REEARTH_FLOW_WS_PORT` (default **8000**), `EXPOSE 8000` — **not** Cloud Run `$PORT`. |
| `.github/workflows/build_deploy_websocket_go.yml` | Active deploy: `go build` → `../Dockerfile` → push image `reearth/reearth-flow-websocket-go` → `gcloud run deploy reearth-flow-websocket-go`, codifying the full runtime env + `--port 8000` in the deploy step (a fresh service inherits no out-of-band env). |
| `.github/workflows/ci_websocket_go.yml` | Active Go CI: gofmt, `go vet`, golangci-lint, `go test -race`, govulncheck. |
| `../docs/build-hygiene.md` | golangci-lint-for-go1.25 resolution + govulncheck triage. |
| `../docs/rust-test-coverage-map.md` | Rust acceptance-test → Go-test coverage map + gaps. |

## Required GitHub Actions secrets / vars

The deploy workflow uses `secrets: inherit`. It **reuses** the shared infra
secrets/vars the Rust deploy already relies on: `GC_SA_EMAIL`,
`GC_WORKLOAD_IDENTITY_PROVIDER`, `GC_REGION`, `DOCKERHUB_USERNAME`,
`DOCKERHUB_TOKEN`, `WS_REDIS_URL_SECRET` (shared Redis), and every `vars.WS_*`
config value (shared bucket, origins, DoS caps, OTEL knobs — same values as Rust).

Two **new** repo secrets must exist before the first `nightly`/`main` deploy (they
point at the Go service's own resources, provisioned by
`eukarya-inc/infrastructure`):

| Secret | Value |
|---|---|
| `WEBSOCKET_GO_IMAGE_GC` | GCP Artifact Registry path for the Go image (the `reearth-flow-websocket-go` repo), mirroring `WEBSOCKET_IMAGE_GC` for Rust. |
| `WS_GO_API_SECRET` | Secret Manager name of the Go service's API secret (`reearth-flow-websocket-api-secret`). Its **value MUST equal** the API server's `REEARTH_FLOW_WEBSOCKET_API_SECRET`, or the API's document HTTP calls to the Go service 401. |

Until both exist the deploy job fails loudly at the push/tag step — it never
touches the Rust service.

## Runtime env contract (codified in the deploy step)

A fresh Cloud Run service does not inherit out-of-band env, so the deploy step sets
every var the Go binary reads (names match `internal/config/config.go`):

- **Secrets** (`--set-secrets`): `REEARTH_FLOW_REDIS_URL` (= `WS_REDIS_URL_SECRET`),
  `REEARTH_FLOW_API_SECRET` (= `WS_GO_API_SECRET`; cross-service invariant above).
- **Config** (`--set-env-vars`, `^@@^`-delimited so commas in values aren't split):
  `REEARTH_FLOW_WS_PORT=8000`, `_APP_ENV=production`, `_GCS_BUCKET_NAME`,
  `_GCS_ENDPOINT=` (empty ⇒ real GCS), `_THRIFT_AUTH_URL`, `_ORIGINS`,
  `_WS_PROTECTED=false`, `_GCS_PHASE2=false`, the
  `_MAX_CONNECTIONS`/`_MAX_PEERS_PER_ROOM`/`_MAX_ROOMS` caps, and the OTEL knobs.
  Set `_LOG_LEVEL=debug` to surface ygo + relay detail.
- **Container port:** `--port 8000` (the binary binds `REEARTH_FLOW_WS_PORT`, not
  `$PORT`).

## Blue-green cutover checklist (HUMAN-RUN)

> Preconditions (hard go/no-go gates): the acceptance gate is green
> (`../docs/rust-test-coverage-map.md`), `/health` returns 200 on the Go service,
> and protected mode is OFF for the overlap. Live race signals to watch through
> every ramp step: evict race (0 lost updates on reconnect-during-evict), stale
> canvas (editor unfreezes post-rollback), spurious future versions (none survive
> rollback), position drift (no cursor jump under cross-instance load).

Traffic is shifted at the **load balancer** (routing user WS traffic from the Rust
service to the Go service), NOT via Cloud Run revision tags — the two servers are
independent services.

1. **Provision + wire infra.** Ensure `reearth-flow-websocket-go` exists with a
   backend/NEG on the LB (see `eukarya-inc/infrastructure`), and that
   `WEBSOCKET_GO_IMAGE_GC` + `WS_GO_API_SECRET` are set (above). A push to `main`
   then deploys the Go service live on its own URL (0 user traffic).

2. **Smoke the Go service directly** (its own Cloud Run URL, before any LB change):
   `/health` → 200; open a WS room served by a Rust instance and confirm it syncs
   into the Go instance over the shared Redis stream (cross-instance fan-out), and
   that awareness propagates both directions.

3. **Canary 5–10%** at the LB (point a small share of the WS backend at the Go
   service). Watch the four race gates as live signals.

4. **Ramp 25 → 50 → 100%**, holding at each step for ≥1 GCS-flush + ≥1
   heartbeat-expiry window (~2 min) and re-running the regression smoke. Any red
   gate ⇒ roll back (below).

5. **Drain Rust.** At 100% Go, keep the Rust service running but LB-detached for ≥1
   full retention window (instant restore). Verify no Rust instance still holds the
   last-active-instance election for any live doc (all `doc:instances:{doc}`
   heartbeats are Go).

6. **Decommission Rust.** Remove the Rust service from the LB and stop deploying it.
   Only now is "Rust fully drained" true (precondition for Phase-2 + protected mode).

7. **Enable Phase-2 `{projectId}/` layout.** Set `REEARTH_FLOW_GCS_PHASE2=true` (new
   revision). The adapter dual-reads new-prefix → legacy-root and lazily backfills
   the legacy base on first write; backfill wiring at startup/activation is a
   follow-up before relying on it broadly. **Only after step 6** (Phase-2 changes
   the keyspace; Rust must not be reading/writing it).

8. **(Separate launch) Flip auth ON.** Set `REEARTH_FLOW_WS_PROTECTED=true` on its
   own gate with `REEARTH_FLOW_THRIFT_AUTH_URL` verified reachable; canary again.
   Pre-req: add the e2e upgrade-rejection test (gap G3 in the coverage map).

**Rollback (any gate):** point the LB WS backend back at the Rust service (100%).
Both wrote byte-identical Redis/GCS state, so there is no data rollback. If
protected mode was the regression, roll back the `REEARTH_FLOW_WS_PROTECTED` env
flag only. Keep the Rust service attachable for ≥1 retention window before removal.
**Do not start Phase-2 or enable protected mode until Rust is fully decommissioned
and all gates are green.**

## What stays HUMAN-DRIVEN (not done by CI)

- LB routing changes (attach the Go backend/NEG, ramp, drain) — all at the load
  balancer, per `eukarya-inc/infrastructure`.
- The go/no-go gate observation at each ramp step.
- Enabling Phase-2 and flipping protected mode ON — each behind its own gate.

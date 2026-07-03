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
| `../Dockerfile` | Multi-stage `golang:1.25` → `gcr.io/distroless/static-debian12` (CGO off, static). Binds `REEARTH_FLOW_WS_PORT` — **not** Cloud Run `$PORT`. Binary default is 8000, but Terraform sets `REEARTH_FLOW_WS_PORT=8080` + `container_port=8080` in prod, so it runs on **8080**. |
| `.github/workflows/build_deploy_websocket_go.yml` | Active deploy: `go build` → `../Dockerfile` → push image `reearth/reearth-flow-websocket-go` → `gcloud run deploy reearth-flow-websocket-go` **image-only** (like the Rust workflow). Env, secrets, and container port are owned by the Terraform module, not this step. |
| `.github/workflows/ci_websocket_go.yml` | Active Go CI: gofmt, `go vet`, golangci-lint, `go test -race`, govulncheck. |
| `../docs/build-hygiene.md` | golangci-lint-for-go1.25 resolution + govulncheck triage. |
| `../docs/rust-test-coverage-map.md` | Rust acceptance-test → Go-test coverage map + gaps. |

## Required GitHub Actions secrets / vars

The deploy workflow uses `secrets: inherit` and mirrors the Rust deploy's secret
usage. It **reuses** the shared secrets the Rust build/deploy already relies on:
`GC_SA_EMAIL`, `GC_WORKLOAD_IDENTITY_PROVIDER`, `GC_REGION`, `DOCKERHUB_USERNAME`,
`DOCKERHUB_TOKEN`.

One **new** repo secret must exist before the first `nightly`/`main` deploy — the
exact analog of the Rust `WEBSOCKET_IMAGE_GC`:

| Secret | Value |
|---|---|
| `WEBSOCKET_GO_IMAGE_GC` | GCP Artifact Registry path for the Go image (`.../reearth/reearth-flow-websocket-go`), matching the `image` the Terraform module points the service at. |

Until it exists the deploy job fails loudly at the push step — it never touches the
Rust service.

## Service config is owned by Terraform (not this workflow)

The deploy is **image-only** (`gcloud run deploy --image …:nightly`), exactly like
the Rust workflow. Everything else about the service — env vars, the Redis URL and
API-secret bindings, and the container port (**8080**, with
`REEARTH_FLOW_WS_PORT=8080`) — is set by the `reearth_flow_websocket` Terraform
module in `eukarya-inc/infrastructure`, which `ignore_changes` the image so
`terraform apply` never fights this deploy. `gcloud run deploy --image` rolls a new
revision and preserves that config, so this workflow must not re-set env/port/
secrets (doing so would diverge from Rust and flap against terraform apply).

Cross-service invariant (owned by Terraform via `api_secret_id`): the Go service's
`REEARTH_FLOW_API_SECRET` MUST equal the API server's
`REEARTH_FLOW_WEBSOCKET_API_SECRET`, or the API's document HTTP calls 401.

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

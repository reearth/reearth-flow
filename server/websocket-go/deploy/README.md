# websocket-go deploy artifacts & blue-green runbook

This directory holds **draft, not-yet-activated** deploy artifacts for the Go
Y-WebSocket server and the **human-run** cutover checklist. The live Rust
workflows under `.github/workflows/` are intentionally untouched.

## Files

| File | What it is |
|---|---|
| `../Dockerfile` | Multi-stage `golang:1.25` → `gcr.io/distroless/static-debian12` (CGO off, static). Binds `REEARTH_FLOW_WS_PORT` (default **8000**), `EXPOSE 8000` — replicates the Rust deployed-port contract (finding G), **not** Cloud Run `$PORT`. |
| `build_deploy_websocket.go.yml.draft` | Proposed replacement for `.github/workflows/build_deploy_websocket.yml`: cargo→go build, builds `../Dockerfile`, **keeps** image `reearth/reearth-flow-websocket` + service `reearth-flow-websocket` + all registry secrets, deploys **green at 0% traffic**, and **codifies the full runtime env + `--port 8000`** in the deploy step (defeats the Cloud-Run env-preservation hazard, finding G §4). |
| `ci_websocket.go.yml.draft` | Proposed Go CI: gofmt, `go vet`, golangci-lint **v2.4.0** (go1.25-built), `go test -race`, govulncheck. |
| `../docs/build-hygiene.md` | golangci-lint-for-go1.25 resolution + govulncheck triage. |
| `../docs/rust-test-coverage-map.md` | Rust acceptance-test → Go-test coverage map + gaps. |

## Runtime env contract (codified in the deploy step)

A fresh/swapped Cloud Run service does **not** inherit out-of-band env (finding G
§4). The deploy draft therefore sets every var the Go binary reads. Names match
the live Rust vars (`internal/config/config.go`):

- **Secrets** (`--set-secrets`, from Secret Manager): `REEARTH_FLOW_REDIS_URL`,
  `REEARTH_FLOW_API_SECRET`.
  - **Cross-service invariant:** `REEARTH_FLOW_API_SECRET` MUST equal the API
    server's `REEARTH_FLOW_WEBSOCKET_API_SECRET`, or the API's document HTTP calls
    get 401.
- **Config** (`--set-env-vars`): `REEARTH_FLOW_WS_PORT=8000`, `_APP_ENV`,
  `_GCS_BUCKET_NAME`, `_GCS_ENDPOINT` (empty ⇒ real GCS), `_THRIFT_AUTH_URL`,
  `_ORIGINS` (incl. prod `*.netlify.app` allow-list — use the `^@@^` delimiter so
  commas in the list aren't split), `_WS_PROTECTED=false`, `_GCS_PHASE2=false`,
  the `_MAX_CONNECTIONS`/`_MAX_PEERS_PER_ROOM`/`_MAX_ROOMS` DoS caps, and the OTEL
  knobs (`_ENABLE_OTLP`, `_OTLP_ENDPOINT`, `_GCP_PROJECT_ID`, `_SERVICE_NAME`,
  `_OTEL_EXPORTER_TYPE`, `_OTEL_SAMPLING_RATIO`, `_OTEL_BATCH_TIMEOUT`,
  `_OTEL_MAX_EXPORT_BATCH_SIZE`, `_OTEL_MAX_QUEUE_SIZE`).
- **Container port:** `--port 8000` (the Go binary binds `REEARTH_FLOW_WS_PORT`,
  not `$PORT`). **Confirm the live service's configured container port before
  cutover** — if it's already 8000, this is a no-op; if not, this aligns it.

## Blue-green cutover checklist (HUMAN-RUN — finding G §8)

> Preconditions (hard go/no-go gates): the global acceptance gate is green
> (`../docs/rust-test-coverage-map.md`), `/health` returns 200 on green, the env
> contract + port are codified above, and protected mode is OFF for the overlap.
> The known races are live go/no-go signals: evict race (0 lost updates on
> reconnect-during-evict), stale canvas (editor unfreezes post-rollback), spurious
> future versions (none survive rollback), position drift (no cursor jump under
> cross-instance load).

0. **ygo is already pinned (done).** `server/websocket-go/go.mod` requires
   `github.com/reearth/ygo v1.22.0` from the module proxy, with no local-path
   `replace`, so the Docker build resolves ygo from the proxy. No action is
   needed here; this step is kept only to record that the precondition the draft
   workflow assumes is already satisfied.

1. **Activate the workflows.** Rename `build_deploy_websocket.go.yml.draft` →
   `.github/workflows/build_deploy_websocket.yml` and `ci_websocket.go.yml.draft`
   → `.github/workflows/ci_websocket.yml`, replacing the Rust versions. Wire the
   new `vars.WS_*` / `secrets.WS_*` referenced in the deploy step.

2. **Deploy green at 0% traffic.** The deploy job runs
   `gcloud run deploy … --no-traffic --tag green --port 8000 --set-env-vars … --set-secrets …`.
   Smoke the tagged URL: `/health` → 200; open a WS room and confirm a doc served
   by a Rust instance syncs into the green Go instance over the shared Redis
   stream (cross-instance fan-out), and that awareness propagates both directions.

3. **Canary 5–10%.**
   `gcloud run services update-traffic reearth-flow-websocket --to-tags green=10`.
   Watch the four race gates above as live signals.

4. **Ramp 25 → 50 → 100%**, holding at each step for ≥1 GCS-flush + ≥1
   heartbeat-expiry window (~2 min), re-running the regression smoke. Any red gate
   ⇒ roll back (step R).

5. **Drain Rust.** At 100% Go, keep the Rust revision deployed-but-0% for ≥1 full
   retention window (instant restore). Verify no Rust instance still holds the
   last-active-instance election for any live doc (all `doc:instances:{doc}`
   heartbeats are Go).

6. **Decommission Rust.** Delete the Rust revision. Only now is "Rust fully
   drained" true (precondition for Phase-2 + protected mode).

7. **Enable Phase-2 `{projectId}/` layout.** Set
   `REEARTH_FLOW_GCS_PHASE2=true` (new revision). The adapter dual-reads
   new-prefix → legacy-root; lazy-backfill on first open of legacy projects;
   verify `state_vector(new) == state_vector(legacy)` for a sample before pruning
   legacy. Drops the global OID index + `lock:oid_generation`. **Only after step
   6** (Phase 2 changes the keyspace; Rust must not be reading/writing).

8. **(Separate launch) Flip auth ON.** Set `REEARTH_FLOW_WS_PROTECTED=true` on its
   own gate with `REEARTH_FLOW_THRIFT_AUTH_URL` verified reachable; canary again.
   This is the first time WS auth is live in prod (the UI already sends `?token=`).
   Pre-req: add the e2e upgrade-rejection test (gap G3 in the coverage map).

**Rollback (any gate):**
`gcloud run services update-traffic reearth-flow-websocket --to-revisions <rust-revision>=100`.
Both wrote byte-identical Redis/GCS state, so there is no data rollback. If
protected mode was the regression, roll back the `REEARTH_FLOW_WS_PROTECTED` env
flag only. Keep the Rust revision at 0% for ≥1 retention window before deletion.
**Do not start Phase-2 or enable protected mode until Rust is fully decommissioned
and all gates are green.**

## What stays HUMAN-DRIVEN (not done by the code workstream)

- Push the ygo branch, open/merge the ygo PR, tag the ygo release.
- Drop the `replace` directive + pin `require github.com/reearth/ygo vX.Y.Z`;
  `go mod tidy`.
- Activate (rename) the draft workflows over the live Rust ones; wire `WS_*`
  vars/secrets.
- The actual `gcloud` rollout: green deploy, traffic ramp, drain, decommission,
  Phase-2 enable, protected-mode flip — each behind its own go/no-go gate.

# Preview Schema — server/api support design

**Date:** 2026-06-16
**Status:** Approved design (pre-implementation)
**Owners:** server/api (lead), engine (gating dependency), ui (consumer)

## 1. Summary

Expose the engine's dynamic `probe-schema` capability to the UI as a **"Preview Schema"**
step. While editing a workflow in the visual builder, a user triggers a preview; the server
runs the engine's per-node attribute-schema probe against the **live editor graph** and
returns the resulting `SchemaReport` to the UI.

Execution **mirrors the debug-run orchestration** — a Job dispatched through the Cloud Run
worker (GCP Batch fallback), status via the existing job monitoring + subscriptions, the report
delivered as a **GCS artifact URL** on the completed Job — but on a **dedicated path**. It does
**not** reuse `runProject`/`Project.Run`, because the worker invokes a distinct `probe-schema`
subcommand rather than `run`. Only the orchestration shape is shared, not the run code path.

### Vocabulary mapping (deliberate split)

| Layer | Term |
|-------|------|
| Engine CLI | `probe-schema` (unchanged) |
| Server GraphQL | `previewSchema` mutation, job `Mode = preview-schema` |
| UI | "Preview Schema" |

The engine command name stays `probe-schema`; server + UI speak "Preview Schema". This is
intentional and documented here so the divergence is not mistaken for a bug.

## 2. Background: what `probe-schema` produces

The engine `probe-schema` subcommand (merged to `main` in PR #2134) walks a workflow DAG in
topological order, **samples the first N features** from each source reader to discover real
attribute names/types, and propagates transforms downstream. It prints a JSON `SchemaReport` to
stdout. It is read-only and has no side effects. Per-source failures degrade to an `open`
schema with a `note` rather than failing the whole probe.

`SchemaReport` shape (stable, versioned):

```jsonc
{
  "version": 1,
  "sampleSize": 10,
  "nodes": {
    "<node-id>": {
      "name": "buildings_reader",
      "ports": {
        "default": {
          "open": false,
          "fields": [
            { "name": "id",   "type": "String", "presence": "always" },
            { "name": "year", "type": "Number", "presence": "maybe" }
          ]
        }
      },
      "note": "source run failed: ..."   // optional; present only on failure
    }
  }
}
```

- `type`: `Bool|Number|String|DateTime|Array|Map|Bytes|Null|Unknown` (PascalCase).
- `presence`: `always|maybe` (lowercase).
- `open: true` + empty `fields` = couldn't enumerate (e.g. a source that failed to sample).
- Output is on **stdout**; engine logs go to **stderr**.

Datasets in real workflows are expressed as `env.get("var")` and resolved from the workflow's
`with:` vars merged with `--var` (the server passes editor parameters as `--var`).

## 3. Decisions (locked during brainstorming)

| # | Decision | Choice |
|---|----------|--------|
| D1 | Capability | **Dynamic** probe (reads/samples data, real per-node schema). Not the static build pass. |
| D2 | Probe source | **Live editor graph** (flush Yjs→GCS, UploadWorkflow). Not saved deployments. |
| D3 | Execution model | **Exactly like debug runs**: Job-based, Cloud Run debug worker (Batch fallback), async status via subscriptions. |
| D4 | Result delivery | `SchemaReport` JSON written to **GCS**, surfaced as a **result URL** on the completed Job; UI fetches + parses. |
| D5 | Name | **Preview Schema** → `previewSchema` mutation. |
| D6 | Integration approach | **B — dedicated preview pipeline**. `previewSchema` does NOT reuse `runProject`/`Project.Run`; the worker runs a distinct `probe-schema` subcommand via its own route. Only the debug-run *orchestration* (Job, worker dispatch, monitoring, subscriptions, GCS artifact) is mirrored. |
| D7 | Engine packaging | `probe-schema` is exposed as a **`reearth-flow-worker` subcommand** (`reearth-flow-worker probe-schema`), not a separate CLI shipped in the image. The worker binary gains subcommands (`run` vs `probe-schema`). |

### Defaults adopted (open to revision during spec review)

- **D4a** — dedicated nullable `Job.previewSchemaUrl: URL` field (clearer UI contract) rather
  than reusing `outputURLs`.
- **D6a** — when no Cloud Run debug worker is configured, fall back to **GCP Batch** exactly
  like `Project.Run` (async, slower, still works) rather than erroring "preview unavailable".
- **D6b** — a **persisted Job** is created (consistent with debug runs), but preview jobs are
  **excluded from normal `jobs(...)` queries** so they don't pollute run history.

## 4. Scope

**In (v1):**
- Probe the live editor graph for a project.
- Job-based execution via the debug-run pipeline (Cloud Run worker, Batch fallback).
- Per-node `SchemaReport` delivered as a GCS artifact URL on the Job.
- `sampleSize` override (optional, server default + cap) and workflow variables.

**Out (v1):**
- Probing saved deployments / versions.
- Per-node streaming of schema (one report at completion).
- Persisting/caching reports beyond the GCS artifact + Job.
- A dedicated RBAC resource (reuse the run path's `ActionEdit`).
- The engine's **static** build/infer pass.

## 5. GraphQL contract (UI integration surface)

New file `server/api/gql/previewSchema.graphql`, then `make gql`:

```graphql
input PreviewSchemaInput {
  projectId: ID!
  workspaceId: ID!
  file: Upload!                    # the editor graph, same as runProject
  parameters: [...]                # reuse RunProjectInput's existing parameter type → engine --var
  sampleSize: Int                  # optional; server default (10) + capped
}

type PreviewSchemaPayload {
  job: Job!
}

extend type Mutation {
  previewSchema(input: PreviewSchemaInput!): PreviewSchemaPayload!
}
```

- Returns a **Job** immediately (mirrors `runProject` / `RunProjectPayload`).
- `Job` type gains a nullable field **`previewSchemaUrl: URL`**, populated only when a
  preview job reaches `COMPLETED`.
- The UI flow is identical to a debug run plus one fetch:
  1. `previewSchema(input)` → `Job`.
  2. Subscribe to the **existing** `jobStatus(jobId)` subscription.
  3. On `COMPLETED`, read `job.previewSchemaUrl`, GET the JSON from GCS, parse the versioned
     `SchemaReport`.
  4. On `FAILED`, surface the error (logs available via the existing `logs(jobId)` channel).

Because the graph is sent as `Upload` (multipart), this must be a **Mutation** (gqlgen does
not support `Upload` on a Query).

## 6. Server execution path (Approach B — dedicated pipeline)

A **dedicated** path that *mirrors the debug-run orchestration* but does **not** reuse the run
interactor/mutation. It may reuse generic low-level helpers (`FlushToGCS`, `UploadWorkflow`, job
create/save, `StartMonitoring`) — those are not run-specific — but it is its own method, not a
flag threaded through `Project.Run`. Strict clean architecture layering: resolver → interface →
interactor → gateway → infrastructure. Reference (for orchestration shape only):
`Project.Run` at `server/api/internal/usecase/interactor/project.go:177`.

### 6.1 Resolver
- `server/api/internal/adapter/gql/resolver_mutation_previewSchema.go` (new): parse IDs
  (`gqlmodel.ToID[...]`), `gqlmodel.FromFile(&input.File)`, call
  `usecases(ctx).Project.PreviewSchema(ctx, param)`, wrap in `PreviewSchemaPayload`. Thin,
  modeled on `resolver_mutation_project.go:68` (`RunProject`).

### 6.2 Usecase interface
- `server/api/internal/usecase/interfaces/project.go`: add
  `PreviewSchema(ctx, PreviewSchemaParam) (*job.Job, error)` + a `PreviewSchemaParam` struct
  (`ProjectID`, `Workspace`, `Workflow *file.File`, `Variables`, `SampleSize`).

### 6.3 Interactor
- Add a **dedicated** `PreviewSchema` interactor method (its own code path — NOT a flag on
  `Run`). Placement: either a new `interactor/previewschema.go` or alongside the Project
  interactor; it reuses generic helpers but never calls `Run`. Steps:
  - `checkPermission(ctx, ActionEdit)` — same permission as the run path.
  - `websocket.FlushToGCS` + `projectRepo.FindByID` + `websocket.GetLatest` (snapshot+version).
  - Build a `job.Job` with `Debug=true` and `Mode = preview-schema`.
  - `file.UploadWorkflow(workflow)` → URL. **No** `UploadMetadata` (probe-schema does not
    consume metadata).
  - `jobRepo.Save`.
  - Dispatch via a **dedicated gateway method** (§6.5) that targets the worker's
    `probe-schema` route, then `StartMonitoring`. If the worker gateway is unconfigured, fall
    back to Batch invoking the `probe-schema` subcommand (D6a).

### 6.4 Domain / persistence
- `server/api/pkg/job/job.go`: add a `Mode` discriminator (`run` | `preview-schema`) and the
  `previewSchemaUrl` field (+ getter/setter/builder).
- `server/api/internal/infrastructure/mongo/mongodoc/job.go`: persist the new fields. Job is
  **Mongo-only** today (Postgres port not yet done — see `infrastructure/postgres/container.go`),
  so no Postgres repo work is required.
- Job list queries (`repo/job.go` filters / the `jobs(...)` resolver): exclude
  `Mode = preview-schema` by default (D6b).

### 6.5 Gateway + worker transport
- `server/api/internal/usecase/gateway/cloudrunworker.go`: add a **dedicated**
  `PreviewSchema(ctx, ProbeParam) error` method (distinct from `RunJob`), where `ProbeParam`
  carries `JobID`, `WorkflowURL`, `Variables`, `SampleSize`. (A separate `gateway.PreviewSchema`
  interface is an acceptable alternative; a method on the worker gateway keeps one HTTP seam.)
- `server/api/internal/infrastructure/cloudrunworker/worker.go`: POST to a **dedicated worker
  route** (e.g. `/probe-schema`) with its own request struct (`job_id`, `workflow_url`,
  `variables`, `sample_size`) — NOT a `preview_schema` flag on the `/run` request. idtoken HTTP
  client reused.
- `server/api/internal/infrastructure/gcpbatch/batch.go`: Batch fallback builds the CLI string
  invoking the `probe-schema` subcommand (`reearth-flow-worker probe-schema --workflow ... --var ... --sample-size ...`).
- No new run-path coupling — `RunJob`/`runProject` are left untouched.

### 6.6 Result pickup
- The monitoring loop's `updateJobArtifacts`
  (`server/api/internal/usecase/interactor/job.go:462`) sets `previewSchemaUrl` from the known
  GCS path for preview jobs on completion.

## 7. Engine / worker changes (cross-team — the gating dependency)

`probe-schema` currently lives in the `reearth-flow-cli` binary, but the Cloud Run worker image
(`engine/Dockerfile.worker-trial`) ships only `reearth-flow-worker` + `reearth-flow-worker-server`.
Per **D7**, the engine work is:

1. **Worker subcommand** — add `probe-schema` as a subcommand of `reearth-flow-worker`
   (`engine/worker/src/main.rs` gains subcommand dispatch: `run` vs `probe-schema`), refactoring
   the probe logic so it is reachable from the worker binary. The image stays single-binary; no
   separate CLI is shipped.
2. **Dedicated worker route** — `engine/worker/src/bin/server.rs` gains a **new route**, separate
   from `/run` (e.g. `POST /probe-schema`), that invokes `reearth-flow-worker probe-schema`,
   **captures stdout via `Stdio::piped`** (the `/run` handler uses `Stdio::inherit`), writes the
   `SchemaReport` JSON to the job's GCS output path, and returns `COMPLETED`.
   `engine/worker/src/wrapper.rs` gets a `build_probe_args` builder (`--workflow`, repeatable
   `--var`, `--sample-size`) — distinct from `build_worker_args`.

This is the **critical path**. Server work (§5–§6, §8) can proceed in parallel against a mocked
gateway; integration requires (1)+(2) to land first.

## 8. Auth, limits, hygiene

- **Auth/scoping:** identical to debug runs — `checkPermission(ActionEdit)`. No new RBAC
  resource, no `make gen-policies`. (Note the existing run path's known looseness: it accepts
  `workspaceId` but doesn't verify the project belongs to it; preview should be at least as
  strict — consider verifying `prj.Workspace()` membership against the operator. Flagged, not
  blocking.)
- **Bounds:** `sampleSize` optional, server default `10` (engine default), with a hard cap to
  bound cost; worker-call timeout shorter than the 65-min run ceiling.
- **Job hygiene:** preview jobs excluded from `jobs(...)` listings (D6b).

## 9. Error handling

- **Bad/unreachable source:** `probe-schema` degrades that source to `open` + a `note`; the job
  still completes `COMPLETED` with a report containing notes — partial failures are data, not
  job failures.
- **Infra failure / timeout:** job → `FAILED` with a message, exactly like debug runs; logs via
  the existing `logs(jobId)` channel.
- **No debug worker configured:** Batch fallback (D6a).

## 10. Testing

- **Interactor:** `PreviewSchema` test with mocked repos + gateway, mirroring `project_test.go`
  Run tests (asserts Job created with `Mode=preview-schema`, the dedicated `PreviewSchema`
  gateway method invoked, metadata NOT uploaded, run path untouched).
- **Gateway:** `httptest`-based test asserting the request hits the `/probe-schema` route with
  `workflow_url`/`variables`/`sample_size` and parses the response, mirroring the existing
  `cloudrunworker` worker test.
- **Engine worker:** test for the dedicated `/probe-schema` route + `probe-schema` subcommand
  dispatch + stdout capture into a GCS path.
- **GraphQL:** schema compiles + `make gql` produces the expected models; resolver smoke test.

## 11. Cross-team coordination

1. **Engine (blocking):** `probe-schema` `reearth-flow-worker` subcommand + dedicated worker
   route (§7).
2. **Server:** §5–§10 — startable now against a mock gateway; integrate once §7 lands.
3. **UI:** `previewSchema` mutation → `jobStatus` subscription → fetch `previewSchemaUrl` →
   parse `SchemaReport` (§2 shape). Can integrate against the documented contract before the
   backend is wired (mock the mutation/job).

## 12. Open items for spec review

- D4a: dedicated `previewSchemaUrl` field vs reusing `outputURLs[0]`.
- D6a: Batch fallback vs "preview unavailable" error when no debug worker.
- D6b: persisted preview Job (excluded from lists) vs ephemeral / no Job.
- §8: tighten project↔workspace membership check beyond the existing run path's looseness?
- §6.5: dedicated `PreviewSchema` method on the worker gateway vs a fully separate
  `gateway.PreviewSchema` interface.
- `sampleSize` default (10) and hard cap value.

# 05 — Verifying all four issues locally

How to reproduce and verify every item in this plan **without GCP, MongoDB, pubsub, the API
server, or the frontend** — and without relying on any test we write.

The harness lives in [`repro/`](repro/). Everything below was run on `main` @ `e3d85ffae`,
engine `0.0.560`; the outputs quoted are real, not illustrative.

---

## Setup (once)

```bash
cd engine
cargo build -p reearth-flow-worker --bin reearth-flow-worker
```

## Run

```bash
cd engine/dev-docs/diagnostics-plan
./repro/run.sh repro/expr-fatal.yml
```

## Why this works

The worker writes the **full, uncapped `JobCompleteEvent`** — the exact object whose
`failedNodes` reaches the frontend — to a local file before it publishes anything
([`artifact.rs:16`](../../worker/src/artifact.rs#L16)):

```
~/Library/Caches/reearth.flow.workers/jobs/<job-id>/diagnostics.json
```

`--pubsub-backend noop` ([`backend.rs:40`](../../worker/src/pubsub/backend.rs#L40)) removes
the only cloud dependency in the path, and a `file://` `artifactBaseUrl` keeps the artifact
sweep local. Nothing between that file and the GraphQL response rewrites `message` — the Go
side copies it verbatim — so this file **is** the ground truth for all four issues.

The script also counts `"Error operation"` lines in the worker log, which is how many times
the action *actually* failed. The gap between that number and the number of diagnostics is
issue [04](04-fatal-occurrence-count.md) made visible.

---

## The fixtures

| File | Reproduces |
|---|---|
| `expr-fatal.yml` | The reported payload: `ExecutionError(Processor(Diagnostic {...}))` |
| `expr-factory-error.yml` | The build-time shape: `ExecutionError(Factory {...})`, `nodeId: null` |
| `expr-fatal-continue.yml` | Same failure under `onFatal: continue` — **the oracle**, see below |
| `expr-fatal-override.yml` | Proves `errorPolicy` overrides are silently ignored |
| `expr-factory-error-continue.yml` | Proves `onFatal: continue` does **not** rescue build-time errors |

All four feed 5 features into a Statistics Calculator whose expression references a
non-existent attribute.

---

## ⭐ The oracle: `onFatal: continue` already produces the target output

This is the most useful thing the harness revealed. The **same failure** under
`onFatal: continue` takes the `fold_outcomes` path, which downcasts the diagnostic correctly
— so it already shows what [01](01-worker-diagnostic-recovery.md) must make the default path produce.

```bash
./repro/run.sh repro/expr-fatal.yml           # broken (default: onFatal terminate)
./repro/run.sh repro/expr-fatal-continue.yml  # correct (the target)
```

| field | `terminate` (default, broken) | `continue` (target) |
|---|---|---|
| `message` | `ExecutionError(Processor(Diagnostic { code: ... }))` | `StatisticsCalculator error: Failed to evaluate expression for attribute 'mean_height_m': ...attribute 'test' not found` |
| `featureId` | `null` | `"92602c4e-152f-409f-a410-2f7dcfb0378a"` |
| `actionType` | `"StatisticsCalculator"` ← the **node name** | `"Statistics Calculator"` ← the **action type** |
| `code` | `internal.unclassified` | `internal.unclassified` (unchanged — that is [03](03-expression-error-classification.md)) |
| `aggregated` | `null` | `null` (unchanged — that is [04](04-fatal-occurrence-count.md)) |

**Acceptance test for [01](01-worker-diagnostic-recovery.md): the two files' `failedNodes`
become identical apart from `jobId`/`timestamp`.**

> The oracle's reach is limited to errors that pass through `join()`. Run
> `./repro/run.sh repro/expr-factory-error-continue.yml`: a build-time error under
> `onFatal: continue` still Debug-dumps, because it is raised before any node thread starts.
> So the oracle defines 01's *target*, but 01 must also fix the paths the oracle cannot reach.

```bash
diff <(python3 -m json.tool ...terminate.../diagnostics.json | grep -v '"jobId"\|"timestamp"') \
     <(python3 -m json.tool ...continue.../diagnostics.json  | grep -v '"jobId"\|"timestamp"')
```

> Incidental finding from this comparison: the wire field **`actionType` currently carries the
> node's `name`, not its action type**, because `failure_summary` fills it from
> `FailedNode.name` (the `ProcessorFailed` event's `name`, which is `node_name`).
> Fix 01 corrects this for free by using the recovered diagnostic's real `action_type`.
> Add an assertion for it.

---

## Per-issue verification

### 01 — Worker discards the real Diagnostic

```bash
./repro/run.sh repro/expr-fatal.yml
```
- **Before:** `message` starts `ExecutionError(Processor(Diagnostic {`; `featureId` is `null`.
- **After:** `message` is the plain error chain; `featureId` is populated; matches the oracle.
- Also run `repro/expr-factory-error.yml` — it has no boxed `Diagnostic`, so it must fall
  through to the rendered chain **without** a doubled message, and must not regress.

### 02 — `Terminate` discards the run summary

- **Before:** `aggregatedDiagnostics` is `[]` on every failed run, even when other nodes
  produced warnings.
- **After:** populated. To make this visible, extend a fixture with a second node that emits
  warn-level diagnostics (e.g. a geometry action fed bad geometry) and confirm its warnings
  survive a failure elsewhere in the graph.
- Also: add a second failing node and confirm **both** appear in `failedNodes`.

### 03 — Expression failures are unclassified

```bash
./repro/run.sh repro/expr-fatal.yml
```
- **Before:** `"code": "internal.unclassified"`, `"category": "internal"`, generic `help`.
- **After:** `"code": "expression.evaluation_failed"`, `"category": "expression"`, and `help`
  that names the likely cause.

**And the part that actually matters** — `errorPolicy` must start working:

```bash
./repro/run.sh repro/expr-fatal-override.yml
```

That fixture sets `allowRelaxInternal: true` and an override mapping the failing node to
`warn_drop`. **Verified today: the override validates, compiles, and has *zero* effect** —
the run still fails fatally with byte-identical output, because
[`synthesize_process_error_fatal`](../../runtime/runtime/src/executor/processor_node.rs#L635)
hardcodes `Fatal` and never calls `resolve()`.

After 03, this fixture should change behaviour: `result` becomes `success` (or the diagnostic
moves to `aggregatedDiagnostics` as a `warn_drop`). **That is the regression test for 03
being load-bearing rather than cosmetic.**

### 04 — Occurrence count

```bash
./repro/run.sh repro/expr-fatal.yml
```
- The script prints `5` (error log lines) against `1` diagnostic with `"aggregated": null`.
- **After A (frontend):** the UI stops rendering `null` as `1`.
- **After B (engine):** `"aggregated": { "count": 5, ... }`.
- The fixture emits exactly 5 features, so the expected count is exactly 5 — vary the
  `creator` array to check the count tracks.

---

## Notes

- Each run uses a **fresh job id**, so runs never clobber each other and you can diff any two.
- Job dirs accumulate under `~/Library/Caches/reearth.flow.workers/jobs/`
  (`~/.cache/...` on Linux). Delete freely.
- The script never fails the shell on a failed workflow — these fixtures are *supposed* to fail.
- `--pubsub-backend noop` is what makes this offline. The default is `google` and will try to
  authenticate.

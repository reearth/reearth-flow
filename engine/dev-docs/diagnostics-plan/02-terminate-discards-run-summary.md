# 02 — `Terminate` discards the run summary

**Scope:** `engine/runtime/runtime/src/executor/dag_executor.rs`, the runner's public
`Result<RunSummary, Error>` contract, golden-log tests.
**Fixes:** other failed nodes, `aggregatedDiagnostics`.
**Largest blast radius of the four. Do this last, on its own branch.**

---

## The defect

[`dag_executor.rs:377`](../../runtime/runtime/src/executor/dag_executor.rs#L377):

```rust
if self.disposition_policy.on_fatal() == OnFatalInput::Terminate {
    if let Some(pos) = results.iter().position(|(_, (_, result))| result.is_err()) {
        let (_, (_, result)) = results.remove(pos);
        return Err(result.expect_err("position() above guarantees Err"));
    }
}
Ok(fold_outcomes(results))
```

The `while` loop above has already joined **every** node thread, so `results` holds the
complete outcome set — every node's error *and* every node's drained warn/reject buckets.
The `Terminate` branch then returns the first `Err` and drops the entire vector.

Consequences on the default policy:

1. Only the first failing node is ever reported. Other failed nodes survive only as
   `NodeFailureHandler` rows, which carry no diagnostic of their own (see [01](01-worker-diagnostic-recovery.md)).
2. **Every aggregated diagnostic in the run is lost** — all `warn_drop`, `warn_continue`
   and `reject` buckets from all nodes, including nodes that succeeded. The worker then
   sets `aggregated_diagnostics: Vec::new()` in `failure_summary`, so `aggregatedDiagnostics`
   arrives empty.
3. `dropped_event_count` is also lost — `run_dag_executor` only populates it on the
   `Ok` branch ([`executor.rs:83`](../../runtime/runner/src/executor.rs#L83):
   `if let Ok(summary) = join_result.as_mut()`).

`Terminate` is `#[default]` ([`policy.rs:6`](../../runtime/diagnostics/src/policy.rs#L6)),
so this is the path almost every run takes.

---

## Why it is built this way

[`executor.rs:76`](../../runtime/runner/src/executor.rs#L76) documents the intent:

> `// `Terminate` still returns `Err` (golden logs byte-identical); `Continue` folds every outcome into `Ok(summary)`.`

The `Err` return is load-bearing for **golden-log byte-identity tests**. Changing it is not
a refactor — it is a contract change with a test suite pinned to the current behaviour.

---

## Options

> **"Why not just make `onFatal: continue` the default?"** That is this option, arrived at
> from the other direction — and on its own it is **not sufficient**. Verified with
> `repro/expr-factory-error-continue.yml` ([05](05-local-verification.md)): a build-time
> `Factory` error under `onFatal: continue` still produces the Debug dump, byte-identical to
> the default, because build errors are raised in `DagExecutor::new()` **before any node
> thread starts** and therefore never reach `join()` at all. The same is true of
> `PolicyValidationError` and `RuntimeError`. Flipping the default would also leave the
> broken path in place for anyone who sets `terminate` explicitly.
> **[01](01-worker-diagnostic-recovery.md) is required either way.**

> **Naming caveat:** `Continue` does *not* mean "the run survives a fatal". Both policies
> report `"result": "failed"` for the same failure (measured). `OnFatalInput` is documented as
> *"Applied at the join level only — never as a disposition demotion"* — it changes only how
> the outcome is **reported**, not whether the run fails, and not whether nodes keep
> processing (they always do — see [04](04-fatal-occurrence-count.md)).

### A — Fold, then signal fatality separately (preferred)

Always `Ok(fold_outcomes(results))`. `RunSummary.failed_nodes` being non-empty already
means "the run failed", and the worker already derives `JobResult` from exactly that
(`derive_job_result`, [`command.rs:574`](../../worker/src/command.rs#L574)).

- ✅ Nothing is lost; `Terminate` and `Continue` produce identical, complete summaries.
- ✅ Makes [01](01-worker-diagnostic-recovery.md)'s fallback path nearly dead code — the
  worker would almost always get `Ok(summary)` with properly downcast diagnostics.
- ❌ Changes the runner's outward contract. `summary_into_unit_result`
  ([`runner.rs:43`](../../runtime/runner/src/runner.rs#L43)) already converts a non-empty
  `failed_nodes` back into `Err` for the unit-returning `Runner::run` wrappers and the CLI,
  so those callers are covered — **verify this before relying on it**.
- ❌ Golden-log tests must be re-baselined. Establish *why* each diff appears before
  accepting it; a byte-identity suite is only worth having if re-baselining is deliberate.

Difference in semantics to settle: under `Terminate` the nodes have already all run to
completion by the time `join()` is reached, so "terminate" here only ever meant *how the
result is reported*, not early cancellation. Worth confirming that reading against the
`on_fatal` docs before changing anything — if `Terminate` is also expected to cancel
in-flight work elsewhere, that is a separate mechanism and this change does not touch it.

### B — Keep `Err`, attach the summary to it

Add a variant carrying both, e.g. `Error::FailedRun { summary: RunSummary, first: ExecutionError }`.

- ✅ Golden logs likely unaffected (still an `Err`).
- ❌ Awkward type; every consumer must learn to look inside an error for success-shaped data.
- ❌ Leaves two divergent shapes for the same run outcome — the root cause of this whole cluster.

### C — Do nothing; accept partial reporting on the default policy

Only defensible if `Terminate` is redefined as explicitly "report the first fatal, discard
the rest", and that is documented in the policy type and surfaced in the UI. Not recommended,
but cheaper than A. Note it would leave `aggregatedDiagnostics` permanently empty for
failed runs, which is likely to be reported as a separate bug later.

---

## Open questions — resolve before implementing

- [ ] Which golden-log tests actually pin the `Err` return? Enumerate them first;
      the scope of A is unknowable until this is answered.
- [ ] Does anything besides `summary_into_unit_result` depend on `join()` returning `Err`
      under `Terminate`?
- [ ] Is `Terminate` expected to mean early cancellation anywhere? (It does not here.)
- [ ] Should a `Terminate` run report *all* failed nodes, or first-fatal plus a count?
      This is a product question — it changes what the UI shows.

## Checklist

- [ ] Enumerate affected golden-log tests **before** writing code
- [ ] `cargo set-version --bump patch`
- [ ] Full `cargo make test`, plus a fresh `workflow-tests` baseline taken on `origin/main`
      at branch time (it is broken on `main` independently — do not attribute its failures here)

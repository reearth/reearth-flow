# 02 — `Terminate` discards the run summary

**Scope:** `engine/runtime/runtime/src/executor/dag_executor.rs`, the runner's public
`Result<RunSummary, Error>` contract, golden-log tests.
**Fixes:** other failed nodes, `aggregatedDiagnostics`.
> ## ✅ IMPLEMENTED — branch `fix/terminate-preserves-run-summary`
>
> `join()` always folds. The `Terminate` early-return is gone, so both policies produce the same
> complete summary and a non-empty `failed_nodes` is the run-failed signal
> (`summary_into_unit_result` still converts that to `Err` for the unit-returning wrappers).
>
> **Measured before/after** on `repro/fatal-plus-warns.yml` — a run with one fatal node and one
> warn-emitting node — by reverting just the `join()` change and rebuilding:
> `aggregatedDiagnostics: []` → `[('expr.attribute_operation_failed', 3)]`.
>
> **Exactly the 3 predicted tests broke**, no golden-log re-baselining, as the enumeration said.
>
> ### Two things this surfaced that the plan did not anticipate
>
> **1. `onFatal` now has no behavioural consumer.** Decided (option A): keep parsing it for
> compatibility, documented on the type and in `workflow.json` as having no effect, with a note
> that if `terminate` is ever given real meaning it should be actual early cancellation. Also
> corrected `PolicyDisposition::Fatal`'s doc, which described the removed behaviour.
> `workflow.json` turned out to be **stale on main** — `doc-workflow` is not part of
> `check-schema`, so the regeneration picks up unrelated drift.
>
> **2. Cascade failures became visible.** Reporting every node surfaced
> `JSON Writer: Cannot receive from channel: RecvError` next to the real failure — the writer
> noticing its upstream died, not an independent fault. `fold_outcomes` now drops those rows
> **only when a real failure exists**, so a lone channel failure is never hidden. This mirrors the
> existing `start_source` behaviour, which already swallows `CannotSendToChannel` when a listener
> quits. Note this also improves `onFatal: continue`, where the noise already existed — two
> pre-existing tests asserted `failed_nodes.len() == 2` to codify it and now expect 1.

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

## Open questions — RESOLVED

The blocking question is answered, and **the scope is far smaller than this doc assumed.**

### Which tests pin the `Err` return? — 3, all in `11_run_summary_threading.rs`

Nine `expect_err` assertions exist across the logging suite. Only three reach `join()`:

| Test | Affected? | Why |
|---|---|---|
| `failing_source_workflow_yields_err` | ✅ **yes** | calls `run_with_event_handler` directly |
| `processor_failure_event_converges_with_thread_result` | ✅ **yes** | same |
| `branch_completion_terminate_default_still_errors_for_same_workflow` | ✅ **yes** | same |
| `run_with_sandbox_root_wrapper_still_returns_err_for_failing_source` | ❌ no | unit wrapper → `summary_into_unit_result` converts back to `Err` |
| `run_with_sandbox_root_wrapper_still_returns_err_under_continue_policy` | ❌ no | same — and this test **already proves** the conversion holds under `Continue` |
| `reject_promoting_override_on_a_sink_without_side_file_aborts_the_run` | ❌ no | `validate_reject_routing`, pre-`join` |
| `reject_promoting_override_on_a_processor_with_unwired_rejected_port_aborts_the_run` | ❌ no | same |
| `unknown_error_code_in_policy_aborts_before_dag_construction` | ❌ no | policy validation, pre-`join` |
| `unmatched_node_selector_aborts_after_dag_construction` | ❌ no | same |

Those three assert *"the run fails"*, which stays true under A — only the reporting shape moves
from `Err(e)` to `Ok(summary)` with a non-empty `failed_nodes`. They need rewriting, not deleting.

### No golden-log re-baselining is needed

The `user-facing.log` / `action.log` fixtures capture **user-facing** entries, not the runner's
raw tracing. `grep -rl "Finish workflow\|Failed to workflow"` over
`runtime/tests/fixture/testdata/logging/` returns nothing. `result.json` is *workflow output
data* and exists only for the three succeeding scenarios.

The "golden logs byte-identical" note at [`executor.rs:76`](../../runtime/runner/src/executor.rs#L76)
overstates the constraint for this change.

### The worker's log parser is unaffected — and improves

`worker/src/action_log_parser.rs` scrapes runner tracing with regexes. Two matter:

- `workflow_completed` **already accepts both forms**:
  `r"Finish workflow = .* \((success|failed|completed with \d+ failed node\(s\))\)"` —
  the `Ok(summary)` branch's wording is already handled.
- `workflow_failed` is `r"Failed nodes:"`, emitted by `command.rs` — **only in its `Ok(summary)`
  branch**. Under today's `Terminate` that line never fires for a node failure; under A it would.
  So A gives the parser a failure signal it currently lacks.
- `factory_error` parses the **Debug shape** of `ExecutionError::Factory`. Factory errors are
  raised pre-`join` and keep returning `Err`, so this is untouched — but it is a live coupling to
  a `{:?}` rendering and worth noting before anyone "cleans up" that log line.

### Remaining answers

- **Anything else depending on the `Err`?** Only `summary_into_unit_result`, which already
  converts and is covered by two passing tests.
- **Does `Terminate` mean early cancellation?** No. Every node thread is joined before `join()`
  is reached; "terminate" only ever described how the result is reported.
- **Report all failed nodes, or first-fatal plus a count?** A reports all, which is what
  `Continue` already does. Keeping the two policies' summaries identical is the point.

## Checklist

- [ ] Enumerate affected golden-log tests **before** writing code
- [ ] `cargo set-version --bump patch`
- [ ] Full `cargo make test`, plus a fresh `workflow-tests` baseline taken on `origin/main`
      at branch time (it is broken on `main` independently — do not attribute its failures here)

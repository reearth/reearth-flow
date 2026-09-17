# Terminal Diagnostics — Fidelity Plan

Working doc for a set of fixes to what the frontend receives in `Job.failedNodes`.
Delete this directory when all four items have landed.

**Trigger:** the frontend receives a `Diagnostic` whose `message` is a Rust `Debug` dump
of a struct, while every other field on the row is a generic placeholder.

```json
{
  "code": "internal.unclassified",
  "category": "internal",
  "nodeId": "798af2c6-ccba-54aa-8148-f82fab95596d",
  "actionType": "Statistics Calculator",
  "featureId": null,
  "message": "ExecutionError(Processor(Diagnostic { code: InternalUnclassified, ..., feature_id: Some(d4270fa5-4ea2-41ab-9bf5-ba62b78b789f), message: \"StatisticsCalculator error: Failed to evaluate expression for attribute 'mean_height_m': ...\", ... }))",
  "help": "See the message and the failing action's log for details. ...",
  "aggregatedCount": null
}
```

---

## The chain that produces it

All four issues below sit on one path. Verified against `main` @ `e3d85ffae`.

1. An action's `process()` returns a plain `Err`.
   → [`statistics_calculator.rs:413`](../../runtime/action-processor/src/attribute/statistics_calculator.rs#L413)

2. The runtime blanket-wraps it as a `Diagnostic` hardcoded to `ErrorCode::InternalUnclassified`,
   and stores it in the node's first-wins fatal slot.
   → [`processor_node.rs:635 synthesize_process_error_fatal`](../../runtime/runtime/src/executor/processor_node.rs#L635)

3. At terminate, the fatal slot becomes `ExecutionError::Processor(Box<Diagnostic>)`.
   → [`processor_node.rs:652 reconcile_terminate_result`](../../runtime/runtime/src/executor/processor_node.rs#L652)
   The box is deliberately preserved — [`errors.rs:144`](../../runtime/runtime/src/errors.rs#L144) warns:
   *"a Diagnostic carrier must not be collapsed via `format!()`, since the join fold later downcasts it back out."*

4. `join()` branches on error policy — [`dag_executor.rs:377`](../../runtime/runtime/src/executor/dag_executor.rs#L377):
   - **`Continue`** → `fold_outcomes` → `diagnostic_from_execution_error` **downcasts the box back**. Correct.
   - **`Terminate`** (the `#[default]`, [`policy.rs:6`](../../runtime/diagnostics/src/policy.rs#L6)) → returns the first `Err`
     and discards every other node outcome and all aggregated diagnostics.

5. The worker `Debug`-formats that `Err` and synthesizes a fresh placeholder row around it.
   → [`command.rs:266`](../../worker/src/command.rs#L266) and [`command.rs:590 failure_summary`](../../worker/src/command.rs#L590)

Step 5 is the bug that produces the visible symptom. Steps 2 and 4 are why the fix
recovers less than it first appears to.

**Why the regular logs look fine:** [`log_event_handler.rs:96`](../../runtime/runner/src/log_event_handler.rs#L96)
does `let message = &d.message;` — it reads the structured field off the real `Diagnostic`
delivered live over the event hub, and never sees an `ExecutionError` at all.

---

## The four issues

| # | Issue | Fixes | Doc |
|---|---|---|---|
| 1 | ✅ **DONE** — Worker Debug-dumps the error and discards the real `Diagnostic` | `message`, `featureId`, `actionType` | [01](01-worker-diagnostic-recovery.md) |
| 2 | `Terminate` discards all other node outcomes + every aggregated diagnostic | other failed nodes, `aggregatedDiagnostics` | [02](02-terminate-discards-run-summary.md) |
| 3 | ✅ **DONE** — Expression failures have no error code | `code`, `category`, `help`, + `errorPolicy` now applies | [03](03-expression-error-classification.md) |
| 4 | ⬜ A ✅ **DONE** / B pending 02 — Fatals never carry a count | `aggregatedCount` | [04](04-fatal-occurrence-count.md) |

**A fatal does not stop the node.** `process()` returns `()`, is spawned on a thread pool, and
the fatal slot is not read until terminate — so an action processes *every* feature after the
first fatal, discarding each subsequent diagnostic into an already-full first-wins slot. N error
logs with 1 diagnostic is the design working as written. Full trace in [04](04-fatal-occurrence-count.md#0-a-fatal-does-not-stop-the-node--verified).

**Verifying any of this locally:** [05 — Local verification](05-local-verification.md).
No GCP, MongoDB, pubsub or frontend needed; the worker writes the exact `failedNodes` payload
to a local `diagnostics.json`. Harness in [`repro/`](repro/).

**None of these is a subset of another.** Fixing only #1 leaves the row still coded
`internal.unclassified` with the generic help, still showing one occurrence, and still
missing every other node that failed.

---

## Recommended order

1. **#1 first, on its own.** Smallest, unblocks the open frontend PR, no behaviour change
   outside the failure path. Ship it alone.
2. **#3 next.** Independent of the others, and self-contained per action.
3. **#4** — option A (stop rendering `null` as 1) is shippable immediately alongside the
   frontend PR. Option B (count repeat fatals) is recommended but must follow #2.
4. **#2 last.** Largest blast radius — touches golden-log tests and the runner's
   `Result<RunSummary, Error>` contract.

---

## Constraints that apply to every item here

- **Engine version bump is mandatory.** `.github/workflows/ci_engine.yml:66` compares
  `engine/Cargo.toml` version across `HEAD~1..HEAD` whenever anything under `engine/`
  changed. `cargo set-version --bump patch`. Currently `0.0.560`.
  - ⚠️ Re-read the version after any merge-up from `main`, **even when `Cargo.toml` did not
    conflict** — if both sides bumped to the same patch, the merge is clean and the merge
    commit then matches its parent, which fails this check with nothing flagging it.
- **`cargo make check-schema` runs in CI** (`ci_engine.yml:118`). It invokes `schema-base`,
  which regenerates `schema/error-codes.json` among others (`Makefile.toml:194`). Item #3
  changes the registry, so that file must be regenerated and committed.
- **Error codes have no i18n.** The `message`/`help` in `schema/error-codes/*.toml` ship
  as-is, English only. They are user-facing text and fall under action-standard §2.
- **`workflow-tests` is broken on `main`** independently of this work. Take a fresh baseline
  on `origin/main` at branch time rather than trusting any recorded failure count.

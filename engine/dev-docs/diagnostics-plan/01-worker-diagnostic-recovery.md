# 01 — Worker discards the real Diagnostic

**Scope:** `engine/worker/src/command.rs` (+ one small `pub` addition in the runtime crate).
**Fixes:** `message`, `featureId`. **Does not fix:** `code`, `category`, `help`, `aggregatedCount`.
**Ship alone.** This is the change that unblocks the open frontend PR.

> ## ✅ IMPLEMENTED — branch `fix/diagnostic-recovery-worker`, engine `0.0.561`
>
> - `recover_diagnostic` + `render_error_chain` added to `runtime/src/errors.rs` (public);
>   `diagnostic_from_execution_error` refactored onto them, so both paths share one implementation.
> - `failure_summary` now takes `&runner::errors::Error`, recovers the carried `Diagnostic`,
>   and falls back to the rendered source chain only when there is nothing to recover.
> - `is_same_node` handles the composed-id/handle-id mismatch.
> - 2 existing tests updated (they asserted the Debug shape); 5 added. All green, clippy clean.
> - **Verified against the oracle: default-path `failedNodes` is now byte-identical to
>   `onFatal: continue`** (modulo the per-run feature UUID). Factory path renders readably
>   with no doubled source.
> - Not committed — awaiting review.
>
> ### Follow-up surfaced during verification (NOT fixed here)
> The Factory message now reads:
> `Factory error for node … (StatisticsCalculator): StatisticsCalculator Factory error: Lex { pos: 11, msg: "unexpected character" }`
> The trailing `Lex { … }` is the **action** formatting the expr crate's error with `{:?}`
> into its own message string — the same class of bug one layer down, inside
> `statistics_calculator.rs`. Out of scope for 01; fold into [03](03-expression-error-classification.md).

---

## The defect

[`command.rs:266`](../../worker/src/command.rs#L266):

```rust
Err(e) => failure_summary(format!("{e:?}"), &node_failure_handler),
```

`{e:?}` is `Debug`. When the error is `ExecutionError::{Processor,Sink,Source}(Box<Diagnostic>)`
— the common case for any action fatal — this dumps the carried `Diagnostic` as a Rust struct
literal into the `message` of a **different, freshly synthesized** diagnostic.

[`failure_summary`](../../worker/src/command.rs#L590) then builds rows hardcoded to
`ErrorCode::InternalUnclassified`, passing `None` for `feature_id`.

Net effect: a fully-formed `Diagnostic` is stringified and replaced with a placeholder
carrying the stringification. [`errors.rs:144`](../../runtime/runtime/src/errors.rs#L144)
explicitly warns against exactly this, one layer down.

### Why Display alone is not the fix

`e.to_string()` yields `"Processor error: [FATAL] internal.unclassified @ node (action): msg"`
— readable, but it still discards `feature_id`, `severity`, `source_span`, and still stamps
`internal.unclassified`. It would make the frontend *look* fixed while the data stays lost.
**Recover the object; do not re-render the string.**

---

## The fix

### Change 1 — expose the existing recovery logic

The runtime already does this correctly in
[`dag_executor.rs:426 diagnostic_from_execution_error`](../../runtime/runtime/src/executor/dag_executor.rs#L426)
(private `fn`, used by `fold_outcomes` on the `Continue` path).

Do **not** duplicate it in the worker — two copies of this logic drifting apart is what
produced this bug. Make it reusable instead. Preferred shape:

```rust
// runtime/runtime/src/errors.rs (or a small `diagnostic_recovery` module)
/// Recovers a Diagnostic carried by a node-kind ExecutionError, if present.
pub fn recover_diagnostic(e: &ExecutionError) -> Option<&Diagnostic> {
    match e {
        ExecutionError::Processor(b) | ExecutionError::Sink(b) | ExecutionError::Source(b) =>
            b.downcast_ref::<Diagnostic>(),
        _ => None,
    }
}
```

Then rewrite `diagnostic_from_execution_error` to call it, so both paths share one
implementation. `downcast_ref` (not `downcast`) because the worker only holds `&Error` —
`command.rs` matches on `&result` and still needs `result` afterwards for `job_result`.
`Diagnostic` is `Clone`, so `.cloned()` at the call site is fine.

> ⚠️ `ExecutionError::Factory` is deliberately **not** in that match, and must stay out
> unless a factory starts boxing a `Diagnostic`. Verified at time of writing: no
> `build()` impl in `action-processor` / `action-source` / `action-sink` constructs a
> `DiagnosticDraft`. Re-check before adding it.

### Change 2 — a source-chain renderer for the non-Diagnostic case

Errors that carry no `Diagnostic` (`Factory`, `PolicyValidationError`, `RuntimeError`, …)
still need a readable string. Use Display plus a `source()` walk — this is the one thing
Debug genuinely did better, and it is worth keeping:

```rust
fn render_chain(e: &dyn std::error::Error) -> String {
    let mut out = e.to_string();
    let mut cur = e.source();
    while let Some(s) = cur {
        out.push_str(": ");
        out.push_str(&s.to_string());
        cur = s.source();
    }
    out
}
```

`BoxedError` is `Box<dyn Error + Send + Sync>` ([`errors.rs:135`](../../runtime/runtime/src/errors.rs#L135)),
so the walk works through the whole chain.

> Guard against duplication: several `ExecutionError` variants **already interpolate their
> source** into their Display string (`Factory` uses `{error}`, `Source`/`Processor`/`Sink`
> use `{0}`), so a naive walk will print the inner message twice. Decide one of:
> dedupe consecutive repeats, or only walk when the parent Display does not already contain
> the child's. **Write a test for this specific case** — it is easy to ship a doubled message.

### Change 3 — `failure_summary` takes the error, not a string

```rust
fn failure_summary(error: &reearth_flow_runner::errors::Error, handler: &NodeFailureHandler) -> RunSummary
```

Behaviour:

1. Try `recover_diagnostic`. If it yields a `Diagnostic`:
   - clone it, stamp `effective_disposition = Some(Disposition::Fatal)` (matching
     `fold_outcomes`, which stamps Fatal on every `failed_nodes` entry regardless of severity),
   - emit it as the row **for its own node**.
2. Emit one synthesized `internal.unclassified` row per *other* node in
   `handler.failure_details()`.
   - **Message for these rows:** use the registry default (omit `.with_message(...)` and
     `from_draft` falls back to `code.default_message()` → *"node failed with an unclassified
     error"*). Do **not** reuse the recovered node's error text — today every row gets the
     same whole-run string, which on a second node's row describes a failure that happened
     somewhere else. That is a pre-existing correctness flaw; fix it here.
3. If nothing was recovered, keep today's shape but with `render_chain(error)` instead of
   `format!("{e:?}")`.
4. If `failed_nodes` ends up empty, keep the existing workflow-level fallback row.

---

## ⚠️ The node-id matching problem

To avoid emitting two rows for the same node, step 2 must exclude the recovered node.
**The two id namespaces do not match.**

- A `Diagnostic`'s `node_id` is a **composed id** —
  [`builder_dag.rs:58`](../../runtime/runtime/src/builder_dag.rs#L58):
  `format!("{prefix}.{}", handle.id)` for a node inside a subgraph, else `handle.id`.
- `NodeFailureHandler` records the **raw handle id** —
  [`event_handler.rs:78`](../../worker/src/event_handler.rs#L78): `node.id.to_string()`.
  `NodeHandle` has only an `id` field ([`node.rs:157`](../../runtime/runtime/src/node.rs#L157)),
  so the handler *cannot* reconstruct the prefix from the event.

So for a node inside a subgraph the ids differ and a plain `==` comparison silently fails,
producing a duplicate row.

**Recommended:** a documented helper with a test —

```rust
fn same_node(composed_id: &str, handle_id: &str) -> bool {
    composed_id == handle_id || composed_id.ends_with(&format!(".{handle_id}"))
}
```

This mirrors the documented `composed_id` format rather than guessing. The proper long-term
fix is to carry the composed id on the events themselves, which is out of scope here —
note it in the code comment.

> Also note, unrelated but visible in the same data: `SinkFinishFailed` pushes
> `FailedNode { id: name.clone(), .. }` ([`event_handler.rs:85`](../../worker/src/event_handler.rs#L85))
> — an action *name* in the `id` field. Sink failure rows therefore already carry a
> non-id `nodeId`. Out of scope; do not fix silently, log it as a follow-up.

---

## Tests

Existing tests assert the broken shape and **must** be updated, not deleted:

- `failure_summary_builds_a_fatal_row_per_failed_node_deduped`
  ([`command.rs:626`](../../worker/src/command.rs#L626)) — currently passes the literal
  `"ExecutionError(Source(..))"` and asserts `row.message` equals it. This is exactly how the
  Debug shape got baked in rather than caught.
- `failure_summary_falls_back_to_a_workflow_level_row` ([`command.rs:664`](../../worker/src/command.rs#L664)).

New tests to add:

1. A boxed `Diagnostic` in `Processor` is recovered intact — asserts `code`, `feature_id`,
   `message` all survive, and that `message` contains no `"Diagnostic {"`.
2. Same for `Sink` and `Source`.
3. A non-Diagnostic error renders via `render_chain` — asserts readable text and, explicitly,
   **no duplicated segment**.
4. `ExecutionError::Factory` does **not** attempt recovery and renders via the chain.
5. Recovered node + one other failed node → exactly 2 rows, no duplicate for the recovered
   node, and the other row carries the registry default message.
6. `same_node` matches both `"<uuid>"` and `"sub.<uuid>"` against `"<uuid>"`.
7. Round-trip: recovered `Diagnostic` → `WireDiagnostic` → JSON has a populated `featureId`.

---

## Acceptance

**There is a live oracle: `onFatal: continue` already produces the correct output**, because
it takes the `fold_outcomes` path that downcasts properly. See [05](05-local-verification.md).

```bash
cd engine && cargo build -p reearth-flow-worker --bin reearth-flow-worker
cd dev-docs/diagnostics-plan
./repro/run.sh repro/expr-fatal.yml            # broken
./repro/run.sh repro/expr-fatal-continue.yml   # target
```

The fix is done when the two `failedNodes` arrays are identical apart from `jobId`/`timestamp`.
That also catches the `actionType` bug (today the default path emits the **node name**, the
oracle emits the real **action type**) — assert it explicitly.

Expect on the reported workflow:

```json
"message": "StatisticsCalculator error: Failed to evaluate expression for attribute 'mean_height_m': Error while running action: eval error at position 0: attribute 'test' not found",
"featureId": "d4270fa5-4ea2-41ab-9bf5-ba62b78b789f",
"code": "internal.unclassified",     // unchanged — see 03
"aggregatedCount": null              // unchanged — see 04
```

Confirm `code` and `aggregatedCount` are **expected to stay as-is** so the frontend reviewer
does not read them as a failed fix.

## Checklist

- [ ] `cargo set-version --bump patch` in `engine/`
- [ ] `cargo make test` (or at least `-p reearth-flow-worker -p reearth-flow-runtime`)
- [ ] `cargo clippy` clean
- [ ] No schema regeneration needed — this item touches no registry or action schema

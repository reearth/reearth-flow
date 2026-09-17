# 04 — Fatals carry no occurrence count

**Scope:** frontend null-handling (A), then `engine/runtime/diagnostics/src/aggregator.rs` (B).
**Fixes:** `aggregatedCount`.
**Option A is unblocked and shippable now. Option B is recommended but sequenced after
[02](02-terminate-discards-run-summary.md).**

---

## The symptom

The UI shows **1 occurrence** while the run log shows dozens of the identical error.

**Measured** with `repro/expr-fatal.yml` (5 features, all failing) — see [05](05-local-verification.md):
5 `"Error operation"` log lines, 1 diagnostic, `"aggregated": null`.

## Why — three compounding causes

### 0. A fatal does NOT stop the node — verified

This is the piece that makes the rest make sense, and it contradicts the natural reading of
the word "fatal". A per-feature error **cannot** halt processing:

- [`processor_node.rs:557`](../../runtime/runtime/src/executor/processor_node.rs#L557) — the
  `process` free function returns `()`. It has no error channel. On `Err` it sets `has_failed`,
  logs, emits `ProcessorFailed`, records the fatal, and returns normally.
- [`on_op_with_failure_tracking`](../../runtime/runtime/src/executor/processor_node.rs#L460)
  spawns it on a **thread pool** and returns `Ok(())` unconditionally — it never inspects the
  outcome, and features are in flight concurrently.
- `take_fatal` is called in exactly two places, both at terminate:
  [`processor_node.rs:338`](../../runtime/runtime/src/executor/processor_node.rs#L338) and
  [`sink_node.rs:353`](../../runtime/runtime/src/executor/sink_node.rs#L353).
  `record_fatal` triggers nothing. **There is no fatal-driven abort anywhere in the runtime.**

So the observed sequence — dozens of error logs, one diagnostic — is the design working as
written, not a second bug.

**Where the error originates decides the shape:**

| Origin | Log lines | Diagnostics |
|---|---|---|
| `build()` (factory/params) | 1 | 1 |
| `process()` (per feature) | **N** | **1** |
| `finish()` (drain end) | 1 | 1 |

### 1. The fatal slot is first-wins

[`aggregator.rs:66`](../../runtime/diagnostics/src/aggregator.rs#L66):

```rust
pub fn record_fatal(&self, diagnostic: Diagnostic) {
    let mut slot = self.fatal.lock().unwrap();
    if slot.is_none() { *slot = Some(diagnostic); }   // subsequent fatals dropped
}
```

Feature 1 fills the slot; features 2..N are logged and their diagnostics discarded.

### 2. Fatals are excluded from aggregation *by design*

[`aggregator.rs:10`](../../runtime/diagnostics/src/aggregator.rs#L10):

> `/// Fatal is never aggregated here — it goes to the per-node fatal slot and fails the node.`

`DiagnosticKind` has only `WarnContinue`, `WarnDrop`, `Reject` — there is no `Fatal` bucket.
Consistent with [`types.rs`](../../runtime/diagnostics/src/types.rs), where `AggregateInfo`
is documented *"`Some` for finish()-time summaries; `None` for per-feature/fatal diagnostics."*

So `aggregated` is `None` → [`convert_diagnostic.go:29`](../../../server/api/internal/adapter/gql/gqlmodel/convert_diagnostic.go#L29)
only sets `AggregatedCount` when `Aggregated()` is non-nil → `aggregatedCount: null`.

**This is a documented design decision, not an oversight.** Any change here argues against
an explicit invariant and must say why.

---

## What the control flow implies for the decision

Trace the consequences on the default policy after the first fatal:

1. The node is already doomed — the filled slot guarantees it fails at terminate.
2. It processes **every remaining feature anyway**.
3. Those features' diagnostics are dropped by the full slot.
4. Every *other* node's aggregated diagnostics are then discarded by the `Terminate` branch
   in `join()` (see [02](02-terminate-discards-run-summary.md)).

**The engine pays the full cost of a doomed run and reports almost nothing it learned.**
That is the worst of both available trade-offs, and it is the current default.

This inverts the intuitive framing of the question. "Fatal implies one occurrence" is true
for `build()` and `finish()` errors but **false for `process()` errors**, which are inherently
per-feature and repeat. For those, the count is genuinely informative: *1 bad record* versus
*50,000 features failed* is the difference between a data problem and a wrong expression.
Today that signal is discarded.

---

## The honest framing

`aggregatedCount: null` means *"this diagnostic does not carry a count"*, not *"this happened
once"*. The frontend is rendering null as 1. That is the immediate defect: **the data is
correct and the presentation invents a number.**

---

## Options

### A — Frontend only: stop inventing a count ✅ IMPLEMENTED

`diagnosticOccurrences` now returns `number | undefined` instead of `aggregatedCount ?? 1`,
and the table renders `t("Unknown")` (already present in all five locales) rather than `1`.
The docstring's old claim — *"every other row is a single occurrence"* — was the bug in prose
and is replaced with why that is false.

- ✅ Removes the false claim immediately.
- ✅ Costs nothing and is correct regardless of which engine option follows.
- ❌ User still cannot tell 1 failure from 500.

### B — Count repeat fatals per node (recommended)

Add a counter alongside the fatal slot — increment on every `record_fatal` while keeping
first-wins for the stored `Diagnostic` — and populate `AggregateInfo.count` when the slot is
taken.

- ✅ Answers the real question ("how bad is it?") with a small, local change.
- ✅ Keeps the first-wins diagnostic, so nothing else moves.
- ❌ Contradicts the `AggregateInfo` doc comment (*`None` for per-feature/fatal*) — update
      that comment as part of the change, do not leave the two disagreeing.
- ❌ Decide whether the count is per `(node, code)` or per node. Per `(node, code)` matches
      how warn buckets key and is the more useful answer.
- ⚠️ Interacts with [02](02-terminate-discards-run-summary.md): under today's `Terminate`
      the count would still only survive for the one node whose error is returned.
      **Sequence B after 02** or the fix is invisible on the default policy.
- 📌 Evidence the shape is right: once [03](03-expression-error-classification.md) lets a policy
      demote the failure to `warn_drop`, the count appears correctly (`"count": 5`, 5 sample
      ids). The aggregation machinery already works — only the **fatal slot** bypasses it.

### C — Fail fast: stop processing at the first fatal

Makes "1 occurrence" *true* rather than better-reported.

- ✅ Arguably what "fatal" should mean; saves real work on a doomed run (see the control-flow
      section above — today the engine processes every remaining feature after the first fatal).
- ❌ Genuine behaviour change; users lose the "how widespread is this?" signal entirely.
- ❌ Requires a cancellation mechanism that does not exist — nothing currently reads the fatal
      slot before terminate, and `process()` runs on a thread pool with features already in flight.
- ❌ Conflicts with the existing `report()` design, where an action may swallow the `Err`
      and the fatal slot exists precisely as a backstop
      ([`processor_node.rs:621`](../../runtime/runtime/src/executor/processor_node.rs#L621)).
- Not recommended without a broader discussion of fatal semantics. Note B and C are not
  mutually exclusive in principle, but C subsumes the value of B if it ever lands.

---

## Recommendation

**A now** (with the frontend PR), **B after [02](02-terminate-discards-run-summary.md)**.

A is strictly correct on its own, costs nothing, and is right regardless of what follows.
B is the real feature: the control-flow analysis above shows a `process()` fatal is inherently
per-feature and repeats, so the count carries information the user currently cannot get
anywhere except by reading raw logs. It is wasted effort until 02 lands, because under
today's `Terminate` the count would survive only for the one node whose error is returned.

## Decision needed

- [x] ~~Does "occurrences" on a fatal row mean anything?~~ **Yes** — resolved by the
      control-flow analysis above. A `process()` fatal repeats per feature; `build()` and
      `finish()` fatals do not. The count distinguishes a bad record from a bad workflow.
- [ ] If counting: per `(node, code)` or per node? (Recommend `(node, code)` — matches how
      warn buckets key, and a node can in principle hit two distinct fatal codes.)
- [ ] Should the *stored* diagnostic stay first-wins while only the counter increments?
      (Recommend yes — changing which diagnostic is kept is an unrelated behaviour change.)
- [ ] Who owns A — is it inside the scope of the open frontend PR?

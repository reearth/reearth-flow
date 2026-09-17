# 03 — Expression failures are unclassified at the source

**Scope:** `engine/schema/error-codes/`, `engine/runtime/action-processor/`.
**Fixes:** `code`, `category`, `help` on the reported row.
**Independent of [01](01-worker-diagnostic-recovery.md) and [02](02-terminate-discards-run-summary.md).**

> ## ✅ IMPLEMENTED — engine `0.0.561`
>
> - `expr.evaluation_failed` added to `schema/error-codes/expr.toml` (domain `expr` to match the
>   existing file, **not** `expression` as originally drafted); `error-codes.json` regenerated,
>   regeneration verified idempotent.
> - Converted three plain-`Err` eval sites to `ctx.report`: `statistics_calculator.rs`,
>   `aggregator.rs`, `fragmenter.rs`. Each skips just the failing calculation when the policy
>   resolves below Fatal.
> - Fixed `{e:?}` → `{e}` in user-facing messages in those files, and the PascalCase
>   `"StatisticsCalculator error"` → `"Statistics Calculator error"`.
> - **Verified:** the reported payload now carries `code: expr.evaluation_failed`,
>   `category: expression`, and actionable help.
>
> ### ⚠️ Finding: demoting requires naming the CODE, not the node
> `repro/expr-fatal-override.yml` (node selector, `warn_drop`) still fails — by design.
> [`policy.rs:224`](../../runtime/diagnostics/src/policy.rs#L224) `codeless_wins` only lets a
> **codeless** override *promote* severity, never relax it. Only a `code`-selector override can
> demote. `repro/expr-fatal-code-override.yml` proves the working form:
>
> ```yaml
> errorPolicy:
>   overrides:
>     - code: "expr.evaluation_failed"
>       disposition: warn_drop
> ```
>
> → `"result": "success"`, the failure moves to `aggregatedDiagnostics` as `warn_drop`
> **with `"count": 5` and 5 sample feature ids**. Worth documenting for users: a node-level
> override cannot make a node more tolerant.
>
> ### Follow-up NOT done here
> `format!("{e:?}")` in user-facing text is widespread (`mapper.rs`, `table_extractor.rs`,
> `manager.rs`, `conversion_table.rs`, …). Only the files touched by this change were fixed.
> A repo-wide sweep deserves its own PR.

---

## The defect

The reported payload is `internal.unclassified` **all the way down** — the outer synthesized
row *and* the `Diagnostic` nested inside its message both carry that code. So [01](01-worker-diagnostic-recovery.md)
recovers the real object but the real object has nothing better to offer.

Why: [`statistics_calculator.rs:413`](../../runtime/action-processor/src/attribute/statistics_calculator.rs#L413)
returns a plain `thiserror` value from `process()` —

```rust
expr.eval(feature, Arc::clone(&variables)).map_err(|e| {
    AttributeProcessorError::StatisticsCalculator(format!(
        "Failed to evaluate expression for attribute '{}': {e}", calculation.new_attribute))
})?;
```

Any plain `Err` out of `process()` is blanket-wrapped by
[`synthesize_process_error_fatal`](../../runtime/runtime/src/executor/processor_node.rs#L635),
which hardcodes `ErrorCode::InternalUnclassified`. The action never had a code to lose.

The registry text for `internal.unclassified` is *"node failed with an unclassified error"* /
*"See the message and the failing action's log for details. If it points at a workflow
parameter or input, fix that and retry."* That is not *wrong* — it is simply generic, and it
cannot be anything else, because the code carries no information about what failed.

(For the record: the *"This is an engine bug"* help belongs to the adjacent
`internal.invariant_violation`, not to `internal.unclassified`. Being blanket-classified as
unclassified does not currently accuse the user of hitting an engine bug.)

The real cost is that a typo'd attribute name in a user's expression — a **workflow authoring
error**, and a very common one — arrives with a category of `internal` and no actionable
guidance. Neither [01](01-worker-diagnostic-recovery.md) nor [02](02-terminate-discards-run-summary.md)
improves that; only a real code does.

---

## The fix

### 1. Add a registry code

`engine/schema/error-codes/` is the source of truth; the `ErrorCode` enum is **generated**
from it by [`diagnostics/build.rs`](../../runtime/diagnostics/build.rs). Never edit the enum.

Registry constraints, verified from `build.rs`:

- Code must match `<domain>.<reason>`, snake_case, exactly two dot-separated segments.
- `category` must be one of: `io parse validation geometry schema expression config network
  resource internal`. → **`expression` already exists and is the right one.**
- `default_disposition` must be one of `warn_drop reject fatal`.
- `help` is optional; `message` is required.
- Duplicate codes panic the build.

Proposed entry in a new or existing `expression`-domain file:

```toml
[[codes]]
code = "expr.evaluation_failed"
category = "expression"
default_disposition = "fatal"
message = "expression evaluation failed"
help = "Check the expression for this parameter. A common cause is referencing an attribute that does not exist on the incoming feature — verify the attribute name and that an upstream action produces it."
```

> Decide `default_disposition` deliberately. `fatal` preserves today's behaviour (the run
> fails). `reject` would route the offending feature aside and let the run continue, which
> is arguably the better default for a per-feature expression failure — but it is a
> **behaviour change** for existing workflows and should not be smuggled in with a
> classification fix. Recommend `fatal` now, and raise `reject` as its own proposal.

### 2. Report it from the action

Replace the plain `Err` with `ctx.report(DiagnosticDraft::new(ErrorCode::ExpressionEvaluationFailed).with_message(...))`
so the real code reaches the fatal slot instead of the blanket wrapper.

Check the surrounding function's signature first — `ctx.report` lives on `ExecutorContext`
([`executor_operation.rs:239`](../../runtime/runtime/src/executor_operation.rs#L239)) and
returns `Result<Disposition, Diagnostic>`; the `Diagnostic` is deliberately unboxed and
`?`-propagates as a `BoxedError`. The call site at `statistics_calculator.rs:413` is inside
an attribute-accumulation loop — **confirm it has access to `ctx`** before assuming this is
a drop-in change. If it does not, the options are to thread it in or to keep the plain `Err`
and map it to the code one level up.

### 2b. Why this is load-bearing, not cosmetic — `errorPolicy` is silently bypassed

[`synthesize_process_error_fatal`](../../runtime/runtime/src/executor/processor_node.rs#L635)
hardcodes `effective_disposition = Some(Disposition::Fatal)` and **never calls
`handle.resolve()`**. Compare [`ctx.report()`](../../runtime/runtime/src/executor_operation.rs#L239),
which resolves the code against the policy before deciding.

**Verified empirically**, not inferred — see [05](05-local-verification.md).
`repro/expr-fatal-override.yml` sets:

```yaml
errorPolicy:
  allowRelaxInternal: true
  overrides:
    - node: "798af2c6-ccba-54aa-8148-f82fab95596d"
      disposition: warn_drop
```

The override **parses, validates, and compiles** — and has **zero effect**. The run fails
fatally with byte-identical output to the no-policy run. The plain-`Err` path never reaches
the policy at all.

Note the selector here is `node`, not `code` — so this is not merely "there is no code to
match on". Even a node-targeted override is ignored, because `resolve()` is never called.
(A `code` selector would additionally need a real code to name, and demoting an `internal`
category code below its registry default also requires `allowRelaxInternal: true`.)

**Consequence:** a user cannot currently make a workflow tolerate a per-feature expression
failure by any available means. Routing the failure through `ctx.report` with a real code is
what makes `errorPolicy` apply to it.

> Scope note: "tolerate a failing feature and carry on" is the capability the `warn_drop` /
> `reject` dispositions exist to provide — it is not a user request sourced from a ticket.
> Treat it as *"the policy mechanism does not reach this path"*, which is the verifiable claim,
> rather than as evidence of demand.

### 3. How wide is this, really? — measured

Counted across `action-processor`, `action-sink`, `action-source` on this branch:

- **113** action files have a `process()`/`finish()` that can return an error
- **11** use the diagnostics API (`ctx.report`/`ctx.warn`/`ctx.warn_once`) at all
- **102** have **zero** classification — every error they return is blanket-wrapped as
  `internal.unclassified`, hardcoded Fatal, and bypasses `errorPolicy`

So this is **systemic, not three actions**. But 102 is not 102 bugs: some failures genuinely
should be fatal and unclassifiable. Which is which is a per-action judgement.

The 11 that do classify correlate strongly with recently-audited actions (`area_calculator`,
`spatial_filter`, `grid_divider`, `table_extractor`, `image_rasterizer`, the three sinks).
**Classification is already arriving via the action audit, action by action.** What is missing
is a *rule*: [action-standard.md](../action-standard.md) §2 governs diagnostic `message`/`help`
text, but nothing requires an action to classify its failures in the first place, so an auditor
has no criterion prompting them to add any.

**Recommendation:** add that criterion to the action standard in its own PR, rather than opening
a 102-file sweep. The remaining unaudited actions then get it for free, and a sweep would need
the same per-action judgement anyway, without the audit's structure.

### 4. Sweep, don't spot-fix

`expr.eval(...)` failures are not unique to Statistics Calculator. Grep for other actions
mapping an eval failure into a plain factory/processor error and give them the same code.
A code that only one action emits is not a classification.

---

## Incidental finding — `{:?}` inside the action's own error string

Surfaced by [01](01-worker-diagnostic-recovery.md)'s verification. The factory error reads:

```
StatisticsCalculator Factory error: Lex { pos: 11, msg: "unexpected character" }
```

`Lex { … }` is a Rust struct dump: the action formats the expr crate's error with `{:?}` when
building its message. Same class of defect as the one 01 fixed, one layer down. Grep the
action for `{:?}`/`{e:?}` in text that reaches a user and use Display (or a real code) instead.

## Incidental finding — user-facing name

[`attribute/errors.rs:32`](../../runtime/action-processor/src/attribute/errors.rs#L32):

```rust
#[error("StatisticsCalculator error: {0}")]
```

`StatisticsCalculator` is PascalCase in text a user reads, while the registered action is
named **"Statistics Calculator"**, and sibling variants in the same enum use the spaced form
(*"Attribute Keeper error: …"*). Also applies to `StatisticsCalculatorFactory` on the line
above. Fix while in the file; check the rest of the enum for the same slip.

---

## Tests

- Registry: `build.rs` panics on a malformed code, so a bad entry fails the build. No test needed.
- A unit test that an expression failure surfaces `ErrorCode::ExpressionEvaluationFailed`
  (not `InternalUnclassified`).
- Assert the emitted `help` is the registry's, not the generic internal one.

## Checklist

- [ ] `cargo make schema-base` → regenerates `engine/schema/error-codes.json`; **commit it**
      (`cargo make check-schema` runs in CI at `ci_engine.yml:118` and will fail otherwise)
- [ ] `cargo set-version --bump patch`
- [ ] Error codes have **no i18n** — this `message`/`help` ships English-only, and is
      user-facing text subject to action-standard §2
- [ ] `cargo make test`

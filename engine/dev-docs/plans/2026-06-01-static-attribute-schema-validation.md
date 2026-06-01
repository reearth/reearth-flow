# Static Attribute-Schema Validation — Implementation Spec (v1 prototype)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this spec task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. TDD throughout: write the failing test, see it fail, implement, see it pass, commit.

**Goal:** Add a static `build` (alias `check`) CLI command to the reearth-flow engine that, *without executing the workflow or touching data*, infers and propagates feature-attribute schemas through the DAG and reports pre-run validation diagnostics (e.g. "node references attribute `X` that no upstream node produces").

> **STATUS — e2e reached (2026-06-01).** `reearth-flow build`/`check` runs end-to-end on the real PLATEAU workflows in-repo (`worker/workflow/cms/plateau4/quality-check/bldg/template/workflow.yml` and `.../data-convert/bldg/...`). It expands the 4 `!include`s, builds the full subgraph-expanded DAG (~87 nodes, routers/mergers, 20+ action types), and runs the inference pass without panicking, exit 0. `--show-schema` dumps the inferred per-node schemas: on the quality-check workflow the pass records **87 nodes, 395 inferred attribute fields, 49 `(open)` ports, 20 `Maybe`-presence fields**. Implemented transfer functions: `AttributeManager`, `AttributeMapper`, `DateTimeConverter`, `FeatureFilter` (others fall back to schema-transparent passthrough; sources seed `open`). Tests: 271 (runtime+processor) + 4 (cli, incl. 2 PLATEAU e2e) + others, all green; fmt+clippy clean. Types are all `Unknown` on the PLATEAU run (every attribute is expression-derived) — concrete typed inference (String/Number/DateTime) is proven by the DateTimeConverter unit tests. Hard validation **errors remain unreachable end-to-end** until a source seeds a *closed* schema (see "error reachability" note below); this is the next milestone.

**Architecture:** A new attribute-schema type (`AttrSchema`) in `reearth-flow-types`. Two optional, default-`None` trait methods on `ProcessorFactory`/`SourceFactory`/`SinkFactory` — `infer_output_schema` (what a node produces per output port) and `referenced_input_attributes` (what a node consumes). A propagation pass in `reearth-flow-runtime` that walks the DAG in topological order, joining multi-edge inputs and calling the transfer functions, collecting diagnostics. A `build` CLI subcommand mirroring `dot.rs`.

> **AS-BUILT CORRECTION (hook point).** This spec originally prescribed walking the **`BuilderDag`**. During implementation that proved unworkable: `BuilderDag::new` *consumes* the factories into built trait objects (`Box<dyn Processor>` etc., `builder_dag.rs:71-75`) and is `async`/may do I/O — so the factory methods `infer_output_schema`/`referenced_input_attributes` are not reachable there. The pass therefore runs over **`DagSchemas`**, which retains the factory in `SchemaNodeType.kind: Option<NodeKind>` and is built non-executing by `from_graphs` (the same call `dot.rs` uses). **Consequence to track:** the dynamic, config-derived output ports of branching nodes (`FeatureFilter` condition ports, `OutputRouter.routingPort`) are synthesized only in `BuilderDag`, so they do NOT exist at the `DagSchemas` stage. An edge leaving such a node on a condition port won't match the producer's declared output port, so `gather_inputs` falls back to `AttrSchema::open()` — i.e. schemas downstream of a branching filter/router degrade to `open` (safe: no false positives, but no inference either). Making branch children carry real schemas later requires either moving to `BuilderDag` or factoring the dynamic-port logic out of `builder_dag.rs` into a shared helper the pass can call.

**Tech stack:** Rust (edition per workspace, toolchain 1.93.1), `petgraph`, `indexmap`, `schemars`, `serde_json`, `thiserror`, `pretty_assertions` (tests). Build/test via `cargo make` / `cargo test`.

**Scope of this prototype:** Prove the pipeline end-to-end with real validation value on a *representative subset* of processors — `AttributeManager`, `DateTimeConverter` (adds a statically-known-typed attribute; substituted for the originally-planned `AttributeKeeper`, which does not exist in this codebase), `FeatureFilter` (passthrough), plus generic identity/opaque defaults and `open` source seeds. NOT in scope: geometry typing, Rhai value-type inference (treated as `Unknown`; Rhai is being replaced), the full processor catalog, `actions.json` enrichment, node selection. Those follow once the prototype is green.

> **AS-BUILT NOTE (error reachability).** Because every source seeds an `open` schema and `open` suppresses all diagnostics, a hard validation **Error is not reachable end-to-end from a real workflow yet** — the Error/Warning paths are proven only by the `schema_infer.rs` unit tests, which use closed-schema stub producers. The `invalid.yml` fixture therefore exits `0` today (it documents this in its header). Errors become reachable with zero structural change once a real source seeds a *closed* schema (Phase 4: CSV → closed `String` columns, DB → catalog) or a processor closes one. This is the central, intentional limitation of the prototype.

---

## Background facts (verified against the code)

- `Feature.attributes: Arc<IndexMap<Attribute, AttributeValue>>` — `runtime/types/src/feature.rs:48,53`. `Attribute` is a `nutype` String newtype (`attribute.rs:37`); `.inner()` returns the `String`.
- `AttributeValue` variants — `attribute.rs:45-56`: `Null, Bool, Number, String, DateTime, Array, Map, Bytes`. No int/float split.
- Factory traits — `runtime/runtime/src/node.rs`: `SourceFactory` (`:218`), `ProcessorFactory` (`:282`), `SinkFactory` (`:347`). All are used as `Box<dyn …>` (object-safe) and already have defaulted methods (`description`, `categories`), so adding more defaulted methods is safe and touches no existing impls.
- Each factory exposes `parameter_schema()`, `get_input_ports()`, `get_output_ports()`.
- `DagSchemas::from_graphs(entry_graph_id, graphs, factories, global_params) -> crate::Result<DagSchemas>` (`dag_schemas.rs:131`) — non-executing; **panics** internally on unknown action (`:250`), missing from-node (`:271`), missing to-node (`:275`). `SchemaNodeType` exposes `node`, `kind: Option<NodeKind>`, `with: Option<HashMap<String, serde_json::Value>>` (`:36-45`).
- `BuilderDag::new(ctx, dag_schemas)` materializes per-node `output_ports` incl. dynamic router/filter ports (`builder_dag.rs:262-303`). `NodeType { handle, name, kind: NodeKind, output_ports: Vec<Port>, … }`.
- `dot.rs` is the template for a non-executing CLI command: read workflow → `Workflow::try_from` → `DagSchemas::from_graphs(…, ALL_ACTION_FACTORIES + SYSTEM_ACTION_FACTORY_MAPPINGS, None)` (`dot.rs:42-66`).
- CLI wiring: `cli/src/cli.rs` — `build_cli()` registers subcommands (`:12`), `CliCommand` enum (`:27`), `parse_cli_args` match (`:41`), `execute` match (`:57`).
- `cli/src/errors.rs` — `Error { Parse, Init, Run, UnknownCommand }`, `Result<T>`. `main.rs` prints `✘ Command failed` + exit 1 on `Err`.

---

## File structure

| File | Responsibility | Action |
|---|---|---|
| `runtime/types/src/attr_schema.rs` | `AttrType`, `Presence`, `AttrField`, `AttrSchema`, `AttrRef` types + join/util methods | Create |
| `runtime/types/src/lib.rs` | export `attr_schema` module + re-exports | Modify |
| `runtime/runtime/src/node.rs` | add default-`None` `infer_output_schema` + `referenced_input_attributes` to the 3 factory traits | Modify |
| `runtime/runtime/src/schema_infer.rs` | propagation pass over `BuilderDag` + `Diagnostic`/`Severity` + `infer_and_validate()` | Create |
| `runtime/runtime/src/lib.rs` | declare `pub mod schema_infer;` | Modify |
| `runtime/action-processor/src/attribute/manager.rs` | implement the two trait methods for `AttributeManagerFactory` | Modify |
| `runtime/action-processor/src/attribute/keeper.rs` | implement the two trait methods for `AttributeKeeperFactory` | Modify |
| `runtime/action-processor/src/feature/filter.rs` | implement identity passthrough + (no) references for `FeatureFilterFactory` | Modify |
| `cli/src/build.rs` | `build`/`check` subcommand | Create |
| `cli/src/cli.rs` | register the subcommand | Modify |
| `cli/src/main.rs` | `mod build;` | Modify |
| `cli/tests/build_command.rs` (or inline) | integration test of the command on a fixture workflow | Create |

---

## Type definitions (single source of truth — used by all later tasks)

```rust
// runtime/types/src/attr_schema.rs
use indexmap::IndexMap;
use crate::attribute::Attribute;

/// Coarse attribute type, mirroring AttributeValue variants but value-free.
/// `Unknown` = key known, type not statically determinable (e.g. behind an expression).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    Bool,
    Number,
    String,
    DateTime,
    Array,
    Map,
    Bytes,
    Null,
    Unknown,
}

/// Whether a field is guaranteed present, or only conditionally produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Always,
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrField {
    pub ty: AttrType,
    pub presence: Presence,
}

impl AttrField {
    pub fn always(ty: AttrType) -> Self { Self { ty, presence: Presence::Always } }
    pub fn maybe(ty: AttrType) -> Self { Self { ty, presence: Presence::Maybe } }
}

/// A node's attribute schema on one port.
/// `open == true` means the node may emit attributes whose names we can't
/// enumerate statically (sources, flatten, multi-expr) — disables "missing attr" errors.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttrSchema {
    pub fields: IndexMap<Attribute, AttrField>,
    pub open: bool,
}

impl AttrSchema {
    /// A fully-unknown schema: any attribute may exist. Used to seed sources.
    pub fn open() -> Self { Self { fields: IndexMap::new(), open: true } }

    /// An empty, closed schema: no attributes (e.g. a node that drops all input).
    pub fn empty() -> Self { Self { fields: IndexMap::new(), open: false } }

    pub fn insert(&mut self, name: Attribute, field: AttrField) {
        self.fields.insert(name, field);
    }

    /// Join two schemas arriving on the SAME port (e.g. a router rejoin):
    /// - field in both, same type   -> that type, presence = stricter? NO: presence = Maybe unless both Always
    /// - field in both, diff type    -> AttrType::Unknown, presence as above
    /// - field in only one branch    -> Presence::Maybe
    /// - open if either is open
    pub fn join(&self, other: &AttrSchema) -> AttrSchema {
        let mut out = AttrSchema { fields: IndexMap::new(), open: self.open || other.open };
        for (name, a) in &self.fields {
            match other.fields.get(name) {
                Some(b) => {
                    let ty = if a.ty == b.ty { a.ty } else { AttrType::Unknown };
                    let presence = if a.presence == Presence::Always && b.presence == Presence::Always {
                        Presence::Always
                    } else {
                        Presence::Maybe
                    };
                    out.fields.insert(name.clone(), AttrField { ty, presence });
                }
                None => { out.fields.insert(name.clone(), AttrField { ty: a.ty, presence: Presence::Maybe }); }
            }
        }
        for (name, b) in &other.fields {
            if !self.fields.contains_key(name) {
                out.fields.insert(name.clone(), AttrField { ty: b.ty, presence: Presence::Maybe });
            }
        }
        out
    }
}

/// An attribute a node reads from its input — checked against the inferred input schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrRef {
    pub name: Attribute,
    /// Input port the reference applies to (use DEFAULT_PORT name "default" when single-input).
    pub port: String,
}
```

```rust
// runtime/runtime/src/schema_infer.rs (diagnostics + entry point — signatures only here)
use std::collections::HashMap;
use reearth_flow_types::attr_schema::AttrSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity { Error, Warning }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub node_id: String,
    pub node_name: String,
    pub message: String,
}

pub struct InferResult {
    pub diagnostics: Vec<Diagnostic>,
    /// Inferred output schema per node handle id, keyed by output port name.
    pub node_outputs: HashMap<String, HashMap<String, AttrSchema>>,
}

impl InferResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
}

/// Walk the builder dag in topo order, propagate schemas, collect diagnostics.
/// Returns Err on a structural problem the walk itself can't proceed past (e.g. a cycle).
pub fn infer_and_validate(
    dag: &reearth_flow_runtime::builder_dag::BuilderDag,
) -> Result<InferResult, crate::errors::ExecutionError>; // exact path resolved in Task 5
```

```rust
// New trait methods (added to ProcessorFactory; SourceFactory & SinkFactory get the same
// two methods with identical default bodies). node.rs.
fn infer_output_schema(
    &self,
    _inputs: &std::collections::HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>,
    _with: &Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Option<std::collections::HashMap<Port, reearth_flow_types::attr_schema::AttrSchema>> {
    None
}

fn referenced_input_attributes(
    &self,
    _with: &Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Vec<reearth_flow_types::attr_schema::AttrRef> {
    Vec::new()
}
```

---

## Task 1: `AttrSchema` types + join semantics

**Files:**
- Create: `runtime/types/src/attr_schema.rs`
- Modify: `runtime/types/src/lib.rs`
- Test: inline `#[cfg(test)]` in `attr_schema.rs`

- [ ] **Step 1: Write the failing test** (append to `attr_schema.rs` after writing the types)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::Attribute;
    use pretty_assertions::assert_eq;

    fn attr(s: &str) -> Attribute { Attribute::new(s.to_string()) }

    #[test]
    fn join_same_type_both_always_stays_always() {
        let mut a = AttrSchema::empty();
        a.insert(attr("x"), AttrField::always(AttrType::String));
        let mut b = AttrSchema::empty();
        b.insert(attr("x"), AttrField::always(AttrType::String));
        let j = a.join(&b);
        assert_eq!(j.fields.get(&attr("x")), Some(&AttrField::always(AttrType::String)));
        assert!(!j.open);
    }

    #[test]
    fn join_diff_type_becomes_unknown() {
        let mut a = AttrSchema::empty();
        a.insert(attr("x"), AttrField::always(AttrType::String));
        let mut b = AttrSchema::empty();
        b.insert(attr("x"), AttrField::always(AttrType::Number));
        let j = a.join(&b);
        assert_eq!(j.fields.get(&attr("x")).unwrap().ty, AttrType::Unknown);
    }

    #[test]
    fn join_one_branch_only_becomes_maybe() {
        let mut a = AttrSchema::empty();
        a.insert(attr("x"), AttrField::always(AttrType::String));
        let b = AttrSchema::empty();
        let j = a.join(&b);
        assert_eq!(j.fields.get(&attr("x")).unwrap().presence, Presence::Maybe);
    }

    #[test]
    fn join_open_propagates() {
        let a = AttrSchema::open();
        let b = AttrSchema::empty();
        assert!(a.join(&b).open);
    }
}
```

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p reearth-flow-types attr_schema` → FAIL (module/types not defined).
- [ ] **Step 3: Implement** — write the full type block from "Type definitions" above into `attr_schema.rs`; add `pub mod attr_schema;` to `runtime/types/src/lib.rs` (place near other `pub mod` lines). Confirm `Attribute::new` exists (nutype generates it; `attribute.rs:20-37`) — it does.
- [ ] **Step 4: Run test** — `cargo test -p reearth-flow-types attr_schema` → PASS (4 tests).
- [ ] **Step 5: Commit** — `git add runtime/types/src/attr_schema.rs runtime/types/src/lib.rs && git commit -m "feat(types): add AttrSchema attribute-schema model with join semantics"`

---

## Task 2: Default-`None` trait methods on the factory traits

**Files:**
- Modify: `runtime/runtime/src/node.rs` (the three `*Factory` traits: `:218`, `:282`, `:347`)
- Test: build-only (no behavior change; existing impls must still compile)

- [ ] **Step 1: Write the failing test** — add a test module to `node.rs` that asserts the default returns `None`/empty for a trivial factory. Use an existing in-repo factory if importable; otherwise assert via a tiny local stub:

```rust
#[cfg(test)]
mod schema_trait_tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    struct DummyProc;
    impl ProcessorFactory for DummyProc {
        fn name(&self) -> &str { "Dummy" }
        fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> { None }
        fn get_input_ports(&self) -> Vec<Port> { vec![DEFAULT_PORT.clone()] }
        fn get_output_ports(&self) -> Vec<Port> { vec![DEFAULT_PORT.clone()] }
        fn build(&self, _c: NodeContext, _e: EventHub, _a: String, _w: Option<HashMap<String, Value>>)
            -> Result<Box<dyn Processor>, BoxedError> { unreachable!() }
    }

    #[test]
    fn defaults_are_none_and_empty() {
        let f = DummyProc;
        assert!(f.infer_output_schema(&HashMap::new(), &None).is_none());
        assert!(f.referenced_input_attributes(&None).is_empty());
    }
}
```

- [ ] **Step 2: Run test** — `cargo test -p reearth-flow-runtime schema_trait_tests` → FAIL (methods don't exist).
- [ ] **Step 3: Implement** — add the two defaulted methods (exact bodies from "Type definitions") to `ProcessorFactory`, `SourceFactory`, `SinkFactory`. Add `use reearth_flow_types::attr_schema::{AttrSchema, AttrRef};` if not already imported (it isn't — add it near the existing `use reearth_flow_types::Feature;` at `node.rs:7`).
- [ ] **Step 4: Run test** — `cargo test -p reearth-flow-runtime schema_trait_tests` → PASS; then `cargo build -p reearth-flow-runtime` and `cargo build -p reearth-flow-action-processor` → both compile (proves no existing impl was broken).
- [ ] **Step 5: Commit** — `git add runtime/runtime/src/node.rs && git commit -m "feat(runtime): add default schema-inference hooks to factory traits"`

---

## Task 3: `AttributeManager` transfer function + references

**Files:**
- Modify: `runtime/action-processor/src/attribute/manager.rs`
- Test: inline `#[cfg(test)]` in `manager.rs`

> Behavior to model (verified `manager.rs`): operations are Convert / Create / Rename / Remove keyed by `operations[].attribute`/`value`/`method`. Convert & Create set a key (value type from a Rhai expr → `Unknown`). Rename moves `attribute`→`value`. Remove drops `attribute`. Unlisted attributes pass through. Output port = `default`. References = the `attribute` of each Convert/Rename/Remove op (the source key it reads).

- [ ] **Step 1: Write the failing test** — covering create-adds-key, remove-drops-key, rename-moves-key, passthrough, and that a Convert op produces an `AttrRef` to its source. (Read the actual `AttributeManagerParam`/`Method` enum names from the file and match them exactly; do not invent variant names.) Build input schema with two known attrs, run `infer_output_schema`, assert resulting fields. Assert `referenced_input_attributes` lists the read keys.
- [ ] **Step 2: Run test** — `cargo test -p reearth-flow-action-processor attribute::manager` → FAIL.
- [ ] **Step 3: Implement** — `impl` the two methods on `AttributeManagerFactory`. Deserialize `with` via the existing `serde_json::from_value::<AttributeManagerParam>` path the factory's `build` already uses. Start the output from `inputs.get(DEFAULT_PORT)` cloned (or `AttrSchema::open()` if absent); apply each op: Create/Convert → `insert(name, AttrField::always(AttrType::Unknown))`; Rename → remove old, insert new (carry old field's type if present else Unknown); Remove → `fields.shift_remove`. Return `{ DEFAULT_PORT: out }`. Collect `AttrRef` for Convert/Rename/Remove source keys.
- [ ] **Step 4: Run test** — PASS.
- [ ] **Step 5: Commit** — `git add runtime/action-processor/src/attribute/manager.rs && git commit -m "feat(processor): static attribute-schema transfer for AttributeManager"`

---

## Task 4: `AttributeKeeper` + `FeatureFilter` transfer functions

**Files:**
- Modify: `runtime/action-processor/src/attribute/keeper.rs`, `runtime/action-processor/src/feature/filter.rs`
- Test: inline in each file

> `AttributeKeeper` (verify param name, likely `keep: Vec<Attribute>` or `keepAttributes`): output schema = input restricted to the kept keys; references = the kept keys; `open` becomes false (it produces a closed set). `FeatureFilter`: filters rows, not attributes → identity passthrough of input schema on every output port it declares; no attribute references (its conditions are expressions — treat as no static AttrRef for v1).

- [ ] **Step 1: Write failing tests** — Keeper: input {a,b,c}, keep [a,c] → output {a,c}, closed; refs = [a,c]. Filter: input schema in → identical schema out on each output port; refs empty.
- [ ] **Step 2: Run** → FAIL.
- [ ] **Step 3: Implement** both. For `FeatureFilter`, iterate `self.get_output_ports()` and map each to the (cloned) default input schema.
- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** — `git commit -m "feat(processor): static schema transfer for AttributeKeeper and FeatureFilter"`

---

## Task 5: Propagation pass over `BuilderDag`

**Files:**
- Create: `runtime/runtime/src/schema_infer.rs`
- Modify: `runtime/runtime/src/lib.rs` (add `pub mod schema_infer;`)
- Test: inline `#[cfg(test)]` building a small `BuilderDag` from a fixture workflow

> Resolve the exact `BuilderDag` accessors first (Read `builder_dag.rs`): how to get nodes (`graph()`), each `NodeType.output_ports`, `NodeType.kind` (`NodeKind::{Source,Processor,Sink}`), `NodeType.with`, and edges (`from`/`to` ports). Use `petgraph::algo::toposort` for ordering; on `Err(Cycle)` return an `ExecutionError`. Pick or add an `ExecutionError` variant for "schema inference cycle" (Read `runtime/runtime/src/errors.rs` and reuse an existing variant if one fits, else add `SchemaInferenceCycle`).

- [ ] **Step 1: Write the failing test** — construct a `DagSchemas` then `BuilderDag` from an inline 3-node workflow (source → AttributeManager(create "foo") → AttributeKeeper(keep ["foo","missing"])). Call `infer_and_validate`. Assert: no panic; `node_outputs` for the manager contains `foo`; and a **Warning or Error** diagnostic is produced for `AttributeKeeper` referencing `missing` (because the manager's output is closed and lacks `missing`). Assert the source's downstream `open` flag suppresses errors where appropriate.
- [ ] **Step 2: Run** — `cargo test -p reearth-flow-runtime schema_infer` → FAIL.
- [ ] **Step 3: Implement** — algorithm:
  1. `toposort(dag.graph(), None)` → ordered node indices, or `Err` → return cycle error.
  2. For each node in order: gather inputs: for each incoming edge, look up the producer node's already-computed output schema for the edge's `from` port; group by the edge's `to` port; `join` multiple schemas landing on the same `to` port. Source nodes (no incoming edges): seed each output port with `AttrSchema::open()`.
  3. Get the node's factory from `NodeType.kind` (match the three `NodeKind` arms → call the trait method on the inner factory). Call `infer_output_schema(inputs_by_port, with)`. If `None`: produce, for each declared output port, an identity passthrough of the joined input (or `open()` if it has no inputs) — i.e. unknown processors are schema-transparent rather than schema-erasing, since most non-attribute processors don't touch attributes. Store result in `node_outputs[node_id]`.
  4. Reference check: `referenced_input_attributes(with)`; for each `AttrRef`, find the joined input schema for that port. If the schema is **not `open`** and the field is absent → `Severity::Error` diagnostic ("references attribute `X` not present in input"). If present but `Presence::Maybe` → `Severity::Warning`. If `open` → no diagnostic.
- [ ] **Step 4: Run** → PASS.
- [ ] **Step 5: Commit** — `git add runtime/runtime/src/schema_infer.rs runtime/runtime/src/lib.rs && git commit -m "feat(runtime): topological attribute-schema propagation + reference validation"`

---

## Task 6: `build` / `check` CLI command

**Files:**
- Create: `cli/src/build.rs`
- Modify: `cli/src/cli.rs`, `cli/src/main.rs`
- Test: `cli/tests/build_command.rs` (or `#[cfg(test)]` in `build.rs`)

> Mirror `dot.rs` exactly for load + DAG build, then add `BuilderDag::new` + `infer_and_validate`. `BuilderDag::new` needs a `NodeContext`/`EventHub` — Read `dag_executor.rs:49-64` for how `run`/`dot` construct these and replicate the minimal ctx (no execution, no storage writes). Print diagnostics to stderr; exit non-zero (via `crate::errors::Error::run`) iff `has_errors()`. Add `--workflow` arg like `dot.rs`. Register as subcommand `"build"` with visible alias `"check"`.

- [ ] **Step 1: Write the failing test** — a fixture workflow YAML with a known-bad reference; run `BuildCliCommand::parse_cli_args` + `execute` (or invoke the binary via `assert_cmd` if that's the repo pattern — check `cli/tests/`), assert it returns `Err`/non-zero and emits the expected diagnostic text. Add a second fixture that is valid → returns `Ok`.
- [ ] **Step 2: Run** → FAIL (command not defined).
- [ ] **Step 3: Implement** — `build.rs` with `build_build_command()`, `BuildCliCommand { workflow_path }`, `parse_cli_args`, `execute`. Wire into `cli.rs`: import, `.subcommand(build_build_command().display_order(7))`, `CliCommand::Build(BuildCliCommand)` arm, `"build" | "check" =>` in both matches. Add `mod build;` to `main.rs`.
- [ ] **Step 4: Run** → PASS; manual smoke: `cargo run -p reearth-flow-cli -- build --workflow <fixture>`.
- [ ] **Step 5: Commit** — `git add cli/src/build.rs cli/src/cli.rs cli/src/main.rs cli/tests/build_command.rs && git commit -m "feat(cli): add 'build'/'check' static workflow validation command"`

---

## Task 7: End-to-end fixtures + final verification

**Files:**
- Create: 2 fixture workflows under `runtime/tests/fixture/workflow/schema_infer/` (valid + invalid-reference)
- Test: covered by Task 6 integration test

- [ ] **Step 1** — write `valid.yml` (source → AttributeManager creates `area` → AttributeKeeper keeps `area`) and `invalid.yml` (AttributeKeeper keeps `area` with no producer). Use valid UUIDs for all ids (monorepo constraint).
- [ ] **Step 2** — point the Task 6 integration test at these fixtures.
- [ ] **Step 3** — run the full gate: `cargo make format`, `cargo make clippy`, `cargo test -p reearth-flow-types -p reearth-flow-runtime -p reearth-flow-action-processor -p reearth-flow-cli`. All green.
- [ ] **Step 4: Commit** — `git commit -m "test: end-to-end fixtures for static schema validation"`

---

## Self-review checklist (run before handing to executor)

- Spec coverage: schema model (T1), trait hooks (T2), ≥3 processor transfer fns (T3,T4), propagation+validation (T5), CLI (T6), e2e (T7). ✓
- Type consistency: `AttrSchema`/`AttrField`/`AttrType`/`Presence`/`AttrRef`/`Diagnostic`/`Severity`/`InferResult`/`infer_and_validate` defined once in "Type definitions", referenced unchanged thereafter. `infer_output_schema` returns `Option<HashMap<Port, AttrSchema>>` everywhere. ✓
- Known unknowns to resolve *in-task by Reading the file first* (flagged inline, not placeholders): exact `AttributeManagerParam`/`Method`/`AttributeKeeperParam` field & variant names (T3,T4); exact `BuilderDag` accessors & `NodeContext`/`EventHub` construction (T5,T6); whether `errors.rs` needs a new cycle variant (T5); repo's CLI test harness style (T6).
- Risks carried from investigation: sources seed `open` → keep "unknown processor = passthrough" so info survives; reference errors only fire against **closed** schemas (no false positives on `open`).

# Static Attribute-Schema "Build" Step for reearth-flow/engine

*Investigation + implementation plan. Inspired by dbt-fusion's static SQL/column comprehension.*

## 1. The problem (in the user's words)

> Workflows that involve `Attribute*` processors require knowing the schema of the
> field, and currently it's impossible to know that without looking into the data or
> running the workflow itself.

Today a reearth-flow `Feature` carries a fully **dynamic, untyped** attribute bag
(`Attributes = IndexMap<Attribute, AttributeValue>`, `runtime/types/src/feature.rs:48,53`).
Nothing in the engine records what attributes a node *produces*, so the only way to
learn a node's output schema is to run the graph and inspect features. We want a
**static "build" pass** that infers and propagates an attribute schema through the DAG
*without executing it or touching data* — the same value dbt-fusion delivers for SQL
columns.

**Primary goal (confirmed): pre-run validation.** v1 is a `build`/`check` command that
fails *before* execution when a workflow references an attribute that a node doesn't (or
may not) produce — plus the structural checks the engine currently does via `panic!`.
The emitted schema artifact is secondary to the pass/fail validation result; diagnostics
quality and severity tiers (error vs. warning) are first-class. (Rhai is being replaced
soon, so its current type-opacity is treated as temporary — see §7.)

## 2. What dbt-fusion does (the inspiration)

dbt-fusion does NOT have a single "schema inferencer." It reuses its **Jinja type
checker** (a vendored minijinja under `crates/dbt-jinja/`) as a static-analysis engine,
plus a persisted schema store. Key mechanics:

- **One recursive type lattice.** `Type` enum (`crates/dbt-jinja/minijinja/src/types/mod.rs:46`).
  A relation/record schema is a `StructType { fields: BTreeMap<String, Type> }`
  (`types/struct_.rs:13`). Column access = `get_attribute`; missing column → warning +
  degrade to `Any{hard:false}`.
- **Two-tier "unknown".** `Any{hard:true}` = genuinely dynamic (lattice **top**);
  `Any{hard:false}` = "analysis gap," kept *below* top so unions preserve partial info
  (`mod.rs:82`). A permissive `is_compatible_with` (accept any unknown at use sites, no
  false positives) is paired with a strict `is_subtype_of` (used only for joins).
- **Per-node inference = abstract interpretation.** Each model's compiled bytecode is
  run with *types* as abstract values (`vm/typemeta.rs`: `TypeChecker::check` worklist
  fixpoint, `transfer_block` per-instruction transfer, `merge_into` lattice join via
  `union`).
- **DAG propagation = topological order + a shared schema store**, NOT a global graph
  fixpoint. Upstream model schema enters downstream analysis injected behind `ref()`
  (`dbt-jinja-utils/.../resolve_model_context.rs:350`). Ground-truth leaf schemas come
  from `SchemaStore` (Arrow schemas cached to Parquet, `dbt-schema-store/src/lib.rs`).
  Only leaf sources need real catalog introspection; everything downstream is pure
  static propagation.
- **Diagnostics are a decoupled listener** with span-precise, suppressible (`noqa`),
  on/off-switchable warnings (`dbt-jinja-utils/src/listener.rs:311`). Type mismatches
  are *warnings*, not hard failures.
- **Command model = two axes.** *What* (`CoreCommand`: parse/compile/build/run,
  `dbt-clap-core/src/commands.rs:15`) × *how far* (`Phases`: Parse→Compile→…→All,
  `dbt-common/src/io_args.rs:802`). `compile` is literally `build` minus execution — same
  pipeline, an earlier `return` at a phase checkpoint (`dbt-main/src/dbt_lib.rs:1260`).
  `build` always writes `manifest.json` (the resolved graph), even on failure.
- **Caching is two independent layers** (worth borrowing the *separation*): a parse-state
  cache keyed by `(file mtime, blake3 of config, vars hash, binary version)`
  (`dbt-metadata/src/partial_parse.rs`), and a node-execution cache keyed by
  `node-content-hash + upstream-hashes (+ freshness)` (`dbt-run-cache`).

## 3. What the engine looks like today (the target)

- **Feature/attribute model.** `Feature { id, attributes: Arc<Attributes>, geometry }`
  (`runtime/types/src/feature.rs:50`). `AttributeValue` is an untagged enum:
  `Null | Bool | Number | String | DateTime | Array | Map | Bytes` (`attribute.rs:45`).
  **No int/float distinction** (both `Number`). Attribute keys are arbitrary strings;
  the set mutates freely at runtime.
- **Three graph stages, all `petgraph::DiGraph`:** parsed DTOs (`types/workflow.rs`) →
  **`DagSchemas`** (resolved nodes + subgraph expansion, `runtime/runtime/src/dag_schemas.rs:122`)
  → `BuilderDag` (instantiated factories, `builder_dag.rs`) → `ExecutionDag` (channels).
- **`DagSchemas` is the static gold mine.** `from_graphs(entry, graphs, factories, params)`
  fully expands subgraphs (depth cap 1000, cycle detection) **without executing**. Each
  `SchemaNodeType` holds `node`, `kind: Option<NodeKind>` (the factory), and `with`
  (resolved params). `cli/src/dot.rs:61` already calls this to emit DOT — the exact
  template a `build` command should follow.
- **No schema/type notion exists anywhere.** Searches for `attribute_schema`,
  `infer_schema`, `output_attributes`, etc. → 0 hits. The only "schema" today is
  `parameter_schema()` (schemars JSON Schema of an action's *config*, not its features)
  and value-driven CityGML conversions.
- **Ports are untyped strings.** Factories declare `get_input_ports`/`get_output_ports`
  (`runtime/runtime/src/node.rs`) returning `Vec<Port>` (names only). Some ports are
  config-derived at build time (`builder_dag.rs:262`).
- **No real pre-run validation.** Unknown actions / dangling edges currently `panic!`
  inside `from_graph` (`dag_schemas.rs:246,271,275`).

## 4. Attribute processors: how knowable is each output schema?

13 processors under `runtime/action-processor/src/attribute/`. Classification:

- **Fully static** (names + types from params alone):
  `StatisticsCalculator` (aggregated port), `DateTimeConverter` (default port; type from
  `output_format`), `AttributeDuplicateFilter` (identity on schema).
- **Partial** (key *names* static; value *types* and/or presence runtime-dependent):
  `AttributeManager`, `AttributeAggregator`, `AttributeMapper` (attribute/expr mappers),
  `BulkAttributeRenamer`, `AttributeConversionTable`, `FilePathInfoExtractor`,
  `AttributeRangeMapper`, `NullAttributeMapper`, `BulkArrayJoiner`.
- **Opaque** (output key *set* unknowable without data):
  `AttributeFlattener` (promoted keys = runtime Map contents),
  `AttributeMapper` *with `multiple_expr`* (Rhai returns an arbitrary Map).

Recurring opacity sources: (1) **Rhai expressions** — value type only known at eval;
(2) **runtime Map/Array contents** — Flattener, BulkArrayJoiner, multiple_expr;
(3) **conditional presence** — add/remove gated on runtime values.

Critical subtlety: `AttributeMapper` calls `with_attributes(...)` which **replaces** the
whole attribute set (no passthrough); most others clone+mutate (passthrough). The
inference functions must model this per-processor.

**Sources don't declare schemas.** CSV → all `String`, names = header row (data-derived);
`FeatureCreator` → fully opaque (Rhai); DB/GeoPackage → names+types from DB catalog.
So there is usually **no static seed** except hardcoded keys some processors add. This is
the single biggest divergence from dbt (dbt's leaves have declared/catalog schemas).

## 5. Proposed design

### 5.1 Schema model (`runtime/types`, next to `AttributeValue`)

```rust
pub enum AttrType { Bool, Number, String, DateTime, Array, Map, Bytes, Null, Unknown }

pub struct AttrSchema {
    pub fields: indexmap::IndexMap<Attribute, AttrField>, // ordered like Attributes
    pub open: bool, // node may emit attrs we can't enumerate (Flattener, sources, multiple_expr)
}

pub struct AttrField { pub ty: AttrType, pub presence: Presence } // Always | Maybe
```

Mirrors dbt's two opacity dimensions: `AttrType::Unknown` = known key / unknown type;
`open` = unknown key set; `Presence::Maybe` = conditional add. Keep `Unknown` *below*
top so joins preserve info (dbt's soft-any lesson).

### 5.2 Per-node transfer function (trait method)

Add an **optional** method to `ProcessorFactory`/`SourceFactory`/`SinkFactory`
(`runtime/runtime/src/node.rs`), default returns `None` = opaque, so rollout is incremental
and unconverted actions stay sound:

```rust
fn infer_output_schema(
    &self,
    inputs: &HashMap<Port, AttrSchema>,
    with: &Option<HashMap<String, serde_json::Value>>, // same params build() gets
) -> Option<HashMap<Port, AttrSchema>> { None }
```

Keyed by `Port` on both sides (DateTimeConverter has `default`+`failed`,
StatisticsCalculator `default`+`complete`, NullAttributeMapper 3 ports). Each impl reuses
the existing `serde_json::from_value::<XxxParam>` deserialization.

**For validation, add a second method describing what a node *references*.** Production
alone can't validate — the pass must also know which attribute names each node *consumes*
from its input, to check them against the inferred input schema:

```rust
fn referenced_input_attributes(
    &self,
    with: &Option<HashMap<String, serde_json::Value>>,
) -> Option<Vec<AttrRef>> { None } // AttrRef { name, port, required: bool }
```

Many processors reference attributes by static param name — e.g. `AttributeKeeper.keep`,
`AttributeManager.operations[].attribute`, `Aggregator.group_by`, `filter_by`. The
validator compares each `AttrRef` against the node's inferred input schema: name absent &
schema not `open` → **error**; name present but `Presence::Maybe` → **warning**; schema
`open` → no diagnostic (can't disprove). Once Rhai is replaced, references inside
expressions become statically extractable and flow through the same `AttrRef` path.

### 5.3 Propagation pass (`runtime/runtime`, new module)

**Hook point (corrected by adversarial review): run on `BuilderDag`, not `DagSchemas`.**
The dynamic config-derived output ports — `OutputRouter`'s `routingPort` and
`FeatureFilter`'s `conditions[].outputPort` — **do not exist at the `DagSchemas` stage**;
`get_output_ports()` returns `vec![]` / `["unfiltered"]` there, and the real ports are
synthesized only in `BuilderDag::new` (`builder_dag.rs:262-303`, also duplicated in
`incremental.rs:341-374`). Keying inference outputs by `Port` on `DagSchemas` would
silently drop the very ports branching workflows route through. Two options:
- **(a) Preferred:** run the pass on `BuilderDag.graph()`, whose `NodeType.output_ports`
  (`builder_dag.rs:27`) are already materialized. Cost: factories are instantiated
  (`build()` runs) — acceptable for a build command.
- **(b)** Stay pre-instantiation but extract the dynamic-port logic into ONE shared
  function called by builder_dag, incremental, and the pass — do not add a fourth copy.

Walk in topological order:
1. **Cycle check first.** Use `petgraph::algo::toposort`; surface `Cycle` as a `build`
   diagnostic. The graph is NOT guaranteed acyclic — the only existing cycle handling is
   the *subgraph-recursion* depth cap (`dag_schemas.rs:147,214`), not action-node cycles.
2. Seed source nodes (mostly `open` / `Unknown`, or declared where possible).
3. For each node, gather input schemas per port from incoming edges (`SchemaEdgeType.from/to`).
   Use the generic `union` join **only when multiple edges land on the SAME port** (router
   rejoin: same key/different type → `Unknown`; key in one branch → `Presence::Maybe`).
4. Call `infer_output_schema(inputs_by_port, with)`; `None` → outputs `open`/`Unknown`.
   **Join-style nodes** (e.g. `FeatureMerger` with distinct `requestor`/`supplier` inputs
   → `merged`/`unmerged` outputs, `merger.rs:51-57`) combine their *named* input ports
   inside their own impl — they must NOT be pre-unioned into one bag.
5. **System ports** (`rejected`/`remain`/`unfiltered`, `node.rs:19,24`, `filter.rs:18`)
   default to identity-passthrough of the input schema, not `open`.
6. Record per-edge / per-node schema for emission.

Reuse `collect_ancestor_sources` (`dag_schemas.rs:303`) and the prefix-chain/port-naming
machinery in `runtime/runtime/src/incremental.rs`.

**Out of scope for v1:** geometry typing (`Feature.geometry`, `feature.rs:54`) — the
artifact is an *attribute* contract, not a full feature contract. State this explicitly.

### 5.4 `build` / `check` CLI command (`cli/src/build.rs`, mirror `dot.rs`)

- Register in `cli/src/cli.rs` (add `CliCommand::Build` arm + subcommand). Reuse the
  `run.rs`/`dot.rs` load seam: stdin-or-storage read → `expand_yaml_includes` →
  `Workflow::try_from` → `--var` merge.
- Build the static DAG exactly as `dot.rs:61`:
  `DagSchemas::from_graphs(entry, graphs, ALL_ACTION_FACTORIES + SYSTEM_ACTION_FACTORY_MAPPINGS, params)`.
- Run the propagation pass; emit an attribute-annotated artifact to stdout (data) and
  diagnostics via `tracing`; non-zero exit on error via `crate::errors::Error`.
- **Harden validation (its own prerequisite phase — see Phase 1).** Convert the
  `panic!`s in `dag_schemas.rs` (entry graph `:140`, subgraph not found `:173`, action
  not found `:251`, dangling edge endpoints `:271`/`:275`, router rewiring `:532`) into
  recoverable errors. This requires threading `Result` through `from_graph` /
  `add_subgraph_after_node`, which today don't return `Result` for these — non-trivial,
  not a footnote.
- **Use `dot.rs`'s factory set, not `run.rs`'s.** `dot.rs:57-59` extends
  `ALL_ACTION_FACTORIES` with `SYSTEM_ACTION_FACTORY_MAPPINGS`; `run.rs:309` does not.
  Without the system mappings, `InputRouter`/`OutputRouter` hit `panic!("Action not
  found")` (`dag_schemas.rs:251`).

### 5.5 Registry + artifact

- Enrich `ActionSchema` / `create_action_schema` (`cli/src/utils.rs:208`) and
  `schema/actions.json` with optional per-port attribute-transform metadata, regenerated
  by `cargo make schema-base`. Add a `check-schema`-style CI freshness gate.
- The `build` artifact (attribute-annotated DAG, JSON) becomes the contract other tools
  (UI autocomplete, validation, docs) consume — the analog of dbt's `manifest.json`.

## 6. Phased implementation plan

**Phase 0 — Schema model + plumbing (no behavior change).**
`AttrType`/`AttrSchema`/`AttrField` in `runtime/types`; add default-`None`
`infer_output_schema` to the three factory traits. Compiles, no-op.

**Phase 1a — Harden graph construction (prerequisite refactor).**
Thread `Result` through `from_graph`/`add_subgraph_after_node`; replace the six `panic!`s
(`dag_schemas.rs:140,173,251,271,275,532`) with `ExecutionError` variants. This is the
secretly-hardest plumbing item and gates a usable validator. Do it before inference.

**Phase 1b — Propagation engine + `build` command skeleton.**
New propagation module over **`BuilderDag`** (materialized ports — see §5.3) with a
`toposort` cycle check; `cli/src/build.rs` mirroring `dot.rs` (incl. its factory set).
With all factories returning `None`, output = "everything open/unknown" — proves the
pipeline end to end.

**Phase 2 — Implement inference for the fully-static three** (StatisticsCalculator,
DateTimeConverter, DuplicateFilter). First real schema output; golden-file tests.

**Phase 3 — The "partial" processors** (Manager, Aggregator, Mapper attribute/expr,
BulkRenamer, ConversionTable, FilePathInfoExtractor, RangeMapper, NullAttributeMapper,
BulkArrayJoiner). Names static, types `Unknown`, conditional adds `Presence::Maybe`.

**Phase 4 — Source seeds.** Best-effort: CSV → all `String` (names `open`), DB/GeoPackage →
read catalog once (the only place a live peek is justified, dbt-style), FeatureCreator →
`open`. Non-attribute pass-through processors → identity.

**Phase 4.5 — Reference checking (the actual v1 validation payoff).** Add
`referenced_input_attributes` to the processors that consume named attributes; the pass
compares references against inferred input schemas and emits error/warning diagnostics.
This is what makes `build` a *validator* rather than just a schema printer.

**Phase 5 — Registry + artifact + UX.** Enrich `actions.json`, emit the annotated-DAG
artifact, decoupled diagnostics (severity tiers: warning vs error, span = node id +
attribute), optional `--select` node scoping later.

## 7. Key risks / open questions

1. **v1 = pre-run validation, scoped to attribute presence/provenance** (not absolute
   typing). No static seed from sources means most schemas are `open` at the leaves
   (CSV → all `String`, FeatureCreator/DB → opaque), so *type* info is thin — but the
   high-value validation is presence-based anyway: "node Z references attribute `X`, but
   no upstream path reliably produces `X`" (error), or "X is `Presence::Maybe` here"
   (warning). Key *names* come from static params and survive propagation, so this works
   even with `open` leaves. Implication: the diagnostics layer (decoupled listener,
   error/warning severity, span = node id + attribute) is core scope, not a nicety.
2. **Rhai opacity is temporary — do not architect around it.** Value types (and, for
   `multiple_expr`, the key *set*) behind today's `expr` fields are `Unknown`/`open`, but
   Rhai is being replaced soon. Design `infer_output_schema` and `AttrType` so a future
   expression layer can return real types/keys (i.e. the trait already passes params and
   returns full types) — the only thing missing is a typeable expression language. Don't
   bake "expr ⇒ Unknown" assumptions into the propagation core; keep it in the per-expr
   resolution that the new language will replace.
3. **`Number` granularity** (no int/float) caps type precision; acceptable for v1.
4. **Warning vs error severity.** dbt treats mismatches as warnings; a `build` step may
   want a stricter error tier. Decide policy.
   - **As-built (prototype):** a missing reference against a *closed* schema is an **Error**;
     a `Presence::Maybe` reference is a **Warning**. Note this is intentionally *stricter
     than runtime*: `AttributeManager` Convert/Rename silently skip a missing source key
     (`manager.rs`), and `DateTimeConverter` routes a missing source to its `failed` port
     rather than failing — so when source seeds land and these Errors become reachable, a
     workflow that runs fine could fail `build`. Revisit whether nodes with a graceful
     runtime fallback (e.g. DateTimeConverter's `failed` port) should downgrade to Warning.
5. **Conditional presence** (`Presence::Maybe`) — decide whether downstream consumers need
   it or whether over-approximating to "present" is fine for v1.
6. **Structural malformations still panic.** The `build` command inherits the `panic!`s in
   `DagSchemas::from_graphs` (unknown action, dangling edge, etc.) — a validator that
   panics on a typo is a sharp edge. Hardening those into clean diagnostics (Phase 1a) was
   NOT done in the prototype; it's flagged in a code comment in `cli/src/build.rs`.
7. **`InferResult.node_outputs`** is fully computed but not yet consumed by the CLI — it's
   the seam Phase 5 (schema-artifact emission) will read.

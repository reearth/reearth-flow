# Memory Management Improvements

## Current Implementation and Issues

Sections are ordered by importance.

### 1. Feature Ownership and Cloning

Features use owned data throughout:
```rust
pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: IndexMap<Attribute, AttributeValue>,  // Can be large (large strings, extracted geometry, etc)
    pub metadata: Metadata,
    pub geometry: Geometry,  // Can be VERY large
}
```

**ISSUE:** Features are cloned many times on each action. All clone points for Feature / ExecutorContext in the runtime core:

| Location | What | Clones per feature |
|----------|------|-------------------|
| `source_node.rs:288` | Feature cloned to create ExecutorContext | 1 (per source-emitted feature) |
| `forwarder.rs:107` | Feature cloned for async feature writer | N (one per output edge) |
| `feature_store.rs:103` | Feature cloned again for JSON serialization | N (one per output edge, if writer enabled) |
| `forwarder.rs:140` | ExecutorContext cloned for multiple downstream senders | N-1 |
| `forwarder.rs:60` | ExecutorContext cloned for port multiplexing | P-1 (P = output ports) |
| `sink_node.rs:220` | ExecutorContext cloned before sink's `process()` | 1 (per sink input) |

For a single forward operation to N downstream nodes with feature writing enabled:
total ≈ **3N - 1 deep clones** (N for feature writers, N for feature store serialization, N-1 for sender fan-out). The source and sink clones add further copies at the pipeline boundaries.

Feature flow through the system:
```
Source creates Feature (owned)
    │
    ▼ feature.clone() to create ExecutorContext (source_node.rs:288)
Channel holds ExecutorContext { feature: Feature, ... }
    │
    ▼ moved into Rayon closure
process() takes ctx by value (ownership)
    │
    ├─► If not forwarded: Feature dropped when process() returns
    │
    └─► If forwarded to N nodes:
            ├─► N clones for feature writers (forwarder.rs:107)
            ├─► N clones for feature store serialization (feature_store.rs:103)
            └─► N-1 clones for downstream senders (forwarder.rs:140)
```

Note: Feature store serialization can be disabled with `FLOW_RUNTIME_FEATURE_WRITER_DISABLE=true`, which skips the `feature_store.rs:103` clone. However, the `forwarder.rs:107` clone still happens unconditionally (the disable check is inside `writer.write()`, after the clone). So with writer disabled: N + (N-1) = **2N - 1 deep clones**.

### 2. Processor Trait Method Signatures

The Processor trait defines three lifecycle methods:
```rust
pub trait Processor: Send + Sync + Debug + ProcessorClone {
    fn initialize(&mut self, _ctx: NodeContext) -> Result<(), BoxedError>;
    fn process(&mut self, ctx: ExecutorContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError>;
    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError>;
}
```

**ISSUE:** `finish(&self)` forces accumulating processors (e.g., `AreaOnAreaOverlayer`, `RayIntersector`) to copy their accumulated features when emitting results. It also prevents early resource cleanup — processor memory is held until `Arc<RwLock<...>>` is dropped.

### 3. Feature Store (Intermediate Data Capture)

The feature store captures all intermediate feature data for debugging/replay. Though this issue is partially covered in 1, there is a little more to say about it.

Write flow (`feature_store.rs`):
1. `feature.clone().into()` — deep clone + convert to `serde_json::Value`
2. Serialize to JSON string
3. Push string into `VecDeque<String>` (per-edge in-memory buffer)
4. When buffer exceeds 512 items → batch flush to disk as JSONL

The batching reduces I/O syscalls, which is reasonable.

**ISSUE:** The feature store runs on the hot path of every feature on every edge, even in production. Specifically:
1. Every feature is deep-cloned before serialization; serializing from a reference would suffice (already discussed above)
2. 512 JSON strings per edge sit in memory before flush.
3. `VecDeque::drain()` does not release capacity — memory is not returned after flush

### 4. DAG Orchestration and Backpressure

All nodes start simultaneously in `DagExecutor::start()`. The only flow control mechanism is implicit backpressure from bounded channels:

- Bounded crossbeam channels (default 256 items) between every node pair
- When a channel is full, `sender.send()` blocks the sending thread

```
Source (goes full speed)
  → bounded channel (256 items) → blocks when full
    → Processor (pulls via sel.ready())
      → bounded channel (256 items) → blocks when full
        → Sink (processes synchronously)
```

**ISSUE:** There is no memory-aware flow control and no way to prioritize draining sinks over feeding sources. Memory pressure from multiple accumulating processors running concurrently is unmanaged.

### 5. Threading and Channel Architecture

One OS thread is spawned per action:

| Node Type | OS Thread | Internal Thread Pool |
|-----------|-----------|---------------------|
| Source | 1 shared | Tokio async tasks |
| Processor | 1 per node | Rayon pool (num_threads workers) |
| Sink | 1 per node | None (synchronous) |

Threads are spawned upfront in `DagExecutor::start()`. Termination occurs when input channels are exhausted and the in-flight work counter reaches 0.

Each processor node creates its own `rayon::ThreadPool`. With N processor nodes: N OS threads (for `receiver_loop()`) + Σ `num_threads(i)` Rayon threads (for `process()`).

Processor node execution flow:

1. **OS thread** runs `receiver_loop()`:
   - Receives features from crossbeam channels
   - Dispatches each feature to Rayon pool via `thread_pool.spawn()`
   - After all channels exhausted and in-flight counter reaches 0, calls `on_terminate()` which runs `finish()` — **directly on this OS thread, not on Rayon**

2. **Rayon threads** run `process()` only:
   - Executes `processor.process(ctx, channel_manager)`

Channel types:

| Component | Channel Type | Actual Work |
|-----------|-------------|-------------|
| Source | `tokio::sync::mpsc` (async) | `source.start().await` (async) |
| Processor | `crossbeam::channel` (sync) | `processor.process()` (sync) |
| Sink | `crossbeam::channel` (sync) | `sink.process()` (sync) |

Rayon spawns parallel tasks, but there is a write lock on the processor:
```rust
// processor_node.rs:404
let mut processor_guard = processor.write();
let result = processor.process(ctx, channel_manager);
```

`num_threads > 1` only helps if features arrive faster than processing (buffering benefit) or the processor releases the lock during I/O. If the processor holds the lock for the entire `process()` call, parallelism is effectively serialized by lock contention.

The processor's OS thread mostly waits on crossbeam channels and dispatches to Rayon:
```rust
loop {
    let index = sel.ready();  // Blocking wait - OS thread idle
    // ... dispatch to Rayon
}
```

Tokio is still used inside Processor/Sink paths. Inside `ChannelManager::send_op()`:
```rust
// forwarder.rs:117
self.runtime.block_on(async move {
    writer.write(&feature).await;
});
```

The actual flow:
```
OS Thread (sync, blocked on crossbeam select)
  → Rayon Thread (sync, process())
    → block_on(async feature writing)  // Rayon thread blocked here
```

Processor/Sink traits are sync while Source is async. There is no technical reason for this — async I/O is still needed, so Processor/Sink work around it with `block_on()`.

**ISSUE:** Per-processor Rayon pools are wasteful when processors are serialized by lock contention. They also cause crossbeam_epoch TLS accumulation (1.72M per thread is observed by experiments via `heaptrack`, scales workflow size) — orphaned TLS when threads exit appears as a memory leak. One OS thread per processor node is mostly idle waiting on channels. Rayon threads block on `block_on()` during I/O.

### 6. Unnecessary Processor Cloning During DAG Creation

Two nearly identical `NodeType` structs exist:

```rust
// builder_dag.rs:20
pub struct NodeType {
    pub handle: NodeHandle,
    pub name: String,
    pub kind: NodeKind,  // Not Option
}

// execution_dag.rs:25
pub struct NodeType {
    pub handle: NodeHandle,
    pub name: String,
    pub kind: Option<NodeKind>,  // Always `Some` until `.take()`d once during executor setup
    pub is_source: bool,         // Added field
}
```

At `execution_dag.rs:145-156`, every processor is deep-cloned to convert between these structs:
```rust
let graph = graph.map(
    |_, node| NodeType {
        kind: match &node.kind {
            NodeKind::Processor(processor) => Some(NodeKind::Processor(processor.clone())),  // Deep clone!
            // ...
        },
    },
);
```

The `Option<NodeKind>` wrapping works as follows:
- `kind` starts as `Some(...)` — always
- `.take()` extracts it exactly once per node (panics if None)
- After `.take()`, `kind` is `None` but never accessed again
- `is_source` exists only because after `.take()`, you can't tell what type it was

**ISSUE:** Every processor is deep-cloned during DAG creation solely to wrap `kind` in `Some()` and add an `is_source` boolean. This can be avoided by merging into one struct with `Option<NodeKind>` from the start, or passing ownership directly.

## Plans for the Fix

Sections are ordered by importance.

### 1. Feature Sharing with `Arc`

`Arc` is **much cheaper** than cloning moderate-sized data, so we will wrap the feature values by `Arc`. The issue is *how* we wrap them. The following four candidate strategies are ordered from coarsest to finest granularity of Arc partitioning:

#### Strategy 1: `Arc<Feature>`
```rust
type SharedFeature = Arc<Feature>;
```
- **Forwarding (fan-out):** `Arc::clone` — no data copied
- **Any modification (attribute or geometry):** `Arc::make_mut` clones the **entire Feature** (attributes + geometry + metadata) if refcount > 1 (i.e., upstream branched and other branches still hold a reference). If refcount == 1, mutation is in-place with zero copies.
- Best when: Features rarely modified, mostly pass-through

#### Strategy 2: `Arc<Geometry>` only
```rust
struct Feature {
    id: uuid::Uuid,
    attributes: IndexMap<Attribute, AttributeValue>,
    metadata: Metadata,
    geometry: Arc<Geometry>,
}
```
- **Forwarding (fan-out):** Deep-clones attributes + metadata; `Arc::clone` geometry (cheap)
- **Attribute modification:** Direct mutation — attributes are always owned (already deep-cloned on forward)
- **Geometry modification:** `Arc::make_mut` clones geometry only if refcount > 1. If unique, mutates in-place.
- Best when: Attributes frequently edited, geometry static

#### Strategy 3: `Arc<Geometry>` + `Arc<Attributes>` **(Front-Runner)**
```rust
struct Feature {
    id: uuid::Uuid,
    attributes: Arc<IndexMap<Attribute, AttributeValue>>,
    metadata: Metadata,
    geometry: Arc<Geometry>,
}
```
- **Forwarding (fan-out):** `Arc::clone` both — no data copied
- **Attribute modification:** `Arc::make_mut` clones the **entire attribute map** if refcount > 1 (e.g., upstream branched). If unique, mutates in-place.
- **Geometry modification:** `Arc::make_mut` clones geometry only if refcount > 1. If unique, mutates in-place.
- Modifications are independent — editing attributes never clones geometry, and vice versa. The refcount > 1 condition arises when the upstream node fans out to multiple downstream nodes.
- Best when: Sometimes geometry, sometimes attributes edited

#### Strategy 4: `Arc<Geometry>` + `HashMap<K, Arc<V>>`
```rust
struct Feature {
    id: uuid::Uuid,
    attributes: IndexMap<Attribute, Arc<AttributeValue>>,
    metadata: Metadata,
    geometry: Arc<Geometry>,
}
```
- **Forwarding (fan-out):** Clones the IndexMap structure (keys + Arc pointers) + `Arc::clone` each value + `Arc::clone` geometry. Map structure is cloned but individual values are not.
- **Single attribute modification:** `Arc::make_mut` on that one value — clones only that value if refcount > 1. Other attribute values remain shared.
- **Geometry modification:** Same as strategies 2/3.
- Best when: Individual attribute values are large and frequently copied

**Note 1: Why not subdivide geometries as well?** (e.g., `MultiPolygon(Vec<Arc<Polygon>>)`)
The sharing boundary is the Feature level, not the polygon level. When a feature fans out to downstream nodes, all nodes get the same whole geometry — no action selectively shares individual polygons across different nodes. Finer-grained Arc wrapping adds indirection on every geometry operation without matching any real access pattern.

**Note 2: Benchmarking difficulty with real workflows.** One might be tempted to measure the estimated clone savings for each Arc strategy using intermediate data from the existing implementation (i.e. determine the best strategy by experiment). However, this is very difficult. Whether a given action's output is a clone of its input or newly constructed data varies per action, and determining this requires heavy profiling of each action individually. Synthetic benchmarks can measure the sharing/cloning mechanics, but predicting real-world gains across the full action set is hard without per-action analysis, which itself takes too much time.

### 2. `finish(&mut self)` for Accumulating Processors

The current `finish(&self)` signature forces accumulating processors (e.g., `AreaOnAreaOverlayer`, `RayIntersector`) to **copy their accumulated features** when emitting results — they can't move data out of `&self`.

Changing to `finish(&mut self)` lets these processors `.take()` or `.drain()` their internal buffers, transferring ownership of accumulated features to downstream nodes instead of copying them.

### 3. Feature Store Fix

The feature store runs on the hot path of every feature on every edge, even in production. The fix:

1. **Disable by default (?)** — The single best solution for at least cloud run without UI. Make feature writing opt-in for debugging rather than opt-out. Production workflows should not pay the serialization cost.
2. **Serialize from reference** — replace `feature.clone().into()` with direct serialization from `&Feature`, eliminating a deep clone per feature per edge
3. **Shrink `VecDeque` after drain** — `VecDeque::drain()` does not release capacity. Call `shrink_to_fit()` after flush to return memory (especially important to clean up at the last flush). The `VecDeque` batching pattern itself is correct, but consider reducing the batch size.

Note: writes go through OpenDAL which may target cloud storage (GCS/S3), so per-feature writes would mean 3 network round-trips per feature. Application-level batching is necessary.

### 4. DAG-Level Memory-Aware Scheduling (Deferred)

**Only pursue this if the three major fixes (Arc sharing, `finish(&mut self)`, feature store) prove insufficient.**

We deliberately avoid a centralized orchestrator — it would make the system overly complicated and non-scalable. Instead, the approach is local adjustment only:

- Add an **attribute macro** (e.g., `#[heavy_finish]`) that marks processors with potentially expensive `finish()` functions (those that accumulate and emit large amounts of data)
- When a marked processor's input channels are exhausted, instead of immediately calling `finish()`, it **waits until the shared thread pool becomes relatively silent** (low in-flight task count) before proceeding
- This prevents memory spikes from multiple expensive `finish()` calls running concurrently, without requiring global knowledge or a central scheduler
- The "silence" threshold can be a simple heuristic (e.g., in-flight tasks < N% of pool size)

### 5. Other Improvements

1. **Merge `NodeType` structs** - Eliminate unnecessary processor cloning during DAG creation

2. **Share Rayon thread pool** - Single global pool instead of per-processor pools. The current implementation already uses a per-node `AtomicU32` counter (`thread_counter`) to track in-flight tasks. This counter is independent of the pool, so switching to a shared pool requires no change to the completion detection logic.

3. **Consider async channels** - Replace crossbeam with tokio channels to eliminate OS thread per processor. For workflows with 100 nodes, this can reduce memory up to a GB. `process()` is dispatched to the Rayon pool as before. **Important:** `finish()` currently runs on the OS thread — when replacing with a tokio task, `finish()` must be dispatched to a blocking thread or the Rayon pool, not run on the tokio runtime (it can be CPU-heavy for accumulating processors).

4. **Remove `FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS`** - This env var controls `NODE_STATUS_PROPAGATION_DELAY`, a sleep used in source/processor/sink termination paths and the processor spin-wait loop. It seems that the only reason for its existence is to "wait for events to propagate" but serves no correctness purpose — it accidentally suppresses non-deterministic libxml2 failures by reducing CPU contention. The root cause was a race on `std::env::set_var("XML_CATALOG_FILES")` in XMLValidator's `get_or_create_schema_context()`, called concurrently from multiple threads. Fixed by introducing `XML_CATALOG_LOCK` (global mutex) around `set_var` + `create_xml_schema_validation_context` in `validator.rs`. With the fix applied, all `NODE_STATUS_PROPAGATION_DELAY` sleeps can be removed. The only concern about this change is that the memory pressure becoming high, but it is no longer an issue after the above fix of reducing data clones.

   **Caveat:** Removing the delay causes workflow tests to fire requests much faster, which can trigger IP-based rate limiting from schema servers (blocked for an unknown duration). This needs to be addressed before removing the delay — e.g., by adding request throttling or mocking external endpoints in tests.
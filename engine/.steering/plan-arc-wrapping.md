# Implementation Plan: Arc-Wrapping Feature Attributes and Geometry (Strategy 3)

## Goal

Reduce memory usage and cloning overhead by wrapping `attributes` and `geometry` fields of `Feature` in `Arc`, enabling cheap clones during fan-out operations while preserving copy-on-write semantics for mutations.

Also includes Fix #3: eliminate unnecessary clones in feature store serialization.

## Current State

```rust
pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: IndexMap<Attribute, AttributeValue>,  // Deep-cloned on every forward
    pub metadata: Metadata,
    pub geometry: Geometry,  // Deep-cloned on every forward
}
```

Clone cost per feature forward to N downstream nodes: **2N-1 deep clones** (with feature writer disabled).

## Target State

```rust
pub type Attributes = IndexMap<Attribute, AttributeValue>;

pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: Arc<Attributes>,
    pub metadata: Metadata,
    pub geometry: Arc<Geometry>,
}
```

Clone cost per feature forward to N downstream nodes: **N-1 Arc::clone operations** (pointer copies).

---

## Phase 1: Core Type Changes (`runtime/types`)

### 1.1 Add Type Alias and Update Feature Struct

**File:** `runtime/types/src/feature.rs`

```rust
use std::sync::Arc;

/// Type alias for feature attributes to reduce verbosity
pub type Attributes = IndexMap<Attribute, AttributeValue>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: Arc<Attributes>,
    pub metadata: Metadata,
    pub geometry: Arc<Geometry>,
}
```

**Note:** Serde handles `Arc<T>` transparently when `T: Serialize/Deserialize`. No custom serializers needed.

### 1.2 Remove `Feature::new()` and `Default`

**Delete these implementations:**

```rust
// DELETE: Feature::new() - forces bad pattern of creating empty then mutating
impl Default for Feature {
    fn default() -> Self {
        Self::new()
    }
}

impl Feature {
    pub fn new() -> Self { ... }  // DELETE
}
```

**Rationale:** Every usage of `Feature::new()` immediately sets attributes/geometry. The proper pattern is to build attributes first, then construct the Feature.

### 1.3 Update Remaining Constructors

All constructors must wrap values in `Arc::new()`:

```rust
impl Feature {
    pub fn new_with_id_and_attributes(
        id: uuid::Uuid,
        attributes: Attributes,
    ) -> Self {
        Self {
            id,
            attributes: Arc::new(attributes),
            metadata: Metadata::default(),
            geometry: Arc::new(Geometry::default()),
        }
    }

    pub fn new_with_attributes(attributes: Attributes) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            metadata: Metadata::default(),
            geometry: Arc::new(Geometry::default()),
        }
    }

    pub fn new_with_attributes_and_geometry(
        attributes: Attributes,
        geometry: Geometry,
        metadata: Metadata,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            geometry: Arc::new(geometry),
            metadata,
        }
    }
}
```

### 1.4 Update Feature Mutation Methods with `Arc::make_mut`

The key insight: `Arc::make_mut(&mut arc)` returns `&mut T`. If refcount == 1, it mutates in-place. If refcount > 1, it clones the inner value, replaces the Arc, and returns a mutable reference to the new copy.

```rust
impl Feature {
    /// Insert an attribute. Uses copy-on-write if shared.
    pub fn insert<T: AsRef<str> + std::fmt::Display>(
        &mut self,
        key: T,
        value: AttributeValue,
    ) -> Option<AttributeValue> {
        Arc::make_mut(&mut self.attributes)
            .insert(Attribute::new(key.to_string()), value)
    }

    /// Extend attributes. Uses copy-on-write if shared.
    pub fn extend(&mut self, attributes: HashMap<Attribute, AttributeValue>) {
        Arc::make_mut(&mut self.attributes).extend(attributes);
    }

    /// Extend attributes from string keys. Uses copy-on-write if shared.
    pub fn extend_attributes(&mut self, attributes: HashMap<String, AttributeValue>) {
        Arc::make_mut(&mut self.attributes)
            .extend(attributes.into_iter().map(|(k, v)| (Attribute::new(k), v)));
    }

    /// Remove an attribute. Uses copy-on-write if shared.
    pub fn remove<T: AsRef<str> + std::fmt::Display>(&mut self, key: T) -> Option<AttributeValue> {
        Arc::make_mut(&mut self.attributes)
            .swap_remove(&Attribute::new(key.to_string()))
    }

    /// Get mutable access to attributes. Uses copy-on-write if shared.
    pub fn attributes_mut(&mut self) -> &mut Attributes {
        Arc::make_mut(&mut self.attributes)
    }

    /// Get mutable access to geometry. Uses copy-on-write if shared.
    pub fn geometry_mut(&mut self) -> &mut Geometry {
        Arc::make_mut(&mut self.geometry)
    }
}
```

### 1.5 Read-Only Access Methods (No Changes Needed)

Methods that only read don't need changes - `Arc<T>` derefs to `&T`:

```rust
impl Feature {
    // These work unchanged because Arc<T>: Deref<Target=T>
    pub fn contains_key<T: AsRef<str> + std::fmt::Display>(&self, key: T) -> bool {
        self.attributes.contains_key(&Attribute::new(key.to_string()))
    }

    pub fn get<T: AsRef<str> + std::fmt::Display>(&self, key: T) -> Option<&AttributeValue> {
        self.attributes.get(&Attribute::new(key.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Attribute, &AttributeValue)> {
        self.attributes.iter()
    }
}
```

### 1.6 Update Builder Methods

```rust
impl Feature {
    /// Replace attributes, keeping other fields. Wraps in new Arc.
    pub fn with_attributes(&self, attributes: Attributes) -> Self {
        Self {
            id: self.id,
            attributes: Arc::new(attributes),
            geometry: Arc::clone(&self.geometry),  // Cheap Arc clone
            metadata: self.metadata.clone(),
        }
    }

    /// Replace attributes by consuming self. More efficient - reuses Arc for geometry.
    pub fn into_with_attributes(self, attributes: Attributes) -> Self {
        Self {
            id: self.id,
            attributes: Arc::new(attributes),
            geometry: self.geometry,  // Move the Arc
            metadata: self.metadata,
        }
    }
}
```

### 1.7 Delete Unnecessary From/Into Implementations

**Delete:**

```rust
// DELETE: Unused
impl From<Feature> for AttributeValue { ... }

// DELETE: Should use serde_json::to_value(&feature) instead
impl From<Feature> for serde_json::Value { ... }
```

### 1.8 Update Remaining From Implementations

```rust
impl From<Attributes> for Feature {
    fn from(v: Attributes) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(v),
            metadata: Metadata::default(),
            geometry: Arc::new(Geometry::default()),
        }
    }
}

impl From<IndexMap<String, AttributeValue>> for Feature {
    fn from(v: IndexMap<String, AttributeValue>) -> Self {
        let attributes = v
            .into_iter()
            .map(|(k, v)| (Attribute::new(k), v))
            .collect::<Attributes>();
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            metadata: Metadata::default(),
            geometry: Arc::new(Geometry::default()),
        }
    }
}

impl From<Geometry> for Feature {
    fn from(v: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            geometry: Arc::new(v),
            metadata: Metadata::default(),
            attributes: Arc::new(Attributes::new()),
        }
    }
}

impl From<serde_json::Value> for Feature {
    fn from(v: serde_json::Value) -> Self {
        // ... parse and wrap in Arc::new()
    }
}
```

### 1.9 Export Type Alias

**File:** `runtime/types/src/lib.rs`

```rust
pub use feature::{Attributes, Feature, MetadataKey};
```

---

## Phase 2: Engine Core Changes (`runtime/runtime`)

### 2.1 Why Clone Sites Don't Need Code Changes

The engine core has these clone hotspots (from the steering doc):

| Location | What | Current Cost |
|----------|------|--------------|
| `forwarder.rs:60` | `ctx.clone()` for port multiplexing | Deep clone of Feature |
| `forwarder.rs:107` | `ctx.feature.clone()` for feature writer | Deep clone of Feature |
| `forwarder.rs:140` | `ctx.clone()` for multiple downstream senders | Deep clone of Feature |
| `sink_node.rs:220` | `ctx.clone()` before sink's process() | Deep clone of Feature |
| `source_node.rs:387` | `ctx.clone()` | Deep clone of Feature |
| `processor_node.rs:365` | `ctx.clone()` in finish() | Deep clone of Feature |

**No code changes needed at these sites.** The `#[derive(Clone)]` on `Feature` automatically generates:
```rust
impl Clone for Feature {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),           // 16 bytes (UUID)
            attributes: self.attributes.clone(),  // Arc::clone - 8 bytes
            metadata: self.metadata.clone(),      // Small struct
            geometry: self.geometry.clone(),      // Arc::clone - 8 bytes
        }
    }
}
```

After Arc wrapping, all existing `.clone()` calls automatically become cheap.

### 2.2 Delete Dead Code in ExecutorContext

**File:** `runtime/runtime/src/executor_operation.rs`

The following impls use `Feature::default()` and are **dead code** (confirmed by removing them and verifying compilation succeeds):

**Delete Line 88-98:**
```rust
// DELETE - dead code
impl From<Context> for ExecutorContext {
    fn from(ctx: Context) -> Self {
        Self {
            feature: Feature::default(),
            // ...
        }
    }
}
```

**Delete Line 101-112:**
```rust
// DELETE - dead code
impl Default for ExecutorContext {
    fn default() -> Self {
        Self {
            feature: Feature::default(),
            // ...
        }
    }
}
```

### 2.3 Summary of Engine Core Files

| File | Change Required |
|------|----------------|
| `forwarder.rs` | **None** - clones automatically cheap |
| `sink_node.rs` | **None** - clones automatically cheap |
| `source_node.rs` | **None** - clones automatically cheap |
| `processor_node.rs` | **None** - clones automatically cheap |
| `executor_operation.rs` | **Delete** dead code: `impl From<Context>` and `impl Default` |
| `feature_store.rs` | Fix #3 changes (separate phase) |

---

## Phase 3: Fix Feature Store (Fix #3 from Steering Doc)

### 3.1 Eliminate Clone in Feature Serialization

**File:** `runtime/runtime/src/feature_store.rs`

**Before (line 103):**
```rust
let item: serde_json::Value = feature.clone().into();
```

**After:**
```rust
let item = serde_json::to_value(feature)
    .map_err(|e| FeatureWriterError::Serialize(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
```

This eliminates a deep clone per feature per edge on the hot path.

### 3.2 Shrink VecDeque After Drain

**File:** `runtime/runtime/src/feature_store.rs`

**After line 117 (after drain):**
```rust
let elements = buffer.drain(..).collect::<Vec<_>>();
buffer.shrink_to_fit();  // Release memory
```

### 3.3 Update Echo Processors

**File:** `runtime/action-sink/src/echo.rs`

**Before:**
```rust
let feature: serde_json::Value = ctx.feature.clone().into();
```

**After:**
```rust
let feature = serde_json::to_value(&ctx.feature)
    .unwrap_or_else(|_| serde_json::Value::Null);
```

**File:** `runtime/action-processor/src/echo.rs` - Same change.

---

## Phase 4: Migrate `Feature::new()` Usages

All ~25 usages of `Feature::new()` must be migrated to build attributes first, then construct.

### Pattern A: Insert-based construction

**Before:**
```rust
let mut feature = Feature::new();
feature.insert("key1", value1);
feature.insert("key2", value2);
```

**After:**
```rust
let mut attrs = Attributes::new();
attrs.insert(Attribute::new("key1"), value1);
attrs.insert(Attribute::new("key2"), value2);
let feature = Feature::new_with_attributes(attrs);
```

### Pattern B: Direct field assignment

**Before:**
```rust
let mut feature = Feature::new();
feature.attributes = some_attributes;
feature.geometry = some_geometry;
```

**After:**
```rust
let feature = Feature::new_with_attributes_and_geometry(
    some_attributes,
    some_geometry,
    Metadata::default(),
);
```

### Files requiring `Feature::new()` migration:

| File | Count |
|------|-------|
| `action-source/src/file/geopackage.rs` | 4 |
| `action-plateau-processor/src/plateau4/attribute_flattener/processor.rs` | 2 |
| `action-plateau-processor/src/plateau4/missing_attribute_detector.rs` | 3 |
| `action-plateau-processor/src/plateau4/domain_of_definition_validator.rs` | 3 |
| `action-plateau-processor/src/plateau4/flooding_area_surface_generator.rs` | 1 |
| `action-plateau-processor/src/plateau4/solid_intersection_test_pair_creator.rs` | 1 |
| `action-plateau-processor/src/plateau4/unshared_edge_detector.rs` | 1 |
| `action-plateau-processor/src/plateau3/domain_of_definition_validator.rs` | 1 |
| `action-processor/src/geometry/area_on_area_overlayer.rs` | 2 |
| `action-processor/src/geometry/line_on_line_overlayer.rs` | 2 |
| `action-processor/src/geometry/dissolver.rs` | 1 |
| `action-processor/src/geometry/convex_hull_accumulator.rs` | 1 |
| `action-processor/src/geometry/image_rasterizer.rs` | 1 |
| `action-processor/src/geometry/csg/csg_builder.rs` | 3 |
| `action-processor/src/attribute/aggregator.rs` | 1 |
| `action-processor/src/attribute/statistics_calculator.rs` | 1 |

---

## Phase 5: Update Action Processors

Most processors follow one of these patterns:

### Pattern A: Read-Only Access (No Changes Needed)

```rust
// Arc<T>: Deref<Target=T> makes this transparent
let value = feature.get("key");
for (k, v) in feature.attributes.iter() { ... }
```

### Pattern B: Mutation via Feature Methods (No Changes Needed)

```rust
// Feature methods already use Arc::make_mut internally
feature.insert("key", value);
feature.remove("key");
feature.extend(new_attrs);
```

### Pattern C: Direct Attributes Field Mutation (Needs Update)

**Before:**
```rust
feature.attributes.insert(key, value);
feature.attributes.extend(map);
feature.attributes = new_map;
```

**After:**
```rust
feature.attributes_mut().insert(key, value);
feature.attributes_mut().extend(map);
feature.attributes = Arc::new(new_map);
```

### Pattern D: Direct Geometry Field Assignment (Needs Update)

**Before:**
```rust
feature.geometry = new_geometry;
feature.geometry.value = new_value;
```

**After:**
```rust
feature.geometry = Arc::new(new_geometry);
feature.geometry_mut().value = new_value;
```

### Pattern E: Building New Feature with Existing Components

**Before:**
```rust
Feature::new_with_attributes_and_geometry(
    feature.attributes.clone(),  // Deep clone
    new_geometry,
    feature.metadata.clone(),
)
```

**After:**
```rust
Feature {
    id: uuid::Uuid::new_v4(),
    attributes: Arc::clone(&feature.attributes),  // Cheap
    geometry: Arc::new(new_geometry),
    metadata: feature.metadata.clone(),
}
```

---

## Phase 6: Files Requiring Changes

### Priority 1: Core Types and Engine Core (Must Change)

| File | Changes |
|------|---------|
| `runtime/types/src/feature.rs` | Add `Attributes` alias, wrap fields in Arc, remove `new()`, delete unused From impls |
| `runtime/types/src/lib.rs` | Export `Attributes` type alias |
| `runtime/runtime/src/executor_operation.rs` | Delete dead code: `impl From<Context>` and `impl Default` |
| `runtime/runtime/src/feature_store.rs` | Serialize from reference, shrink VecDeque |

### Priority 2: Echo Processors (Fix #3)

| File | Changes |
|------|---------|
| `runtime/action-sink/src/echo.rs` | Use `serde_json::to_value(&feature)` |
| `runtime/action-processor/src/echo.rs` | Use `serde_json::to_value(&feature)` |

### Priority 3: High-Impact Action Processors

Files with direct `.attributes.insert()` or `.geometry = ` patterns:

| File | Pattern |
|------|---------|
| `action-processor/src/attribute/flattener.rs` | Direct attributes mutation |
| `action-processor/src/attribute/mapper.rs` | Builds new attribute maps |
| `action-processor/src/attribute/bulk_renamer.rs` | Remove and rebuild cycles |
| `action-processor/src/geometry/coercer.rs` | 11 geometry assignments |
| `action-processor/src/geometry/remover.rs` | `feature.geometry = Geometry::default()` |
| `action-processor/src/geometry/clipper.rs` | 6 geometry assignments |

### Priority 4: PLATEAU Processors

| File | Pattern |
|------|---------|
| `action-plateau-processor/src/plateau4/attribute_flattener/processor.rs` | 20+ attribute mutations |
| `action-plateau-processor/src/plateau4/face_extractor.rs` | Geometry and attribute mutations |

---

## Phase 7: Migration Strategy

### Step 1: Make Feature Changes Compile

1. Add `Attributes` type alias
2. Update `Feature` struct definition with `Arc` wrapping
3. Remove `Feature::new()` and `Default` impl
4. Update remaining constructors to use `Arc::new()`
5. Add `attributes_mut()` and `geometry_mut()` helpers
6. Delete `impl From<Feature> for AttributeValue`
7. Delete `impl From<Feature> for serde_json::Value`
8. Update remaining `From` implementations

**At this point:** Code won't compile due to downstream usage.

### Step 2: Fix Feature Store (Fix #3)

1. Change `feature.clone().into()` to `serde_json::to_value(feature)?`
2. Add `shrink_to_fit()` after drain
3. Update echo.rs files

### Step 3: Migrate `Feature::new()` Usages

Fix all ~25 compilation errors from removed `Feature::new()`.

### Step 4: Fix Remaining Compilation Errors

1. **Direct field assignments** (`feature.geometry = x`):
   - Change to `feature.geometry = Arc::new(x)`

2. **Direct attribute mutations** (`feature.attributes.insert(k, v)`):
   - Change to `feature.attributes_mut().insert(k, v)`

3. **Direct geometry mutations** (`feature.geometry.value = x`):
   - Change to `feature.geometry_mut().value = x`

### Step 5: Verify

```bash
cargo make format
cargo make clippy
cargo make test
```

---

## Phase 8: Implementation Checklist

**Phase 1: Core Type Changes**
- [ ] **1.1:** Add `Attributes` type alias
- [ ] **1.2:** Remove `Feature::new()` and `Default` impl
- [ ] **1.3:** Wrap `Feature` fields in `Arc`
- [ ] **1.4:** Update remaining constructors
- [ ] **1.5:** Add `attributes_mut()` and `geometry_mut()` methods
- [ ] **1.6:** Update mutation methods to use `Arc::make_mut`
- [ ] **1.7:** Delete `impl From<Feature> for AttributeValue`
- [ ] **1.8:** Delete `impl From<Feature> for serde_json::Value`
- [ ] **1.9:** Update remaining `From` implementations
- [ ] **1.10:** Export `Attributes` from `lib.rs`

**Phase 2: Engine Core Changes**
- [ ] **2.1:** Delete dead code in `executor_operation.rs` (`impl From<Context>`, `impl Default`)

**Phase 3: Fix Feature Store**
- [ ] **3.1:** Fix feature_store.rs serialization (use `serde_json::to_value`)
- [ ] **3.2:** Add `shrink_to_fit()` after drain
- [ ] **3.3:** Fix echo.rs files

**Phase 4: Migrate Feature::new() Usages**
- [ ] **4.x:** Migrate all ~25 `Feature::new()` usages

**Phase 5-6: Fix Downstream Compilation Errors**
- [ ] **5.x:** Fix all direct attribute/geometry field mutations
- [ ] **6.x:** Fix all downstream compilation errors

**Final Verification**
- [ ] Run `cargo make format`
- [ ] Run `cargo make clippy` and fix warnings
- [ ] Run `cargo make test` and fix failures

---

## Expected Outcomes

### Memory Reduction

- **Before:** Each forward to N nodes = N-1 deep clones of entire Feature
- **After:** Each forward to N nodes = N-1 Arc pointer copies (8 bytes each)

For a 10KB geometry forwarded to 3 downstream nodes:
- Before: 2 × 10KB = 20KB copied
- After: 2 × 8 bytes = 16 bytes copied

### Feature Store Improvement

- **Before:** Clone feature → convert to JSON → serialize
- **After:** Serialize directly from reference

Eliminates one deep clone per feature per edge on the hot path.

---

## Notes

1. **Why not `Arc<RwLock<T>>`?** We don't need interior mutability - features flow through channels and are owned by one processor at a time. Arc alone suffices with `make_mut` for CoW.

2. **Why not `Cow<'a, T>`?** Lifetime complexity would infect the entire codebase. Arc is ownership-based and doesn't require lifetime annotations.

3. **Metadata stays owned** because it's small (a few `Option<String>` fields). Arc overhead wouldn't be worth it.

4. **Serde compatibility:** `Arc<T>` where `T: Serialize/Deserialize` works transparently - serializes/deserializes the inner value.

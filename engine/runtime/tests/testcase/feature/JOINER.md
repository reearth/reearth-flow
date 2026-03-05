# FeatureJoiner Tests

This document describes the test suite for the `FeatureJoiner` processor.

## Overview

`FeatureJoiner` is an accumulating processor that performs SQL-style JOIN operations between two feature streams (requestor and supplier) with support for:
- **Inner join**: Only matched features
- **Left join**: All requestor features (default)
- **Full join**: All features from both sides
- **Many-to-many**: One output feature per requestor-supplier match pair

## Test Structure

```
runtime/tests/
├── testcase/feature/joiner.rs              # Test runner (Rust code)
├── fixture/workflow/feature/
│   ├── joiner.yaml                         # Left join test
│   ├── joiner_inner.yaml                   # Inner join test
│   └── joiner_full.yaml                    # Full join test
└── fixture/testdata/feature/joiner/
    └── joiner.json                         # Test data placeholder
```

## Running the Tests

### Run all FeatureJoiner tests
```bash
cd /home/zw/code/rust_programming/reearth-flow/engine
cargo test -p reearth-flow-tests --test test-main -- joiner
```

### Run specific test
```bash
# Left join test
cargo test -p reearth-flow-tests --test test-main -- test_joiner_left

# Inner join test
cargo test -p reearth-flow-tests --test test-main -- test_joiner_inner

# Full join test
cargo test -p reearth-flow-tests --test test-main -- test_joiner_full
```

### Run all feature tests (including FeatureMerger)
```bash
cargo test -p reearth-flow-tests --test test-main -- feature
```

### Run full test suite
```bash
cargo test -p reearth-flow-tests --test test-main
```

## Test Cases

### Test 1: Left Join (`test_joiner_left`)

**Workflow**: `fixture/workflow/feature/joiner.yaml`

**Purpose**: Verify that left join emits:
- Matched features to `joined` port
- Unmatched requestors to `unjoinedRequestor` port
- Nothing to `unjoinedSupplier` port

**Test Data**:
| Requestor | Supplier | Expected Output |
|-----------|----------|-----------------|
| Tokyo | Tokyo | `joined` (merged) |
| Osaka | Osaka | `joined` (merged) |
| Nagoya | Nagoya | `joined` (merged) |
| Yokohama | Yokohama | `joined` (merged) |
| UnmatchedCity1 | - | `unjoinedRequestor` |
| - | UnmatchedCity2 | (dropped - left join) |

**Verification**: Test passes if workflow completes without error.

---

### Test 2: Inner Join (`test_joiner_inner`)

**Workflow**: `fixture/workflow/feature/joiner_inner.yaml`

**Purpose**: Verify that inner join:
- Only emits matched features to `joined` port
- Drops unmatched features from both sides

**Test Data**:
| Requestor | Supplier | Expected Output |
|-----------|----------|-----------------|
| Tokyo | Tokyo | `joined` (merged) |
| Osaka | Osaka | `joined` (merged) |
| UnmatchedCity1 | - | (dropped - inner join) |
| - | UnmatchedCity2 | (dropped - inner join) |

**Verification**: Test passes if workflow completes without error.

---

### Test 3: Full Join (`test_joiner_full`)

**Workflow**: `fixture/workflow/feature/joiner_full.yaml`

**Purpose**: Verify that full join emits:
- Matched features to `joined` port
- Unmatched requestors to `unjoinedRequestor` port
- Unmatched suppliers to `unjoinedSupplier` port

**Test Data**:
| Requestor | Supplier | Expected Output |
|-----------|----------|-----------------|
| Tokyo | Tokyo | `joined` (merged) |
| Osaka | Osaka | `joined` (merged) |
| UnmatchedCity1 | - | `unjoinedRequestor` |
| - | UnmatchedCity2 | `unjoinedSupplier` |

**Verification**: Test passes if workflow completes without error.

## Common Test Data Attributes

### Requestor Features
```json
{
  "city": "Tokyo",
  "country": "Japan",
  "population": 37977000
}
```

### Supplier Features
```json
{
  "city": "Tokyo",
  "lat": 35.6897,
  "lng": 139.6922
}
```

### Join Configuration
```yaml
action: FeatureJoiner
with:
  joinType: left        # or "inner", "full"
  requestorAttribute:
    - city
  supplierAttribute:
    - city
```

## Adding New Tests

To add a new FeatureJoiner test case:

1. **Create workflow YAML** in `fixture/workflow/feature/joiner_<name>.yaml`
2. **Add test function** in `testcase/feature/joiner.rs`:
   ```rust
   #[test]
   fn test_joiner_<name>() {
       let result = execute("feature/joiner_<name>", vec![]);
       assert!(result.is_ok());
   }
   ```
3. **Update this README** with test description

## Debugging Failed Tests

### Check workflow syntax
```bash
cargo run -p reearth-flow-cli -- validate workflow runtime/tests/fixture/workflow/feature/joiner.yaml
```

### Enable logging
```bash
RUST_LOG=debug cargo test -p reearth-flow-tests --test test-main -- test_joiner_left
```

### Inspect temp output
The test framework creates a temp directory with output files. The test returns this directory which can be inspected for debugging.

## Related Tests

- **FeatureMerger**: `testcase/feature/merger.rs` - Similar accumulating processor with different semantics
- See comparison in `tmp/feature-joiner/feature_merger_explain.md`

## References

- Implementation: `runtime/action-processor/src/feature/joiner.rs`
- Factory registration: `runtime/action-processor/src/feature/mapping.rs`
- Test helper: `runtime/tests/helper.rs`

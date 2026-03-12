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
```

## Running the Tests

### Run all FeatureJoiner tests
```bash
# From the repository root:
cd engine
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

**Verification**: Workflow completes without error (EchoSink receives data on expected ports).

**Expected Counts**:
- Joined: 4 features
- UnjoinedRequestor: 1 feature
- UnjoinedSupplier: 0 features

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

**Verification**: Workflow completes without error.

**Expected Counts**:
- Joined: 2 features
- UnjoinedRequestor: 0 features
- UnjoinedSupplier: 0 features (not connected in workflow)

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

**Verification**: Workflow completes without error.

**Expected Counts**:
- Joined: 2 features
- UnjoinedRequestor: 1 feature
- UnjoinedSupplier: 1 feature

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

## Implementation Details

The tests use `EchoSink` to verify that the FeatureJoiner produces output. The EchoSink receives features from each output port and logs them (when logging is enabled).

### Why EchoSink Instead of File Assertions?

Currently the tests use EchoSink because:
1. It's simpler - no need to manage output file paths
2. It's consistent with the FeatureMerger test approach
3. The primary goal is to verify the join logic works, not to test file I/O
4. The workflow execution would fail if the ports didn't receive data

### Potential Improvements

To add output verification with file assertions:
1. Replace `EchoSink` with `JsonWriter` sinks in the workflows
2. Configure output paths via `env.get("outputFilePath")` or similar
3. Read and verify the output files in the test code

Example improvement:
```rust
#[test]
fn test_joiner_left() {
    let tempdir = execute("feature/joiner", vec![]).unwrap();
    let temp_path = tempdir.path();
    
    // Read output files and verify counts
    let joined = read_json_file(temp_path, "joined.json");
    let unjoined_requestor = read_json_file(temp_path, "unjoined_requestor.json");
    
    assert_eq!(joined.len(), 4, "Should have 4 joined features");
    assert_eq!(unjoined_requestor.len(), 1, "Should have 1 unjoined requestor");
}
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

### Enable logging
```bash
RUST_LOG=debug cargo test -p reearth-flow-tests --test test-main -- test_joiner_left
```

### Check workflow syntax
```bash
cargo run -p reearth-flow-cli -- validate workflow runtime/tests/fixture/workflow/feature/joiner.yaml
```

### Verify test data exists
```bash
ls -la runtime/tests/fixture/testdata/feature/
```

## Related Tests

- **FeatureMerger**: `testcase/feature/merger.rs` - Similar accumulating processor with different semantics

## References

- Implementation: `runtime/action-processor/src/feature/joiner.rs`
- Factory registration: `runtime/action-processor/src/feature/mapping.rs`
- Test helper: `runtime/tests/helper.rs`

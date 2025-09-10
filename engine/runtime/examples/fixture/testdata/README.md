# Workflow Test System

This directory contains the test data and configuration for the Re:Earth Flow workflow testing system. The system allows you to create comprehensive tests for workflow executions with support for intermediate data validation and output comparison.

## Test Structure

### Test Configuration (`workflow_test.json`)

Each testdata is a folder consisting of a test profile named `workflow_test.json` and various supporting files. `workflow_test.json` specifies the testing configurations such as input variables, input files, expected output files, expected intermediate data, and so on. The testing system automatically detects the testdata and write the rust unit test at compile time.

### Configuration Fields

- **workflowPath**: Path to the workflow YAML file (relative to fixture directory)
- **description**: Human-readable description of the test
- **expectedOutput**: Main output validation configuration
  - **expectedFile**: Expected output file (relative to testdata root)
  - **comparison**: Comparison method (`exact`, `jsonEquals`)
  - **except**: Fields to exclude from comparison (for TSV: column names, for JSON: field exclusion)
- **cityGmlPath**: Path to input CityGML file
- **codelists/schemas**: codelists and schemas paths
- **intermediateAssertions**: Validate intermediate data at specific workflow edges
  - **edgeId**: Workflow edge identifier to capture data from
  - **expectedFile**: Expected intermediate data file (relative to testdata root)
  - **comparison**: Comparison method
  - **jsonFilter**: Optional JSON filtering (see JSON Filtering section)
- **skip**: Skip this test (useful for debugging)

## JSON Filtering System

The test system includes a native JSON filtering capability that allows you to focus tests on specific parts of the data without external dependencies.

### Supported Filter Patterns

1. **Object Construction**: `{field1, field2}`
   ```json
   "jsonFilter": "{attributes}"
   ```
   Creates: `{"attributes": {"field": "value"}}`

2. **Field Access**: `.field`
   ```json
   "jsonFilter": ".attributes"
   ```
   Extracts: `{"field": "value"}`

3. **JSONPath**: `$.field.subfield`
   ```json
   "jsonFilter": "$.attributes._num_invalid_geom"
   ```

4. **Multiple Fields**: `{id, attributes, metadata}`
   ```json
   "jsonFilter": "{id, attributes}"
   ```

## How to Run Tests

### From Engine Root

#### Run All Tests
From the engine's root, run:
```bash
cargo test -p workflow-tests
```

#### Run Individual Tests
One can also run tests individually. From the engines root:
```bash
cargo test -p workflow-tests test_quality_check_plateau4_02_bldg_l_bldg_06
```

## Example Test Case

### Folder Structure
```
T-BLDG-02/
├── workflow_test.json                 # Test configuration
├── udx/
│   └── bldg/
│       └── input.gml                  # Input CityGML file
├── codelists/                         # PLATEAU codelists
├── schemas/                           # XML schemas
├── 02_bldg_t_bldg_02エラー.tsv        # Expected TSV output. Required to be of the same name as the actual output.
└── error_count_test_filtered.jsonl   # Expected filtered intermediate data
```

### Example `workflow_test.json`
```json
{
  "workflowPath": "quality-check/plateau4/02-bldg/workflow.yml",
  "description": "Test PLATEAU4 T-bldg-02 validation with geometry type errors",
  "expectedOutput": {
    "expectedFile": "02_bldg_t_bldg_02エラー.tsv",
    "comparison": "exact",
    "except": "\"gmlPath\""
  },
  "cityGmlPath": "udx/bldg/input.gml",
  "codelists": "codelists",
  "schemas": "schemas",
  "intermediateAssertions": [
    {
      "edgeId": "7a561c34-2e94-4883-91e4-4026b09c2f8a.66b6c7d8-9e0f-1a2b-3c4d-5e6f7a8b9c0d",
      "expectedFile": "error_count_test_filtered.jsonl",
      "comparison": "jsonEquals",
      "jsonFilter": "{attributes}"
    }
  ],
  "skip": false
}
```

### Expected Filtered Intermediate Data (`error_count_test_filtered.jsonl`)
```json
{
  "attributes": {
    "_num_invalid_bldginst_geom_type": 2
  }
}
```

## Field Exclusion

### TSV Files
Use the `except` field to exclude columns:
```json
"except": "\"GmlFilePath\""           // Single column
"except": ["\"Col1\"", "\"Col2\""]   // Multiple columns
```

### JSON Files  
Use `jsonFilter` instead of field exclusion for cleaner tests:
```json
"jsonFilter": "{attributes}"         // Focus on specific fields
```

## Adding New Tests

1. Create a new directory under the appropriate category
2. Add input data (CityGML, supporting files)
3. Create `workflow_test.json` with test configuration
4. Run the workflow manually to generate expected outputs
5. Apply JSON filters to reduce expected data file sizes (Optional)
6. Run tests to verify everything works correctly

The test system automatically discovers new test cases on the next build.
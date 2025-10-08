# Workflow Test System

This directory contains the test data and configuration for the Re:Earth Flow workflow testing system. The system allows you to create comprehensive tests for workflow executions with support for intermediate data validation and output comparison.

## Test Structure

### Test Configuration (`workflow_test.json`)

Each testdata is a folder consisting of a test profile named `workflow_test.json` and various supporting files. `workflow_test.json` specifies the testing configurations such as input variables, input files, expected output files, expected intermediate data, and so on. The testing system automatically detects the testdata and writes the corresponding rust unit tests at compile time.

### Configuration Fields

- **workflowPath**: Path to the workflow YAML file (relative to fixture directory)
- **description**: Human-readable description of the test
- **expectedOutput**: Main output validation configuration
  - **expectedFile**: Expected output file (relative to testdata root). The file format is automatically detected based on the file extension.
  - **except**: Fields to exclude from comparison (for TSV/CSV: column names)
- **cityGmlPath**: Path to input CityGML file
- **codelists/schemas**: codelists and schemas paths
- **intermediateAssertions**: Validate intermediate data at specific workflow edges
  - **edgeId**: Workflow edge identifier to capture data from
  - **expectedFile**: Expected intermediate data file (relative to testdata root). The file format is automatically detected.
  - **jsonFilter**: Optional JSON filtering (see JSON Filtering section)
- **summaryOutput**: Validate aggregated summary outputs (see Summary Output Validation section)
  - **errorCountSummary**: Validate JSON-based error count summary files
  - **fileErrorSummary**: Validate CSV-based per-file error summary files
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
    "except": "gmlPath"
  },
  "cityGmlPath": "udx/bldg/input.gml",
  "codelists": "codelists",
  "schemas": "schemas",
  "intermediateAssertions": [
    {
      "edgeId": "7a561c34-2e94-4883-91e4-4026b09c2f8a.66b6c7d8-9e0f-1a2b-3c4d-5e6f7a8b9c0d",
      "expectedFile": "error_count_test_filtered.jsonl",
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

## Supported File Formats

The test system automatically detects file formats based on file extensions:
- **JSON** (`.json`): Parsed and compared as JSON objects
- **JSONL** (`.jsonl`): Each line parsed as a separate JSON object
- **CSV** (`.csv`): Compared with column order independence
- **TSV** (`.tsv`): Compared with column order independence

**Note**: Files with unsupported extensions will result in an error message indicating that only json, jsonl, csv, and tsv files are supported.

## Field Exclusion

### TSV/CSV Files
Use the `except` field to exclude columns:
```json
"except": "GmlFilePath"           // Single column
"except": ["Col1", "Col2"]   // Multiple columns
```

### JSON/JSONL Files
Use `jsonFilter` for focused testing:
```json
"jsonFilter": "{attributes}"         // Focus on specific fields
```

## Summary Output Validation

The test system supports validation of aggregated summary outputs, such as error count summaries and per-file error details. This is useful for testing workflows that produce summary reports alongside detailed error outputs.

### Configuration Options

#### Error Count Summary (`errorCountSummary`)
Validates JSON-based summary files containing aggregated error counts across the entire workflow execution.

- **expectedFile**: Name of the expected summary file (e.g., `summary_bldg.json`)
  - The actual output file must have the same name in the workflow output directory
  - The expected file is placed in the testdata directory
- **includeFields**: Array of field names to validate (optional)
  - If omitted, all fields in the expected file are compared
  - Useful for testing only specific error types

#### File Error Summary (`fileErrorSummary`)
Validates CSV-based summary files containing per-file error details.

- **expectedFile**: Name of the expected summary file (e.g., `02_建築物_検査結果一覧.csv`)
  - The actual output file must have the same name in the workflow output directory
  - The expected file is placed in the testdata directory
- **includeColumns**: Array of column names to validate (optional)
  - If omitted, all columns are compared
  - Key columns (e.g., `Filename`) are automatically included
- **keyColumns**: Columns used to identify rows (default: `["Filename"]`)
  - Used to match rows between actual and expected files

### Example Configuration

```json
{
  "workflowPath": "quality-check/plateau4/02-bldg/workflow.yml",
  "description": "Test PLATEAU4 city code validation with summary output",
  "expectedOutput": {
    "expectedFile": "02_建築物_市区町村コードエラー.csv"
  },
  "summaryOutput": {
    "errorCountSummary": {
      "expectedFile": "summary_bldg.json",
      "includeFields": [
        "市区町村コードエラー"
      ]
    },
    "fileErrorSummary": {
      "expectedFile": "02_建築物_検査結果一覧.csv",
      "includeColumns": [
        "市区町村コードエラー"
      ]
    }
  },
  "cityGmlPath": "udx/bldg/input.gml",
  "codelists": "codelists",
  "schemas": "schemas"
}
```

### Expected Summary Files

#### JSON Error Count Summary (`summary_bldg.json`)
```json
[
  {
    "name": "市区町村コードエラー",
    "count": 2
  },
  {
    "name": "C-bldg-01 エラー",
    "count": 0
  }
]
```

#### CSV File Error Summary (`02_建築物_検査結果一覧.csv`)
```csv
Index,Filename,市区町村コードエラー,C-bldg-01 エラー,不正な建物ID
1,input.gml,2,0,0
```

### Use Cases

1. **Selective Field Testing**: Test only specific error types by using `includeFields` or `includeColumns`
2. **Multi-File Workflows**: Validate aggregated error counts across multiple input files
3. **Summary Reports**: Ensure summary reports are generated correctly alongside detailed outputs

## Adding New Tests

1. Create a new directory under the appropriate category
2. Add input data (CityGML, supporting files)
3. Create `workflow_test.json` with test configuration
4. Run the workflow manually to generate expected outputs
5. Apply JSON filters to reduce expected data file sizes (Optional)
6. Run tests to verify everything works correctly

The test system automatically discovers new test cases on the next build.

---

# テストデータについて / About Test Data

このディレクトリには、テスト用のサンプルファイルが含まれています。一部のテストデータにG空間情報センターで公開されているPLATEAUコンテンツを使用しています。

This directory contains sample files for testing. Some test data uses PLATEAU content published on G-spatial Information Center.

## データ出典 / Data Attribution

出典：国土交通省 PLATEAUウェブサイト (https://www.mlit.go.jp/plateau/)
Source: Ministry of Land, Infrastructure, Transport and Tourism PLATEAU Website (https://www.mlit.go.jp/plateau/)

PLATEAU 3D都市モデル - G空間情報センター公開コンテンツの一部 (https://www.geospatial.jp/ckan/dataset/plateau)
PLATEAU 3D City Models - Part of content published on G-spatial Information Center (https://www.geospatial.jp/ckan/dataset/plateau)

## ライセンス / License

使用しているPLATEAUデータのライセンスについては、以下をご確認ください:
For licensing information of the PLATEAU data used, please refer to:

https://www.mlit.go.jp/plateau/site-policy/

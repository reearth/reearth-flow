use anyhow::{Context, Result};
use csv::{ReaderBuilder, StringRecord};
use jsonpath_lib as jsonpath;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use yaml_include::Transformer;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTestProfile {
    /// Path to the workflow file (relative to fixture/workflow/)
    pub workflow_path: String,

    /// Description of what this test is testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Expected output configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<TestOutput>,

    /// Path to the CityGML file (relative to test folder)
    pub city_gml_path: String,

    /// Path to codelists directory (relative to test folder, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codelists: Option<String>,

    /// Path to schemas directory (relative to test folder, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<String>,

    /// Intermediate data assertions (edge_id -> expected file)
    #[serde(default)]
    pub intermediate_assertions: Vec<IntermediateAssertion>,

    /// Whether to skip this test
    #[serde(default)]
    pub skip: bool,

    /// Reason for skipping (required if skip is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestOutput {
    /// Path to expected output file (relative to test folder) - treated as answer data for the file with same name in output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_file: Option<String>,

    /// Inline expected data for small outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_inline: Option<serde_json::Value>,

    /// Column names to exclude from comparison (for TSV/CSV files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub except: Option<ExceptColumns>,

    /// Node ID to capture output from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExceptColumns {
    Single(String),
    Multiple(Vec<String>),
}

impl ExceptColumns {
    fn contains(&self, column: &str) -> bool {
        match self {
            ExceptColumns::Single(name) => name == column,
            ExceptColumns::Multiple(names) => names.contains(&column.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExceptFields {
    Single(String),
    Multiple(Vec<String>),
}

impl ExceptFields {
    fn contains(&self, field: &str) -> bool {
        match self {
            ExceptFields::Single(name) => name == field,
            ExceptFields::Multiple(names) => names.contains(&field.to_string()),
        }
    }
}

#[derive(Debug)]
enum FileComparisonMethod {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntermediateAssertion {
    /// Edge ID to check
    pub edge_id: String,

    /// Path to expected data file (relative to test folder)
    pub expected_file: String,

    /// JSON fields to exclude from comparison
    #[serde(skip_serializing_if = "Option::is_none")]
    pub except: Option<ExceptFields>,

    /// JSON filter to apply to both actual and expected data before comparison
    /// Supports JSONPath syntax ($.field) and object construction ({field1, field2})
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_filter: Option<String>,

    /// Whether to check only a subset of features
    #[serde(default)]
    pub partial_match: bool,
}

pub struct TestContext {
    pub test_name: String,
    pub test_dir: PathBuf,
    pub fixture_dir: PathBuf,
    pub profile: WorkflowTestProfile,
    pub temp_dir: PathBuf,
    _temp_base: TempDir,
}

impl TestContext {
    pub fn new(
        test_name: String,
        test_dir: PathBuf,
        fixture_dir: PathBuf,
        profile: WorkflowTestProfile,
    ) -> Result<Self> {
        let temp_base = TempDir::new()?;
        let temp_dir = temp_base.path().join(&test_name);

        // Remove existing temp directory if it exists to ensure clean state
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }

        // Create fresh temp directory
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            test_name,
            test_dir,
            fixture_dir,
            profile,
            temp_dir,
            _temp_base: temp_base,
        })
    }

    pub fn setup_environment(&self) -> Result<()> {
        // Change to temp directory for the test execution
        std::env::set_current_dir(&self.temp_dir)?;

        Ok(())
    }

    pub fn get_workflow_path(&self) -> PathBuf {
        self.fixture_dir
            .join("workflow")
            .join(&self.profile.workflow_path)
    }

    pub fn run_workflow(&self) -> Result<()> {
        // Get the path to the reearth-flow binary
        // The binary is located in the engine/target/debug directory
        // CARGO_MANIFEST_DIR = .../engine/runtime/examples/fixture/tests
        // We need to go up to engine: tests -> fixture -> examples -> runtime -> engine
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let cli_path = PathBuf::from(manifest_dir)
            .parent() // -> fixture
            .unwrap()
            .parent() // -> examples
            .unwrap()
            .parent() // -> runtime
            .unwrap()
            .parent() // -> engine
            .unwrap()
            .join("target")
            .join("debug")
            .join("reearth-flow");

        if !cli_path.exists() {
            anyhow::bail!(
                "reearth-flow binary not found at {:?}. Please run 'cargo build --package reearth-flow-cli' first.",
                cli_path
            );
        }

        // Pre-process the workflow YAML to handle includes
        let workflow_path = self.get_workflow_path();
        let yaml_transformer = Transformer::new(workflow_path, false)
            .with_context(|| "Failed to create YAML transformer")?;
        let processed_yaml = yaml_transformer.to_string();

        // Save the processed workflow to a temporary file
        let temp_workflow_path = self.temp_dir.join("processed_workflow.yml");
        fs::write(&temp_workflow_path, processed_yaml).with_context(|| {
            format!(
                "Failed to write processed workflow to {temp_workflow_path:?}",
            )
        })?;

        // Build the CLI command
        let mut cmd = Command::new(&cli_path);

        cmd.arg("run")
            .arg("--workflow")
            .arg(&temp_workflow_path)
            .arg("--working-dir")
            .arg(&self.temp_dir);

        // Add workflow variables as CLI arguments
        let city_gml_path = self.test_dir.join(&self.profile.city_gml_path);
        let city_gml_url = format!("file://{}", city_gml_path.display());
        cmd.arg("--var")
            .arg(format!("cityGmlPath={city_gml_url}"));

        if let Some(codelists) = &self.profile.codelists {
            let codelists_path = self.test_dir.join(codelists);
            let codelists_url = format!("file://{}", codelists_path.display());
            cmd.arg("--var").arg(format!("codelists={codelists_url}"));
        }

        if let Some(schemas) = &self.profile.schemas {
            let schemas_path = self.test_dir.join(schemas);
            let schemas_url = format!("file://{}", schemas_path.display());
            cmd.arg("--var").arg(format!("schemas={schemas_url}"));
        }

        cmd.arg("--var")
            .arg(format!("outputPath={}", self.temp_dir.display()));
        cmd.arg("--var")
            .arg(format!("currentPath={}", self.temp_dir.display()));

        // Run the command
        let output = cmd
            .output()
            .with_context(|| format!("Failed to execute CLI command: {cli_path:?}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "CLI command failed with status {}:\nstdout: {}\nstderr: {}",
                output.status,
                stdout,
                stderr
            );
        }

        Ok(())
    }

    pub fn verify_output(&self) -> Result<()> {
        if let Some(output) = &self.profile.expected_output {
            // Check if expected file exists, fail test if not
            if let Some(expected_file_name) = &output.expected_file {
                let expected_file = self.test_dir.join(expected_file_name);
                if !expected_file.exists() {
                    anyhow::bail!("Expected output file does not exist: {:?}", expected_file);
                }

                // Validate file format and route to appropriate verification method
                self.verify_file_based_on_extension(output, expected_file_name)?;
            }
        }
        Ok(())
    }

    fn verify_file_based_on_extension(&self, output: &TestOutput, file_name: &str) -> Result<()> {
        // Determine file format based on extension
        if file_name.ends_with(".json") {
            self.verify_json_file(file_name)?;
        } else if file_name.ends_with(".jsonl") {
            self.verify_jsonl_file(file_name)?;
        } else if file_name.ends_with(".csv") {
            self.verify_csv_file(output, file_name, b',')?;
        } else if file_name.ends_with(".tsv") {
            self.verify_csv_file(output, file_name, b'\t')?;
        } else {
            // Extract extension for error message
            let extension = file_name.rsplit('.').next().unwrap_or("unknown");
            anyhow::bail!(
                "Unsupported file format '.{}'. Only json, jsonl, csv, and tsv files are supported.",
                extension
            );
        }
        Ok(())
    }

    pub fn verify_intermediate_data(&self) -> Result<()> {
        for assertion in &self.profile.intermediate_assertions {
            // Check if expected file exists, fail test if not
            let expected_path = self.test_dir.join(&assertion.expected_file);
            if !expected_path.exists() {
                anyhow::bail!(
                    "Expected intermediate data file does not exist: {:?}",
                    expected_path
                );
            }

            // Look for the edge data in the projects/engine/jobs directory structure
            let jobs_dir = self.temp_dir.join("projects").join("engine").join("jobs");

            // Find the job directory (there should be one)
            let job_dirs: Vec<_> = if jobs_dir.exists() {
                fs::read_dir(&jobs_dir)?
                    .filter_map(|entry| entry.ok())
                    .collect()
            } else {
                anyhow::bail!("No jobs directory found at {:?}", jobs_dir);
            };

            if job_dirs.is_empty() {
                anyhow::bail!("No job directories found in {:?}", jobs_dir);
            }

            // Use the first (and should be only) job directory
            let job_dir = &job_dirs[0].path();
            let edge_data_path = job_dir
                .join("feature-store")
                .join(format!("{}.jsonl", assertion.edge_id));

            if !edge_data_path.exists() {
                anyhow::bail!(
                    "Intermediate data not found for edge {}: {:?}",
                    assertion.edge_id,
                    edge_data_path
                );
            }

            let mut expected_data = fs::read_to_string(&expected_path)?;
            let mut actual_data = fs::read_to_string(&edge_data_path)?;

            // Apply JSON filter if provided
            if let Some(json_filter) = &assertion.json_filter {
                expected_data = self.apply_json_filter(&expected_data, json_filter)?;
                actual_data = self.apply_json_filter(&actual_data, json_filter)?;
            }

            // Determine comparison method based on file extension
            let comparison_method = self.determine_comparison_method(&assertion.expected_file)?;
            self.compare_data(
                &actual_data,
                &expected_data,
                &comparison_method,
                assertion.except.as_ref(),
            )?;
        }
        Ok(())
    }

    fn verify_csv_file(&self, output: &TestOutput, file_name: &str, delimiter: u8) -> Result<()> {
        let expected_file = self.test_dir.join(file_name);
        let actual_file = self.temp_dir.join(file_name);

        if !actual_file.exists() {
            anyhow::bail!("Output file not found at {:?}", actual_file);
        }

        let expected = fs::read_to_string(&expected_file)?;
        let actual = fs::read_to_string(&actual_file)?;

        self.compare_csv(&actual, &expected, delimiter, output.except.as_ref())?;
        Ok(())
    }

    fn verify_json_file(&self, file_name: &str) -> Result<()> {
        let expected_file = self.test_dir.join(file_name);
        let actual_file = self.temp_dir.join(file_name);

        if !actual_file.exists() {
            anyhow::bail!("Output file not found at {:?}", actual_file);
        }

        let expected: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&expected_file)?)?;
        let actual: serde_json::Value = serde_json::from_str(&fs::read_to_string(&actual_file)?)?;

        assert_eq!(
            actual, expected,
            "JSON output mismatch for {}",
            self.test_name
        );
        Ok(())
    }

    fn verify_jsonl_file(&self, file_name: &str) -> Result<()> {
        let expected_file = self.test_dir.join(file_name);
        let actual_file = self.temp_dir.join(file_name);

        if !actual_file.exists() {
            anyhow::bail!("Output file not found at {:?}", actual_file);
        }

        let expected_content = fs::read_to_string(&expected_file)?;
        let actual_content = fs::read_to_string(&actual_file)?;

        // Parse each line as JSON and collect
        let mut expected_values: Vec<serde_json::Value> = Vec::new();
        for line in expected_content.lines() {
            if !line.trim().is_empty() {
                expected_values.push(serde_json::from_str(line)?);
            }
        }

        let mut actual_values: Vec<serde_json::Value> = Vec::new();
        for line in actual_content.lines() {
            if !line.trim().is_empty() {
                actual_values.push(serde_json::from_str(line)?);
            }
        }

        assert_eq!(
            actual_values, expected_values,
            "JSONL output mismatch for {}",
            self.test_name
        );
        Ok(())
    }

    fn determine_comparison_method(&self, file_name: &str) -> Result<FileComparisonMethod> {
        if file_name.ends_with(".json") || file_name.ends_with(".jsonl") {
            Ok(FileComparisonMethod::Json)
        } else if file_name.ends_with(".csv") || file_name.ends_with(".tsv") {
            Ok(FileComparisonMethod::Text)
        } else {
            let extension = file_name.rsplit('.').next().unwrap_or("unknown");
            anyhow::bail!(
                "Unsupported file format '.{}'. Only json, jsonl, csv, and tsv files are supported.",
                extension
            )
        }
    }

    fn compare_data(
        &self,
        actual: &str,
        expected: &str,
        method: &FileComparisonMethod,
        except: Option<&ExceptFields>,
    ) -> Result<()> {
        match method {
            FileComparisonMethod::Text => {
                assert_eq!(actual, expected);
            }
            FileComparisonMethod::Json => {
                let mut actual_json: serde_json::Value = serde_json::from_str(actual)?;
                let mut expected_json: serde_json::Value = serde_json::from_str(expected)?;

                // Remove excluded fields if specified
                if let Some(except_fields) = except {
                    Self::remove_json_fields(&mut actual_json, except_fields);
                    Self::remove_json_fields(&mut expected_json, except_fields);
                }

                assert_eq!(actual_json, expected_json);
            }
        }
        Ok(())
    }

    fn compare_csv(
        &self,
        actual: &str,
        expected: &str,
        delimiter: u8,
        except: Option<&ExceptColumns>,
    ) -> Result<()> {
        // Build CSV readers with the specified delimiter
        let mut actual_reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(Cursor::new(actual));
        let mut expected_reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .from_reader(Cursor::new(expected));

        // Get headers
        let actual_headers = actual_reader.headers()?.clone();
        let expected_headers = expected_reader.headers()?.clone();

        // Filter out excluded columns
        let actual_cols_filtered: Vec<&str> = actual_headers
            .iter()
            .filter(|col| {
                if let Some(except) = except {
                    !except.contains(col)
                } else {
                    true
                }
            })
            .collect();

        let expected_cols_filtered: Vec<&str> = expected_headers
            .iter()
            .filter(|col| {
                if let Some(except) = except {
                    !except.contains(col)
                } else {
                    true
                }
            })
            .collect();

        // Check that both files have the same columns (regardless of order, excluding excepted columns)
        let mut actual_cols_sorted = actual_cols_filtered.clone();
        let mut expected_cols_sorted = expected_cols_filtered.clone();
        actual_cols_sorted.sort();
        expected_cols_sorted.sort();

        if actual_cols_sorted != expected_cols_sorted {
            let file_type = if delimiter == b'\t' { "TSV" } else { "CSV" };
            anyhow::bail!(
                "{} column mismatch. Expected columns: {:?}, Actual columns: {:?}",
                file_type,
                expected_cols_sorted,
                actual_cols_sorted
            );
        }

        // Collect all rows from both readers
        let actual_rows: Vec<StringRecord> =
            actual_reader.records().collect::<Result<Vec<_>, _>>()?;
        let expected_rows: Vec<StringRecord> =
            expected_reader.records().collect::<Result<Vec<_>, _>>()?;

        // Check same number of data rows
        if actual_rows.len() != expected_rows.len() {
            let file_type = if delimiter == b'\t' { "TSV" } else { "CSV" };
            anyhow::bail!(
                "{} row count mismatch. Expected {} rows, got {} rows",
                file_type,
                expected_rows.len(),
                actual_rows.len()
            );
        }

        // Create a set of excluded column indices for expected data
        let excluded_expected_indices: HashSet<usize> = if let Some(except) = except {
            expected_headers
                .iter()
                .enumerate()
                .filter_map(|(idx, col)| {
                    if except.contains(col) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            HashSet::new()
        };

        // Process and sort rows for comparison (ignoring row order)
        let mut expected_processed_rows = Vec::new();
        let mut actual_processed_rows = Vec::new();

        // Process expected rows
        for expected_row in expected_rows {
            let mut filtered_expected_values = Vec::new();
            for (idx, value) in expected_row.iter().enumerate() {
                if !excluded_expected_indices.contains(&idx) {
                    filtered_expected_values.push(value.to_string());
                }
            }
            expected_processed_rows.push(filtered_expected_values);
        }

        // Process actual rows and reorder columns to match expected order
        for actual_row in actual_rows {
            let mut reordered_actual_values = Vec::new();

            // Reorder actual values to match expected column order and filter excluded columns
            for (expected_idx, expected_col_name) in expected_headers.iter().enumerate() {
                // Skip excluded columns
                if excluded_expected_indices.contains(&expected_idx) {
                    continue;
                }

                // Find corresponding actual value and add it
                if let Some(actual_col_idx) = actual_headers
                    .iter()
                    .position(|col| col == expected_col_name)
                {
                    reordered_actual_values
                        .push(actual_row.get(actual_col_idx).unwrap_or("").to_string());
                } else {
                    anyhow::bail!("Column '{}' not found in actual data", expected_col_name);
                }
            }
            actual_processed_rows.push(reordered_actual_values);
        }

        // Sort both row sets for comparison (ignoring row order)
        expected_processed_rows.sort();
        actual_processed_rows.sort();

        // Compare sorted rows
        if expected_processed_rows != actual_processed_rows {
            let file_type = if delimiter == b'\t' { "TSV" } else { "CSV" };
            anyhow::bail!(
                "{} data mismatch (excluding excepted columns, ignoring row and column order).\nExpected rows (sorted): {:?}\nActual rows (sorted): {:?}", 
                file_type,
                expected_processed_rows,
                actual_processed_rows
            );
        }

        Ok(())
    }

    fn remove_json_fields(json: &mut serde_json::Value, except: &ExceptFields) {
        match json {
            serde_json::Value::Object(map) => {
                // Remove excluded fields from this object
                let keys_to_remove: Vec<String> = map
                    .keys()
                    .filter(|key| except.contains(key))
                    .cloned()
                    .collect();

                for key in keys_to_remove {
                    map.remove(&key);
                }

                // Recursively process nested objects and arrays
                for value in map.values_mut() {
                    Self::remove_json_fields(value, except);
                }
            }
            serde_json::Value::Array(arr) => {
                // Recursively process array elements
                for value in arr.iter_mut() {
                    Self::remove_json_fields(value, except);
                }
            }
            _ => {
                // Primitive values don't need processing
            }
        }
    }

    fn apply_json_filter(&self, data: &str, filter: &str) -> Result<String> {
        let mut results = Vec::new();

        // Try to parse as a single JSON object first (for pretty-printed JSON)
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
            let filtered = self.apply_filter_to_json(&json, filter)?;
            return Ok(serde_json::to_string(&filtered)?);
        }

        // Otherwise, parse each line as JSON (for JSONL files)
        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let json: serde_json::Value = serde_json::from_str(line)
                .with_context(|| format!("Failed to parse JSON line: {line}"))?;

            let filtered = self.apply_filter_to_json(&json, filter)?;
            results.push(serde_json::to_string(&filtered)?);
        }

        Ok(results.join("\n"))
    }

    fn apply_filter_to_json(
        &self,
        json: &serde_json::Value,
        filter: &str,
    ) -> Result<serde_json::Value> {
        let filter = filter.trim();

        // Handle different filter patterns
        if filter.starts_with('{') && filter.ends_with('}') {
            // Object construction: {field1, field2, field3}
            self.construct_object(json, filter)
        } else if filter.starts_with("$.") {
            // JSONPath expression: $.field or $.field1.field2
            self.apply_jsonpath(json, filter)
        } else if let Some(field) = filter.strip_prefix('.') {
            // Simple field access: .field
            Ok(json.get(field).unwrap_or(&serde_json::Value::Null).clone())
        } else {
            // Direct field access: field
            Ok(json.get(filter).unwrap_or(&serde_json::Value::Null).clone())
        }
    }

    fn construct_object(
        &self,
        json: &serde_json::Value,
        filter: &str,
    ) -> Result<serde_json::Value> {
        // Parse {field1, field2, field3} format
        let inner = &filter[1..filter.len() - 1]; // Remove { and }
        let fields: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        let mut result = serde_json::Map::new();

        for field in fields {
            if let Some(value) = json.get(field) {
                result.insert(field.to_string(), value.clone());
            }
        }

        Ok(serde_json::Value::Object(result))
    }

    fn apply_jsonpath(&self, json: &serde_json::Value, path: &str) -> Result<serde_json::Value> {
        let mut selector = jsonpath::selector(json);

        let selected =
            selector(path).with_context(|| format!("Invalid JSONPath expression: {path}"))?;

        match selected.len() {
            0 => Ok(serde_json::Value::Null),
            1 => Ok(selected[0].clone()),
            _ => {
                let values: Vec<serde_json::Value> = selected.into_iter().cloned().collect();
                Ok(serde_json::Value::Array(values))
            }
        }
    }
}

// Include the generated tests
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_json_filter_functionality() -> Result<()> {
        // Create a temporary test context
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().to_path_buf();
        let fixture_dir = PathBuf::from("dummy");

        let profile = WorkflowTestProfile {
            workflow_path: "dummy".to_string(),
            description: None,
            expected_output: None,
            city_gml_path: "dummy".to_string(),
            codelists: None,
            schemas: None,
            intermediate_assertions: vec![],
            skip: false,
            skip_reason: None,
        };

        let ctx = TestContext::new(
            "test_json_filter".to_string(),
            test_dir,
            fixture_dir,
            profile,
        )?;

        // Test data
        let input_data = r#"{
  "id": "test-feature",
  "attributes": {
    "_num_invalid_bldginst_geom_type": 2,
    "other_field": "should be filtered"
  },
  "geometry": {
    "epsg": null,
    "value": "none"
  }
}"#;

        // Test 1: Object construction filter {attributes}
        let result = ctx.apply_json_filter(input_data, "{attributes}")?;
        let parsed: serde_json::Value = serde_json::from_str(&result)?;
        assert!(parsed.get("attributes").is_some());
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("geometry").is_none());

        // Test 2: Simple field access .attributes
        let result = ctx.apply_json_filter(input_data, ".attributes")?;
        let parsed: serde_json::Value = serde_json::from_str(&result)?;
        assert!(parsed.get("_num_invalid_bldginst_geom_type").is_some());

        // Test 3: JSONPath expression $.attributes
        let result = ctx.apply_json_filter(input_data, "$.attributes")?;
        let parsed: serde_json::Value = serde_json::from_str(&result)?;
        assert!(parsed.get("_num_invalid_bldginst_geom_type").is_some());

        // Test 4: Multiple field construction {id, attributes}
        let result = ctx.apply_json_filter(input_data, "{id, attributes}")?;
        let parsed: serde_json::Value = serde_json::from_str(&result)?;
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("attributes").is_some());
        assert!(parsed.get("geometry").is_none());
        Ok(())
    }

    #[test]
    fn test_csv_comparison_functionality() -> Result<()> {
        // Create a temporary test context
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().to_path_buf();
        let fixture_dir = PathBuf::from("dummy");

        let profile = WorkflowTestProfile {
            workflow_path: "dummy".to_string(),
            description: None,
            expected_output: None,
            city_gml_path: "dummy".to_string(),
            codelists: None,
            schemas: None,
            intermediate_assertions: vec![],
            skip: false,
            skip_reason: None,
        };

        let ctx = TestContext::new(
            "test_csv_comparison".to_string(),
            test_dir,
            fixture_dir,
            profile,
        )?;

        // Test CSV data with different column orders
        let actual_csv = "name,age,city\nJohn,30,NYC\nJane,25,LA\n";
        let expected_csv = "age,name,city\n30,John,NYC\n25,Jane,LA\n";

        // Test CSV comparison (should pass - reorders columns)
        ctx.compare_csv(actual_csv, expected_csv, b',', None)?;

        // Test TSV data with different column orders
        let actual_tsv = "name\tage\tcity\nJohn\t30\tNYC\nJane\t25\tLA\n";
        let expected_tsv = "age\tname\tcity\n30\tJohn\tNYC\n25\tJane\tLA\n";

        // Test TSV comparison (should pass - reorders columns)
        ctx.compare_csv(actual_tsv, expected_tsv, b'\t', None)?;

        // Test with excluded columns
        let except_columns = ExceptColumns::Single("age".to_string());
        let actual_csv_with_extra = "name,age,city,country\nJohn,30,NYC,USA\nJane,25,LA,USA\n";
        let expected_csv_with_extra = "city,name,country\nNYC,John,USA\nLA,Jane,USA\n";

        ctx.compare_csv(
            actual_csv_with_extra,
            expected_csv_with_extra,
            b',',
            Some(&except_columns),
        )?;

        // Test with different column orders AND different row orders
        let actual_csv_mixed =
            "name,age,city\nAlice,35,Boston\nBob,40,Chicago\nCharlie,28,Seattle\n";
        let expected_csv_mixed =
            "city,age,name\nSeattle,28,Charlie\nBoston,35,Alice\nChicago,40,Bob\n";

        // Test CSV comparison with both column and row reordering (should pass)
        ctx.compare_csv(actual_csv_mixed, expected_csv_mixed, b',', None)?;

        Ok(())
    }
}

fn main() {
    // The tests will be run by cargo test
    println!("Use 'cargo test' to run the workflow tests");
}

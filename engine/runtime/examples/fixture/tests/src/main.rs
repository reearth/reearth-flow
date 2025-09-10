use anyhow::{Context, Result};
use jsonpath_lib as jsonpath;
use pretty_assertions::assert_eq;
use reearth_flow_runner::runner::Runner;
use reearth_flow_types::Workflow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

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

    /// Comparison method
    #[serde(default)]
    pub comparison: ComparisonMethod,

    /// Column names to exclude from comparison (for TSV files)
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
    pub fn contains(&self, column: &str) -> bool {
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
    pub fn contains(&self, field: &str) -> bool {
        match self {
            ExceptFields::Single(name) => name == field,
            ExceptFields::Multiple(names) => names.contains(&field.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonMethod {
    #[default]
    Exact,
    JsonEquals,
    JsonSubset,
    Contains,
    Regex,
    FileCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntermediateAssertion {
    /// Edge ID to check
    pub edge_id: String,

    /// Path to expected data file (relative to test folder)
    pub expected_file: String,

    /// Comparison method for intermediate data
    #[serde(default)]
    pub comparison: ComparisonMethod,

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
}

impl TestContext {
    pub fn new(
        test_name: String,
        test_dir: PathBuf,
        fixture_dir: PathBuf,
        profile: WorkflowTestProfile,
    ) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("workflow-tests").join(&test_name);

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
        })
    }

    pub fn setup_environment(&self) -> Result<()> {
        // Set working directory for intermediate data capture
        std::env::set_var("FLOW_RUNTIME_WORKING_DIRECTORY", &self.temp_dir);

        // Change to temp directory for the test execution
        std::env::set_current_dir(&self.temp_dir)?;

        Ok(())
    }

    pub fn load_workflow(&self) -> Result<Workflow> {
        let workflow_path = self
            .fixture_dir
            .join("workflow")
            .join(&self.profile.workflow_path);

        // Use yaml-include to process the YAML with includes first
        let yaml_transformer = yaml_include::Transformer::new(workflow_path.clone(), false)?;
        let yaml_str = yaml_transformer.to_string();

        let mut workflow: Workflow = Workflow::try_from(yaml_str.as_str())?;

        // Add current path context like in example_main.rs
        let current_dir = std::env::current_dir()?;
        let current_path = current_dir.to_string_lossy().to_string();
        workflow.extend_with(HashMap::from([("currentPath".to_string(), current_path)]))?;

        Ok(workflow)
    }

    pub fn run_workflow(&self, mut workflow: Workflow) -> Result<()> {
        use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
        use reearth_flow_action_plateau_processor::mapping::ACTION_FACTORY_MAPPINGS as PLATEAU_MAPPINGS;
        use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
        use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
        use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
        use reearth_flow_state::State;
        use reearth_flow_storage::resolve::StorageResolver;

        // Inject test-specific variables directly into workflow instead of using environment variables
        let mut test_variables = HashMap::new();

        let city_gml_path = self.test_dir.join(&self.profile.city_gml_path);
        let city_gml_url = format!("file://{}", city_gml_path.display());
        test_variables.insert("cityGmlPath".to_string(), city_gml_url);

        if let Some(codelists) = &self.profile.codelists {
            let codelists_path = self.test_dir.join(codelists);
            let codelists_url = format!("file://{}", codelists_path.display());
            test_variables.insert("codelists".to_string(), codelists_url);
        }

        if let Some(schemas) = &self.profile.schemas {
            let schemas_path = self.test_dir.join(schemas);
            let schemas_url = format!("file://{}", schemas_path.display());
            test_variables.insert("schemas".to_string(), schemas_url);
        }

        test_variables.insert(
            "outputPath".to_string(),
            self.temp_dir.display().to_string(),
        );
        test_variables.insert(
            "currentPath".to_string(),
            self.temp_dir.display().to_string(),
        );

        // Extend workflow with test variables
        workflow.extend_with(test_variables)?;

        // Setup action factories
        let mut action_factories = HashMap::new();
        action_factories.extend(SINK_MAPPINGS.clone());
        action_factories.extend(SOURCE_MAPPINGS.clone());
        action_factories.extend(PROCESSOR_MAPPINGS.clone());
        action_factories.extend(PLATEAU_MAPPINGS.clone());

        // Setup logging and state
        let job_id = uuid::Uuid::new_v4();
        let action_log_path = self.temp_dir.join("action-log");
        fs::create_dir_all(&action_log_path)?;
        let state_path = self.temp_dir.join("feature-store");
        fs::create_dir_all(&state_path)?;

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_path.clone()),
            action_log_path,
        ));

        let storage_resolver = Arc::new(StorageResolver::new());
        let state_uri = format!("file://{}", state_path.display());
        let state_uri = reearth_flow_common::uri::Uri::from_str(&state_uri)?;
        let state = Arc::new(State::new(&state_uri, &storage_resolver).unwrap());

        // Run workflow
        Runner::run(
            job_id,
            workflow,
            action_factories,
            logger_factory,
            storage_resolver,
            state,
        )?;

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
            }

            match output.comparison {
                ComparisonMethod::Exact => self.verify_exact_output(output)?,
                ComparisonMethod::JsonEquals => self.verify_json_equals(output)?,
                ComparisonMethod::JsonSubset => self.verify_json_subset(output)?,
                ComparisonMethod::Contains => self.verify_contains(output)?,
                ComparisonMethod::Regex => self.verify_regex(output)?,
                ComparisonMethod::FileCount => self.verify_file_count(output)?,
            }
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

            let edge_data_path = self
                .temp_dir
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

            self.compare_data(
                &actual_data,
                &expected_data,
                &assertion.comparison,
                assertion.except.as_ref(),
            )?;
        }
        Ok(())
    }

    fn verify_exact_output(&self, output: &TestOutput) -> Result<()> {
        if let Some(expected_file_name) = &output.expected_file {
            // Expected file contains the answer data
            let expected_file = self.test_dir.join(expected_file_name);
            if !expected_file.exists() {
                anyhow::bail!("Expected output file does not exist: {:?}", expected_file);
            }

            // The actual output file has the same name but is in the temp directory
            let actual_file = self.temp_dir.join(expected_file_name);

            if !actual_file.exists() {
                anyhow::bail!("Output file not found at {:?}", actual_file);
            }

            let expected = fs::read_to_string(&expected_file)?;
            let actual = fs::read_to_string(&actual_file)?;

            // Check if this is a TSV file and handle column order differences
            if expected_file_name.ends_with(".tsv") {
                self.compare_tsv(&actual, &expected, output.except.as_ref())?;
            } else {
                assert_eq!(actual, expected, "Output mismatch for {}", self.test_name);
            }
        }
        Ok(())
    }

    fn verify_json_equals(&self, output: &TestOutput) -> Result<()> {
        if let Some(expected_file_name) = &output.expected_file {
            // Expected file contains the answer data
            let expected_file = self.test_dir.join(expected_file_name);
            if !expected_file.exists() {
                anyhow::bail!("Expected output file does not exist: {:?}", expected_file);
            }

            // The actual output file has the same name but is in the temp directory
            let actual_file = self.temp_dir.join(expected_file_name);
            if !actual_file.exists() {
                anyhow::bail!("Output file not found at {:?}", actual_file);
            }

            let expected: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&expected_file)?)?;
            let actual: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&actual_file)?)?;

            assert_eq!(
                actual, expected,
                "JSON output mismatch for {}",
                self.test_name
            );
        }
        Ok(())
    }

    fn verify_json_subset(&self, _output: &TestOutput) -> Result<()> {
        // TODO: Implement JSON subset comparison
        Ok(())
    }

    fn verify_contains(&self, _output: &TestOutput) -> Result<()> {
        // TODO: Implement contains comparison
        Ok(())
    }

    fn verify_regex(&self, _output: &TestOutput) -> Result<()> {
        // TODO: Implement regex comparison
        Ok(())
    }

    fn verify_file_count(&self, output: &TestOutput) -> Result<()> {
        if let Some(expected_file_name) = &output.expected_file {
            let dir = self.temp_dir.join(expected_file_name);
            let count = fs::read_dir(&dir)?.count();

            if let Some(expected_inline) = &output.expected_inline {
                if let Some(expected_count) = expected_inline.as_u64() {
                    assert_eq!(
                        count as u64, expected_count,
                        "File count mismatch for {}",
                        self.test_name
                    );
                }
            }
        }
        Ok(())
    }

    fn compare_data(
        &self,
        actual: &str,
        expected: &str,
        method: &ComparisonMethod,
        except: Option<&ExceptFields>,
    ) -> Result<()> {
        match method {
            ComparisonMethod::Exact => {
                assert_eq!(actual, expected);
            }
            ComparisonMethod::JsonEquals => {
                let mut actual_json: serde_json::Value = serde_json::from_str(actual)?;
                let mut expected_json: serde_json::Value = serde_json::from_str(expected)?;

                // Remove excluded fields if specified
                if let Some(except_fields) = except {
                    Self::remove_json_fields(&mut actual_json, except_fields);
                    Self::remove_json_fields(&mut expected_json, except_fields);
                }

                assert_eq!(actual_json, expected_json);
            }
            _ => {
                // TODO: Implement other comparison methods
            }
        }
        Ok(())
    }

    fn compare_tsv(
        &self,
        actual: &str,
        expected: &str,
        except: Option<&ExceptColumns>,
    ) -> Result<()> {
        let actual_lines: Vec<&str> = actual.trim().lines().collect();
        let expected_lines: Vec<&str> = expected.trim().lines().collect();

        if actual_lines.is_empty() || expected_lines.is_empty() {
            assert_eq!(actual, expected, "TSV comparison failed: empty content");
            return Ok(());
        }

        // Parse headers and data rows
        let actual_header = actual_lines[0];
        let expected_header = expected_lines[0];

        let actual_columns: Vec<&str> = actual_header.split('\t').collect();
        let expected_columns: Vec<&str> = expected_header.split('\t').collect();

        // Filter out excluded columns
        let actual_cols_filtered: Vec<&str> = actual_columns
            .iter()
            .filter(|col| {
                if let Some(except) = except {
                    !except.contains(col)
                } else {
                    true
                }
            })
            .copied()
            .collect();

        let expected_cols_filtered: Vec<&str> = expected_columns
            .iter()
            .filter(|col| {
                if let Some(except) = except {
                    !except.contains(col)
                } else {
                    true
                }
            })
            .copied()
            .collect();

        // Check that both files have the same columns (regardless of order, excluding excepted columns)
        let mut actual_cols_sorted = actual_cols_filtered.clone();
        let mut expected_cols_sorted = expected_cols_filtered.clone();
        actual_cols_sorted.sort();
        expected_cols_sorted.sort();

        if actual_cols_sorted != expected_cols_sorted {
            anyhow::bail!(
                "TSV column mismatch. Expected columns: {:?}, Actual columns: {:?}",
                expected_cols_sorted,
                actual_cols_sorted
            );
        }

        // Check same number of data rows
        if actual_lines.len() != expected_lines.len() {
            anyhow::bail!(
                "TSV row count mismatch. Expected {} rows, got {} rows",
                expected_lines.len(),
                actual_lines.len()
            );
        }

        // Process and sort rows for comparison (ignoring row order)
        let mut expected_processed_rows = Vec::new();
        let mut actual_processed_rows = Vec::new();

        // Process expected rows
        for expected_row in expected_lines.iter().skip(1) {
            let expected_values: Vec<&str> = expected_row.split('\t').collect();
            if expected_values.len() != expected_columns.len() {
                anyhow::bail!("Expected TSV row has incorrect number of columns");
            }

            // Filter out values for excluded columns
            let mut filtered_expected_values = Vec::new();
            for (expected_col_idx, expected_col_name) in expected_columns.iter().enumerate() {
                // Skip excluded columns
                if let Some(except) = except {
                    if except.contains(expected_col_name) {
                        continue;
                    }
                }
                filtered_expected_values.push(expected_values[expected_col_idx]);
            }
            expected_processed_rows.push(filtered_expected_values);
        }

        // Process actual rows and reorder columns to match expected order
        for actual_row in actual_lines.iter().skip(1) {
            let actual_values: Vec<&str> = actual_row.split('\t').collect();
            if actual_values.len() != actual_columns.len() {
                anyhow::bail!("Actual TSV row has incorrect number of columns");
            }

            // Reorder actual values to match expected column order and filter excluded columns
            let mut reordered_actual_values = Vec::new();
            for expected_col_name in expected_columns.iter() {
                // Skip excluded columns
                if let Some(except) = except {
                    if except.contains(expected_col_name) {
                        continue;
                    }
                }

                // Find corresponding actual value and add it
                if let Some(actual_col_idx) = actual_columns
                    .iter()
                    .position(|col| col == expected_col_name)
                {
                    reordered_actual_values.push(actual_values[actual_col_idx]);
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
            anyhow::bail!(
                "TSV data mismatch (excluding excepted columns, ignoring row and column order).\nExpected rows (sorted): {:?}\nActual rows (sorted): {:?}", 
                expected_processed_rows, actual_processed_rows
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
}

fn main() {
    // The tests will be run by cargo test
    println!("Use 'cargo test' to run the workflow tests");
}

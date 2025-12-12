use anyhow::{Context, Result};
use csv::{ReaderBuilder, StringRecord};
use jsonpath_lib as jsonpath;
use pretty_assertions::assert_eq;
use reearth_flow_runner::runner::Runner;
use reearth_flow_types::Workflow;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Once};
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Initialize tracing subscriber once for all tests
fn init_tracing() {
    INIT.call_once(|| {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::EnvFilter;

        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExpectedFiles {
    Single(String),
    Multiple(Vec<String>),
}

impl ExpectedFiles {
    fn as_vec(&self) -> Vec<String> {
        match self {
            ExpectedFiles::Single(file) => vec![file.clone()],
            ExpectedFiles::Multiple(files) => files.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CityGmlPath {
    /// Shorthand: single GML file path
    GmlFile(String),

    /// Object notation
    Config(CityGmlPathConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum CityGmlPathConfig {
    /// Single file
    File(FileSource),

    /// ZIP generation
    Zip(ZipSource),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileSource {
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZipSource {
    pub source: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
    pub city_gml_path: CityGmlPath,

    /// Path to codelists directory (relative to test folder, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codelists: Option<String>,

    /// Path to schemas directory (relative to test folder, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<String>,

    /// Path to object lists file (relative to test folder, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_lists: Option<String>,

    /// Intermediate data assertions (edge_id -> expected file)
    #[serde(default)]
    pub intermediate_assertions: Vec<IntermediateAssertion>,

    /// Summary output validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_output: Option<SummaryOutput>,

    /// Whether qc_result_ok file should exist (same level as zip)
    /// - Some(true): file must exist
    /// - Some(false): file must NOT exist
    /// - None: do not check (default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect_result_ok_file: Option<bool>,

    /// Whether to skip this test
    #[serde(default)]
    pub skip: bool,

    /// Reason for skipping (required if skip is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestOutput {
    /// Path(s) to expected output file(s) (relative to test folder) - treated as answer data for the file with same name in output
    /// Can be either a single file (String) or multiple files (Vec<String>)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_file: Option<ExpectedFiles>,

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
    Jsonl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SummaryOutput {
    /// Global error count summary (e.g., summary_bldg.json)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_count_summary: Option<ErrorCountSummaryValidation>,

    /// Per-file error detail summary (e.g., 02_建築物_検査結果一覧.csv)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_error_summary: Option<FileErrorSummaryValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorCountSummaryValidation {
    /// Expected output file name (relative to test directory)
    /// The actual output file will have the same name in the temp output directory
    pub expected_file: String,

    /// Fields to include in comparison (only these fields will be checked)
    /// If omitted, all fields are compared
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileErrorSummaryValidation {
    /// Expected output file name (relative to test directory)
    /// The actual output file will have the same name in the temp output directory
    pub expected_file: String,

    /// Columns to include in comparison (only these columns will be checked)
    /// If omitted, all columns are compared
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_columns: Option<Vec<String>>,

    /// Columns to exclude from comparison
    /// These columns will be ignored when comparing CSV files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_columns: Option<Vec<String>>,

    /// Key columns used to identify rows (e.g., ["Filename", "Index"])
    /// Default: ["Filename"]
    #[serde(default = "default_key_columns")]
    pub key_columns: Vec<String>,
}

fn default_key_columns() -> Vec<String> {
    vec!["Filename".to_string()]
}

pub struct TestContext {
    pub test_name: String,
    pub test_dir: PathBuf,
    pub fixture_dir: PathBuf,
    pub profile: WorkflowTestProfile,
    pub temp_dir: PathBuf,
    pub actual_output_dir: PathBuf,
    pub last_job_id: Option<uuid::Uuid>,
    _temp_base: TempDir,
}

impl TestContext {
    pub fn new(
        test_name: String,
        test_dir: PathBuf,
        fixture_dir: PathBuf,
        profile: WorkflowTestProfile,
    ) -> Result<Self> {
        // Initialize tracing subscriber for logging
        init_tracing();

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
            actual_output_dir: temp_dir.clone(),
            temp_dir,
            last_job_id: None,
            _temp_base: temp_base,
        })
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

    fn resolve_city_gml_path(&self) -> Result<PathBuf> {
        match &self.profile.city_gml_path {
            CityGmlPath::GmlFile(path) => Ok(self.test_dir.join(path)),
            CityGmlPath::Config(config) => self.resolve_config(config),
        }
    }

    fn resolve_config(&self, config: &CityGmlPathConfig) -> Result<PathBuf> {
        match config {
            CityGmlPathConfig::File(file_src) => Ok(self.test_dir.join(&file_src.source)),
            CityGmlPathConfig::Zip(zip_src) => {
                self.create_zip_from_directory(&zip_src.name, &zip_src.source)
            }
        }
    }

    fn create_zip_from_directory(
        &self,
        zip_file_name: &str,
        source_dir_name: &str,
    ) -> Result<PathBuf> {
        use std::fs::File;
        use walkdir::WalkDir;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let source_dir = self.test_dir.join(source_dir_name);
        if !source_dir.exists() {
            anyhow::bail!("Source directory does not exist: {}", source_dir.display());
        }

        let zip_path = self.test_dir.join(zip_file_name);

        let file = File::create(&zip_path)
            .with_context(|| format!("Failed to create ZIP file: {}", zip_path.display()))?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for entry in WalkDir::new(&source_dir) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&source_dir)?;

            if relative_path.as_os_str().is_empty() {
                continue;
            }

            let relative_path_str = relative_path.to_string_lossy();

            if path.is_file() {
                zip.start_file(relative_path_str.as_ref(), options)?;
                let mut f = File::open(path)?;
                std::io::copy(&mut f, &mut zip)?;
            } else if path.is_dir() {
                let dir_path = format!("{relative_path_str}/");
                zip.add_directory(dir_path, options)?;
            }
        }

        zip.finish()?;

        Ok(zip_path)
    }

    pub fn run_workflow(&mut self, mut workflow: Workflow) -> Result<()> {
        use reearth_flow_action_log::factory::{create_root_logger, LoggerFactory};
        use reearth_flow_action_plateau_processor::mapping::ACTION_FACTORY_MAPPINGS as PLATEAU_MAPPINGS;
        use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
        use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
        use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
        use reearth_flow_state::State;
        use reearth_flow_storage::resolve::StorageResolver;

        // Inject test-specific variables directly into workflow instead of using environment variables
        let mut test_variables = HashMap::new();

        // Resolve cityGmlPath (handles both single file and ZIP generation)
        let city_gml_path = self.resolve_city_gml_path()?;
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

        if let Some(object_lists) = &self.profile.object_lists {
            let object_lists_path = self.test_dir.join(object_lists);
            let object_lists_url = format!("file://{}", object_lists_path.display());
            test_variables.insert("objectLists".to_string(), object_lists_url);
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
        self.last_job_id = Some(job_id);

        // Use FLOW_RUNTIME_WORKING_DIRECTORY if set, otherwise use temp_dir
        let working_dir = if let Ok(work_dir) = std::env::var("FLOW_RUNTIME_WORKING_DIRECTORY") {
            PathBuf::from(work_dir)
                .join("workflow_test_debug")
                .join(job_id.to_string())
        } else {
            self.temp_dir.clone()
        };

        let action_log_path = working_dir.join("action-log");
        fs::create_dir_all(&action_log_path)?;
        let feature_state_path = working_dir.join("feature-store");
        fs::create_dir_all(&feature_state_path)?;

        let logger_factory = Arc::new(LoggerFactory::new(
            create_root_logger(action_log_path.clone()),
            action_log_path,
        ));

        let storage_resolver = Arc::new(StorageResolver::new());
        let feature_state_uri = format!("file://{}", feature_state_path.display());
        let feature_state_uri = reearth_flow_common::uri::Uri::from_str(&feature_state_uri)?;
        let feature_state = Arc::new(State::new(&feature_state_uri, &storage_resolver).unwrap());
        let ingress_state = Arc::clone(&feature_state);

        // Run workflow
        Runner::run(
            job_id,
            workflow,
            action_factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
        )?;

        Ok(())
    }

    pub fn verify_output(&mut self) -> Result<()> {
        // Ensure zip is extracted if it exists
        if let Err(e) = self.ensure_extracted() {
            tracing::error!(
                test_name = %self.test_name,
                error = %e,
                "Failed to extract output zip"
            );
            return Err(e);
        }

        if let Some(output) = &self.profile.expected_output {
            // Check if expected file(s) exist, fail test if not
            if let Some(expected_files) = &output.expected_file {
                let files = expected_files.as_vec();
                for expected_file_name in &files {
                    let expected_file = self.test_dir.join(expected_file_name);
                    if !expected_file.exists() {
                        tracing::error!(
                            test_name = %self.test_name,
                            expected_file = ?expected_file,
                            "Expected output file does not exist"
                        );
                        anyhow::bail!("Expected output file does not exist: {expected_file:?}");
                    }

                    // Validate file format and route to appropriate verification method
                    if let Err(e) = self.verify_file_based_on_extension(output, expected_file_name)
                    {
                        tracing::error!(
                            test_name = %self.test_name,
                            file_name = %expected_file_name,
                            error = %e,
                            "Failed to verify output file"
                        );
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_extracted(&mut self) -> Result<()> {
        use zip::ZipArchive;

        // Look for any _qc_result.zip file in temp_dir
        let zip_pattern = format!("{}", self.temp_dir.join("*_qc_result.zip").display());
        let zip_files: Vec<PathBuf> = glob::glob(&zip_pattern)
            .ok()
            .map(|paths| paths.filter_map(|p| p.ok()).collect())
            .unwrap_or_default();

        if let Some(zip_path) = zip_files.first() {
            // Extract zip to a subdirectory
            let extract_dir = self.temp_dir.join("extracted");
            fs::create_dir_all(&extract_dir)?;

            let zip_file = fs::File::open(zip_path)?;
            let mut archive = ZipArchive::new(zip_file)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = extract_dir.join(file.name());

                if file.is_dir() {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        fs::create_dir_all(p)?;
                    }
                    let mut outfile = fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }

            // Update actual_output_dir to the extracted directory
            self.actual_output_dir = extract_dir;
        }
        // If no zip file exists, actual_output_dir remains as temp_dir

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
                "Unsupported file format '.{extension}'. Only json, jsonl, csv, and tsv files are supported."
            );
        }
        Ok(())
    }

    pub fn verify_intermediate_data(&self) -> Result<()> {
        for assertion in &self.profile.intermediate_assertions {
            // Check if expected file exists, fail test if not
            let expected_path = self.test_dir.join(&assertion.expected_file);
            if !expected_path.exists() {
                tracing::error!(
                    test_name = %self.test_name,
                    expected_path = ?expected_path,
                    "Expected intermediate data file does not exist"
                );
                anyhow::bail!("Expected intermediate data file does not exist: {expected_path:?}");
            }

            // Use FLOW_RUNTIME_WORKING_DIRECTORY if set, otherwise use temp_dir
            let working_dir = if let Ok(work_dir) = std::env::var("FLOW_RUNTIME_WORKING_DIRECTORY")
            {
                let job_id = self.last_job_id.ok_or_else(|| {
                    tracing::error!(
                        test_name = %self.test_name,
                        "No job_id available - run_workflow must be called first"
                    );
                    anyhow::anyhow!("No job_id available - run_workflow must be called first")
                })?;
                PathBuf::from(work_dir)
                    .join("workflow_test_debug")
                    .join(job_id.to_string())
            } else {
                self.temp_dir.clone()
            };

            let edge_data_path = working_dir
                .join("feature-store")
                .join(format!("{}.jsonl", assertion.edge_id));

            if !edge_data_path.exists() {
                tracing::error!(
                    test_name = %self.test_name,
                    edge_id = %assertion.edge_id,
                    edge_data_path = ?edge_data_path,
                    "Intermediate data not found for edge"
                );
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
            if let Err(e) = self.compare_data(
                &actual_data,
                &expected_data,
                &comparison_method,
                assertion.except.as_ref(),
            ) {
                tracing::error!(
                    test_name = %self.test_name,
                    edge_id = %assertion.edge_id,
                    error = %e,
                    "Failed to compare intermediate data"
                );
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn verify_summary_output(&mut self) -> Result<()> {
        // Ensure zip is extracted if it exists
        if let Err(e) = self.ensure_extracted() {
            tracing::error!(
                test_name = %self.test_name,
                error = %e,
                "Failed to extract output zip for summary verification"
            );
            return Err(e);
        }

        if let Some(summary) = &self.profile.summary_output {
            // Error count summary validation
            if let Some(error_count) = &summary.error_count_summary {
                if let Err(e) = self.verify_error_count_summary(error_count) {
                    tracing::error!(
                        test_name = %self.test_name,
                        error = %e,
                        "Failed to verify error count summary"
                    );
                    return Err(e);
                }
            }

            // File error summary validation
            if let Some(file_error) = &summary.file_error_summary {
                if let Err(e) = self.verify_file_error_summary(file_error) {
                    tracing::error!(
                        test_name = %self.test_name,
                        error = %e,
                        "Failed to verify file error summary"
                    );
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn verify_error_count_summary(&self, config: &ErrorCountSummaryValidation) -> Result<()> {
        // Load actual output
        let actual_file = self.actual_output_dir.join(&config.expected_file);
        if !actual_file.exists() {
            anyhow::bail!("Error count summary file not found: {actual_file:?}");
        }
        let actual_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&actual_file)?)?;

        // Load expected output
        let expected_file = self.test_dir.join(&config.expected_file);
        if !expected_file.exists() {
            anyhow::bail!("Expected error count summary file not found: {expected_file:?}");
        }
        let expected_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&expected_file)?)?;

        // Extract values from array format (name/count fields)
        let actual_map = self.extract_from_array(&actual_json, "name", "count")?;
        let expected_map = self.extract_from_array(&expected_json, "name", "count")?;

        // Filter by include_fields if specified
        let fields_to_check: Vec<String> = if let Some(include_fields) = &config.include_fields {
            include_fields.clone()
        } else {
            // Check all fields from expected
            expected_map.keys().cloned().collect()
        };

        // Compare
        for field in &fields_to_check {
            let expected_value = expected_map
                .get(field)
                .ok_or_else(|| anyhow::anyhow!("Field '{field}' not found in expected summary"))?;
            let actual_value = actual_map
                .get(field)
                .ok_or_else(|| anyhow::anyhow!("Field '{field}' not found in actual summary"))?;

            if actual_value != expected_value {
                anyhow::bail!(
                    "Error count summary mismatch for '{field}': expected {expected_value:?}, got {actual_value:?}"
                );
            }
        }

        Ok(())
    }

    fn verify_file_error_summary(&self, config: &FileErrorSummaryValidation) -> Result<()> {
        // Load actual CSV
        let actual_file = self.actual_output_dir.join(&config.expected_file);
        if !actual_file.exists() {
            anyhow::bail!("File error summary not found: {actual_file:?}");
        }
        let actual_content = fs::read_to_string(&actual_file)?;

        // Load expected CSV
        let expected_file = self.test_dir.join(&config.expected_file);
        if !expected_file.exists() {
            anyhow::bail!("Expected file error summary not found: {expected_file:?}");
        }
        let expected_content = fs::read_to_string(&expected_file)?;

        // Parse CSV
        let mut actual_reader = ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(Cursor::new(&actual_content));
        let mut expected_reader = ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(Cursor::new(&expected_content));

        let actual_headers = actual_reader.headers()?.clone();
        let expected_headers = expected_reader.headers()?.clone();

        // Determine columns to check
        let columns_to_check: Vec<String> = if let Some(include_cols) = &config.include_columns {
            // Always include key columns + specified columns
            let mut cols = config.key_columns.clone();
            cols.extend(include_cols.clone());
            cols.sort();
            cols.dedup();
            cols
        } else {
            // Check all columns
            let mut cols: Vec<String> = expected_headers.iter().map(|s| s.to_string()).collect();

            // Remove excluded columns if specified
            if let Some(exclude_cols) = &config.exclude_columns {
                cols.retain(|col| !exclude_cols.contains(col));
            }

            cols
        };

        // Verify columns exist
        for col in &columns_to_check {
            if !actual_headers.iter().any(|h| h == col) {
                anyhow::bail!("Column '{col}' not found in actual CSV");
            }
            if !expected_headers.iter().any(|h| h == col) {
                anyhow::bail!("Column '{col}' not found in expected CSV");
            }
        }

        // Build row maps keyed by key_columns
        let actual_rows = self.build_row_map(
            &actual_reader.records().collect::<Result<Vec<_>, _>>()?,
            &actual_headers,
            &config.key_columns,
        )?;
        let expected_rows = self.build_row_map(
            &expected_reader.records().collect::<Result<Vec<_>, _>>()?,
            &expected_headers,
            &config.key_columns,
        )?;

        // Compare rows
        for (key, expected_row) in &expected_rows {
            let actual_row = actual_rows
                .get(key)
                .ok_or_else(|| anyhow::anyhow!("Row with key {key:?} not found in actual CSV"))?;

            // Compare specified columns
            for col in &columns_to_check {
                if config.key_columns.contains(col) {
                    continue; // Already matched by key
                }

                let expected_val = expected_row
                    .get(col)
                    .ok_or_else(|| anyhow::anyhow!("Column '{col}' not found in expected row"))?;
                let actual_val = actual_row
                    .get(col)
                    .ok_or_else(|| anyhow::anyhow!("Column '{col}' not found in actual row"))?;

                if actual_val != expected_val {
                    anyhow::bail!(
                        "File error summary mismatch for row {key:?}, column '{col}': expected '{expected_val}', got '{actual_val}'"
                    );
                }
            }
        }

        Ok(())
    }

    fn extract_from_array(
        &self,
        json: &serde_json::Value,
        name_field: &str,
        value_field: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let array = json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected JSON array"))?;

        let mut result = HashMap::new();
        for item in array {
            if let Some(obj) = item.as_object() {
                if let (Some(name), Some(value)) = (obj.get(name_field), obj.get(value_field)) {
                    if let Some(name_str) = name.as_str() {
                        result.insert(name_str.to_string(), value.clone());
                    }
                }
            }
        }
        Ok(result)
    }

    fn build_row_map(
        &self,
        rows: &[StringRecord],
        headers: &StringRecord,
        key_columns: &[String],
    ) -> Result<HashMap<Vec<String>, HashMap<String, String>>> {
        let mut row_map = HashMap::new();

        for row in rows {
            // Build key from key_columns
            let mut key = Vec::new();
            for key_col in key_columns {
                let col_idx = headers
                    .iter()
                    .position(|h| h == key_col)
                    .ok_or_else(|| anyhow::anyhow!("Key column '{key_col}' not found"))?;
                key.push(row.get(col_idx).unwrap_or("").to_string());
            }

            // Build column map for this row
            let mut col_map = HashMap::new();
            for (idx, header) in headers.iter().enumerate() {
                col_map.insert(header.to_string(), row.get(idx).unwrap_or("").to_string());
            }

            row_map.insert(key, col_map);
        }

        Ok(row_map)
    }

    fn verify_csv_file(&self, output: &TestOutput, file_name: &str, delimiter: u8) -> Result<()> {
        let expected_file = self.test_dir.join(file_name);
        let actual_file = self.actual_output_dir.join(file_name);

        if !actual_file.exists() {
            anyhow::bail!("Output file not found at {actual_file:?}");
        }

        let expected = fs::read_to_string(&expected_file)?;
        let actual = fs::read_to_string(&actual_file)?;

        self.compare_csv(&actual, &expected, delimiter, output.except.as_ref())?;
        Ok(())
    }

    fn verify_json_file(&self, file_name: &str) -> Result<()> {
        let expected_file = self.test_dir.join(file_name);
        let actual_file = self.actual_output_dir.join(file_name);

        if !actual_file.exists() {
            anyhow::bail!("Output file not found at {actual_file:?}");
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
        let actual_file = self.actual_output_dir.join(file_name);

        if !actual_file.exists() {
            anyhow::bail!("Output file not found at {actual_file:?}");
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
        if file_name.ends_with(".jsonl") {
            Ok(FileComparisonMethod::Jsonl)
        } else if file_name.ends_with(".json") {
            Ok(FileComparisonMethod::Json)
        } else if file_name.ends_with(".csv") || file_name.ends_with(".tsv") {
            Ok(FileComparisonMethod::Text)
        } else {
            let extension = file_name.rsplit('.').next().unwrap_or("unknown");
            anyhow::bail!(
                "Unsupported file format '.{extension}'. Only json, jsonl, csv, and tsv files are supported."
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
            FileComparisonMethod::Jsonl => {
                // Parse each line as JSON and compare
                let actual_lines: Vec<&str> =
                    actual.lines().filter(|l| !l.trim().is_empty()).collect();
                let expected_lines: Vec<&str> =
                    expected.lines().filter(|l| !l.trim().is_empty()).collect();

                assert_eq!(
                    actual_lines.len(),
                    expected_lines.len(),
                    "Number of lines differs: actual={}, expected={}",
                    actual_lines.len(),
                    expected_lines.len()
                );

                for (i, (actual_line, expected_line)) in
                    actual_lines.iter().zip(expected_lines.iter()).enumerate()
                {
                    let mut actual_json: serde_json::Value = serde_json::from_str(actual_line)
                        .with_context(|| {
                            format!("Failed to parse actual line {}: {}", i + 1, actual_line)
                        })?;
                    let mut expected_json: serde_json::Value = serde_json::from_str(expected_line)
                        .with_context(|| {
                            format!("Failed to parse expected line {}: {}", i + 1, expected_line)
                        })?;

                    // Remove excluded fields if specified
                    if let Some(except_fields) = except {
                        Self::remove_json_fields(&mut actual_json, except_fields);
                        Self::remove_json_fields(&mut expected_json, except_fields);
                    }

                    assert_eq!(
                        actual_json,
                        expected_json,
                        "JSON mismatch at line {}",
                        i + 1
                    );
                }
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
                "{file_type} column mismatch. Expected columns: {expected_cols_sorted:?}, Actual columns: {actual_cols_sorted:?}"
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
                    anyhow::bail!("Column '{expected_col_name}' not found in actual data");
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
                "{file_type} data mismatch (excluding excepted columns, ignoring row and column order).\nExpected rows (sorted): {expected_processed_rows:?}\nActual rows (sorted): {actual_processed_rows:?}"
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

    pub fn verify_result_ok_file(&self) -> Result<()> {
        // If no check is defined, skip verification
        let Some(expect_exists) = self.profile.expect_result_ok_file else {
            return Ok(());
        };

        // Look for files ending with "qc_result_ok" in temp_dir (same level as zip)
        let mut file_exists = false;
        for entry in fs::read_dir(&self.temp_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with("qc_result_ok") {
                        file_exists = true;
                        break;
                    }
                }
            }
        }

        if expect_exists && !file_exists {
            tracing::error!(
                test_name = %self.test_name,
                temp_dir = ?self.temp_dir,
                "Expected qc_result_ok file (suffix match) was not found in output directory"
            );
            anyhow::bail!(
                "Expected qc_result_ok file (suffix match) was not found in output directory"
            );
        }

        if !expect_exists && file_exists {
            tracing::error!(
                test_name = %self.test_name,
                temp_dir = ?self.temp_dir,
                "qc_result_ok file should not exist but was found in output directory"
            );
            anyhow::bail!("qc_result_ok file should not exist but was found in output directory");
        }

        Ok(())
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
            city_gml_path: CityGmlPath::GmlFile("dummy".to_string()),
            codelists: None,
            schemas: None,
            object_lists: None,
            intermediate_assertions: vec![],
            summary_output: None,
            expect_result_ok_file: None,
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
            city_gml_path: CityGmlPath::GmlFile("dummy".to_string()),
            codelists: None,
            schemas: None,
            object_lists: None,
            intermediate_assertions: vec![],
            summary_output: None,
            expect_result_ok_file: None,
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

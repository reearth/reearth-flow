// Shared type definitions for workflow test framework
// This file is included in both build.rs and src/main.rs via include! macro

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExpectedFiles {
    Single(String),
    Multiple(Vec<String>),
}

impl ExpectedFiles {
    pub fn as_vec(&self) -> Vec<String> {
        match self {
            ExpectedFiles::Single(file) => vec![file.clone()],
            ExpectedFiles::Multiple(files) => files.clone(),
        }
    }
}

/// Configuration for creating a ZIP file before running the test
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Zipped {
    /// Path to the file or directory to zip (relative to test folder)
    pub path: String,

    /// Name of the output ZIP file (optional)
    /// If not provided, defaults to "{original_name}.zip"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A single workflow variable with a name and value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowVariable {
    /// Variable name (e.g., "cityGmlPath", "prcs")
    pub name: String,

    /// Variable value - can be any JSON value (string, number, boolean, object, array, null)
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowTestProfile {
    /// Description of what this test is testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Whether to skip this test
    #[serde(default, skip_serializing_if = "is_false")]
    pub skip: bool,

    /// Reason for skipping (required if skip is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,

    /// Path to the workflow file (relative to fixture/workflow/)
    pub workflow_path: String,
    
    /// Workflow variables to inject (name-value pairs)
    /// Values are relative paths for file-based variables, or raw values for others.
    /// File-based variables (paths) will be converted to file:// URLs automatically.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflow_variables: Vec<WorkflowVariable>,
    
    /// Files or directories to zip before running the test
    /// Useful for testing workflows that expect ZIP file inputs.
    /// Generated ZIP files are automatically cleaned up after the test.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zip_before_test: Vec<Zipped>,
    
    /// Intermediate data assertions (edge_id -> expected file)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intermediate_assertions: Vec<IntermediateAssertion>,

    /// Expected output configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<TestOutput>,

    /// Summary output validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_output: Option<SummaryOutput>,

    /// Whether qc_result_ok file should exist (same level as zip)
    /// - Some(true): file must exist
    /// - Some(false): file must NOT exist
    /// - None: do not check (default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect_result_ok_file: Option<bool>,

    /// Validation settings for unexpected output files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unexpected_output_validation: Option<UnexpectedOutputValidation>,
}

fn is_false(b: &bool) -> bool {
    !*b
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
    #[serde(default, skip_serializing_if = "is_false")]
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

/// Configuration for a single error count summary file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorCountSummaryFileConfig {
    /// Expected output file name (relative to test directory)
    /// The actual output file will have the same name in the temp output directory
    pub expected_file: String,

    /// Fields to include in comparison (only these fields will be checked)
    /// If omitted, all fields are compared
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_fields: Option<Vec<String>>,
}

/// Error count summary validation - supports single or multiple files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorCountSummaryValidation {
    /// Single file configuration (backward compatible)
    Single(ErrorCountSummaryFileConfig),
    /// Multiple file configurations
    Multiple(Vec<ErrorCountSummaryFileConfig>),
}

impl ErrorCountSummaryValidation {
    pub fn as_vec(&self) -> Vec<&ErrorCountSummaryFileConfig> {
        match self {
            ErrorCountSummaryValidation::Single(config) => vec![config],
            ErrorCountSummaryValidation::Multiple(configs) => configs.iter().collect(),
        }
    }
}

/// Configuration for a single file error summary file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileErrorSummaryFileConfig {
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
    #[serde(default = "default_key_columns", skip_serializing_if = "is_default_key_columns")]
    pub key_columns: Vec<String>,
}

/// File error summary validation - supports single or multiple files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileErrorSummaryValidation {
    /// Single file configuration (backward compatible)
    Single(FileErrorSummaryFileConfig),
    /// Multiple file configurations
    Multiple(Vec<FileErrorSummaryFileConfig>),
}

impl FileErrorSummaryValidation {
    pub fn as_vec(&self) -> Vec<&FileErrorSummaryFileConfig> {
        match self {
            FileErrorSummaryValidation::Single(config) => vec![config],
            FileErrorSummaryValidation::Multiple(configs) => configs.iter().collect(),
        }
    }
}

fn default_key_columns() -> Vec<String> {
    vec!["Filename".to_string()]
}

fn is_default_key_columns(cols: &[String]) -> bool {
    cols.len() == 1 && cols[0] == "Filename"
}

/// Unexpected output validation - supports both simple boolean and detailed config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnexpectedOutputValidation {
    /// Simple boolean: true to enable validation with no ignore patterns
    Simple(bool),
    /// Detailed config with ignore patterns
    Detailed(UnexpectedOutputValidationConfig),
}

impl UnexpectedOutputValidation {
    pub fn is_enabled(&self) -> bool {
        match self {
            UnexpectedOutputValidation::Simple(v) => *v,
            UnexpectedOutputValidation::Detailed(_) => true,
        }
    }

    pub fn ignore_patterns(&self) -> Vec<String> {
        match self {
            UnexpectedOutputValidation::Simple(_) => vec![],
            UnexpectedOutputValidation::Detailed(c) => c.ignore_patterns.clone(),
        }
    }
}

/// Detailed configuration for unexpected output validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnexpectedOutputValidationConfig {
    /// File patterns to ignore when checking for unexpected outputs (supports glob patterns)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_patterns: Vec<String>,
}

use std::{
    cmp::Ordering,
    collections::HashMap,
    env,
    fmt::Debug,
    fs::{self, OpenOptions},
    panic,
    path::PathBuf,
    str,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use reearth_flow_action_log::factory::{self, LoggerFactory};
use reearth_flow_action_processor::mapping::ACTION_FACTORY_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_FACTORY_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::runner::Runner;
use reearth_flow_runtime::node::NodeKind;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;
use reearth_flow_worker::logger::{
    UserFacingLogHandler, UserFacingLogLayer, USER_FACING_LOG_FILE_WRITER,
};
use reearth_flow_worker::pubsub::backend::{noop::NoopPubSub, PubSubBackend};
use reearth_flow_worker::types::user_facing_log_event::UserFacingLogLevel;
use regex::Regex;
use rust_embed::RustEmbed;
use serde::{de::DeserializeOwned, Deserialize};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

pub static BUILTIN_ACTION_FACTORIES: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut common = HashMap::new();
    common.extend(SINK_MAPPINGS.clone());
    common.extend(SOURCE_MAPPINGS.clone());
    common.extend(PROCESSOR_MAPPINGS.clone());
    common
});

#[derive(RustEmbed)]
#[folder = "fixture/testdata/"]
pub struct Fixtures;

// ---------------------------------------------------------------------------
// Execute workflow and collect logs
// ---------------------------------------------------------------------------

pub struct LogTestResult {
    pub _tempdir: TempDir,
    pub action_log_dir: PathBuf,
    pub user_facing_log_path: PathBuf,
    #[allow(dead_code)]
    pub workflow_id: String,
    #[allow(dead_code)]
    pub job_id: Uuid,
}

/// Execute a workflow expected to succeed.
/// Panics if parsing fails or `Runner::run` returns an error.
#[allow(dead_code)]
pub fn execute_logging_test(fixture_dir: &str, workflow_name: &str) -> LogTestResult {
    execute_workflow(fixture_dir, workflow_name, true)
}

/// Execute a workflow expected to fail (parse error, runtime error, etc.).
/// Panics are still re-raised, but `Runner::run` errors and parse failures are tolerated.
#[allow(dead_code)]
pub fn execute_logging_error_test(fixture_dir: &str, workflow_name: &str) -> LogTestResult {
    execute_workflow(fixture_dir, workflow_name, false)
}

/// Shared implementation mirroring the production flow in `worker/src/command.rs`:
///   1. Set up logging infrastructure (action-log, user-facing log file, handler)
///   2. Try to parse the workflow YAML
///      - If parsing fails → `handler.send_workflow_definition_error(&e)`, skip Runner
///      - If parsing succeeds → `Runner::run(...)`
///   3. Return logs for verification
fn execute_workflow(fixture_dir: &str, workflow_name: &str, expect_success: bool) -> LogTestResult {
    // Ensure action logging is enabled regardless of CI environment variables.
    // ACTION_LOG_DISABLE is a Lazy<bool> static, so this must be set before its first evaluation.
    env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "false");

    let workflow_bytes = Fixtures::get(&format!("{fixture_dir}/workflow.yml"))
        .expect("missing workflow.yml fixture");
    let workflow_str = str::from_utf8(workflow_bytes.data.as_ref()).unwrap();

    let tempdir = tempfile::tempdir().unwrap();
    let folder_path = tempdir.path();
    fs::create_dir_all(folder_path).unwrap();

    let action_log_dir = folder_path.join("action-log");
    fs::create_dir_all(&action_log_dir).unwrap();

    let root_logger = factory::create_root_logger(action_log_dir.clone());
    let logger_factory = Arc::new(LoggerFactory::new(root_logger, action_log_dir.clone()));

    let storage_resolver = Arc::new(StorageResolver::new());
    let ingress_state =
        Arc::new(State::new(&Uri::for_test("ram:///ingress/"), &storage_resolver).unwrap());
    let feature_state_dir = folder_path.join("feature-state");
    fs::create_dir_all(&feature_state_dir).unwrap();
    let feature_state_uri = format!("file://{}/", feature_state_dir.to_str().unwrap());
    let feature_state =
        Arc::new(State::new(&Uri::for_test(&feature_state_uri), &storage_resolver).unwrap());

    let job_id = Uuid::new_v4();

    // Set up user-facing log file writer
    let uf_log_dir = folder_path.join("user-facing-log");
    fs::create_dir_all(&uf_log_dir).unwrap();
    let uf_log_path = uf_log_dir.join("user-facing.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&uf_log_path)
        .unwrap();
    let (non_blocking, _uf_guard) = tracing_appender::non_blocking(file);
    *USER_FACING_LOG_FILE_WRITER.write().unwrap() = Some(non_blocking);

    // Create handler + layer (same as production)
    let rt = Runtime::new().unwrap();
    let handler = Arc::new(UserFacingLogHandler::new(
        Uuid::nil(),
        job_id,
        PubSubBackend::Noop(NoopPubSub {}),
        rt.handle().clone(),
    ));
    handler.set_workflow_name(workflow_name.to_string());

    let layer = UserFacingLogLayer::new(handler.clone());
    let subscriber = tracing_subscriber::registry().with(layer);
    // Each test binary has its own process, so set_global_default is safe.
    let _ = tracing::subscriber::set_global_default(subscriber);

    // --- Production flow: try parse, then run ---
    let workflow_id;
    match Workflow::try_from(workflow_str) {
        Err(e) => {
            if expect_success {
                panic!("Expected workflow to parse successfully, but got: {e}");
            }
            // Production path: command.rs calls handler.send_workflow_definition_error(&e)
            handler.send_workflow_definition_error(&e);
            workflow_id = Uuid::nil().to_string();
        }
        Ok(mut workflow) => {
            workflow_id = workflow.id.to_string();

            // Inject workerArtifactPath so FileWriter can resolve its output path
            workflow
                .merge_with(HashMap::from([(
                    "workerArtifactPath".to_string(),
                    folder_path.to_str().unwrap().to_string(),
                )]))
                .unwrap();

            let catch_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                Runner::run(
                    job_id,
                    workflow,
                    BUILTIN_ACTION_FACTORIES.clone(),
                    logger_factory,
                    storage_resolver,
                    ingress_state,
                    feature_state,
                    None,
                )
            }));
            match catch_result {
                Err(panic_payload) => panic::resume_unwind(panic_payload),
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if expect_success {
                        panic!("Expected Runner to succeed, but got: {e:?}");
                    }
                }
            }
        }
    }

    // Drop guard guarantees all buffered writes are flushed.
    *USER_FACING_LOG_FILE_WRITER.write().unwrap() = None;
    drop(_uf_guard);

    LogTestResult {
        _tempdir: tempdir,
        action_log_dir,
        user_facing_log_path: uf_log_path,
        workflow_id,
        job_id,
    }
}

// ---------------------------------------------------------------------------
// User Facing Log
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Eq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct UserFacingLogEntry {
    #[allow(dead_code)]
    pub timestamp: DateTime<Utc>,
    #[allow(dead_code)]
    pub workflow_id: Uuid,
    #[allow(dead_code)]
    pub job_id: Uuid,
    #[allow(dead_code)]
    pub level: UserFacingLogLevel,
    pub node_id: Option<Uuid>,
    pub node_name: Option<String>,
    pub message: String,
}

/// Compare ignoring `workflow_id`, `job_id`, and `timestamp` (they vary per run).
/// The `message` field is normalized to strip variable elapsed timing.
impl PartialEq for UserFacingLogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level
            && self.node_name == other.node_name
            && self.node_id == other.node_id
            && normalize_uf_message(&self.message) == normalize_uf_message(&other.message)
    }
}

impl PartialOrd for UserFacingLogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserFacingLogEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = normalize_uf_message(&self.message);
        let b = normalize_uf_message(&other.message);
        (&self.level, &self.node_name, &self.node_id, &a).cmp(&(
            &other.level,
            &other.node_name,
            &other.node_id,
            &b,
        ))
    }
}

pub fn parse_user_facing_entries(content: &str) -> Vec<UserFacingLogEntry> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("invalid JSON in user-facing.log"))
        .collect()
}

/// Strip variable portions from user-facing log messages (elapsed timing, hashes).
fn normalize_uf_message(msg: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"Finished in [0-9.]+s|hash: \d+").unwrap());
    RE.replace_all(msg, "<REDACTED>").to_string()
}

pub fn verify_user_facing_log(fixture_dir: &str, result: &LogTestResult) {
    let expected_bytes = Fixtures::get(&format!("{fixture_dir}/user-facing.log"))
        .expect("missing expected user-facing.log fixture");
    let expected_str = str::from_utf8(expected_bytes.data.as_ref()).unwrap();
    let actual_str = fs::read_to_string(&result.user_facing_log_path)
        .unwrap_or_else(|e| panic!("Failed to read user-facing.log: {e}"));

    let mut expected = parse_user_facing_entries(expected_str);
    let mut actual = parse_user_facing_entries(&actual_str);
    expected.sort();
    actual.sort();
    assert_eq!(expected, actual);
}

// ---------------------------------------------------------------------------
// Action Log
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionLogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Deserialize, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionLogEntry {
    #[allow(dead_code)]
    pub ts: DateTime<Utc>,
    pub level: ActionLogLevel,
    pub module: String,
    #[allow(dead_code)]
    pub action: Uuid,
    pub msg: String,
}

/// Compare ignoring `ts` and `action` (they vary per run).
/// The `msg` field is normalized to strip variable elapsed/timing data.
/// The `module` field is normalized to strip line numbers.
impl PartialEq for ActionLogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level
            && normalize_module(&self.module) == normalize_module(&other.module)
            && normalize_action_msg(&self.msg) == normalize_action_msg(&other.msg)
    }
}

impl PartialOrd for ActionLogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActionLogEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = normalize_action_msg(&self.msg);
        let b = normalize_action_msg(&other.msg);
        let ma = normalize_module(&self.module);
        let mb = normalize_module(&other.module);
        (&self.level, &a, &ma).cmp(&(&other.level, &b, &mb))
    }
}

/// Strip variable elapsed/timing/stats/hash/feature-id portions from action log messages.
fn normalize_action_msg(msg: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"elapsed = [^,"]+|avg = [^,"]+|stddev = [^,"]+|features = \d+|hash: \d+|feature id = [a-f0-9-]+"#,
        )
        .unwrap()
    });
    RE.replace_all(msg, "<REDACTED>").to_string()
}

/// Strip line numbers from module paths (e.g. "module:68" -> "module").
fn normalize_module(module: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r":\d+$").unwrap());
    RE.replace(module, "").to_string()
}

#[allow(dead_code)]
pub fn verify_action_log(fixture_dir: &str, result: &LogTestResult) {
    let expected = Fixtures::get(&format!("{fixture_dir}/action.log"))
        .expect("missing expected action.log fixture");
    let expected_str = str::from_utf8(expected.data.as_ref()).unwrap();

    let actual_log_path = result.action_log_dir.join("all.log");
    let actual_str = fs::read_to_string(&actual_log_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", actual_log_path.display()));

    let mut expected_entries = parse_action_entries(expected_str);
    let mut actual_entries = parse_action_entries(&actual_str);
    expected_entries.sort();
    actual_entries.sort();
    assert_eq!(expected_entries, actual_entries);
}

#[allow(dead_code)]
pub fn verify_no_action_log(result: &LogTestResult) {
    let actual_log_path = result.action_log_dir.join("all.log");
    if actual_log_path.exists() {
        let content = fs::read_to_string(&actual_log_path).unwrap();
        let entries = parse_action_entries(&content);
        assert!(
            entries.is_empty(),
            "Expected no action log entries but found {}: {:?}",
            entries.len(),
            entries
        );
    }
}

pub fn parse_action_entries(content: &str) -> Vec<ActionLogEntry> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("invalid JSON in action.log"))
        .collect()
}

// ---------------------------------------------------------------------------
// Result Verification
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn verify_result_json<T>(fixture_dir: &str, result: &LogTestResult)
where
    T: DeserializeOwned + Debug + Eq + Ord,
{
    let expected_bytes = Fixtures::get(&format!("{fixture_dir}/result.json"))
        .expect("missing expected result.json fixture");
    let mut expected: Vec<T> =
        serde_json::from_slice(expected_bytes.data.as_ref()).expect("invalid expected result.json");

    let actual_path = result._tempdir.path().join("result.json");
    let actual_str = fs::read_to_string(&actual_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", actual_path.display()));
    let mut actual: Vec<T> = serde_json::from_str(&actual_str)
        .expect("actual result.json does not match expected schema");

    expected.sort();
    actual.sort();

    assert_eq!(expected, actual, "result.json content mismatch");
}

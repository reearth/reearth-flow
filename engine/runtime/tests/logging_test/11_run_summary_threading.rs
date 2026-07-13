//! Phase 2a Task 6: proves the by-value `RunSummary` actually makes it end
//! to end through `Runner::run_with_event_handler` / `run_with_sandbox_root`,
//! using the same numbered `logging/NN_*` fixtures as the golden logging
//! tests (reused here for their workflow shape only — these tests don't
//! touch the action-log/user-facing-log golden files).

// Unlike every other `logging_test/NN_*.rs` binary, this test doesn't drive
// `logging_helper`'s log-file verification apparatus (`execute_logging_test`,
// `verify_action_log`, `verify_user_facing_log`, ...) — it asserts directly
// on the `Result<RunSummary, Error>` returned by `Runner::run_with_event_handler`
// instead. Each `logging_test` binary compiles `logging_helper.rs` fresh as
// part of its own crate, so the unused half of that shared helper module
// would otherwise trip `dead_code` here (and nowhere else, since every other
// binary exercises the full module).
#[allow(dead_code)]
mod logging_helper;

use std::{collections::HashMap, str::FromStr, sync::Arc, sync::Once};

use logging_helper::{Fixtures, BUILTIN_ACTION_FACTORIES};
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::RunSummary;
use reearth_flow_runner::{errors::Error, runner::Runner};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;

/// Per-action log files are gated behind this env var (see
/// `reearth-flow-action-log`'s `ACTION_LOG_DISABLE` `Lazy`, evaluated once
/// on first use). These tests use a `Discard` root logger and never inspect
/// action-log output, so disable it up front — otherwise each node's
/// per-action `FileLoggerBuilder` tries to create a log file under the
/// `ram:///log/` sentinel path used below, which isn't a real directory.
static INIT: Once = Once::new();
fn init_test_env() {
    INIT.call_once(|| {
        std::env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
    });
}

/// Everything a `Runner::run_*` call needs, built fresh per test so each
/// test gets its own tempdir / job id and tests can't interfere with each
/// other.
struct PreparedRun {
    job_id: uuid::Uuid,
    workflow: Workflow,
    logger_factory: Arc<LoggerFactory>,
    storage_resolver: Arc<StorageResolver>,
    ingress_state: Arc<State>,
    feature_state: Arc<State>,
    sandbox_root: Uri,
}

/// Loads `logging/{fixture_dir}/workflow.yml` and wires up the minimal
/// context `Runner::run_with_event_handler` / `run_with_sandbox_root` need.
/// Mirrors `logging_helper::execute_workflow`'s setup but skips the
/// action-log / user-facing-log plumbing entirely: these tests only assert
/// on the returned `Result<RunSummary, Error>`, not on log files.
fn prepare_run(fixture_dir: &str) -> PreparedRun {
    init_test_env();
    let workflow_bytes = Fixtures::get(&format!("logging/{fixture_dir}/workflow.yml"))
        .unwrap_or_else(|| panic!("missing fixture logging/{fixture_dir}/workflow.yml"));
    let workflow_str = std::str::from_utf8(workflow_bytes.data.as_ref()).unwrap();

    let tempdir = tempfile::tempdir().unwrap();
    let folder_path = tempdir.keep();
    let folder_str = folder_path.to_str().unwrap().to_string();

    let storage_resolver = Arc::new(StorageResolver::new());
    let ingress_state =
        Arc::new(State::new(&Uri::for_test("ram:///ingress/"), &storage_resolver).unwrap());
    let feature_state_dir = folder_path.join("feature-state");
    std::fs::create_dir_all(&feature_state_dir).unwrap();
    let feature_state_uri = format!("file://{}/", feature_state_dir.to_str().unwrap());
    let feature_state =
        Arc::new(State::new(&Uri::for_test(&feature_state_uri), &storage_resolver).unwrap());

    let logger_factory = Arc::new(LoggerFactory::new(
        reearth_flow_action_log::ActionLogger::root(
            reearth_flow_action_log::Discard,
            reearth_flow_action_log::o!(),
        ),
        Uri::for_test("ram:///log/").path(),
    ));

    let mut workflow = Workflow::try_from(workflow_str)
        .unwrap_or_else(|e| panic!("failed to parse fixture logging/{fixture_dir}: {e}"));
    workflow
        .merge_with(HashMap::from([(
            "workerArtifactPath".to_string(),
            folder_str.clone(),
        )]))
        .unwrap();

    let sandbox_root = Uri::from_str(&format!("file://{folder_str}/")).unwrap();

    PreparedRun {
        job_id: uuid::Uuid::new_v4(),
        workflow,
        logger_factory,
        storage_resolver,
        ingress_state,
        feature_state,
        sandbox_root,
    }
}

fn run_with_event_handler(fixture_dir: &str) -> Result<RunSummary, Error> {
    let p = prepare_run(fixture_dir);
    Runner::run_with_event_handler(
        p.job_id,
        p.workflow,
        BUILTIN_ACTION_FACTORIES.clone(),
        p.logger_factory,
        p.storage_resolver,
        p.ingress_state,
        p.feature_state,
        None,
        vec![],
        p.sandbox_root,
    )
}

fn run_with_sandbox_root(fixture_dir: &str) -> Result<(), Error> {
    let p = prepare_run(fixture_dir);
    Runner::run_with_sandbox_root(
        p.job_id,
        p.workflow,
        BUILTIN_ACTION_FACTORIES.clone(),
        p.logger_factory,
        p.storage_resolver,
        p.ingress_state,
        p.feature_state,
        None,
        p.sandbox_root,
    )
}

/// scenario-01 (basic sequential processing, no errors, no aggregated
/// diagnostics): `run_with_event_handler` yields `Ok(summary)` with an empty
/// `failed_nodes` and — currently — an empty `aggregated_diagnostics`.
#[test]
fn passing_workflow_yields_ok_summary_with_no_diagnostics() {
    let summary = run_with_event_handler("01_basic_sequential_processing")
        .expect("scenario-01 workflow is expected to succeed");
    assert!(summary.failed_nodes.is_empty());
    assert!(summary.aggregated_diagnostics.is_empty());
    assert_eq!(summary.dropped_event_count, 0);
}

/// scenario-05 (source error, `ExecutionError::Source`): under the Task 5
/// invariant preserved by this task (`Ok(_)` implies `failed_nodes.is_empty()`),
/// a node-*thread* failure still surfaces as `Err`, not as `Ok(summary)` with
/// a populated `failed_nodes`. This is the thread-error invariant Task 6
/// explicitly does not change (a later task relaxes it).
///
/// This uses scenario-05, not scenario-06: scenario-06's "Attribute
/// Aggregator" error is a per-feature `process()` error that — since the
/// "unify node failure precedence" work (`Event::ProcessorFailed` +
/// `NodeStatus::Failed`, `runtime/src/executor/processor_node.rs`
/// `reconcile_terminate_result`) — sets `has_failed`/emits a status event
/// but does *not* propagate as an `ExecutionError`, so the node thread
/// itself still returns `Ok(())`. See
/// `processor_failure_event_can_diverge_from_thread_result` below, which
/// documents that case directly. Scenario-05's source error, by contrast, is
/// a genuine `Err` returned from the source thread's `receiver_loop` and so
/// exercises `DagExecutorJoinHandle::join`'s early-`Err` branch for real.
#[test]
fn failing_source_workflow_yields_err() {
    let err = run_with_event_handler("05_source_error")
        .expect_err("scenario-05 workflow is expected to fail");
    let rendered = err.to_string();
    assert!(
        rendered.contains("Source error"),
        "unexpected error text: {rendered}"
    );
}

/// Wrapper compat: `Runner::run_with_sandbox_root` still returns
/// `Result<(), Error>` and still yields `Err` with the same rendered text
/// for the same failing fixture — proving the thin wrapper's legacy
/// semantics survived the by-value `RunSummary` threading.
#[test]
fn run_with_sandbox_root_wrapper_still_returns_err_for_failing_source() {
    let err = run_with_sandbox_root("05_source_error")
        .expect_err("scenario-05 workflow is expected to fail through the wrapper too");
    let rendered = err.to_string();
    assert!(
        rendered.contains("Source error"),
        "unexpected error text: {rendered}"
    );
}

/// scenario-06 (processor error): documents the *divergence* case the
/// worker's belt-and-braces `JobResult` derivation (`worker/src/command.rs`)
/// exists to catch. The "Attribute Aggregator" node's `process()` returns an
/// `Err` for a missing attribute, which sets `has_failed`, sends
/// `Event::ProcessorFailed`, and emits `NodeStatus::Failed` — the signal
/// `NodeFailureHandler` (event-driven) keys off — but does not itself
/// propagate to the node thread's `Result`, so the thread still returns
/// `Ok(())` and `join()`/`fold_outcomes` sees no failure at all. So, today,
/// `run_with_event_handler` returns `Ok(summary)` with an *empty*
/// `failed_nodes` for this fixture even though the run visibly failed a
/// node. This is exactly why the worker checks both signals and logs when
/// they disagree, rather than trusting `RunSummary` alone.
#[test]
fn processor_failure_event_can_diverge_from_thread_result() {
    let summary = run_with_event_handler("06_processor_error").expect(
        "scenario-06's per-feature processor error does not propagate to an ExecutionError, \
         so the node thread — and therefore run_with_event_handler — still returns Ok",
    );
    assert!(
        summary.failed_nodes.is_empty(),
        "if this now fails, the Attribute Aggregator has started propagating process() errors \
         as a thread-level ExecutionError (e.g. via ctx.fatal()) — update this test's expectation \
         and consider whether NodeFailureHandler is now redundant with RunSummary"
    );
}

/// scenario-10 (warn-drop aggregation): the real prize. Proves the
/// finish()-time aggregated diagnostic (3 dropped features, folded into one
/// `Diagnostic` with `AggregateInfo { count: 3, .. }`) is delivered by value
/// through `join()` -> `run_dag_executor` -> `run_apps`/`run_all` ->
/// `run_with_event_handler`, all the way back to the test.
#[test]
fn warn_drop_aggregation_diagnostic_is_delivered_by_value() {
    let summary = run_with_event_handler("10_warn_drop_aggregation")
        .expect("scenario-10 workflow is expected to succeed (warn-drop, not fatal)");
    assert!(summary.failed_nodes.is_empty());
    assert_eq!(summary.aggregated_diagnostics.len(), 1);
    let diagnostic = &summary.aggregated_diagnostics[0];
    let aggregated = diagnostic
        .aggregated
        .as_ref()
        .expect("finish()-time summary diagnostic should carry AggregateInfo");
    assert_eq!(aggregated.count, 3);
}

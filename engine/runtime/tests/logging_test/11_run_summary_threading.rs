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
    let workflow_bytes = Fixtures::get(&format!("logging/{fixture_dir}/workflow.yml"))
        .unwrap_or_else(|| panic!("missing fixture logging/{fixture_dir}/workflow.yml"));
    let workflow_str = std::str::from_utf8(workflow_bytes.data.as_ref()).unwrap();
    prepare_run_from_yaml(workflow_str)
}

/// Like [`prepare_run`], but parses `workflow_yaml` directly instead of
/// loading a `logging/NN_*` fixture from the `Fixtures` embed — used by
/// tests that construct a workflow inline rather than reusing a numbered
/// golden-adjacent scenario (e.g. the D8 branch-completion test below,
/// which needs a two-branch shape no existing fixture has).
fn prepare_run_from_yaml(workflow_yaml: &str) -> PreparedRun {
    init_test_env();

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

    let mut workflow = Workflow::try_from(workflow_yaml)
        .unwrap_or_else(|e| panic!("failed to parse workflow yaml: {e}"));
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

// ---------------------------------------------------------------------------
// Phase 2a-policy Task 4: `errorPolicy: { onFatal: continue }` counterparts
// to the scenario-05 tripwires above. Same failing fixture (Feature Creator
// source raises for `"not_an_array"`), only the policy differs — proving
// `DagExecutorJoinHandle::join` forks on `disposition_policy.on_fatal()`
// rather than always early-`Err`ing.
// ---------------------------------------------------------------------------

/// Scenario-05's failing "Feature Creator" node id — no subgraphs in that
/// fixture, so composed id == raw id. Read straight out of
/// `logging/05_source_error/workflow.yml`.
const SCENARIO_05_SOURCE_NODE_ID: &str = "a1f90a3e-61d3-48e2-a328-e7226c2ad1ae";

/// Scenario-05's failing "JSON Writer" sink node id — the source's only
/// downstream, orphaned by the same failure (see the test doc below).
const SCENARIO_05_SINK_NODE_ID: &str = "c1f90a3e-61d3-48e2-a328-e7226c2ad1ae";

/// Under `onFatal: continue`, `run_with_event_handler` — the summary-
/// returning entry point `join()` feeds directly — yields `Ok(summary)`
/// instead of `Err`: every node thread's outcome is folded into
/// `failed_nodes` (composed id + Fatal stamp, per `fold_outcomes`) rather
/// than short-circuiting the run.
///
/// That's TWO failed nodes here, not one: scenario-05's only sink ("JSON
/// Writer") is entirely downstream of the failing source, with no
/// independent branch of its own (unlike the D8 test below). When
/// `Feature Creator.start()` errors, `SourceNode::run` returns immediately
/// — without calling `send_to_all_nodes` for the source's already-connected
/// downstream channels — so the sink never receives so much as a
/// `Terminate` op. Once the source thread exits and drops its senders, the
/// orphaned sink's `recv()` observes a disconnected channel and reports its
/// own `CannotReceiveFromChannel` failure. This cascade is pre-existing
/// `SourceNode` behavior, orthogonal to this task's `join()` policy fork —
/// it was always latent, just invisible before now because `Terminate`'s
/// early-`Err` only ever surfaced whichever of the two threads finished
/// first and discarded the other. `Continue` surfaces every thread's
/// outcome, so both show up here.
#[test]
fn failing_source_workflow_under_continue_policy_yields_ok_with_failed_nodes() {
    use reearth_flow_diagnostics::Disposition;
    use reearth_flow_types::{ErrorPolicy, OnFatal};

    let mut p = prepare_run("05_source_error");
    p.workflow.error_policy = Some(ErrorPolicy {
        on_fatal: OnFatal::Continue,
        ..Default::default()
    });

    let summary = Runner::run_with_event_handler(
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
    .expect("onFatal: continue must turn the failing source's Err into Ok(summary)");

    assert_eq!(summary.failed_nodes.len(), 2);
    assert!(
        summary
            .failed_nodes
            .iter()
            .any(|d| d.node_id.as_deref() == Some(SCENARIO_05_SOURCE_NODE_ID)),
        "the source's own failure must be present in failed_nodes: {:?}",
        summary.failed_nodes
    );
    assert!(
        summary
            .failed_nodes
            .iter()
            .any(|d| d.node_id.as_deref() == Some(SCENARIO_05_SINK_NODE_ID)),
        "the orphaned sink's cascaded failure must be present in failed_nodes: {:?}",
        summary.failed_nodes
    );
    assert!(
        summary
            .failed_nodes
            .iter()
            .all(|d| d.effective_disposition == Some(Disposition::Fatal)),
        "every failed_nodes entry must be Fatal-stamped by fold_outcomes: {:?}",
        summary.failed_nodes
    );
}

/// Same `onFatal: continue` input as above, but through the unit-returning
/// `Runner::run_with_sandbox_root` wrapper: it must still yield `Err`,
/// proving `summary_into_unit_result`'s defensive mapping (commit
/// `77109d4b5`) is now load-bearing — this is exactly the case that mapping
/// was added ahead of time for.
#[test]
fn run_with_sandbox_root_wrapper_still_returns_err_under_continue_policy() {
    use reearth_flow_types::{ErrorPolicy, OnFatal};

    let mut p = prepare_run("05_source_error");
    p.workflow.error_policy = Some(ErrorPolicy {
        on_fatal: OnFatal::Continue,
        ..Default::default()
    });

    let err = Runner::run_with_sandbox_root(
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
    .expect_err(
        "the unit-returning wrapper must still surface a failed node as Err, \
         even though run_with_event_handler now returns Ok(summary) for it",
    );
    let rendered = err.to_string();
    assert!(
        // 2 failed_nodes here (source + cascaded-orphaned sink — see the
        // sibling test above), so `summary_into_unit_result`'s "{N} node(s)
        // failed" formatting reports 2.
        rendered.contains("2 node(s) failed"),
        "unexpected error text (expected the failed-node count): {rendered}"
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

// ---------------------------------------------------------------------------
// Phase 2a-policy Task 3: composed node ids + policy threading, end to end.
//
// Reuses scenario-10's workflow shape (Feature Creator -> Cesium 3D Tiles
// Writer, 3 features dropped for empty geometry) but injects an `errorPolicy`
// directly on the parsed `Workflow` in Rust rather than editing the golden
// fixture — the golden fixture must stay policy-free so the existing
// `10_warn_drop_aggregation` test keeps proving zero-policy == zero
// behavior change. The writer node's id, `42a900a6-0f2a-4364-a4d2-
// dd8fa63d3dd0`, is read straight out of `logging/10_warn_drop_aggregation/
// workflow.yml`; since that fixture has no subgraphs, composed id == raw id.
// ---------------------------------------------------------------------------

const SCENARIO_10_WRITER_NODE_ID: &str = "42a900a6-0f2a-4364-a4d2-dd8fa63d3dd0";

/// The reject-override runner test (spec 4.2/4.7, this task's binding
/// end-to-end proof): overriding `cesium3dtiles.empty_geometry` to `reject`
/// on the writer node must resolve through `NodeDiagnosticsHandle::resolve`
/// at `report()` time instead of the registry default (`warn_drop`) — the
/// run still succeeds (`Ok`, no failed nodes; Reject in a sink with no
/// side-file just aggregates at this task's point, see the brief's Task 5
/// note), but the aggregated summary's `effective_disposition` is `Reject`,
/// not `WarnDrop`, proving the resolve() ladder is live end to end, not just
/// unit-tested in isolation.
#[test]
fn reject_override_promotes_disposition_through_resolve_end_to_end() {
    use reearth_flow_diagnostics::Disposition;
    use reearth_flow_types::{ErrorPolicy, PolicyDisposition, PolicyOverride};

    let mut p = prepare_run("10_warn_drop_aggregation");
    p.workflow.error_policy = Some(ErrorPolicy {
        overrides: vec![PolicyOverride {
            node: Some(SCENARIO_10_WRITER_NODE_ID.to_string()),
            code: Some("cesium3dtiles.empty_geometry".to_string()),
            category: None,
            disposition: PolicyDisposition::Reject,
        }],
        ..Default::default()
    });

    let summary = Runner::run_with_event_handler(
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
    .expect("reject-override run is expected to succeed (Reject, not fatal)");

    assert!(summary.failed_nodes.is_empty());
    assert_eq!(summary.aggregated_diagnostics.len(), 1);
    let diagnostic = &summary.aggregated_diagnostics[0];
    assert_eq!(diagnostic.effective_disposition, Some(Disposition::Reject));
    let aggregated = diagnostic
        .aggregated
        .as_ref()
        .expect("finish()-time summary diagnostic should carry AggregateInfo");
    assert_eq!(aggregated.count, 3);
}

/// An `errorPolicy` that fails `DispositionPolicy::compile` (here: an
/// unknown error code) must abort the run before DAG construction, as a
/// `PolicyValidationError` naming the bad code — never silently fall back
/// to the registry default.
#[test]
fn unknown_error_code_in_policy_aborts_before_dag_construction() {
    use reearth_flow_types::{ErrorPolicy, PolicyDisposition, PolicyOverride};

    let mut p = prepare_run("10_warn_drop_aggregation");
    p.workflow.error_policy = Some(ErrorPolicy {
        overrides: vec![PolicyOverride {
            node: None,
            code: Some("not.a.real.code".to_string()),
            category: None,
            disposition: PolicyDisposition::Reject,
        }],
        ..Default::default()
    });

    let err = Runner::run_with_event_handler(
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
    .expect_err("an unknown error code in errorPolicy must abort the run");
    let rendered = err.to_string();
    assert!(
        rendered.contains("not.a.real.code"),
        "unexpected error text: {rendered}"
    );
}

/// An override naming a node id absent from the workflow graph must abort
/// the run once the DAG has been built (load-time node matching, spec 4.2) —
/// this exercises the path that needs the flattened DAG, distinct from the
/// compile-time check above.
#[test]
fn unmatched_node_selector_aborts_after_dag_construction() {
    use reearth_flow_types::{ErrorPolicy, PolicyDisposition, PolicyOverride};

    let mut p = prepare_run("10_warn_drop_aggregation");
    p.workflow.error_policy = Some(ErrorPolicy {
        overrides: vec![PolicyOverride {
            node: Some("node-that-does-not-exist".to_string()),
            code: None,
            category: None,
            disposition: PolicyDisposition::Reject,
        }],
        ..Default::default()
    });

    let err = Runner::run_with_event_handler(
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
    .expect_err("an override naming an unknown node id must abort the run");
    let rendered = err.to_string();
    assert!(
        rendered.contains("node-that-does-not-exist"),
        "unexpected error text: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// D8 (spec): best-effort branch completion under `onFatal: continue`. A
// two-branch workflow — an independent failing branch and an independent
// succeeding branch, sharing no nodes or edges — proves the succeeding
// branch actually finishes (its sink writes real output) rather than merely
// not being counted as failed.
//
// The failing branch's sink ("Failing Branch Writer") fails deterministically
// at `process()` time, once per feature (`attributes["nonexistentField"]`
// evaluates via the `Attributes.__getitem__` CEL binding, which errors on a
// missing key — see `runtime/types/src/expr.rs`). This is a genuine
// *sink-thread*-level `Err` (unlike scenario-06's Attribute Aggregator,
// whose per-feature error never reaches the node thread's `Result`), but —
// critically for branch independence — it happens entirely inside
// `SinkNode::receiver_loop`'s per-op handling, not in source startup: the
// sink still receives (and correctly acks) its own `Terminate` op before its
// thread returns `Err`, so the shared "sources" thread (which hosts BOTH
// branches' `Feature Creator` sources — `DagExecutor::start` collapses every
// `Source`-kind node into one thread) completes normally and reaches
// `send_to_all_nodes` for both branches. Using a *source*-level failure for
// one branch instead would abort that shared thread before it ever
// terminates the other branch's downstream sink, deadlocking the succeeding
// branch — so the failure is deliberately placed in a sink, not a source.
const BRANCH_COMPLETION_WORKFLOW_YAML: &str = r#"
id: 5714128a-66a0-4a38-b0fb-2c77f590767d
name: "Branch Completion Continue Test"
entryGraphId: d1afaf50-ed16-4595-b539-eb081c7f3dc8
with:
graphs:
  - id: d1afaf50-ed16-4595-b539-eb081c7f3dc8
    name: main_graph
    nodes:
      - id: f450d035-e9b7-4847-8541-79d2c7ce59fd
        name: Failing Branch Source
        type: action
        action: Feature Creator
        with:
          creator:
            type: flowExpr
            value: '[{"id": 1}]'
      - id: 448e4d1f-73a4-4d75-a565-832d43619b8d
        name: Failing Branch Writer
        type: action
        action: JSON Writer
        with:
          output:
            type: flowExpr
            value: 'attributes["nonexistentField"]'
      - id: 9cff2417-1129-4ab7-bce1-7a0ad6e24aaf
        name: Succeeding Branch Source
        type: action
        action: Feature Creator
        with:
          creator:
            type: flowExpr
            value: '[{"id": 2}]'
      - id: 1f757bb4-9274-4e0e-8082-ceae9d319164
        name: Succeeding Branch Writer
        type: action
        action: JSON Writer
        with:
          output:
            type: string
            value: succeeding_output.json
    edges:
      - id: abdbe066-4942-4a5a-b457-98d4257e18b9
        from: f450d035-e9b7-4847-8541-79d2c7ce59fd
        to: 448e4d1f-73a4-4d75-a565-832d43619b8d
        fromPort: features
        toPort: features
      - id: b231d52d-7ddc-4234-9b8e-6978270dcfcb
        from: 9cff2417-1129-4ab7-bce1-7a0ad6e24aaf
        to: 1f757bb4-9274-4e0e-8082-ceae9d319164
        fromPort: features
        toPort: features
"#;

const BRANCH_COMPLETION_FAILING_WRITER_NODE_ID: &str = "448e4d1f-73a4-4d75-a565-832d43619b8d";

/// D8, the prize: under `onFatal: continue`, the failing branch is recorded
/// as exactly one failed node while the independent succeeding branch's sink
/// actually completes — verified by its output file existing on disk, not
/// just by the summary's shape.
#[test]
fn branch_completion_continue_completes_independent_branch_and_records_one_failed_node() {
    use reearth_flow_diagnostics::Disposition;
    use reearth_flow_types::{ErrorPolicy, OnFatal};

    let mut p = prepare_run_from_yaml(BRANCH_COMPLETION_WORKFLOW_YAML);
    p.workflow.error_policy = Some(ErrorPolicy {
        on_fatal: OnFatal::Continue,
        ..Default::default()
    });
    let output_path = p.sandbox_root.path().join("succeeding_output.json");

    let summary = Runner::run_with_event_handler(
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
    .expect("onFatal: continue must complete the run despite the failing branch");

    assert_eq!(summary.failed_nodes.len(), 1);
    assert_eq!(
        summary.failed_nodes[0].node_id.as_deref(),
        Some(BRANCH_COMPLETION_FAILING_WRITER_NODE_ID)
    );
    assert_eq!(
        summary.failed_nodes[0].effective_disposition,
        Some(Disposition::Fatal)
    );
    assert!(
        output_path.exists(),
        "the succeeding branch's independent sink must have written its output to {}",
        output_path.display()
    );
}

/// Same two-branch workflow, default `Terminate` policy: the run must still
/// fail overall (`Err`), same as every pre-Task-4 fatal — the D8 relief is
/// opt-in via `onFatal: continue`, not the default.
#[test]
fn branch_completion_terminate_default_still_errors_for_same_workflow() {
    let p = prepare_run_from_yaml(BRANCH_COMPLETION_WORKFLOW_YAML);

    let err = Runner::run_with_event_handler(
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
    .expect_err("default onFatal: terminate must still fail the run");
    let rendered = err.to_string();
    assert!(
        !rendered.is_empty(),
        "expected a non-empty rendered error, got: {rendered}"
    );
}

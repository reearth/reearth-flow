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
/// run still succeeds (`Ok`, no failed nodes), and the aggregated summary's
/// `effective_disposition` is `Reject`, not `WarnDrop`, proving the
/// resolve() ladder is live end to end, not just unit-tested in isolation.
///
/// Phase 2a-policy Task 5 update: `sideFile: true` is now required for a
/// Reject-promoting override on a sink to load at all (spec 4.4's load-time
/// validation, see the two tests below) — this test adds it and additionally
/// asserts the D7 side-file shard (`rejected/{node}.jsonl`) was written with
/// one row per rejected feature, closing the loop this test's own doc
/// comment used to defer to "Task 5".
#[test]
fn reject_override_promotes_disposition_through_resolve_end_to_end() {
    use reearth_flow_diagnostics::Disposition;
    use reearth_flow_types::{ErrorPolicy, PolicyDisposition, PolicyOverride};

    let mut p = prepare_run("10_warn_drop_aggregation");
    p.workflow.error_policy = Some(ErrorPolicy {
        side_file: true,
        overrides: vec![PolicyOverride {
            node: Some(SCENARIO_10_WRITER_NODE_ID.to_string()),
            code: Some("cesium3dtiles.empty_geometry".to_string()),
            category: None,
            disposition: PolicyDisposition::Reject,
        }],
        ..Default::default()
    });
    let sandbox_path = p.sandbox_root.path();

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

    // D7: the side-file shard exists with one JSONL row per rejected
    // feature, matching the aggregator's bucket count above.
    let shard_path = sandbox_path.join(format!("rejected/{SCENARIO_10_WRITER_NODE_ID}.jsonl"));
    let shard_content = std::fs::read_to_string(&shard_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", shard_path.display()));
    let lines: Vec<&str> = shard_content.lines().collect();
    assert_eq!(lines.len(), 3, "expected 3 reject rows, got: {lines:?}");
    for line in lines {
        let row: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(
            row["code"],
            serde_json::json!("cesium3dtiles.empty_geometry")
        );
        assert_eq!(row["hasGeometry"], serde_json::json!(false));
        assert!(row["featureId"].is_string());
    }
}

/// Negative counterpart (spec 4.4, Task 5): the same Reject-promoting
/// override on the same sink, but WITHOUT `sideFile: true`, must abort the
/// run before DAG construction ever starts a node thread — a sink with a
/// possible `Reject` needs somewhere configured to route it.
#[test]
fn reject_promoting_override_on_a_sink_without_side_file_aborts_the_run() {
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
    .expect_err("a reject-promoting override on a sink without sideFile must abort the run");
    let rendered = err.to_string();
    assert!(
        rendered.contains(SCENARIO_10_WRITER_NODE_ID),
        "unexpected error text: {rendered}"
    );
    assert!(
        rendered.contains("errorPolicy.sideFile"),
        "unexpected error text: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// Phase 2a-policy Task 5: load-time Reject-routing validation for processor
// nodes (spec 4.4) — a processor must declare the `rejected` output port AND
// have it wired, or a Reject-promoting override on it aborts the run.
// "Geometry Validator" already declares `rejected` among its output ports
// (`runtime/action-processor/src/geometry/validator.rs`), so it's a real
// action rather than a synthetic fixture; the override's code doesn't need
// to be one the action would ever actually emit — this validation is
// structural (spec 4.4's conservative over-approximation), not behavioral.
// ---------------------------------------------------------------------------

const PROCESSOR_REJECT_SOURCE_ID: &str = "5e96c308-75c7-4d43-81f7-7d7f5be4ae9b";
const PROCESSOR_REJECT_VALIDATOR_ID: &str = "5336c448-2933-4581-977e-5970d5d8e6d4";
const PROCESSOR_REJECT_SUCCESS_WRITER_ID: &str = "905c1f85-93f6-4737-b5b6-9c620f71d0f9";
const PROCESSOR_REJECT_REJECTED_WRITER_ID: &str = "9171d021-305a-4bd7-be13-9bf2cbfbbb02";

/// `wire_rejected`: whether an edge from the Geometry Validator's `rejected`
/// port to a second JSON Writer is included.
fn processor_reject_workflow_yaml(wire_rejected: bool) -> String {
    let rejected_writer_node = if wire_rejected {
        format!(
            r#"
      - id: {PROCESSOR_REJECT_REJECTED_WRITER_ID}
        name: Rejected Writer
        type: action
        action: JSON Writer
        with:
          output:
            type: string
            value: rejected_output.json"#
        )
    } else {
        String::new()
    };
    let rejected_edge = if wire_rejected {
        format!(
            r#"
      - id: 5a01558f-a478-4f12-8bd4-47b7b4bd1a79
        from: {PROCESSOR_REJECT_VALIDATOR_ID}
        to: {PROCESSOR_REJECT_REJECTED_WRITER_ID}
        fromPort: rejected
        toPort: features"#
        )
    } else {
        String::new()
    };
    format!(
        r#"
id: 01b1918e-cb47-48f2-9fca-38751af1556f
name: "Processor Reject Routing Test"
entryGraphId: dc15f659-079a-4527-9c9f-4646ab132380
with:
graphs:
  - id: dc15f659-079a-4527-9c9f-4646ab132380
    name: main_graph
    nodes:
      - id: {PROCESSOR_REJECT_SOURCE_ID}
        name: Feature Creator
        type: action
        action: Feature Creator
        with:
          creator:
            type: flowExpr
            value: '[{{"id": 1}}]'
      - id: {PROCESSOR_REJECT_VALIDATOR_ID}
        name: Geometry Validator
        type: action
        action: Geometry Validator
        with:
          validationTypes:
            - duplicatePoints
      - id: {PROCESSOR_REJECT_SUCCESS_WRITER_ID}
        name: Success Writer
        type: action
        action: JSON Writer
        with:
          output:
            type: string
            value: success_output.json{rejected_writer_node}
    edges:
      - id: b29806b9-5f20-4395-a1ee-4a11e22b8dc3
        from: {PROCESSOR_REJECT_SOURCE_ID}
        to: {PROCESSOR_REJECT_VALIDATOR_ID}
        fromPort: features
        toPort: features
      - id: d3107574-7066-408a-bf46-89002ec0985c
        from: {PROCESSOR_REJECT_VALIDATOR_ID}
        to: {PROCESSOR_REJECT_SUCCESS_WRITER_ID}
        fromPort: success
        toPort: features{rejected_edge}
"#
    )
}

fn processor_reject_error_policy() -> reearth_flow_types::ErrorPolicy {
    use reearth_flow_types::{PolicyDisposition, PolicyOverride};
    reearth_flow_types::ErrorPolicy {
        overrides: vec![PolicyOverride {
            node: Some(PROCESSOR_REJECT_VALIDATOR_ID.to_string()),
            code: Some("gltf.zero_face_solid".to_string()),
            category: None,
            disposition: PolicyDisposition::Reject,
        }],
        ..Default::default()
    }
}

/// A processor with `rejected` declared but NOT wired: the reject-promoting
/// override must abort the run at load time.
#[test]
fn reject_promoting_override_on_a_processor_with_unwired_rejected_port_aborts_the_run() {
    let mut p = prepare_run_from_yaml(&processor_reject_workflow_yaml(false));
    p.workflow.error_policy = Some(processor_reject_error_policy());

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
    .expect_err(
        "a reject-promoting override on a processor with an unwired rejected port must abort",
    );
    let rendered = err.to_string();
    assert!(
        rendered.contains(PROCESSOR_REJECT_VALIDATOR_ID),
        "unexpected error text: {rendered}"
    );
    assert!(
        rendered.contains("rejected"),
        "unexpected error text: {rendered}"
    );
}

/// Same override, but with `rejected` wired to a second sink: the run must
/// load and complete successfully.
#[test]
fn reject_promoting_override_on_a_processor_with_wired_rejected_port_succeeds() {
    let mut p = prepare_run_from_yaml(&processor_reject_workflow_yaml(true));
    p.workflow.error_policy = Some(processor_reject_error_policy());

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
    .expect("wiring the rejected port must let the run load and complete");
    assert!(summary.failed_nodes.is_empty());
}

// ---------------------------------------------------------------------------
// Phase 2a-policy Task 5: reject-shard no-clobber (spec §7). Two independent
// branches, each with its own reject-promoting sink, prove the two sinks'
// `rejected/{composed_id}.jsonl` shards land at distinct paths and neither
// clobbers the other's rows.
// ---------------------------------------------------------------------------

const REJECT_NO_CLOBBER_SINK_A_ID: &str = "a84c1737-48bf-431b-9723-8146bc8975b5";
const REJECT_NO_CLOBBER_SINK_B_ID: &str = "67e1e7a4-4c50-445d-8a63-7b5438bb2ce7";

const REJECT_NO_CLOBBER_WORKFLOW_YAML: &str = r#"
id: bb97151b-e00f-4e7e-b765-cdbc12880430
name: "Reject Shard No Clobber Test"
entryGraphId: 4e522009-85aa-46cc-b710-45c32e4be935
with:
graphs:
  - id: 4e522009-85aa-46cc-b710-45c32e4be935
    name: main_graph
    nodes:
      - id: 56220bc9-588a-4245-8d43-8c5b2ce075f5
        name: Feature Creator A
        type: action
        action: Feature Creator
        with:
          creator:
            type: flowExpr
            value: |
              let features = [];
              for i in range(1, 4) {
                features.append({"id": i});
              }
              features
      - id: a84c1737-48bf-431b-9723-8146bc8975b5
        name: Cesium 3D Tiles Writer A
        type: action
        action: Cesium 3D Tiles Writer
        with:
          output:
            type: string
            value: cesium_output_a
          minZoom: 15
          maxZoom: 18
      - id: 787f935c-228e-4504-8d08-3c8ab351f602
        name: Feature Creator B
        type: action
        action: Feature Creator
        with:
          creator:
            type: flowExpr
            value: |
              let features = [];
              for i in range(1, 6) {
                features.append({"id": i});
              }
              features
      - id: 67e1e7a4-4c50-445d-8a63-7b5438bb2ce7
        name: Cesium 3D Tiles Writer B
        type: action
        action: Cesium 3D Tiles Writer
        with:
          output:
            type: string
            value: cesium_output_b
          minZoom: 15
          maxZoom: 18
    edges:
      - id: b647aeb9-f915-41fa-ab81-e3879b6717bf
        from: 56220bc9-588a-4245-8d43-8c5b2ce075f5
        to: a84c1737-48bf-431b-9723-8146bc8975b5
        fromPort: features
        toPort: features
      - id: 8ee7fbc3-2f98-4d30-a246-19e32325b6e6
        from: 787f935c-228e-4504-8d08-3c8ab351f602
        to: 67e1e7a4-4c50-445d-8a63-7b5438bb2ce7
        fromPort: features
        toPort: features
"#;

#[test]
fn reject_shards_for_two_sinks_do_not_clobber_each_other() {
    use reearth_flow_types::{ErrorPolicy, PolicyDisposition, PolicyOverride};

    let mut p = prepare_run_from_yaml(REJECT_NO_CLOBBER_WORKFLOW_YAML);
    p.workflow.error_policy = Some(ErrorPolicy {
        side_file: true,
        // Codeless-by-node selector: applies uniformly to both sinks
        // without needing two separate overrides.
        overrides: vec![PolicyOverride {
            node: None,
            code: Some("cesium3dtiles.empty_geometry".to_string()),
            category: None,
            disposition: PolicyDisposition::Reject,
        }],
        ..Default::default()
    });
    let sandbox_path = p.sandbox_root.path();

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
    .expect("two-sink reject-shard run is expected to succeed");
    assert!(summary.failed_nodes.is_empty());

    let bucket_count_for = |node_id: &str| -> u64 {
        summary
            .aggregated_diagnostics
            .iter()
            .find(|d| d.node_id.as_deref() == Some(node_id))
            .and_then(|d| d.aggregated.as_ref())
            .map(|a| a.count)
            .unwrap_or_else(|| panic!("no aggregated diagnostic for node {node_id}"))
    };

    let read_shard = |node_id: &str| -> Vec<String> {
        let path = sandbox_path.join(format!("rejected/{node_id}.jsonl"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
            .lines()
            .map(String::from)
            .collect()
    };

    let shard_a = read_shard(REJECT_NO_CLOBBER_SINK_A_ID);
    let shard_b = read_shard(REJECT_NO_CLOBBER_SINK_B_ID);

    assert_eq!(
        shard_a.len() as u64,
        bucket_count_for(REJECT_NO_CLOBBER_SINK_A_ID)
    );
    assert_eq!(
        shard_b.len() as u64,
        bucket_count_for(REJECT_NO_CLOBBER_SINK_B_ID)
    );
    assert_eq!(shard_a.len(), 3, "Writer A: 3 features created");
    assert_eq!(shard_b.len(), 5, "Writer B: 5 features created");
    // Neither shard's rows leaked into the other's file.
    assert_ne!(shard_a, shard_b);
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

// ---------------------------------------------------------------------------
// Phase 2a-policy Task 7: fatal-override end-to-end, runner-level companions
// to the `test-logging-fatal-override` golden (`logging/12_fatal_override`).
// ---------------------------------------------------------------------------

/// scenario-12's writer node id (`errorPolicy` promotes
/// `cesium3dtiles.empty_geometry` to `fatal` on this node), read straight out
/// of `logging/12_fatal_override/workflow.yml`. No subgraphs in that
/// fixture, so composed id == raw id.
const SCENARIO_12_WRITER_NODE_ID: &str = "cbd5f624-b7cd-4a11-b6dd-181063c314d4";

/// The golden `test-logging-fatal-override` scenario proves the default
/// `onFatal: terminate` shape (the run fails with `Err`, no `RunSummary` is
/// ever produced). This is its `onFatal: continue` counterpart -- same
/// fixture, only the policy's `on_fatal` field flipped -- proving the
/// swallowed/superseded-fatal drain-end backstop (spec §7) surfaces the
/// writer as a `failed_nodes` entry when the run is allowed to keep going
/// instead of aborting.
///
/// **Trace (see the report for the full derivation):** all 3 of the writer's
/// `report()` calls resolve `Fatal` via the override; the *first* one's `Err`
/// propagates through `process_default`'s `?` as a real thread-level error
/// (`ExecutionError::Sink`, wrapping the *stringified*
/// `SinkError::Cesium3DTilesWriter(diag.to_string())` -- not the structured
/// `Diagnostic` itself). At the sink's drain end,
/// `reconcile_sink_terminate_result` gives that real returned error
/// precedence over the fatal slot (`first_error` > fatal slot), so the node
/// thread's `Result` carries the *stringified* error, not a boxed
/// `Diagnostic`. `fold_outcomes`' `diagnostic_from_execution_error` can only
/// recover the original structured `Diagnostic` by downcasting the boxed
/// error to `Diagnostic` -- which only succeeds for the *raw fatal-slot-wins*
/// case (`reconcile_sink_terminate_result`'s third arm, `Err(ExecutionError::
/// Sink(Box::new(diag)))`, reached only when the action swallows `report()`'s
/// `Err` entirely and nothing else fails). Since this scenario's `report()`
/// Err is genuinely propagated (not swallowed), the downcast fails and
/// `fold_outcomes` synthesizes a `failed_nodes` entry under the catch-all
/// `internal.unclassified` code instead -- carrying the *original*
/// diagnostic's rendering (including the `cesium3dtiles.empty_geometry` code
/// string and its message) inside its own `message` field, and the writer's
/// composed node id via `meta.composed_id`. This is still the drain-end
/// backstop doing its job: a swallowed/superseded fatal is never silently
/// invisible, it just surfaces as `internal.unclassified` rather than
/// impersonating the original code when the original diagnostic itself
/// wasn't what won the node's final `Result`.
#[test]
fn fatal_override_under_continue_policy_surfaces_the_writer_in_failed_nodes() {
    use reearth_flow_diagnostics::{Disposition, ErrorCode};
    use reearth_flow_types::OnFatal;

    let mut p = prepare_run("12_fatal_override");
    p.workflow
        .error_policy
        .as_mut()
        .expect("fixture bakes in an errorPolicy fatal override")
        .on_fatal = OnFatal::Continue;

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
    .expect("onFatal: continue must turn the fatal-overridden writer's Err into Ok(summary)");

    assert_eq!(summary.failed_nodes.len(), 1);
    let failed = &summary.failed_nodes[0];
    assert_eq!(failed.node_id.as_deref(), Some(SCENARIO_12_WRITER_NODE_ID));
    assert_eq!(failed.effective_disposition, Some(Disposition::Fatal));
    // The node's real thread-level Result carried the stringified
    // SinkError, not a boxed Diagnostic, so fold_outcomes' downcast falls
    // through to the catch-all code -- see the trace above.
    assert_eq!(failed.code, ErrorCode::InternalUnclassified);
    assert!(
        failed.message.contains("cesium3dtiles.empty_geometry"),
        "the original diagnostic's code must still be visible in the synthesized \
         entry's message, got: {}",
        failed.message
    );
}

/// `treatAllAsFatal` runner-level test: scenario-10's workflow (policy-free
/// fixture, 3 features dropped for empty geometry) with `errorPolicy: {
/// treatAllAsFatal: true, onFatal: continue }` -- no per-code override at
/// all. `DispositionPolicy::resolve` applies `treat_all_as_fatal` last,
/// unconditionally promoting the ladder's already-clamped result
/// (`cesium3dtiles.empty_geometry`'s registry default, `warn_drop`) to
/// `Fatal`. Unlike the override-based test above, every one of the 3
/// `report()` calls independently resolves `Fatal` for the *same* reason
/// (the blanket policy, not a per-code override), but the behavior at the
/// sink drain end is identical: the run completes (`Ok`) under `onFatal:
/// continue`, with the writer recorded as the sole failed node.
#[test]
fn treat_all_as_fatal_promotes_warn_drop_to_fatal_end_to_end() {
    use reearth_flow_diagnostics::Disposition;
    use reearth_flow_types::{ErrorPolicy, OnFatal};

    let mut p = prepare_run("10_warn_drop_aggregation");
    p.workflow.error_policy = Some(ErrorPolicy {
        treat_all_as_fatal: true,
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
    .expect("treatAllAsFatal + onFatal: continue must still yield Ok(summary)");

    assert_eq!(summary.failed_nodes.len(), 1);
    let failed = &summary.failed_nodes[0];
    assert_eq!(failed.node_id.as_deref(), Some(SCENARIO_10_WRITER_NODE_ID));
    assert_eq!(failed.effective_disposition, Some(Disposition::Fatal));
}

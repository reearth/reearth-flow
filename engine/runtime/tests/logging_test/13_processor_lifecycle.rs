//! Phase 3 Task 4 (engine lifecycle) coverage:
//!  - a processor whose `initialize()` fails emits exactly one
//!    `NodeStatusChanged{Failed}` (previously: no status event at all, so
//!    the node appeared permanently stuck at `Starting` — `processor_node.rs`
//!    used to propagate the `initialize()` error via a bare `?`).
//!  - a processor that completes successfully emits `Event::ProcessorFinished`
//!    exactly once, alongside `NodeStatusChanged{Completed}`, and that
//!    terminal status event carries `NodeMetrics` with the node's actual
//!    `features_processed` / `finish_feature_count` counts.
//!
//! Uses `Runner::run_with_event_handler` directly (the same harness
//! `11_run_summary_threading.rs` uses for its D8 inline-yaml scenario)
//! rather than the log-file golden harness in `logging_helper.rs`: these
//! tests assert on captured `Event`s, not on action-log/user-facing-log
//! content, and need a custom action factory (`FailInitProcessor`) that no
//! real action provides — no shipped `Processor` impl overrides the
//! trait's default `initialize() -> Ok(())`, so this bug path is otherwise
//! unreachable from any real workflow.

#[allow(dead_code)]
mod logging_helper;

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex, Once},
};

use logging_helper::BUILTIN_ACTION_FACTORIES;
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::runner::Runner;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::{Event, EventHandler, EventHub, NodeMetrics},
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{NodeKind, NodeStatus, Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;
use serde_json::Value;

static INIT: Once = Once::new();
fn init_test_env() {
    INIT.call_once(|| {
        std::env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
    });
}

// ---------------------------------------------------------------------------
// FailInitProcessor: a minimal test-only Processor whose `initialize()`
// always fails. No shipped action does this (every real `Processor` impl
// relies on the trait's default `Ok(())`), so this is the only way to
// exercise `processor_node.rs`'s initialize-failure path end to end.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FailInitProcessor;

impl Processor for FailInitProcessor {
    fn initialize(&mut self, _ctx: NodeContext) -> Result<(), BoxedError> {
        Err("synthetic initialize failure for Task 4 test coverage".into())
    }

    fn process(
        &mut self,
        _ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "FailInitProcessor"
    }
}

#[derive(Debug, Clone)]
struct FailInitProcessorFactory;

impl ProcessorFactory for FailInitProcessorFactory {
    fn name(&self) -> &str {
        "FailInitProcessor"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(FailInitProcessor))
    }
}

fn factories_with_fail_init_processor() -> HashMap<String, NodeKind> {
    let mut factories = BUILTIN_ACTION_FACTORIES.clone();
    factories.insert(
        "FailInitProcessor".to_string(),
        NodeKind::Processor(Box::new(FailInitProcessorFactory)),
    );
    factories
}

// ---------------------------------------------------------------------------
// Event capture
// ---------------------------------------------------------------------------

#[derive(Default)]
struct NodeStatusRecorder {
    statuses: Mutex<Vec<(String, NodeStatus, Option<NodeMetrics>)>>,
    processor_finished: Mutex<Vec<String>>,
}

impl NodeStatusRecorder {
    fn statuses_for(&self, node_id: &str) -> Vec<(NodeStatus, Option<NodeMetrics>)> {
        self.statuses
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _, _)| id == node_id)
            .map(|(_, status, metrics)| (status.clone(), *metrics))
            .collect()
    }

    fn processor_finished_count_for(&self, node_id: &str) -> usize {
        self.processor_finished
            .lock()
            .unwrap()
            .iter()
            .filter(|id| id.as_str() == node_id)
            .count()
    }
}

#[async_trait::async_trait]
impl EventHandler for NodeStatusRecorder {
    async fn on_event(&self, event: &Event) {
        match event {
            Event::NodeStatusChanged {
                node_handle,
                status,
                metrics,
                ..
            } => {
                self.statuses.lock().unwrap().push((
                    node_handle.id.to_string(),
                    status.clone(),
                    *metrics,
                ));
            }
            Event::ProcessorFinished { node, .. } => {
                self.processor_finished
                    .lock()
                    .unwrap()
                    .push(node.id.to_string());
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Minimal run harness (mirrors `11_run_summary_threading.rs`'s
// `prepare_run_from_yaml` + `run_with_event_handler`, but parameterized on a
// custom action-factory map so `FailInitProcessor` can be registered).
// ---------------------------------------------------------------------------

fn run_workflow_yaml(
    workflow_yaml: &str,
    factories: HashMap<String, NodeKind>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
) -> Result<reearth_flow_diagnostics::RunSummary, reearth_flow_runner::errors::Error> {
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

    Runner::run_with_event_handler(
        uuid::Uuid::new_v4(),
        workflow,
        factories,
        logger_factory,
        storage_resolver,
        ingress_state,
        feature_state,
        None,
        event_handlers,
        sandbox_root,
    )
}

/// source (Feature Creator, `feature_count` features) -> `middle_action` ->
/// sink (JSON Writer), the same shape `logging/06_processor_error`'s fixture
/// uses, generated with fresh UUIDs per call so parallel `#[test]` runs in
/// this binary never collide on node ids. `middle_action` gets no `with`
/// params (fine for `FailInitProcessor`, which ignores them) — use
/// [`source_middle_sink_workflow_with_params`] for an action needing config.
fn source_middle_sink_workflow(middle_action: &str, feature_count: usize) -> String {
    source_middle_sink_workflow_with_params(middle_action, "", feature_count)
}

/// Like [`source_middle_sink_workflow`], but `middle_with_yaml` is spliced in
/// verbatim as the middle node's `with:` block body (already indented to
/// match the surrounding YAML, e.g. `"          renameType: All\n..."`).
fn source_middle_sink_workflow_with_params(
    middle_action: &str,
    middle_with_yaml: &str,
    feature_count: usize,
) -> String {
    let workflow_id = uuid::Uuid::new_v4();
    let graph_id = uuid::Uuid::new_v4();
    let source_id = uuid::Uuid::new_v4();
    let middle_id = uuid::Uuid::new_v4();
    let sink_id = uuid::Uuid::new_v4();
    let edge1_id = uuid::Uuid::new_v4();
    let edge2_id = uuid::Uuid::new_v4();

    let features: Vec<String> = (1..=feature_count)
        .map(|i| format!(r#"{{"id": {i}, "value": "test-{i}"}}"#))
        .collect();
    let creator_value = format!("[{}]", features.join(", "));

    format!(
        r#"
id: {workflow_id}
name: "Processor Lifecycle Test"
entryGraphId: {graph_id}
with:
graphs:
  - id: {graph_id}
    name: main_graph
    nodes:
      - id: {source_id}
        name: Feature Creator
        type: action
        action: Feature Creator
        with:
          creator:
            type: flowExpr
            value: '{creator_value}'
      - id: {middle_id}
        name: MiddleAction
        type: action
        action: {middle_action}
        with:
{middle_with_yaml}
      - id: {sink_id}
        name: JSON Writer
        type: action
        action: JSON Writer
        with:
          output:
            type: string
            value: result.json
    edges:
      - id: {edge1_id}
        from: {source_id}
        to: {middle_id}
        fromPort: features
        toPort: features
      - id: {edge2_id}
        from: {middle_id}
        to: {sink_id}
        fromPort: features
        toPort: features
"#
    )
}

// ---------------------------------------------------------------------------
// Part 1: processor initialize() failure status hole
// ---------------------------------------------------------------------------

#[test]
fn processor_initialize_failure_emits_exactly_one_failed_status() {
    let workflow_yaml = source_middle_sink_workflow("FailInitProcessor", 1);
    let recorder = Arc::new(NodeStatusRecorder::default());
    let handlers: Vec<Arc<dyn EventHandler>> = vec![recorder.clone()];

    // The middle node's initialize() always fails, so the run as a whole is
    // expected to fail — only the per-node status sequence is under test.
    let _ = run_workflow_yaml(
        &workflow_yaml,
        factories_with_fail_init_processor(),
        handlers,
    );

    // Find the FailInitProcessor node's id from the captured statuses: it's
    // whichever node reached `Starting` but never `Processing` (the source
    // and sink both reach further states in this fixture).
    let all_statuses = recorder.statuses.lock().unwrap().clone();
    let mut by_node: HashMap<String, Vec<NodeStatus>> = HashMap::new();
    for (id, status, _) in &all_statuses {
        by_node.entry(id.clone()).or_default().push(status.clone());
    }

    let (failing_node, statuses) = by_node
        .iter()
        .find(|(_, statuses)| {
            statuses.contains(&NodeStatus::Starting) && !statuses.contains(&NodeStatus::Processing)
        })
        .unwrap_or_else(|| {
            panic!(
                "expected exactly one node to reach Starting but never Processing (the \
                     FailInitProcessor node); captured statuses by node: {by_node:?}"
            )
        });

    assert_eq!(
        statuses,
        &vec![NodeStatus::Starting, NodeStatus::Failed],
        "node {failing_node} (FailInitProcessor) must transition Starting -> Failed only — no \
         Processing, no Completed, and no repeated Failed — got {statuses:?}"
    );

    let failed_count = statuses
        .iter()
        .filter(|s| **s == NodeStatus::Failed)
        .count();
    assert_eq!(
        failed_count, 1,
        "expected exactly one NodeStatusChanged{{Failed}} for the failing node, got {failed_count}"
    );

    // The terminal Failed event's metrics field carries no counters — this
    // node never reached process()/finish().
    let terminal_metrics = recorder.statuses_for(failing_node).last().unwrap().1;
    assert!(
        terminal_metrics.is_none() || terminal_metrics == Some(NodeMetrics::default()),
        "an init-failed node has nothing to report; got {terminal_metrics:?}"
    );
}

// ---------------------------------------------------------------------------
// Part 2 + Part 4: ProcessorFinished cardinality + terminal NodeMetrics
// ---------------------------------------------------------------------------

#[test]
fn processor_success_emits_processor_finished_once_with_metrics() {
    const FEATURE_COUNT: usize = 3;
    // "Bulk Attribute Renamer" is a plain per-feature processor (no
    // accumulation), so its finish() never sends anything downstream itself
    // — `finish_feature_count` should read back 0.
    let middle_with_yaml = "          renameType: All\n          renameAction: AddPrefix\n          renameValue: \"test_\"";
    let workflow_yaml = source_middle_sink_workflow_with_params(
        "Bulk Attribute Renamer",
        middle_with_yaml,
        FEATURE_COUNT,
    );
    let recorder = Arc::new(NodeStatusRecorder::default());
    let handlers: Vec<Arc<dyn EventHandler>> = vec![recorder.clone()];

    let summary = run_workflow_yaml(&workflow_yaml, BUILTIN_ACTION_FACTORIES.clone(), handlers)
        .expect("a plain rename-and-write workflow is expected to succeed");
    assert!(summary.failed_nodes.is_empty());

    // Identify the processor node: the one with a Processing status (unlike
    // the init-failure test, this fixture always reaches it) that is neither
    // the source (never reaches Processing/Completed via NodeStatusChanged
    // status-wise the same way) nor emits `features_written` in its metrics.
    let all_statuses = recorder.statuses.lock().unwrap().clone();
    let mut by_node: HashMap<String, Vec<(NodeStatus, Option<NodeMetrics>)>> = HashMap::new();
    for (id, status, metrics) in &all_statuses {
        by_node
            .entry(id.clone())
            .or_default()
            .push((status.clone(), *metrics));
    }

    let (processor_node, processor_events) = by_node
        .iter()
        .find(|(_, events)| {
            events.iter().any(|(status, metrics)| {
                *status == NodeStatus::Completed
                    && metrics.is_some_and(|m| m.features_processed == FEATURE_COUNT as u64)
            })
        })
        .unwrap_or_else(|| {
            panic!(
                "expected exactly one node whose terminal Completed metrics report \
                 features_processed == {FEATURE_COUNT}; captured: {by_node:?}"
            )
        });

    let terminal_metrics = processor_events
        .iter()
        .find(|(status, _)| *status == NodeStatus::Completed)
        .and_then(|(_, m)| *m)
        .expect("Completed status must carry Some(NodeMetrics)");
    assert_eq!(terminal_metrics.features_processed, FEATURE_COUNT as u64);
    assert_eq!(terminal_metrics.finish_feature_count, 0);
    assert_eq!(
        terminal_metrics.features_written, 0,
        "features_written is sink-only and must stay 0 on a processor's metrics"
    );

    assert_eq!(
        recorder.processor_finished_count_for(processor_node),
        1,
        "ProcessorFinished must be emitted exactly once for the successfully-completed \
         processor node — it is a per-NODE event, deliberately not 1:1 with the per-FEATURE \
         ProcessorFailed"
    );

    // The sink's own terminal metrics carry features_written, not
    // features_processed — confirms the two node kinds populate disjoint
    // NodeMetrics fields.
    let sink_terminal = by_node
        .values()
        .find_map(|events| {
            events.iter().find_map(|(status, metrics)| {
                if *status == NodeStatus::Completed {
                    metrics.filter(|m| m.features_written == FEATURE_COUNT as u64)
                } else {
                    None
                }
            })
        })
        .expect("expected the sink node's Completed metrics to report features_written == FEATURE_COUNT");
    assert_eq!(sink_terminal.features_processed, 0);
}

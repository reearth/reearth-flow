use std::{collections::HashMap, sync::Arc};

use reearth_flow_common::future::SharedFuture;
use reearth_flow_diagnostics::RunSummary;
use reearth_flow_runtime::{
    event::EventHandler,
    executor::dag_executor::DagExecutor,
    executor_operation::ExecutorOptions,
    incremental::IncrementalRunConfig,
    kvs::KvStore,
    node::{NodeKind, SYSTEM_ACTION_FACTORY_MAPPINGS},
    shutdown::ShutdownReceiver,
};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tokio::runtime::Handle;

use crate::errors::Error;

pub struct Executor;

impl Executor {
    pub async fn create_dag_executor(
        self,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        executor_options: ExecutorOptions,
    ) -> Result<DagExecutor, Error> {
        let mut factories = factories.clone();
        factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
        let executor = DagExecutor::new(
            env_vars,
            storage_resolver,
            kv_store,
            workflow.entry_graph_id,
            workflow.graphs,
            executor_options,
            factories,
            workflow.with,
        )
        .await?;
        Ok(executor)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_dag_executor(
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    runtime: Arc<Handle>,
    dag_executor: DagExecutor,
    shutdown: ShutdownReceiver,
    ingress_state: Arc<State>,
    feature_state: Arc<State>,
    incremental_run_config: Option<IncrementalRunConfig>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    executor_id: uuid::Uuid,
) -> Result<RunSummary, Error> {
    let shutdown_future = shutdown.create_shutdown_future();

    let mut join_handle = runtime.block_on(dag_executor.start(
        SharedFuture::new(Box::pin(shutdown_future)),
        runtime.clone(),
        env_vars,
        storage_resolver,
        kv_store,
        ingress_state,
        feature_state,
        incremental_run_config,
        event_handlers,
        executor_id,
    ))?;
    // `join()` collects every node thread and folds their diagnostics into a
    // `RunSummary` (Phase 2a Task 5), then forks on the workflow's compiled
    // `errorPolicy.onFatal` (Phase 2a-policy Task 4): under the default
    // `Terminate`, it still returns `Err` with the same raw `ExecutionError`
    // the old fail-fast loop would have returned when any node thread
    // failed, so every golden logging scenario stays byte-identical through
    // this call site, and a successful join can never carry a non-empty
    // `failed_nodes`. Under `Continue`, every thread's outcome — including
    // failed ones — is folded into `Ok(summary)` instead, so independent
    // branches that finished cleanly are reported as such even when a
    // sibling branch's node thread failed (spec D8).
    let mut join_result = join_handle.join().map_err(Error::ExecutionError);
    join_handle.notify();
    // Deterministic replacement for the old fixed 100ms "settle" sleep
    // (previously a defensive guess at how long in-flight async tasks /
    // output flushes might take to drain): the event subscriber's own tokio
    // task is retained on `DagExecutorJoinHandle` (Phase 2a Task 7), so we
    // await it directly. `subscribe_event`'s notify arm drains every event
    // still queued in the broadcast ring — dispatching it to handlers — and
    // runs `on_shutdown` before that task returns, so this blocks exactly
    // as long as the real drain takes, no more and no less.
    if let Some(subscriber) = join_handle.take_subscriber() {
        let _ = runtime.block_on(subscriber);
    }
    if let Ok(summary) = join_result.as_mut() {
        summary.dropped_event_count = join_handle.dropped_events();
    }
    join_result
}

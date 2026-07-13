use std::{collections::HashMap, sync::Arc, time::Duration};

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
    // `RunSummary` (Phase 2a Task 5), but — as an interim measure — still
    // returns `Err` with the same raw `ExecutionError` the old fail-fast loop
    // would have returned when any node thread failed, so every golden
    // logging scenario stays byte-identical through this call site. A
    // successful join can therefore never carry a non-empty `failed_nodes`
    // (Phase 2a Task 5/6 invariant; a later task relaxes it).
    let join_result = join_handle.join().map_err(Error::ExecutionError);
    // Settle delay between join completion and notify. The historical 1000ms
    // was a defensive value (likely waiting for in-flight async tasks / output
    // flushes to drain). 100ms is enough headroom in practice and turns the
    // 1s × N-tests overhead into a much smaller cost. A proper fix would
    // replace this with explicit async-drain logic before notify, but that's
    // a bigger refactor; this is the minimal change that recovers most of the
    // wall-clock cost without exposing the underlying race.
    std::thread::sleep(Duration::from_millis(100));
    join_handle.notify();
    join_result
}

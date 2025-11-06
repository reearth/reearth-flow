use std::{collections::HashMap, sync::Arc, time::Duration};

use reearth_flow_common::future::SharedFuture;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    event::EventHandler,
    executor::dag_executor::DagExecutor,
    executor_operation::ExecutorOptions,
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
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        executor_options: ExecutorOptions,
    ) -> Result<DagExecutor, Error> {
        let mut factories = factories.clone();
        factories.extend(SYSTEM_ACTION_FACTORY_MAPPINGS.clone());
        let executor = DagExecutor::new(
            expr_engine,
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
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    runtime: Arc<Handle>,
    dag_executor: DagExecutor,
    shutdown: ShutdownReceiver,
    ingress_state: Arc<State>,
    feature_state: Arc<State>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
) -> Result<(), Error> {
    let shutdown_future = shutdown.create_shutdown_future();

    let mut join_handle = runtime.block_on(dag_executor.start(
        SharedFuture::new(Box::pin(shutdown_future)),
        runtime.clone(),
        expr_engine,
        storage_resolver,
        kv_store,
        ingress_state,
        feature_state,
        event_handlers,
    ))?;
    let result = join_handle
        .join((*runtime).clone())
        .map_err(Error::ExecutionError);
    std::thread::sleep(Duration::from_millis(1000));
    join_handle.notify();
    result
}

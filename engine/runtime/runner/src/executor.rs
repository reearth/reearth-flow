use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use reearth_flow_common::future::SharedFuture;
use reearth_flow_runtime::{
    event::EventHandler,
    executor::dag_executor::DagExecutor,
    executor_operation::{ExecutorOptions, NodeContext},
    node::{NodeKind, RouterFactory},
    shutdown::ShutdownReceiver,
};
use reearth_flow_state::State;
use reearth_flow_types::workflow::Workflow;
use tokio::runtime::Handle;

use crate::errors::Error;

pub struct Executor;

impl Executor {
    pub async fn create_dag_executor(
        self,
        ctx: NodeContext,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        executor_options: ExecutorOptions,
    ) -> Result<DagExecutor, Error> {
        let mut factories = factories.clone();
        factories.insert(
            "Router".to_string(),
            NodeKind::Processor(Box::<RouterFactory>::default()),
        );
        let executor = DagExecutor::new(
            ctx,
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

pub fn run_dag_executor(
    ctx: NodeContext,
    runtime: &Arc<Handle>,
    dag_executor: DagExecutor,
    shutdown: ShutdownReceiver,
    state: Arc<State>,
    event_handlers: Vec<Box<dyn EventHandler>>,
) -> Result<(), Error> {
    let shutdown_future = shutdown.create_shutdown_future();

    let mut join_handle = runtime.block_on(dag_executor.start(
        SharedFuture::new(Box::pin(shutdown_future)),
        runtime.clone(),
        ctx.expr_engine.clone(),
        ctx.storage_resolver.clone(),
        ctx.logger.clone(),
        ctx.kv_store.clone(),
        state,
        event_handlers,
    ))?;
    let result = join_handle.join().map_err(Error::ExecutionError);
    thread::sleep(Duration::from_millis(1000));
    join_handle.notify();
    result
}

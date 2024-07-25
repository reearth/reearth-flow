use std::{collections::HashMap, sync::Arc};

use reearth_flow_common::future::SharedFuture;
use reearth_flow_runtime::{
    executor::dag_executor::DagExecutor,
    executor_operation::{ExecutorOptions, NodeContext},
    node::{NodeKind, RouterFactory},
    shutdown::ShutdownReceiver,
};
use reearth_flow_state::State;
use reearth_flow_types::workflow::Workflow;
use tokio::runtime::Runtime;

use crate::errors::OrchestrationError;

pub struct Executor;

impl Executor {
    pub async fn create_dag_executor(
        self,
        ctx: NodeContext,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        executor_options: ExecutorOptions,
    ) -> Result<DagExecutor, OrchestrationError> {
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
    runtime: &Arc<Runtime>,
    dag_executor: DagExecutor,
    shutdown: ShutdownReceiver,
    state: Arc<State>,
) -> Result<(), OrchestrationError> {
    let join_handle = runtime.block_on(dag_executor.start(
        SharedFuture::new(Box::pin(shutdown.create_shutdown_future())),
        runtime.clone(),
        ctx.expr_engine.clone(),
        ctx.storage_resolver.clone(),
        ctx.logger.clone(),
        ctx.kv_store.clone(),
        state,
    ));
    join_handle
        .unwrap()
        .join()
        .map_err(OrchestrationError::ExecutionError)
}

use once_cell::sync::Lazy;
use reearth_flow_action_processor::mapping::ACTION_MAPPINGS as PROCESSOR_MAPPINGS;
use reearth_flow_action_sink::mapping::ACTION_MAPPINGS as SINK_MAPPINGS;
use reearth_flow_action_source::mapping::ACTION_MAPPINGS as SOURCE_MAPPINGS;
use reearth_flow_runtime::{
    executor::dag_executor::DagExecutor,
    executor_operation::{ExecutorOptions, NodeContext},
    node::{NodeKind, RouterFactory},
    shutdown::ShutdownReceiver,
};
use reearth_flow_types::workflow::Workflow;
use std::{collections::HashMap, sync::Arc};
use tokio::runtime::Runtime;

use crate::errors::OrchestrationError;

pub static ACTION_MAPPINGS: Lazy<HashMap<String, NodeKind>> = Lazy::new(|| {
    let mut common = HashMap::from([(
        "Router".to_string(),
        NodeKind::Processor(Box::<RouterFactory>::default()),
    )]);
    let sink = SINK_MAPPINGS.clone();
    let source = SOURCE_MAPPINGS.clone();
    let processor = PROCESSOR_MAPPINGS.clone();
    common.extend(sink);
    common.extend(source);
    common.extend(processor);
    common
});

pub struct Executor;

impl Executor {
    pub async fn create_dag_executor(
        self,
        ctx: NodeContext,
        workflow: Workflow,
        executor_options: ExecutorOptions,
    ) -> Result<DagExecutor, OrchestrationError> {
        let executor = DagExecutor::new(
            ctx,
            workflow.entry_graph_id,
            workflow.graphs,
            executor_options,
            ACTION_MAPPINGS.clone(),
            workflow.with,
        )
        .await
        .unwrap();
        Ok(executor)
    }
}

pub fn run_dag_executor(
    ctx: NodeContext,
    runtime: &Arc<Runtime>,
    dag_executor: DagExecutor,
    shutdown: ShutdownReceiver,
) -> Result<(), OrchestrationError> {
    let join_handle = runtime.block_on(dag_executor.start(
        Box::pin(shutdown.create_shutdown_future()),
        runtime.clone(),
        ctx.expr_engine.clone(),
        ctx.storage_resolver.clone(),
        ctx.logger.clone(),
        ctx.kv_store.clone(),
    ));
    join_handle
        .unwrap()
        .join()
        .map_err(OrchestrationError::ExecutionError)
}

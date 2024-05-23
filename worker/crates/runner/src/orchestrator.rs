use std::sync::Arc;

use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::executor_operation::{ExecutorOptions, NodeContext};
use reearth_flow_runtime::kvs::create_kv_store;
use reearth_flow_runtime::shutdown::ShutdownReceiver;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use crate::errors::OrchestrationError;
use crate::executor::{run_dag_executor, Executor};

#[derive(Clone)]
pub struct Orchestrator {
    pub runtime: Arc<Runtime>,
}

impl Orchestrator {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self { runtime }
    }

    pub async fn run_apps(
        &self,
        _job_id: String,
        workflow: Workflow,
        shutdown: ShutdownReceiver,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
    ) -> Result<(), OrchestrationError> {
        let executor = Executor {};
        let options = ExecutorOptions {
            channel_buffer_sz: 10,
            event_hub_capacity: 10,
            error_threshold: None,
            thread_pool_size: 30,
        };
        let expr_engine = Engine::new();
        if let Some(with) = &workflow.with {
            expr_engine.append(with);
        }
        let ctx = NodeContext {
            expr_engine: Arc::new(expr_engine),
            storage_resolver: storage_resolver.clone(),
            logger: logger_factory.clone(),
            kv_store: Arc::new(create_kv_store()),
        };
        let dag_executor = executor
            .create_dag_executor(ctx.clone(), workflow, options)
            .await?;
        let runtime_clone = self.runtime.clone();
        let shutdown_clone = shutdown.clone();
        let pipeline_future = self.runtime.spawn_blocking(move || {
            run_dag_executor(ctx.clone(), &runtime_clone, dag_executor, shutdown_clone)
        });

        let mut futures = FuturesUnordered::new();
        futures.push(flatten_join_handle(pipeline_future).boxed());

        while let Some(result) = futures.next().await {
            result?;
        }
        Ok(())
    }

    pub async fn run_all(
        &self,
        job_id: String,
        workflow: Workflow,
        shutdown: ShutdownReceiver,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
    ) -> Result<(), OrchestrationError> {
        let pipeline_shutdown = shutdown.clone();
        self.run_apps(
            job_id,
            workflow,
            pipeline_shutdown,
            logger_factory,
            storage_resolver,
        )
        .await
    }
}

async fn flatten_join_handle(
    handle: JoinHandle<Result<(), OrchestrationError>>,
) -> Result<(), OrchestrationError> {
    match handle.await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(OrchestrationError::JoinError(err)),
    }
}

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use once_cell::sync::Lazy;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::event::EventHandler;
use reearth_flow_runtime::executor_operation::ExecutorOptions;
use reearth_flow_runtime::kvs::create_kv_store;
use reearth_flow_runtime::node::NodeKind;
use reearth_flow_runtime::shutdown::ShutdownReceiver;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::errors::Error;
use crate::executor::{run_dag_executor, Executor};

static CHANNEL_BUFFER_SIZE: Lazy<usize> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_CHANNEL_BUFFER_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(256)
});

static EVENT_HUB_CAPACITY: Lazy<usize> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_EVENT_HUB_CAPACITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192)
});

static THREAD_POOL_SIZE: Lazy<usize> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_THREAD_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
});

static FEATURE_FLUSH_THRESHOLD: Lazy<usize> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(512)
});

#[derive(Clone)]
pub struct Orchestrator {
    pub runtime: Arc<Handle>,
}

impl Orchestrator {
    pub fn new(runtime: Arc<Handle>) -> Self {
        Self { runtime }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_apps(
        &self,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        shutdown: ShutdownReceiver,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<(), Error> {
        let executor = Executor {};
        let options = ExecutorOptions {
            channel_buffer_sz: *CHANNEL_BUFFER_SIZE,
            event_hub_capacity: *EVENT_HUB_CAPACITY,
            thread_pool_size: *THREAD_POOL_SIZE,
            feature_flush_threshold: *FEATURE_FLUSH_THRESHOLD,
        };
        let expr_engine = Engine::new();
        if let Some(with) = &workflow.with {
            expr_engine.append(with);
        }
        let expr_engine = Arc::new(expr_engine);
        let kv_store = Arc::new(create_kv_store());

        let dag_executor = executor
            .create_dag_executor(
                expr_engine.clone(),
                storage_resolver.clone(),
                kv_store.clone(),
                workflow,
                factories,
                options,
            )
            .await?;

        let runtime_clone = self.runtime.clone();
        let pipeline_future = self.runtime.spawn_blocking(move || {
            run_dag_executor(
                expr_engine,
                storage_resolver,
                kv_store,
                runtime_clone,
                dag_executor,
                shutdown,
                ingress_state,
                feature_state,
                event_handlers,
            )
        });

        let mut futures = FuturesUnordered::new();
        futures.push(flatten_join_handle(pipeline_future).boxed());
        while let Some(result) = futures.next().await {
            result?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_all(
        &self,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        shutdown: ShutdownReceiver,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<(), Error> {
        let pipeline_shutdown = shutdown.clone();
        self.run_apps(
            workflow,
            factories,
            pipeline_shutdown,
            storage_resolver,
            ingress_state,
            feature_state,
            event_handlers,
        )
        .await
    }
}

async fn flatten_join_handle<T>(handle: JoinHandle<Result<T, Error>>) -> Result<T, Error> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(Error::JoinError(e)),
    }
}

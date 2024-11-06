use std::{collections::HashMap, env, sync::Arc, time::Instant};

use once_cell::sync::Lazy;
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_runtime::{event::EventHandler, node::NodeKind, shutdown};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tracing::{error, info, info_span};

use crate::orchestrator::Orchestrator;

/// Controls the number of worker threads in the Tokio runtime.
///
/// # Environment Variable
/// - FLOW_RUNTIME_WORKER_NUM: Number of worker threads (default: 30)
///
/// # Notes
static WORKER_NUM: Lazy<usize> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_WORKER_NUM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
});

pub struct Runner;

impl Runner {
    pub fn run(
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        state: Arc<State>,
    ) -> Result<(), crate::errors::Error> {
        Self::run_with_event_handler(
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            state,
            vec![],
        )
    }

    pub fn run_with_event_handler(
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<(), crate::errors::Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(*WORKER_NUM)
            .enable_all()
            .build()
            .map_err(|e| {
                crate::errors::Error::RuntimeError(format!(
                    "Failed to init tokio runtime with {:?}",
                    e
                ))
            })?;

        let start = Instant::now();
        let span = info_span!(
            "root",
            "otel.name" = workflow.name.as_str(),
            "otel.kind" = "runner",
            "workflow.id" = workflow.id.to_string().as_str(),
        );
        let workflow_name = workflow.name.clone();
        info!(parent: &span, "Start workflow = {:?}", workflow_name.as_str());
        let handle = runtime.handle().clone();
        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&handle);
        let handle = Arc::new(handle);
        let orchestrator = Orchestrator::new(handle.clone());
        let result = runtime.block_on(async move {
            orchestrator
                .run_all(
                    workflow,
                    factories,
                    shutdown_receiver,
                    logger_factory,
                    storage_resolver,
                    state,
                    event_handlers,
                )
                .await
        });

        if let Err(e) = &result {
            error!(parent: &span, "Failed to workflow: {:?}", e);
        }
        info!(parent: &span, "Finish workflow = {:?}, duration = {:?}", workflow_name.as_str(), start.elapsed());
        result
    }
}

pub struct AsyncRunner;

impl AsyncRunner {
    pub async fn run(
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        state: Arc<State>,
    ) -> Result<(), crate::errors::Error> {
        Self::run_with_event_handler(
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            state,
            vec![],
        )
        .await
    }

    pub async fn run_with_event_handler(
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<(), crate::errors::Error> {
        let start = Instant::now();
        let span = info_span!(
            "root",
            "otel.name" = workflow.name.as_str(),
            "otel.kind" = "runner",
            "workflow.id" = workflow.id.to_string().as_str(),
        );
        let workflow_name = workflow.name.clone();
        info!(parent: &span, "Start workflow = {:?}", workflow_name.as_str());
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|e| crate::errors::Error::RuntimeError(format!("{:?}", e)))?;
        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&runtime);
        let orchestrator = Orchestrator::new(Arc::new(runtime));
        let result = orchestrator
            .run_all(
                workflow,
                factories,
                shutdown_receiver,
                logger_factory,
                storage_resolver,
                state,
                event_handlers,
            )
            .await;
        if let Err(e) = &result {
            error!("Failed to workflow: {:?}", e);
        }
        info!(parent: &span, "Finish workflow = {:?}, duration = {:?}", workflow_name.as_str(), start.elapsed());
        result
    }
}

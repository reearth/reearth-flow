use std::{collections::HashMap, env, sync::Arc, time::Instant};

use once_cell::sync::Lazy;
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_runtime::{event::EventHandler, node::NodeKind, shutdown};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tracing::{error, info, info_span};

#[cfg(feature = "analyzer")]
use crate::analyzer_handler::AnalyzerEventHandler;
use crate::{log_event_handler::LogEventHandler, orchestrator::Orchestrator};

#[cfg(feature = "analyzer")]
use reearth_flow_runtime::analyzer::{
    create_analyzer, default_reports_dir, generate_report_filename, AnalyzerSinkBuilder,
    DEFAULT_CHANNEL_CAPACITY,
};

/// Controls the number of worker threads in the Tokio runtime.
///
/// # Environment Variable
/// - FLOW_RUNTIME_ASYNC_WORKER_NUM: Number of worker threads (default: number of CPUs)
///
/// # Notes
static ASYNC_WORKER_NUM: Lazy<usize> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_ASYNC_WORKER_NUM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(num_cpus::get())
});

pub struct Runner;

impl Runner {
    pub fn run(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
    ) -> Result<(), crate::errors::Error> {
        Self::run_with_event_handler(
            job_id,
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
            vec![],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_with_event_handler(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<(), crate::errors::Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(*ASYNC_WORKER_NUM)
            .enable_all()
            .build()
            .map_err(|e| {
                crate::errors::Error::RuntimeError(format!(
                    "Failed to init tokio runtime with {e:?}"
                ))
            })?;

        let start = Instant::now();
        let version = env!("CARGO_PKG_VERSION");
        let span = info_span!(
            "root",
            "engine.version" = version,
            "otel.name" = workflow.name.as_str(),
            "otel.kind" = "runner",
            "workflow.id" = workflow.id.to_string().as_str(),
            "job.id" = job_id.to_string().as_str(),
        );
        let workflow_name = workflow.name.clone();
        let workflow_id = workflow.id;
        info!(parent: &span, "Start workflow = {:?}", workflow_name.as_str());
        let handle = runtime.handle().clone();
        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&handle);
        let handle = Arc::new(handle);
        let orchestrator = Orchestrator::new(handle.clone());
        let mut handlers: Vec<Arc<dyn EventHandler>> = vec![Arc::new(LogEventHandler::new(
            workflow.id,
            job_id,
            logger_factory.clone(),
        ))];

        // Set up analyzer if feature is enabled
        #[cfg(feature = "analyzer")]
        let (analyzer_sink_handle, analyzer_context) = {
            let (context, receiver, shutdown) = create_analyzer(DEFAULT_CHANNEL_CAPACITY);
            let analyzer_handler = AnalyzerEventHandler::new(
                context.sender.clone(),
                workflow_id,
                workflow_name.clone(),
            );
            handlers.push(Arc::new(analyzer_handler));

            // Clone context for sending workflow end event later
            let ctx_for_end = context.clone();

            // Spawn the analyzer sink
            let reports_dir = default_reports_dir();
            let report_path = reports_dir.join(generate_report_filename());
            info!(parent: &span, "Analyzer enabled, report will be saved to: {:?}", report_path);

            let sink_builder = AnalyzerSinkBuilder::new(receiver, shutdown.clone())
                .with_output_path(report_path.clone());

            let sink_handle = handle.spawn(async move {
                match sink_builder.run().await {
                    Ok(report) => {
                        tracing::info!(
                            "Analyzer report saved: {} nodes, {} edges",
                            report.memory_reports.len(),
                            report.edge_reports.len()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to save analyzer report: {:?}", e);
                    }
                }
            });

            (Some((sink_handle, shutdown)), Some(ctx_for_end))
        };

        #[cfg(not(feature = "analyzer"))]
        let (analyzer_sink_handle, analyzer_context): (
            Option<(
                tokio::task::JoinHandle<()>,
                std::sync::Arc<tokio::sync::Notify>,
            )>,
            Option<()>,
        ) = (None, None);

        handlers.extend(event_handlers);

        let result = runtime.block_on(async move {
            orchestrator
                .run_all(
                    workflow,
                    factories,
                    shutdown_receiver,
                    storage_resolver,
                    ingress_state,
                    feature_state,
                    handlers,
                )
                .await
        });

        // Send workflow end event based on result
        #[cfg(feature = "analyzer")]
        if let Some(ctx) = analyzer_context {
            let success = result.is_ok();
            tracing::info!(
                "Analyzer: Workflow {} finished with success={}",
                workflow_id,
                success
            );
            if let Err(ref e) = result {
                tracing::error!("Analyzer: Workflow error: {:?}", e);
            }
            ctx.workflow_end(workflow_id, success);
            // Give the event a moment to be processed before shutdown
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Shutdown analyzer sink if enabled
        if let Some((sink_handle, shutdown)) = analyzer_sink_handle {
            shutdown.notify_one();
            let _ = runtime.block_on(sink_handle);
        }

        if let Err(e) = &result {
            error!(parent: &span, "Failed to workflow: {:?}", e);
            info!(parent: &span, "Finish workflow = {:?} (failed), duration = {:?}", workflow_name.as_str(), start.elapsed());
        } else {
            info!(parent: &span, "Finish workflow = {:?} (success), duration = {:?}", workflow_name.as_str(), start.elapsed());
        }
        result
    }
}

pub struct AsyncRunner;

impl AsyncRunner {
    pub async fn run(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
    ) -> Result<(), crate::errors::Error> {
        Self::run_with_event_handler(
            job_id,
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
            vec![],
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run_with_event_handler(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<(), crate::errors::Error> {
        let start = Instant::now();
        let version = env!("CARGO_PKG_VERSION");
        let span = info_span!(
            "root",
            "engine.version" = version,
            "otel.name" = workflow.name.as_str(),
            "otel.kind" = "runner",
            "workflow.id" = workflow.id.to_string().as_str(),
            "job.id" = job_id.to_string().as_str(),
        );
        let workflow_name = workflow.name.clone();
        let workflow_id = workflow.id;
        info!(parent: &span, "Start workflow = {:?}", workflow_name.as_str());
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|e| crate::errors::Error::RuntimeError(format!("{e:?}")))?;
        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&runtime);
        let orchestrator = Orchestrator::new(Arc::new(runtime.clone()));
        let mut handlers: Vec<Arc<dyn EventHandler>> = vec![Arc::new(LogEventHandler::new(
            workflow.id,
            job_id,
            logger_factory.clone(),
        ))];

        // Set up analyzer if feature is enabled
        #[cfg(feature = "analyzer")]
        let (analyzer_sink_handle, analyzer_context) = {
            let (context, receiver, shutdown) = create_analyzer(DEFAULT_CHANNEL_CAPACITY);
            let analyzer_handler = AnalyzerEventHandler::new(
                context.sender.clone(),
                workflow_id,
                workflow_name.clone(),
            );
            handlers.push(Arc::new(analyzer_handler));

            // Clone context for sending workflow end event later
            let ctx_for_end = context.clone();

            // Spawn the analyzer sink
            let reports_dir = default_reports_dir();
            let report_path = reports_dir.join(generate_report_filename());
            info!(parent: &span, "Analyzer enabled, report will be saved to: {:?}", report_path);

            let sink_builder = AnalyzerSinkBuilder::new(receiver, shutdown.clone())
                .with_output_path(report_path.clone());

            let sink_handle = runtime.spawn(async move {
                match sink_builder.run().await {
                    Ok(report) => {
                        tracing::info!(
                            "Analyzer report saved: {} nodes, {} edges",
                            report.memory_reports.len(),
                            report.edge_reports.len()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to save analyzer report: {:?}", e);
                    }
                }
            });

            (Some((sink_handle, shutdown)), Some(ctx_for_end))
        };

        #[cfg(not(feature = "analyzer"))]
        let (analyzer_sink_handle, analyzer_context): (
            Option<(
                tokio::task::JoinHandle<()>,
                std::sync::Arc<tokio::sync::Notify>,
            )>,
            Option<()>,
        ) = (None, None);

        handlers.extend(event_handlers);
        let result = orchestrator
            .run_all(
                workflow,
                factories,
                shutdown_receiver,
                storage_resolver,
                ingress_state,
                feature_state,
                handlers,
            )
            .await;

        // Send workflow end event based on result
        #[cfg(feature = "analyzer")]
        if let Some(ctx) = analyzer_context {
            let success = result.is_ok();
            tracing::info!(
                "Analyzer: Workflow {} finished with success={}",
                workflow_id,
                success
            );
            if let Err(ref e) = result {
                tracing::error!("Analyzer: Workflow error: {:?}", e);
            }
            ctx.workflow_end(workflow_id, success);
        }

        // Shutdown analyzer sink if enabled
        if let Some((sink_handle, shutdown)) = analyzer_sink_handle {
            shutdown.notify_one();
            let _ = sink_handle.await;
        }

        if let Err(e) = &result {
            error!("Failed to workflow: {:?}", e);
            info!(parent: &span, "Finish workflow = {:?} (failed), duration = {:?}", workflow_name.as_str(), start.elapsed());
        } else {
            info!(parent: &span, "Finish workflow = {:?} (success), duration = {:?}", workflow_name.as_str(), start.elapsed());
        }
        result
    }
}

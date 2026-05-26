use std::{collections::HashMap, env, str::FromStr, sync::Arc, time::Instant};

use once_cell::sync::Lazy;
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    event::EventHandler, incremental::IncrementalRunConfig, node::NodeKind, shutdown,
};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Workflow;
use tracing::{error, info, info_span};

use crate::{log_event_handler::LogEventHandler, orchestrator::Orchestrator};

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

/// Reject the unsandboxed sentinel (`file:///`) when supplied to a production
/// entrypoint. `Runner::run` deliberately constructs this value to opt out of
/// sandboxing for tests / legacy callers; any other path that resolves to it
/// (e.g. a misconfigured `workerArtifactPath`) would silently disable the
/// sandbox, so production wrappers must trap it.
fn reject_unsandboxed_sentinel(sandbox_root: &Uri) -> Result<(), crate::errors::Error> {
    if sandbox_root.as_str() == "file:///" {
        return Err(crate::errors::Error::UnsandboxedSentinelRejected);
    }
    Ok(())
}

pub struct Runner;

#[allow(clippy::too_many_arguments)]
impl Runner {
    /// Run a workflow without a sandboxed output path.
    ///
    /// The executor contexts will have `sandbox_root` set to `file:///`, which
    /// means sink writes are **not** sandboxed to a job-scoped directory.
    /// This is intentional for tests and legacy callers that do not supply an
    /// artifact path. Production callers (CLI, worker) should use
    /// [`Runner::run_with_sandbox_root`] instead.
    pub fn run(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        incremental_run_config: Option<IncrementalRunConfig>,
    ) -> Result<(), crate::errors::Error> {
        let sandbox_root = Uri::from_str("file:///").expect("'file:///' is always a valid URI");
        // Bypass `run_with_sandbox_root`'s sentinel guard — this entrypoint
        // intentionally requests the unsandboxed mode.
        Self::run_with_event_handler(
            job_id,
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
            incremental_run_config,
            vec![],
            sandbox_root,
        )
    }

    /// Run a workflow with a sandboxed output path.
    ///
    /// `sandbox_root` is threaded into every executor context as `sandbox_root`,
    /// so that sink writes are scoped to the supplied directory. Production
    /// callers (CLI, worker) should use this method and pass the resolved
    /// `workerArtifactPath` value.
    #[allow(clippy::too_many_arguments)]
    pub fn run_with_sandbox_root(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        incremental_run_config: Option<IncrementalRunConfig>,
        sandbox_root: Uri,
    ) -> Result<(), crate::errors::Error> {
        reject_unsandboxed_sentinel(&sandbox_root)?;
        Self::run_with_event_handler(
            job_id,
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
            incremental_run_config,
            vec![],
            sandbox_root,
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
        incremental_run_config: Option<IncrementalRunConfig>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
        sandbox_root: Uri,
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
                    incremental_run_config,
                    handlers,
                    sandbox_root,
                )
                .await
        });

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

#[allow(clippy::too_many_arguments)]
impl AsyncRunner {
    /// Run a workflow without a sandboxed output path.
    ///
    /// The executor contexts will have `sandbox_root` set to `file:///`, which
    /// means sink writes are **not** sandboxed to a job-scoped directory.
    /// This is intentional for tests and legacy callers that do not supply an
    /// artifact path. Production callers should use
    /// [`AsyncRunner::run_with_sandbox_root`] instead.
    pub async fn run(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        incremental_run_config: Option<IncrementalRunConfig>,
    ) -> Result<(), crate::errors::Error> {
        let sandbox_root = Uri::from_str("file:///").expect("'file:///' is always a valid URI");
        Self::run_with_event_handler(
            job_id,
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
            incremental_run_config,
            vec![],
            sandbox_root,
        )
        .await
    }

    /// Run a workflow with a sandboxed output path.
    ///
    /// `sandbox_root` is threaded into every executor context as `sandbox_root`,
    /// so that sink writes are scoped to the supplied directory. Production
    /// callers should use this method and pass the resolved
    /// `workerArtifactPath` value.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_with_sandbox_root(
        job_id: uuid::Uuid,
        workflow: Workflow,
        factories: HashMap<String, NodeKind>,
        logger_factory: Arc<LoggerFactory>,
        storage_resolver: Arc<StorageResolver>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        incremental_run_config: Option<IncrementalRunConfig>,
        sandbox_root: Uri,
    ) -> Result<(), crate::errors::Error> {
        reject_unsandboxed_sentinel(&sandbox_root)?;
        Self::run_with_event_handler(
            job_id,
            workflow,
            factories,
            logger_factory,
            storage_resolver,
            ingress_state,
            feature_state,
            incremental_run_config,
            vec![],
            sandbox_root,
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
        incremental_run_config: Option<IncrementalRunConfig>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
        sandbox_root: Uri,
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
        info!(parent: &span, "Start workflow = {:?}", workflow_name.as_str());
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|e| crate::errors::Error::RuntimeError(format!("{e:?}")))?;
        let (_shutdown_sender, shutdown_receiver) = shutdown::new(&runtime);
        let orchestrator = Orchestrator::new(Arc::new(runtime));
        let mut handlers: Vec<Arc<dyn EventHandler>> = vec![Arc::new(LogEventHandler::new(
            workflow.id,
            job_id,
            logger_factory.clone(),
        ))];
        handlers.extend(event_handlers);
        let result = orchestrator
            .run_all(
                workflow,
                factories,
                shutdown_receiver,
                storage_resolver,
                ingress_state,
                feature_state,
                incremental_run_config,
                handlers,
                sandbox_root,
            )
            .await;
        if let Err(e) = &result {
            error!("Failed to workflow: {:?}", e);
            info!(parent: &span, "Finish workflow = {:?} (failed), duration = {:?}", workflow_name.as_str(), start.elapsed());
        } else {
            info!(parent: &span, "Finish workflow = {:?} (success), duration = {:?}", workflow_name.as_str(), start.elapsed());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_unsandboxed_sentinel_rejects_file_root() {
        let uri = Uri::from_str("file:///").unwrap();
        assert!(matches!(
            reject_unsandboxed_sentinel(&uri),
            Err(crate::errors::Error::UnsandboxedSentinelRejected)
        ));
    }

    #[test]
    fn reject_unsandboxed_sentinel_accepts_real_paths() {
        let uri = Uri::from_str("file:///tmp/job").unwrap();
        assert!(reject_unsandboxed_sentinel(&uri).is_ok());
        let uri = Uri::from_str("gs://bucket/job").unwrap();
        assert!(reject_unsandboxed_sentinel(&uri).is_ok());
    }
}

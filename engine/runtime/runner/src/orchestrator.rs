use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{DispositionPolicy, RunSummary};
use reearth_flow_runtime::event::EventHandler;
use reearth_flow_runtime::executor_operation::ExecutorOptions;
use reearth_flow_runtime::incremental::IncrementalRunConfig;
use reearth_flow_runtime::kvs::create_kv_store;
use reearth_flow_runtime::node::NodeKind;
use reearth_flow_runtime::shutdown::ShutdownReceiver;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::{ErrorPolicy, Workflow};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::errors::Error;
use crate::executor::{run_dag_executor, Executor};
use crate::policy::{map_error_policy, validate_node_selectors, validate_reject_routing};

/// Joins a `Vec<String>` of validation messages (`ErrorPolicy::validate`,
/// `DispositionPolicy::compile`, `validate_node_selectors`,
/// `validate_reject_routing`) into the single `Error::PolicyValidationError`
/// line, one message per line.
fn policy_validation_error(errors: Vec<String>) -> Error {
    Error::PolicyValidationError(errors.join("\n"))
}

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
        incremental_run_config: Option<IncrementalRunConfig>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
        sandbox_root: Uri,
    ) -> Result<RunSummary, Error> {
        // Compile the workflow's errorPolicy at load, before DAG
        // construction — structural validation (`ErrorPolicy::validate`)
        // then registry-aware compilation (`DispositionPolicy::compile`).
        // Every violation is surfaced at once as a run-abort error, mirroring
        // how a workflow-parse failure aborts the run upstream (the worker's
        // `Workflow::try_from` path) — this is the same "fail before doing
        // any real work" shape, just for the policy block instead of the
        // whole document.
        let error_policy: ErrorPolicy = workflow.error_policy.clone().unwrap_or_default();
        error_policy.validate().map_err(policy_validation_error)?;
        let disposition_policy = Arc::new(
            DispositionPolicy::compile(map_error_policy(&error_policy))
                .map_err(policy_validation_error)?,
        );

        let executor = Executor {};
        let options = ExecutorOptions {
            channel_buffer_sz: *CHANNEL_BUFFER_SIZE,
            event_hub_capacity: *EVENT_HUB_CAPACITY,
            thread_pool_size: *THREAD_POOL_SIZE,
            feature_flush_threshold: *FEATURE_FLUSH_THRESHOLD,
            sandbox_root,
            disposition_policy: disposition_policy.clone(),
        };
        let env_vars = Arc::new(workflow.with.clone().unwrap_or_default());
        let kv_store = Arc::new(create_kv_store());

        let dag_executor = executor
            .create_dag_executor(
                env_vars.clone(),
                storage_resolver.clone(),
                kv_store.clone(),
                workflow,
                factories,
                options,
            )
            .await?;

        // Load-time node matching (spec 4.2): every override's `node` value
        // must equal a composed id in the now-flattened DAG. Needs the built
        // DAG, so this runs after `create_dag_executor`, unlike the
        // structural/compile checks above.
        validate_node_selectors(&error_policy, &dag_executor.node_identities())
            .map_err(policy_validation_error)?;

        // Load-time Reject-routing validation (spec 4.4, Task 5): colocated
        // with the node-selector check above — same window (needs the built
        // DAG's node kinds/ports/wiring), same abort shape.
        validate_reject_routing(&disposition_policy, &dag_executor.reject_routing_info())
            .map_err(policy_validation_error)?;

        // Generate unique executor ID for cache isolation between concurrent executions
        let executor_id = uuid::Uuid::new_v4();

        let runtime_clone = self.runtime.clone();
        let pipeline_future = self.runtime.spawn_blocking(move || {
            run_dag_executor(
                env_vars,
                storage_resolver,
                kv_store,
                runtime_clone,
                dag_executor,
                shutdown,
                ingress_state,
                feature_state,
                incremental_run_config,
                event_handlers,
                executor_id,
            )
        });

        let mut futures = FuturesUnordered::new();
        futures.push(flatten_join_handle(pipeline_future).boxed());
        let mut summary: Option<RunSummary> = None;
        while let Some(result) = futures.next().await {
            summary = Some(result?);
        }

        Ok(summary.expect(
            "run_apps pushes exactly one pipeline future, so the loop above yields exactly one result",
        ))
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
        incremental_run_config: Option<IncrementalRunConfig>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
        sandbox_root: Uri,
    ) -> Result<RunSummary, Error> {
        let pipeline_shutdown = shutdown.clone();
        self.run_apps(
            workflow,
            factories,
            pipeline_shutdown,
            storage_resolver,
            ingress_state,
            feature_state,
            incremental_run_config,
            event_handlers,
            sandbox_root,
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

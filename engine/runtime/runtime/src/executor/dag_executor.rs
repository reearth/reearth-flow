use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread::{self, Builder};
use std::time::Duration;

use futures::Future;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Graph;
use tokio::runtime::Handle;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Notify;

use super::node::Node;
use super::processor_node::ProcessorNode;
use super::sink_node::SinkNode;
use crate::builder_dag::{BuilderDag, NodeKind};
use crate::dag_schemas::DagSchemas;
use crate::errors::ExecutionError;
use crate::event::{Event, EventHandler, EventHub};
use crate::executor_operation::{ExecutorOptions, NodeContext};
use crate::kvs::KvStore;

use super::execution_dag::ExecutionDag;
use super::source_node::{create_source_node, SourceNode};

/// Helper function to create a node cache for a specific node
fn create_node_cache(
    storage_resolver: &Arc<StorageResolver>,
    project_key: &str,
    job_id: uuid::Uuid,
    node_id: &str,
) -> Result<Arc<reearth_flow_state::State>, std::io::Error> {
    // Create the cache directory path: projects/<project_key>/jobs/<job_id>/node-cache/<node_id>/
    let cache_dir = reearth_flow_common::dir::get_job_root_dir_path(project_key, job_id)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get job root directory: {}", e),
            )
        })?
        .join("node-cache")
        .join(node_id);

    // Don't create the directory here - it will be created lazily when needed

    // Create the cache State using the directory
    let cache_uri =
        reearth_flow_common::uri::Uri::from_str(cache_dir.to_str().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid cache directory path",
            )
        })?)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to create URI from cache directory: {}", e),
            )
        })?;

    let cache_state =
        reearth_flow_state::State::new(&cache_uri, storage_resolver).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create cache State: {}", e),
            )
        })?;

    Ok(Arc::new(cache_state))
}

pub struct DagExecutor {
    builder_dag: BuilderDag,
    options: ExecutorOptions,
}

pub struct DagExecutorJoinHandle {
    event_hub: EventHub,
    join_handles: Vec<JoinHandle<Result<(), ExecutionError>>>,
    notify: Arc<Notify>,
}

impl DagExecutor {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        entry_graph_id: uuid::Uuid,
        graphs: Vec<Graph>,
        options: ExecutorOptions,
        factories: HashMap<String, crate::node::NodeKind>,
        global_params: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<Self, ExecutionError> {
        let dag_schemas = DagSchemas::from_graphs(entry_graph_id, graphs, factories, global_params);
        let event_hub = EventHub::new(options.event_hub_capacity);
        // Node cache is None here as we're just building the DAG structure - actual node caches
        // are created later during execution when we have job-specific context
        let ctx = NodeContext::new(expr_engine, storage_resolver, kv_store, event_hub, None);
        let builder_dag = BuilderDag::new(ctx, dag_schemas).await?;
        Ok(Self {
            builder_dag,
            options,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn start<F: Send + 'static + Future + Unpin + Debug + Clone>(
        self,
        project_key: String,
        job_id: uuid::Uuid,
        shutdown: F,
        runtime: Arc<Handle>,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn crate::kvs::KvStore>,
        state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<DagExecutorJoinHandle, ExecutionError> {
        // Construct execution dag.
        let mut execution_dag = ExecutionDag::new(
            self.builder_dag,
            self.options.channel_buffer_sz,
            self.options.feature_flush_threshold,
            Arc::clone(&state),
        )?;
        let node_indexes = execution_dag.graph().node_indices().collect::<Vec<_>>();

        let event_hub = execution_dag.event_hub().clone();

        // Start the threads.
        let source_node = create_source_node(
            project_key.clone(),
            job_id,
            Arc::clone(&expr_engine),
            Arc::clone(&storage_resolver),
            Arc::clone(&kv_store),
            &mut execution_dag,
            &self.options,
            shutdown.clone(),
            runtime.clone(),
        )
        .await;
        let mut receiver = execution_dag.event_hub().sender.subscribe();
        let notify = Arc::new(Notify::new());
        let notify_publish = Arc::clone(&notify);
        let notify_subscribe = Arc::clone(&notify);
        runtime.spawn(async move {
            subscribe_event(&mut receiver, notify_subscribe.clone(), &event_handlers).await;
        });
        let mut join_handles = vec![start_source(source_node)?];
        for node_index in node_indexes {
            let Some(node) = execution_dag.graph()[node_index].kind.as_ref() else {
                continue;
            };
            match node {
                NodeKind::Source { .. } => continue,
                NodeKind::Processor(_) => {
                    let node_handle = execution_dag.graph()[node_index].handle.clone();
                    let node_cache = create_node_cache(
                        &storage_resolver,
                        &project_key,
                        job_id,
                        node_handle.id.as_ref(),
                    )
                    .map_err(|e| ExecutionError::Io(e.to_string()))?;

                    let ctx = NodeContext::new(
                        Arc::clone(&expr_engine),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&kv_store),
                        execution_dag.event_hub().clone(),
                        Some(node_cache),
                    );
                    let processor_node = ProcessorNode::new(
                        ctx,
                        &mut execution_dag,
                        node_index,
                        shutdown.clone(),
                        runtime.clone(),
                    )
                    .await;
                    join_handles.push(start_processor(processor_node)?);
                }
                NodeKind::Sink(_) => {
                    let node_handle = execution_dag.graph()[node_index].handle.clone();
                    let node_cache = create_node_cache(
                        &storage_resolver,
                        &project_key,
                        job_id,
                        node_handle.id.as_ref(),
                    )
                    .map_err(|e| ExecutionError::Io(e.to_string()))?;

                    let ctx = NodeContext::new(
                        Arc::clone(&expr_engine),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&kv_store),
                        execution_dag.event_hub().clone(),
                        Some(node_cache),
                    );
                    let sink_node = SinkNode::new(
                        ctx,
                        &mut execution_dag,
                        node_index,
                        shutdown.clone(),
                        runtime.clone(),
                    );
                    join_handles.push(start_sink(sink_node)?);
                }
            }
        }

        Ok(DagExecutorJoinHandle {
            event_hub,
            join_handles,
            notify: notify_publish.clone(),
        })
    }
}

async fn subscribe_event(
    receiver: &mut Receiver<Event>,
    notify: Arc<Notify>,
    event_handlers: &[Arc<dyn EventHandler>],
) {
    crate::event::subscribe_event(receiver, notify, event_handlers).await;
}

impl DagExecutorJoinHandle {
    pub fn join(&mut self) -> Result<(), ExecutionError> {
        loop {
            let Some(finished) = self
                .join_handles
                .iter()
                .enumerate()
                .find_map(|(i, handle)| handle.is_finished().then_some(i))
            else {
                thread::sleep(Duration::from_millis(250));

                continue;
            };
            let handle = self.join_handles.swap_remove(finished);
            handle.join().unwrap()?;

            if self.join_handles.is_empty() {
                // All threads have completed, add a delay before returning
                if let Ok(handle) = Handle::try_current() {
                    tracing::info!(
                        "Workflow complete, waiting for final events to be published..."
                    );

                    // Simple delay approach - block for 500ms to let events publish
                    handle.block_on(self.event_hub.simple_flush(500));

                    tracing::info!("Proceeding with workflow termination");
                }

                return Ok(());
            }
        }
    }

    pub fn notify(&self) {
        self.notify.notify_waiters();
    }
}

fn start_source<F: Send + 'static + Future + Unpin + Debug>(
    source: SourceNode<F>,
) -> Result<JoinHandle<Result<(), ExecutionError>>, ExecutionError> {
    let handle = Builder::new()
        .name("sources".into())
        .spawn(move || match source.run() {
            Ok(()) => Ok(()),
            // Channel disconnection means the source listener has quit.
            // Maybe it quit gracefully so we don't need to propagate the error.
            Err(e) => {
                if let ExecutionError::Source(e) = &e {
                    if let Some(ExecutionError::CannotSendToChannel(_)) = e.downcast_ref() {
                        return Ok(());
                    }
                }
                Err(e)
            }
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)?;

    Ok(handle)
}

fn start_processor<F: Send + 'static + Future + Unpin + Debug>(
    processor: ProcessorNode<F>,
) -> Result<JoinHandle<Result<(), ExecutionError>>, ExecutionError> {
    Builder::new()
        .name(processor.handle().to_string())
        .spawn(move || {
            processor.run()?;
            Ok(())
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)
}

fn start_sink<F: Send + 'static + Future + Unpin + Debug>(
    sink: SinkNode<F>,
) -> Result<JoinHandle<Result<(), ExecutionError>>, ExecutionError> {
    Builder::new()
        .name(sink.handle().to_string())
        .spawn(|| {
            sink.run()?;
            Ok(())
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)
}

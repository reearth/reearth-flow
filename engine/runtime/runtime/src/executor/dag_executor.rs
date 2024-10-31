use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread::{self, Builder};
use std::time::Duration;

use futures::Future;
use reearth_flow_action_log::factory::LoggerFactory;
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
use crate::event::{Event, EventHandler};
use crate::executor_operation::{ExecutorOptions, NodeContext};

use super::execution_dag::ExecutionDag;
use super::source_node::{create_source_node, SourceNode};

pub struct DagExecutor {
    builder_dag: BuilderDag,
    options: ExecutorOptions,
}

pub struct DagExecutorJoinHandle {
    join_handles: Vec<JoinHandle<Result<(), ExecutionError>>>,
    notify: Arc<Notify>,
}

impl DagExecutor {
    pub async fn new(
        ctx: NodeContext,
        entry_graph_id: uuid::Uuid,
        graphs: Vec<Graph>,
        options: ExecutorOptions,
        factories: HashMap<String, crate::node::NodeKind>,
        global_params: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<Self, ExecutionError> {
        let dag_schemas = DagSchemas::from_graphs(entry_graph_id, graphs, factories, global_params);
        let builder_dag = BuilderDag::new(ctx, dag_schemas, options.event_hub_capacity).await?;

        Ok(Self {
            builder_dag,
            options,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn start<F: Send + 'static + Future + Unpin + Debug + Clone>(
        self,
        shutdown: F,
        runtime: Arc<Handle>,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: Arc<LoggerFactory>,
        kv_store: Arc<dyn crate::kvs::KvStore>,
        state: Arc<State>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<DagExecutorJoinHandle, ExecutionError> {
        // Construct execution dag.
        let mut execution_dag = ExecutionDag::new(
            self.builder_dag,
            self.options.channel_buffer_sz,
            self.options.error_threshold,
            self.options.feature_flush_threshold,
            Arc::clone(&state),
        )?;
        let node_indexes = execution_dag.graph().node_indices().collect::<Vec<_>>();

        let ctx = NodeContext::new(
            Arc::clone(&expr_engine),
            Arc::clone(&storage_resolver),
            Arc::clone(&logger),
            Arc::clone(&kv_store),
        );
        // Start the threads.
        let source_node = create_source_node(
            ctx,
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
                    let ctx = NodeContext::new(
                        Arc::clone(&expr_engine),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&logger),
                        Arc::clone(&kv_store),
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
                    let ctx = NodeContext::new(
                        Arc::clone(&expr_engine),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&logger),
                        Arc::clone(&kv_store),
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

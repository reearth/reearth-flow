use std::{
    borrow::Cow,
    fmt::Debug,
    mem::swap,
    sync::{atomic::AtomicU64, Arc},
    time,
};

use crossbeam::channel::Receiver;
use futures::Future;
use petgraph::graph::NodeIndex;
use reearth_flow_common::runtime_config::NODE_STATUS_PROPAGATION_DELAY;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::runtime::Handle;
use tracing::info_span;

use crate::{
    builder_dag::NodeKind,
    errors::ExecutionError,
    event::{Event, EventHub},
    executor_operation::{ExecutorContext, ExecutorOperation, NodeContext},
    kvs::KvStore,
    node::{NodeHandle, NodeStatus, Sink},
};

use super::receiver_loop::ReceiverLoop;
use super::{execution_dag::ExecutionDag, receiver_loop::init_select};

/// A sink in the execution DAG.
#[derive(Debug)]
pub struct SinkNode<F> {
    /// Node handle in description DAG.
    node_handle: NodeHandle,
    /// Node name from workflow definition.
    node_name: String,
    /// Input node handles.
    node_handles: Vec<NodeHandle>,
    /// Input data channels.
    receivers: Vec<Receiver<ExecutorOperation>>,
    /// The sink.
    sink: Box<dyn Sink>,
    event_hub: EventHub,
    /// The shutdown future.
    #[allow(dead_code)]
    shutdown: F,
    /// The runtime to run the source in.
    #[allow(dead_code)]
    runtime: Arc<Handle>,
    span: tracing::Span,
    features_written: Arc<AtomicU64>,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
}

impl<F: Future + Unpin + Debug> SinkNode<F> {
    pub fn new(
        ctx: NodeContext,
        dag: &mut ExecutionDag,
        node_index: NodeIndex,
        shutdown: F,
        runtime: Arc<Handle>,
    ) -> Self {
        let node = dag.node_weight_mut(node_index);
        let Some(kind) = node.kind.take() else {
            panic!("Must pass in a node")
        };
        let node_handle = node.handle.clone();
        let node_name = node.name.clone();
        let NodeKind::Sink(sink) = kind else {
            panic!("Must pass in a sink node");
        };

        let (node_handles, receivers) = dag.collect_receivers(node_index);

        let version = env!("CARGO_PKG_VERSION");
        let span = info_span!(
            "action",
            "engine.version" = version,
            "otel.name" = sink.name(),
            "otel.kind" = "Sink Node",
            "workflow.id" = dag.id.to_string().as_str(),
            "node.id" = node_handle.id.to_string().as_str(),
            "node.name" = node_name.as_str(),
        );
        Self {
            node_handle,
            node_name,
            node_handles,
            receivers,
            sink,
            event_hub: ctx.event_hub.clone(),
            shutdown,
            runtime,
            span,
            features_written: Arc::new(AtomicU64::new(0)),
            expr_engine: ctx.expr_engine.clone(),
            storage_resolver: ctx.storage_resolver.clone(),
            kv_store: ctx.kv_store.clone(),
        }
    }

    pub fn handle(&self) -> &NodeHandle {
        &self.node_handle
    }
}

impl<F: Future + Unpin + Debug> ReceiverLoop for SinkNode<F> {
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>> {
        let mut result = vec![];
        swap(&mut self.receivers, &mut result);
        result
    }

    fn receiver_name(&self, index: usize) -> Cow<str> {
        Cow::Owned(self.node_handles[index].to_string())
    }

    fn receiver_loop(mut self) -> Result<(), ExecutionError> {
        let mut has_failed = false;

        let receivers = self.receivers();
        let mut is_terminated = vec![false; receivers.len()];
        let now = time::Instant::now();
        let span = self.span.clone();
        let mut sel = init_select(&receivers);
        let mut first_error: Option<ExecutionError> = None;

        tracing::info!("Sink node {} is starting", self.node_handle.id);
        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Starting,
            feature_id: None,
        });

        self.event_hub.info_log_with_node_info(
            Some(span.clone()),
            self.node_handle.clone(),
            self.node_name.clone(),
            format!("{} sink start...", self.sink.name()),
        );

        let init_result = self
            .sink
            .initialize(NodeContext {
                expr_engine: self.expr_engine.clone(),
                kv_store: self.kv_store.clone(),
                storage_resolver: self.storage_resolver.clone(),
                event_hub: self.event_hub.clone(),
            })
            .map_err(ExecutionError::Sink);

        if let Err(ref e) = init_result {
            tracing::error!("Sink node {} initialization failed", self.node_handle.id);

            self.event_hub.error_log_with_node_info(
                Some(span.clone()),
                self.node_handle.clone(),
                self.node_name.clone(),
                format!("{} sink error: {}", self.sink.name(), e),
            );

            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: self.node_handle.clone(),
                status: NodeStatus::Failed,
                feature_id: None,
            });
            return init_result;
        }

        // Log and emit Processing status
        tracing::info!("Sink node {} is processing", self.node_handle.id);
        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Processing,
            feature_id: None,
        });

        loop {
            let index = sel.ready();
            let op = receivers[index]
                .recv()
                .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{e:?}")))?;
            match op {
                ExecutorOperation::Op { ctx } => {
                    let result = self.on_op(ctx.clone());

                    if let Err(e) = result {
                        has_failed = true;
                        tracing::warn!(
                            "Sink node {} processing failed for feature {:?}",
                            self.node_handle.id,
                            ctx.feature.id
                        );

                        self.event_hub.error_log_with_node_info(
                            Some(span.clone()),
                            self.node_handle.clone(),
                            self.node_name.clone(),
                            format!("{} sink error: {}", self.sink.name(), e),
                        );

                        if first_error.is_none() {
                            first_error = Some(e);
                        }

                        // For sink errors, we want to continue processing to emit terminate log
                        // So we don't propagate the error here
                    } else {
                        self.features_written
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                ExecutorOperation::Terminate { ctx } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                    if is_terminated.iter().all(|value| *value) {
                        let features_count = self
                            .features_written
                            .load(std::sync::atomic::Ordering::Relaxed);
                        let message = if features_count > 0 && !has_failed {
                            format!(
                                "{} sink finish. elapsed = {:?}",
                                self.sink.name(),
                                now.elapsed()
                            )
                        } else {
                            format!(
                                "{} sink terminate. elapsed = {:?}",
                                self.sink.name(),
                                now.elapsed()
                            )
                        };

                        self.event_hub.info_log_with_node_info(
                            Some(span.clone()),
                            self.node_handle.clone(),
                            self.node_name.clone(),
                            message,
                        );

                        // Set final status based on overall success/failure
                        let final_status = if has_failed {
                            NodeStatus::Failed
                        } else {
                            NodeStatus::Completed
                        };

                        self.event_hub.send(Event::NodeStatusChanged {
                            node_handle: self.node_handle.clone(),
                            status: final_status,
                            feature_id: None,
                        });

                        std::thread::sleep(*NODE_STATUS_PROPAGATION_DELAY);

                        let terminate_result = self.on_terminate(ctx);

                        if terminate_result.is_err() && !has_failed {
                            tracing::error!("Sink node {} termination failed", self.node_handle.id);
                            self.event_hub.send(Event::NodeStatusChanged {
                                node_handle: self.node_handle.clone(),
                                status: NodeStatus::Failed,
                                feature_id: None,
                            });
                        }

                        // If there was an error during processing, return that error
                        // Otherwise, return the terminate result
                        if let Some(e) = first_error {
                            return Err(e);
                        }
                        return terminate_result;
                    }
                }
            }
        }
    }

    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        self.sink
            .process(ctx)
            .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{e:?}")))
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        let result = self
            .sink
            .finish(ctx)
            .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{e:?}")));
        self.event_hub.send(Event::SinkFinished {
            node: self.node_handle.clone(),
            name: self.node_name.clone(),
        });
        result
    }
}

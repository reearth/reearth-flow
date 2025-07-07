use std::{
    borrow::Cow,
    env,
    fmt::Debug,
    mem::swap,
    sync::Arc,
    time::{self, Duration},
};

use crossbeam::channel::Receiver;
use futures::Future;
use once_cell::sync::Lazy;
use petgraph::graph::NodeIndex;
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

static NODE_STATUS_PROPAGATION_DELAY: Lazy<Duration> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(500))
});

/// A sink in the execution DAG.
#[derive(Debug)]
pub struct SinkNode<F> {
    /// Node handle in description DAG.
    node_handle: NodeHandle,
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
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    node_cache: Option<Arc<reearth_flow_state::State>>,
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
        );
        Self {
            node_handle,
            node_handles,
            receivers,
            sink,
            event_hub: ctx.event_hub.clone(),
            shutdown,
            runtime,
            span,
            expr_engine: ctx.expr_engine.clone(),
            storage_resolver: ctx.storage_resolver.clone(),
            kv_store: ctx.kv_store.clone(),
            node_cache: ctx.node_cache.clone(),
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

        tracing::info!("Sink node {} is starting", self.node_handle.id);
        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Starting,
            feature_id: None,
        });

        let init_ctx = NodeContext::new(
            self.expr_engine.clone(),
            self.storage_resolver.clone(),
            self.kv_store.clone(),
            self.event_hub.clone(),
            self.node_cache.clone(),
        );
        
        let init_result = self
            .sink
            .initialize(init_ctx)
            .map_err(ExecutionError::Sink);

        if init_result.is_err() {
            tracing::error!("Sink node {} initialization failed", self.node_handle.id);
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

        self.event_hub.info_log_with_node_handle(
            Some(span.clone()),
            self.node_handle.clone(),
            format!("{:?} sink start...", self.sink.name()),
        );

        loop {
            let index = sel.ready();
            let op = receivers[index]
                .recv()
                .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{e:?}")))?;
            match op {
                ExecutorOperation::Op { ctx } => {
                    let result = self.on_op(ctx.clone());

                    if result.is_err() {
                        // Track failure but don't emit per-feature status
                        has_failed = true;
                        tracing::warn!(
                            "Sink node {} processing failed for feature {:?}",
                            self.node_handle.id,
                            ctx.feature.id
                        );
                    }

                    // Propagate the result
                    result?;
                }
                ExecutorOperation::Terminate { ctx: _ } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                    if is_terminated.iter().all(|value| *value) {
                        self.event_hub.info_log_with_node_handle(
                            Some(span.clone()),
                            self.node_handle.clone(),
                            format!(
                                "{:?} sink finish. elapsed = {:?}",
                                self.sink.name(),
                                now.elapsed()
                            ),
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

                        let terminate_ctx = NodeContext::new(
                            self.expr_engine.clone(),
                            self.storage_resolver.clone(),
                            self.kv_store.clone(),
                            self.event_hub.clone(),
                            self.node_cache.clone(),
                        );
                        
                        let terminate_result = self.on_terminate(terminate_ctx);

                        if terminate_result.is_err() && !has_failed {
                            tracing::error!("Sink node {} termination failed", self.node_handle.id);
                            self.event_hub.send(Event::NodeStatusChanged {
                                node_handle: self.node_handle.clone(),
                                status: NodeStatus::Failed,
                                feature_id: None,
                            });
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
            name: self.sink.name().to_string(),
        });
        result
    }
}

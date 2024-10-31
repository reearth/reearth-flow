use std::{
    borrow::Cow,
    fmt::Debug,
    mem::swap,
    sync::Arc,
    time::{self},
};

use crossbeam::channel::Receiver;
use futures::Future;
use petgraph::graph::NodeIndex;
use reearth_flow_action_log::{action_log, factory::LoggerFactory, ActionLogger};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::runtime::Handle;
use tracing::info_span;

use crate::{
    builder_dag::NodeKind,
    error_manager::ErrorManager,
    errors::ExecutionError,
    event::Event,
    executor_operation::{ExecutorContext, ExecutorOperation, NodeContext},
    kvs::KvStore,
    node::{NodeHandle, Sink},
};

use super::receiver_loop::ReceiverLoop;
use super::{execution_dag::ExecutionDag, receiver_loop::init_select};

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
    event_sender: tokio::sync::broadcast::Sender<Event>,
    #[allow(dead_code)]
    error_manager: Arc<ErrorManager>,
    /// The shutdown future.
    #[allow(dead_code)]
    shutdown: F,
    /// The runtime to run the source in.
    #[allow(dead_code)]
    runtime: Arc<Handle>,
    logger: Arc<ActionLogger>,
    logger_factory: Arc<LoggerFactory>,
    span: tracing::Span,
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
        let NodeKind::Sink(sink) = kind else {
            panic!("Must pass in a sink node");
        };

        let (node_handles, receivers) = dag.collect_receivers(node_index);

        let logger = ctx
            .logger
            .clone()
            .action_logger(node_handle.id.to_string().as_str());
        let span = info_span!(
            "action",
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
            event_sender: dag.event_hub().sender.clone(),
            error_manager: dag.error_manager().clone(),
            shutdown,
            runtime,
            logger: Arc::new(logger),
            span,
            logger_factory: ctx.logger.clone(),
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
        // This is just copied from ReceiverLoop
        let receivers = self.receivers();
        let mut is_terminated = vec![false; receivers.len()];
        let now = time::Instant::now();
        let span = self.span.clone();
        let logger = self.logger.clone();
        let mut sel = init_select(&receivers);
        self.sink
            .initialize(NodeContext {
                logger: self.logger_factory.clone(),
                expr_engine: self.expr_engine.clone(),
                kv_store: self.kv_store.clone(),
                storage_resolver: self.storage_resolver.clone(),
            })
            .map_err(ExecutionError::Sink)?;
        action_log!(
            parent: span, logger, "{:?} process start...", self.sink.name(),
        );
        loop {
            let index = sel.ready();
            let op = receivers[index]
                .recv()
                .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{:?}", e)))?;
            match op {
                ExecutorOperation::Op { ctx } => {
                    self.on_op(ctx)?;
                }
                ExecutorOperation::Terminate { ctx } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                    if is_terminated.iter().all(|value| *value) {
                        action_log!(
                            parent: span, logger, "{:?} sink finish. elapsed = {:?}", self.sink.name() , now.elapsed(),
                        );
                        self.on_terminate(ctx)?;
                        return Ok(());
                    }
                }
            }
        }
    }

    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        self.sink
            .process(ctx)
            .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{:?}", e)))
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        let result = self
            .sink
            .finish(ctx)
            .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{:?}", e)));
        let _ = self.event_sender.send(Event::SinkFinished {
            node: self.node_handle.clone(),
            name: self.sink.name().to_string(),
        });
        result
    }
}

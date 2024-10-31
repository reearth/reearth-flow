use std::env;
use std::fmt::Debug;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::{self, Duration};
use std::{borrow::Cow, mem::swap};

use crossbeam::channel::Receiver;
use futures::Future;
use once_cell::sync::Lazy;
use petgraph::graph::NodeIndex;
use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_action_log::{action_error_log, action_log, slow_action_log, ActionLogger};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::runtime::Handle;
use tracing::{info_span, Span};

use crate::error_manager::ErrorManager;
use crate::event::Event;
use crate::executor_operation::{ExecutorContext, ExecutorOperation, NodeContext};
use crate::kvs::KvStore;
use crate::{
    builder_dag::NodeKind,
    errors::ExecutionError,
    forwarder::ChannelManager,
    node::{NodeHandle, Processor},
};

use super::receiver_loop::init_select;
use super::{execution_dag::ExecutionDag, receiver_loop::ReceiverLoop};

static SLOW_ACTION_THRESHOLD: Lazy<Duration> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_SLOW_ACTION_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(300))
});

/// A processor in the execution DAG.
#[derive(Debug)]
pub struct ProcessorNode<F> {
    /// Node handle in description DAG.
    node_handle: NodeHandle,
    /// Input node handles.
    node_handles: Vec<NodeHandle>,
    /// Input data channels.
    receivers: Vec<Receiver<ExecutorOperation>>,
    /// The processor.
    processor: Arc<parking_lot::RwLock<Box<dyn Processor>>>,
    /// This node's output channel manager, for forwarding data, writing metadata and writing port state.
    channel_manager: Arc<parking_lot::RwLock<ChannelManager>>,
    /// The shutdown future.
    #[allow(dead_code)]
    shutdown: F,
    /// The runtime to run the source in.
    #[allow(dead_code)]
    runtime: Arc<Handle>,
    #[allow(dead_code)]
    error_manager: Arc<ErrorManager>,
    logger_factory: Arc<LoggerFactory>,
    logger: Arc<ActionLogger>,
    slow_logger: Arc<ActionLogger>,
    span: tracing::Span,
    thread_pool: rayon::ThreadPool,
    thread_counter: Arc<AtomicU32>,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    #[allow(dead_code)]
    event_sender: tokio::sync::broadcast::Sender<Event>,
}

impl<F: Future + Unpin + Debug> ProcessorNode<F> {
    pub async fn new(
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
        let NodeKind::Processor(processor) = kind else {
            panic!("Must pass in a processor node");
        };
        let (node_handles, receivers) = dag.collect_receivers(node_index);

        let senders = dag.collect_senders(node_index);
        let record_writers = dag.collect_record_writers(node_index).await;

        let channel_manager = ChannelManager::new(
            node_handle.clone(),
            record_writers,
            senders,
            dag.error_manager().clone(),
            runtime.clone(),
            dag.event_hub().sender.clone(),
        );
        let span = info_span!(
            "action",
            "otel.name" = processor.name(),
            "otel.kind" = "Processor Node",
            "workflow.id" = dag.id.to_string().as_str(),
            "node.id" = node_handle.id.to_string().as_str(),
        );

        let logger = ctx
            .logger
            .action_logger(node_handle.id.to_string().as_str());
        let slow_logger = ctx.logger.slow_action_logger();
        let logger_factory = Arc::clone(&ctx.logger);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let kv_store = Arc::clone(&ctx.kv_store);
        let num_threads = processor.num_threads();
        Self {
            node_handle,
            node_handles,
            receivers,
            processor: Arc::new(parking_lot::RwLock::new(processor)),
            channel_manager: Arc::new(parking_lot::RwLock::new(channel_manager)),
            shutdown,
            runtime,
            error_manager: dag.error_manager().clone(),
            logger_factory,
            logger: Arc::new(logger),
            slow_logger: Arc::new(slow_logger),
            span,
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
            thread_counter: Arc::new(AtomicU32::new(0)),
            expr_engine,
            storage_resolver,
            kv_store,
            event_sender: dag.event_hub().sender.clone(),
        }
    }

    pub fn handle(&self) -> &NodeHandle {
        &self.node_handle
    }
}

impl<F: Future + Unpin + Debug> ReceiverLoop for ProcessorNode<F> {
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>> {
        let mut result = vec![];
        swap(&mut self.receivers, &mut result);
        result
    }

    fn receiver_loop(mut self) -> Result<(), ExecutionError>
    where
        Self: Sized,
    {
        let receivers = self.receivers();
        let mut is_terminated = vec![false; receivers.len()];
        let mut sel = init_select(&receivers);

        let span = self.span.clone();
        let logger = self.logger.clone();
        let now = time::Instant::now();
        let processor = Arc::clone(&self.processor);
        processor
            .write()
            .initialize(NodeContext::new(
                self.expr_engine.clone(),
                self.storage_resolver.clone(),
                self.logger_factory.clone(),
                self.kv_store.clone(),
            ))
            .map_err(ExecutionError::Processor)?;
        action_log!(
            parent: span, logger, "{:?} process start...", self.processor.read().name(),
        );

        loop {
            if is_terminated.iter().all(|value| *value) {
                if self
                    .thread_counter
                    .load(std::sync::atomic::Ordering::SeqCst)
                    == 0
                {
                    action_log!(
                        parent: span, logger, "{:?} process finish. elapsed = {:?}", self.processor.read().name() , now.elapsed(),
                    );
                    self.on_terminate(NodeContext::new(
                        self.expr_engine.clone(),
                        self.storage_resolver.clone(),
                        self.logger_factory.clone(),
                        self.kv_store.clone(),
                    ))?;
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            let index = sel.ready();
            let op = receivers[index]
                .recv()
                .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{:?}", e)))?;
            match op {
                ExecutorOperation::Op { ctx } => {
                    self.on_op(ctx)?;
                }
                ExecutorOperation::Terminate { ctx: _ctx } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                }
            }
        }
    }

    fn receiver_name(&self, index: usize) -> Cow<str> {
        Cow::Owned(self.node_handles[index].to_string())
    }

    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        let channel_manager = Arc::clone(&self.channel_manager);
        let processor = Arc::clone(&self.processor);

        let span = self.span.clone();
        let logger = self.logger.clone();
        let slow_logger = self.slow_logger.clone();
        let node_handle = self.node_handle.clone();
        let counter = Arc::clone(&self.thread_counter);
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.thread_pool.spawn(move || {
            process(
                ctx,
                node_handle,
                span,
                logger,
                slow_logger,
                channel_manager,
                processor,
            );
            counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
        Ok(())
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        let channel_manager = Arc::clone(&self.channel_manager);
        let mut channel_manager_guard = channel_manager.write();
        let processor = Arc::clone(&self.processor);
        let channel_manager: &mut ChannelManager = &mut channel_manager_guard;
        let now = time::Instant::now();
        processor
            .write()
            .finish(ctx.clone(), channel_manager)
            .map_err(|e| ExecutionError::CannotSendToChannel(format!("{:?}", e)))?;
        let span = self.span.clone();
        let logger = self.logger.clone();
        action_log!(
            parent: span, logger, "{:?} finish process complete. elapsed = {:?}", processor.read().name(), now.elapsed(),
        );
        channel_manager.send_terminate(ctx)
    }
}

fn process(
    ctx: ExecutorContext,
    node_handle: NodeHandle,
    span: Span,
    logger: Arc<ActionLogger>,
    slow_logger: Arc<ActionLogger>,
    channel_manager: Arc<parking_lot::RwLock<ChannelManager>>,
    processor: Arc<parking_lot::RwLock<Box<dyn Processor>>>,
) {
    let feature_id = ctx.feature.id;
    let mut channel_manager_guard = channel_manager.write();
    let mut processor_guard = processor.write();
    let channel_manager: &mut ChannelManager = &mut channel_manager_guard;
    let processor: &mut Box<dyn Processor> = &mut processor_guard;
    let now = time::Instant::now();
    let result = processor.process(ctx, channel_manager);
    let elapsed = now.elapsed();
    if elapsed >= *SLOW_ACTION_THRESHOLD {
        slow_action_log!(
            parent: span,
            slow_logger,
            "Slow action, processor node name = {:?}, node_id = {}, feature id = {:?}, elapsed = {:?}",
            processor.name(),
            node_handle.id,
            feature_id,
            elapsed,
        )
    }
    if let Err(e) = result {
        action_error_log!(
            parent: span,
            logger,
            "Error operation, processor node name = {:?}, node_id = {}, feature id = {:?}, error = {:?}", processor.name(),
            node_handle.id,
            feature_id,
            e,
        )
    }
}

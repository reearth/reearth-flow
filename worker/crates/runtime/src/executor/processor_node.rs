use std::sync::Arc;
use std::{borrow::Cow, mem::swap};

use crossbeam::channel::Receiver;
use petgraph::graph::NodeIndex;
use reearth_flow_action_log::{action_log, ActionLogger};
use tracing::info_span;

use crate::error_manager::ErrorManager;
use crate::executor_operation::{ExecutorContext, ExecutorOperation, NodeContext};
use crate::{
    builder_dag::NodeKind,
    errors::ExecutionError,
    forwarder::ChannelManager,
    node::{NodeHandle, Processor},
};

use super::{execution_dag::ExecutionDag, name::Name, receiver_loop::ReceiverLoop};

/// A processor in the execution DAG.
#[derive(Debug)]
pub struct ProcessorNode {
    /// Node handle in description DAG.
    node_handle: NodeHandle,
    /// Input node handles.
    node_handles: Vec<NodeHandle>,
    /// Input data channels.
    receivers: Vec<Receiver<ExecutorOperation>>,
    /// The processor.
    processor: Box<dyn Processor>,
    /// This node's output channel manager, for forwarding data, writing metadata and writing port state.
    channel_manager: ChannelManager,
    /// The error manager, for reporting non-fatal errors.
    error_manager: Arc<ErrorManager>,
    logger: Arc<ActionLogger>,
    span: tracing::Span,
}

impl ProcessorNode {
    pub async fn new(ctx: NodeContext, dag: &mut ExecutionDag, node_index: NodeIndex) -> Self {
        let node = dag.node_weight_mut(node_index);
        let Some(kind) = node.kind.take() else {
            panic!("Must pass in a node")
        };
        let node_handle = node.handle.clone();
        let NodeKind::Processor(mut processor) = kind else {
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
        );
        let span = info_span!(
            "root",
            "otel.name" = processor.name(),
            "otel.kind" = "Processor Node",
            "workflow.id" = dag.id.to_string().as_str(),
            "node.id" = node_handle.id.to_string().as_str(),
        );

        let logger = ctx
            .logger
            .action_logger(node_handle.id.to_string().as_str());
        processor.initialize(ctx);
        Self {
            node_handle,
            node_handles,
            receivers,
            processor,
            channel_manager,
            error_manager: dag.error_manager().clone(),
            logger: Arc::new(logger),
            span,
        }
    }

    pub fn handle(&self) -> &NodeHandle {
        &self.node_handle
    }
}

impl Name for ProcessorNode {
    fn name(&self) -> Cow<str> {
        Cow::Owned(self.node_handle.to_string())
    }
}

impl ReceiverLoop for ProcessorNode {
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>> {
        let mut result = vec![];
        swap(&mut self.receivers, &mut result);
        result
    }

    fn receiver_name(&self, index: usize) -> Cow<str> {
        Cow::Owned(self.node_handles[index].to_string())
    }

    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        let feature_id = &ctx.feature.id;
        action_log!(
            parent: &self.span, &self.logger, "Processing operation, feature id = {:?}", feature_id,
        );
        if let Err(e) = self.processor.process(ctx, &mut self.channel_manager) {
            action_log!(
                parent: &self.span, &self.logger, "{:?}", e,
            );
            self.error_manager.report(e);
        }
        Ok(())
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        self.processor
            .finish(ctx.clone(), &mut self.channel_manager)
            .map_err(|e| {
                self.error_manager.report(e);
                ExecutionError::CannotSendToChannel
            })?;
        self.channel_manager.send_terminate(ctx)
    }
}

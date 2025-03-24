use std::{fmt::Debug, future::Future, pin::pin, sync::Arc, time};

use petgraph::visit::IntoNodeIdentifiers;

use async_stream::stream;
use futures::{future::select_all, future::Either, Stream, StreamExt};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::{
    runtime::Handle,
    sync::mpsc::{channel, Receiver, Sender},
};
use tracing::info_span;

use crate::{
    builder_dag::NodeKind,
    errors::ExecutionError,
    event::{Event, EventHub},
    executor_operation::{ExecutorContext, ExecutorOperation, ExecutorOptions, NodeContext},
    forwarder::ChannelManager,
    kvs::KvStore,
    node::{IngestionMessage, NodeStatus, Port, Source, SourceState},
};

use super::execution_dag::ExecutionDag;
use super::node::Node;

/// The source operation collector.
#[derive(Debug)]
pub struct SourceNode<F> {
    /// To decide when to emit `Commit`, we keep track of source state.
    sources: Vec<RunningSource>,
    /// Structs for running a source.
    source_runners: Vec<SourceRunner>,
    /// Receivers from sources.
    receivers: Vec<Receiver<(Port, IngestionMessage)>>,
    /// The shutdown future.
    shutdown: F,
    /// The runtime to run the source in.
    runtime: Arc<Handle>,

    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    span: tracing::Span,
    event_hub: EventHub,
}

impl<F: Future + Unpin> Node for SourceNode<F> {
    fn run(mut self) -> Result<(), ExecutionError> {
        for source in &self.sources {
            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: source.channel_manager.owner().clone(),
                status: NodeStatus::Starting,
                feature_id: None,
            });
        }

        for source in &self.sources {
            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: source.channel_manager.owner().clone(),
                status: NodeStatus::Processing,
                feature_id: None,
            });
        }

        let mut handles = vec![];
        let source_runners = std::mem::take(&mut self.source_runners);

        for (index, source_runner) in source_runners.into_iter().enumerate() {
            let ctx = NodeContext::new(
                Arc::clone(&self.expr_engine),
                Arc::clone(&self.storage_resolver),
                Arc::clone(&self.kv_store),
                self.event_hub.clone(),
            );
            let span = self.span.clone();
            let event_hub = self.event_hub.clone();
            let source_node_handle = self.sources[index].channel_manager.owner().clone();

            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: source_node_handle.clone(),
                status: NodeStatus::Processing,
                feature_id: None,
            });

            let mut source = source_runner.source;
            let sender = source_runner.sender;

            handles.push(Some(self.runtime.spawn(async move {
                let now = time::Instant::now();
                let result = source.start(ctx, sender).await;
                event_hub.info_log(
                    Some(span.clone()),
                    format!(
                        "{:?} finish source complete. elapsed = {:?}",
                        source.name(),
                        now.elapsed()
                    ),
                );

                event_hub.send(Event::NodeStatusChanged {
                    node_handle: source_node_handle,
                    status: NodeStatus::Completed,
                    feature_id: None,
                });

                tracing::info!("Waiting for final status to propagate for all source nodes");
                std::thread::sleep(std::time::Duration::from_millis(300));

                result
            })));
        }

        let mut num_running_sources = handles.len();
        let mut stream = pin!(receivers_stream(self.receivers));

        loop {
            let next = stream.next();
            let next = pin!(next);
            match self
                .runtime
                .block_on(futures::future::select(self.shutdown, next))
            {
                Either::Left((_, _)) => {
                    let ctx = NodeContext::new(
                        Arc::clone(&self.expr_engine),
                        Arc::clone(&self.storage_resolver),
                        Arc::clone(&self.kv_store),
                        self.event_hub.clone(),
                    );

                    for source in &self.sources {
                        self.event_hub.send(Event::NodeStatusChanged {
                            node_handle: source.channel_manager.owner().clone(),
                            status: NodeStatus::Completed,
                            feature_id: None,
                        });
                    }

                    tracing::info!("Waiting for final status to propagate for all source nodes");
                    std::thread::sleep(std::time::Duration::from_millis(300));

                    send_to_all_nodes(&self.sources, ExecutorOperation::Terminate { ctx })?;
                    self.event_hub.send(Event::SourceFlushed);
                    return Ok(());
                }
                Either::Right((next, shutdown)) => {
                    let next = next.expect("We return just when the stream ends");
                    self.shutdown = shutdown;
                    let index = next.0;
                    let Some((port, message)) = next.1 else {
                        match self.runtime.block_on(
                            handles[index]
                                .take()
                                .expect("Shouldn't receive message from dropped receiver"),
                        ) {
                            Ok(Ok(())) => {
                                num_running_sources -= 1;
                                if num_running_sources == 0 {
                                    let ctx = NodeContext::new(
                                        Arc::clone(&self.expr_engine),
                                        Arc::clone(&self.storage_resolver),
                                        Arc::clone(&self.kv_store),
                                        self.event_hub.clone(),
                                    );

                                    for source in &self.sources {
                                        self.event_hub.send(Event::NodeStatusChanged {
                                            node_handle: source.channel_manager.owner().clone(),
                                            status: NodeStatus::Completed,
                                            feature_id: None,
                                        });
                                    }

                                    tracing::info!("Waiting for final status to propagate for all source nodes");
                                    std::thread::sleep(std::time::Duration::from_millis(300));

                                    send_to_all_nodes(
                                        &self.sources,
                                        ExecutorOperation::Terminate { ctx },
                                    )?;
                                    self.event_hub.send(Event::SourceFlushed);
                                    return Ok(());
                                }
                                continue;
                            }
                            Ok(Err(e)) => {
                                self.event_hub.send(Event::NodeStatusChanged {
                                    node_handle: self.sources[index]
                                        .channel_manager
                                        .owner()
                                        .clone(),
                                    status: NodeStatus::Failed,
                                    feature_id: None,
                                });

                                tracing::info!("Waiting for failed status to propagate for source node {}", self.sources[index].channel_manager.owner().id);
                                std::thread::sleep(std::time::Duration::from_millis(300));

                                return Err(ExecutionError::Source(e));
                            }
                            Err(e) => {
                                self.event_hub.send(Event::NodeStatusChanged {
                                    node_handle: self.sources[index]
                                        .channel_manager
                                        .owner()
                                        .clone(),
                                    status: NodeStatus::Failed,
                                    feature_id: None,
                                });

                                panic!("Source panicked: {e}");
                            }
                        }
                    };

                    let source = &mut self.sources[index];
                    match message {
                        IngestionMessage::OperationEvent { feature, .. } => {
                            source.state = SourceState::NonRestartable;

                            self.event_hub.send(Event::NodeStatusChanged {
                                node_handle: source.channel_manager.owner().clone(),
                                status: NodeStatus::Processing,
                                feature_id: Some(feature.id),
                            });

                            source.channel_manager.send_op(ExecutorContext::new(
                                feature.clone(),
                                port.clone(),
                                Arc::clone(&self.expr_engine),
                                Arc::clone(&self.storage_resolver),
                                Arc::clone(&self.kv_store),
                                self.event_hub.clone(),
                            ))?;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct RunningSource {
    channel_manager: ChannelManager,
    state: SourceState,
}

#[derive(Debug)]
struct SourceRunner {
    source: Box<dyn Source>,
    sender: Sender<(Port, IngestionMessage)>,
}

/// Returns if the operation is sent successfully.
fn send_to_all_nodes(
    sources: &[RunningSource],
    op: ExecutorOperation,
) -> Result<(), ExecutionError> {
    for source in sources {
        source.channel_manager.send_non_op(op.clone())?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_source_node<F>(
    ctx: NodeContext,
    dag: &mut ExecutionDag,
    options: &ExecutorOptions,
    shutdown: F,
    runtime: Arc<Handle>,
) -> SourceNode<F> {
    let mut sources = vec![];
    let mut source_runners = vec![];
    let mut receivers = vec![];

    let node_indices = dag.graph().node_identifiers().collect::<Vec<_>>();
    for node_index in node_indices {
        let node = dag.graph()[node_index]
            .kind
            .as_ref()
            .expect("Each node should only be visited once");
        if !matches!(node, NodeKind::Source { .. }) {
            continue;
        }
        let node = dag.node_weight_mut(node_index);
        let node_handle = node.handle.clone();
        let NodeKind::Source(source) = node.kind.take().unwrap() else {
            continue;
        };

        let senders = dag.collect_senders(node_index);
        let record_writers = dag.collect_record_writers(node_index).await;
        let channel_manager = ChannelManager::new(
            node_handle,
            record_writers,
            senders,
            runtime.clone(),
            dag.event_hub().clone(),
        );
        sources.push(RunningSource {
            channel_manager,
            state: SourceState::NotStarted,
        });

        let (sender, receiver) = channel(options.channel_buffer_sz);
        let ctx = ctx.clone();
        source.initialize(ctx).await;
        source_runners.push(SourceRunner { source, sender });
        receivers.push(receiver);
    }

    let version = env!("CARGO_PKG_VERSION");
    let span = info_span!(
        "action",
        "engine.version" = version,
        "otel.name" = "Source Node",
        "otel.kind" = "Source Node",
        "workflow.id" = dag.id.to_string().as_str(),
    );

    SourceNode {
        sources,
        source_runners,
        receivers,
        shutdown,
        runtime,
        expr_engine: Arc::clone(&ctx.expr_engine),
        storage_resolver: Arc::clone(&ctx.storage_resolver),
        kv_store: Arc::clone(&ctx.kv_store),
        span,
        event_hub: dag.event_hub().clone(),
    }
}

/// A convenient way of getting a self-referential struct.
async fn receive_or_drop<T>(
    index: usize,
    mut receiver: Receiver<T>,
) -> (usize, Option<(Receiver<T>, T)>) {
    (index, receiver.recv().await.map(|item| (receiver, item)))
}

/// This is not simply the merge of `ReceiverStream` because we need to know if the source has quit.
pub fn receivers_stream<T>(receivers: Vec<Receiver<T>>) -> impl Stream<Item = (usize, Option<T>)> {
    let mut futures = receivers
        .into_iter()
        .enumerate()
        .map(|(index, receiver)| Box::pin(receive_or_drop(index, receiver)))
        .collect::<Vec<_>>();

    stream! {
        while !futures.is_empty() {
            match select_all(futures).await {
                ((index, Some((receiver, item))), _, remaining) => {
                    yield (index, Some(item));
                    futures = remaining;
                    // Can we somehow remove the allocation here?
                    futures.push(Box::pin(receive_or_drop(index, receiver)));
                }
                ((index, None), _, remaining) => {
                    yield (index, None);
                    futures = remaining;
                }
            }
        }
    }
}

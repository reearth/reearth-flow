use std::{
    collections::HashSet,
    fmt::Debug,
    future::Future,
    pin::pin,
    sync::{atomic::AtomicU64, Arc},
    time,
};

use petgraph::visit::IntoNodeIdentifiers;

use async_stream::stream;
use futures::{future::select_all, future::Either, Stream, StreamExt};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::{
    runtime::Handle,
    sync::mpsc::{channel, Receiver, Sender},
};
use tracing::info_span;

use crate::{
    builder_dag::NodeKind,
    errors::{to_node_error, ExecutionError, NodeErrorKind},
    event::{Event, EventHub},
    executor_operation::{ExecutorContext, ExecutorOptions, NodeContext},
    forwarder::ChannelManager,
    kvs::KvStore,
    node::{IngestionMessage, NodeId, NodeStatus, Port, Source, SourceState},
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

    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    sandbox_root: Uri,
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
                Arc::clone(&self.env_vars),
                Arc::clone(&self.storage_resolver),
                Arc::clone(&self.kv_store),
                self.event_hub.clone(),
                self.sandbox_root.clone(),
            );
            let span = self.span.clone();
            let event_hub = self.event_hub.clone();
            let source_node_handle = self.sources[index].channel_manager.owner().clone();
            let source_composed_id = self.sources[index].composed_id.clone();

            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: source_node_handle.clone(),
                status: NodeStatus::Processing,
                feature_id: None,
            });

            let mut source = source_runner.source;
            let sender = source_runner.sender;
            let node_name = source_runner.node_name;

            handles.push(Some(self.runtime.spawn(async move {
                let node_span = info_span!(
                    parent: &span,
                    "source_node",
                    "node.id" = source_composed_id.as_str(),
                    "node.name" = node_name.as_str(),
                );

                event_hub.info_log_with_node_info(
                    Some(node_span.clone()),
                    source_node_handle.clone(),
                    node_name.clone(),
                    format!("{} source start...", source.name()),
                );
                let now = time::Instant::now();
                let result = source.start(ctx, sender).await;

                if result.is_ok() {
                    let message = format!(
                        "{} source finish. elapsed = {:?}",
                        source.name(),
                        now.elapsed()
                    );

                    event_hub.info_log_with_node_info(
                        Some(node_span.clone()),
                        source_node_handle.clone(),
                        node_name.clone(),
                        message,
                    );

                    event_hub.send(Event::NodeStatusChanged {
                        node_handle: source_node_handle,
                        status: NodeStatus::Completed,
                        feature_id: None,
                    });
                } else if let Err(ref e) = result {
                    event_hub.error_log_with_node_info(
                        Some(node_span.clone()),
                        source_node_handle.clone(),
                        node_name.clone(),
                        format!("{} source error: {:?}", source.name(), e),
                    );

                    event_hub.send(Event::NodeStatusChanged {
                        node_handle: source_node_handle,
                        status: NodeStatus::Failed,
                        feature_id: None,
                    });
                }

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
                        Arc::clone(&self.env_vars),
                        Arc::clone(&self.storage_resolver),
                        Arc::clone(&self.kv_store),
                        self.event_hub.clone(),
                        self.sandbox_root.clone(),
                    );

                    for source in &self.sources {
                        self.event_hub.send(Event::NodeStatusChanged {
                            node_handle: source.channel_manager.owner().clone(),
                            status: NodeStatus::Completed,
                            feature_id: None,
                        });
                    }

                    send_to_all_nodes(&self.sources, ctx)?;
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
                                        Arc::clone(&self.env_vars),
                                        Arc::clone(&self.storage_resolver),
                                        Arc::clone(&self.kv_store),
                                        self.event_hub.clone(),
                                        self.sandbox_root.clone(),
                                    );

                                    for source in &self.sources {
                                        self.event_hub.send(Event::NodeStatusChanged {
                                            node_handle: source.channel_manager.owner().clone(),
                                            status: NodeStatus::Completed,
                                            feature_id: None,
                                        });
                                    }

                                    send_to_all_nodes(&self.sources, ctx)?;
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

                                return Err(to_node_error(e, NodeErrorKind::Source));
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

                            source
                                .features_produced
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            self.event_hub.send(Event::NodeStatusChanged {
                                node_handle: source.channel_manager.owner().clone(),
                                status: NodeStatus::Processing,
                                feature_id: Some(feature.id),
                            });

                            source.channel_manager.send_op(ExecutorContext::new(
                                feature.clone(),
                                port.clone(),
                                Arc::clone(&self.env_vars),
                                Arc::clone(&self.storage_resolver),
                                Arc::clone(&self.kv_store),
                                self.event_hub.clone(),
                                self.sandbox_root.clone(),
                            ))?;
                        }
                    }
                }
            }
        }
    }
}

impl<F> SourceNode<F> {
    /// This thread's node identity for the fold's synthesized diagnostics
    /// (`start_source` in `dag_executor.rs` carries this alongside the
    /// spawned thread's `JoinHandle`). A single `SourceNode` thread can run
    /// more than one source node (`self.sources: Vec<RunningSource>`), so
    /// there is no single composed id to report in general: with exactly
    /// one source, its own identity is used; with zero or several, a
    /// best-effort summary is reported instead — this is only ever consumed
    /// by the catch-all diagnostic-synthesis path (a source thread failure
    /// that isn't a recoverable structured `Diagnostic`), not by
    /// `report()`/`resolve()`, which sources do not call.
    pub fn node_meta(&self) -> super::dag_executor::NodeMeta {
        match self.sources.as_slice() {
            [only] => super::dag_executor::NodeMeta {
                composed_id: only.composed_id.clone(),
                action: only.action.clone(),
            },
            many => super::dag_executor::NodeMeta {
                composed_id: many
                    .iter()
                    .map(|s| s.composed_id.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                action: "source".to_string(),
            },
        }
    }
}

#[derive(Debug)]
struct RunningSource {
    channel_manager: ChannelManager,
    state: SourceState,
    #[allow(dead_code)]
    node_name: String,
    features_produced: Arc<AtomicU64>,
    /// This source's composed id (`execution_dag::NodeType::composed_id`)
    /// and action string, captured before `kind.take()` moves the
    /// `Box<dyn Source>` out of the execution DAG. Used for the per-source
    /// tracing span (spec 4.3: logs must agree with diagnostic identity)
    /// and, aggregated across every source in this thread, `node_meta()`.
    composed_id: String,
    action: String,
}

#[derive(Debug)]
struct SourceRunner {
    source: Box<dyn Source>,
    sender: Sender<(Port, IngestionMessage)>,
    node_name: String,
}

/// Terminates all source nodes by flushing port writers then sending Terminate.
fn send_to_all_nodes(sources: &[RunningSource], ctx: NodeContext) -> Result<(), ExecutionError> {
    for source in sources {
        source.channel_manager.send_terminate(ctx.clone())?;
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
    execute_node_ids: Option<HashSet<NodeId>>,
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

        // ★ incremental: run only selected sources
        let node_id = dag.graph()[node_index].handle.id.clone();
        if let Some(set) = execute_node_ids.as_ref() {
            if !set.contains(&node_id) {
                tracing::info!("Skipping source node {} for incremental run", node_id);
                continue;
            }
        }

        let node = dag.node_weight_mut(node_index);
        let node_handle = node.handle.clone();
        let node_name = node.name.clone();
        let composed_id = node.composed_id();
        let action = node.action.clone();
        let NodeKind::Source(source) = node.kind.take().unwrap() else {
            continue;
        };
        // NOTE: `action` is NOT asserted equal to `source.name()` here. See
        // the matching note in `processor_node.rs::ProcessorNode::new`:
        // `builder_dag.rs`'s `ActionNameMismatch` check validates the
        // *factory's* `SourceFactory::name()` against `node.node.action()` at
        // build time (`action`'s provenance) — the *built instance*'s
        // `Source::name()` is a different trait and can legitimately diverge
        // (e.g. profile-namespaced factory keys vs. a generic instance
        // display name), as proven for the processor case by the
        // quality-check workflow fixtures.
        let senders = dag.collect_senders(node_index);
        let port_writers = dag.collect_port_writers(node_index);
        let channel_manager = ChannelManager::new(
            node_handle,
            port_writers,
            senders,
            runtime.clone(),
            dag.event_hub().clone(),
            dag.executor_id(),
        );
        let features_produced = Arc::new(AtomicU64::new(0));
        sources.push(RunningSource {
            channel_manager,
            state: SourceState::NotStarted,
            node_name: node_name.clone(),
            features_produced: features_produced.clone(),
            composed_id,
            action,
        });

        let (sender, receiver) = channel(options.channel_buffer_sz);
        let ctx = ctx.clone();
        source.initialize(ctx).await;
        source_runners.push(SourceRunner {
            source,
            sender,
            node_name,
        });
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
        env_vars: Arc::clone(&ctx.env_vars),
        storage_resolver: Arc::clone(&ctx.storage_resolver),
        kv_store: Arc::clone(&ctx.kv_store),
        sandbox_root: ctx.sandbox_root.clone(),
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;
use reearth_flow_types::Feature;
use tokio::runtime::Handle;

use crate::errors::ExecutionError;
use crate::event::{Event, EventHub};
use crate::executor_operation::{ExecutorContext, ExecutorOperation, NodeContext};
use crate::feature_store::{FeatureWriter, FeatureWriterKey};

use crate::node::{NodeHandle, Port};

#[derive(Debug, Clone)]
pub enum ProcessorChannelForwarder {
    ChannelManager(ChannelManager),
    Noop(NoopChannelForwarder),
}

impl ProcessorChannelForwarder {
    pub fn node_id(&self) -> String {
        match self {
            ProcessorChannelForwarder::ChannelManager(channel_manager) => channel_manager.node_id(),
            ProcessorChannelForwarder::Noop(noop) => noop.node_id(),
        }
    }
    pub fn send(&self, ctx: ExecutorContext) {
        match self {
            ProcessorChannelForwarder::ChannelManager(channel_manager) => channel_manager.send(ctx),
            ProcessorChannelForwarder::Noop(noop) => noop.send(ctx),
        }
    }

    pub fn send_terminate(&self, ctx: NodeContext) -> Result<(), ExecutionError> {
        match self {
            ProcessorChannelForwarder::ChannelManager(channel_manager) => {
                channel_manager.send_terminate(ctx)
            }
            ProcessorChannelForwarder::Noop(_) => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SenderWithPortMapping {
    pub sender: Sender<ExecutorOperation>,
    pub port_mapping: HashMap<Port, Vec<Port>>,
}

impl SenderWithPortMapping {
    pub fn send_op(&self, mut ctx: ExecutorContext) -> Result<(), ExecutionError> {
        let Some(ports) = self.port_mapping.get(&ctx.port) else {
            // Downstream node is not interested in data from this port.
            return Ok(());
        };

        if let Some((last_port, ports)) = ports.split_last() {
            for port in ports {
                let mut ctx = ctx.clone();
                ctx.port = port.clone();
                self.sender.send(ExecutorOperation::Op { ctx })?;
            }
            ctx.port = last_port.clone();
            self.sender.send(ExecutorOperation::Op { ctx })?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChannelManager {
    owner: NodeHandle,
    feature_writers: HashMap<FeatureWriterKey, Vec<Box<dyn FeatureWriter>>>,
    senders: Vec<SenderWithPortMapping>,
    runtime: Arc<Handle>,
    event_hub: EventHub,
}

impl ChannelManager {
    #[inline]
    pub fn send_op(&self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        let sender_ports: HashMap<Port, Vec<Port>> = {
            let mut sender_port = HashMap::new();
            for sender in &self.senders {
                for (port, ports) in &sender.port_mapping {
                    let entry = sender_port.entry(port.clone()).or_insert_with(Vec::new);
                    for port_item in ports {
                        if !entry.contains(port_item) {
                            entry.push(port_item.clone());
                        }
                    }
                }
            }
            sender_port
        };
        if let Some(sender_ports) = sender_ports.get(&ctx.port) {
            for port in sender_ports {
                if let Some(writers) = self
                    .feature_writers
                    .get(&FeatureWriterKey(ctx.port.clone(), port.clone()))
                {
                    for writer in writers {
                        let edge_id = writer.edge_id();
                        let feature_id = ctx.feature.id;
                        let mut writer = writer.clone();
                        let feature = ctx.feature.clone();
                        let event_hub = self.event_hub.clone();
                        let node_handle = self.owner.clone();

                        let edge_id_clone = edge_id.clone();
                        self.event_hub.send(Event::EdgePassThrough {
                            feature_id,
                            edge_id: edge_id_clone,
                        });

                        self.runtime.block_on(async move {
                            let result = writer.write(&feature).await;
                            let node = node_handle.clone();
                            if let Err(e) = result {
                                event_hub.error_log_with_node_handle(
                                    None,
                                    node,
                                    format!("Failed to write feature: {e}"),
                                );
                            } else {
                                event_hub.send(Event::EdgeCompleted {
                                    feature_id,
                                    edge_id,
                                });
                            }
                        });
                    }
                }
            }
        }

        if let Some((last_sender, senders)) = self.senders.split_last() {
            for sender in senders {
                sender.send_op(ctx.clone())?;
            }
            last_sender.send_op(ctx)?;
        }
        Ok(())
    }

    pub fn send_non_op(&self, op: ExecutorOperation) -> Result<(), ExecutionError> {
        if let Some((last_sender, senders)) = self.senders.split_last() {
            for sender in senders {
                sender.sender.send(op.clone())?;
            }
            last_sender.sender.send(op)?;
        }
        Ok(())
    }

    pub fn send_terminate(&self, ctx: NodeContext) -> Result<(), ExecutionError> {
        let all_writers = self
            .feature_writers
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let node_handle = self.owner.clone();
        self.runtime.block_on(async {
            let futures = all_writers.iter().map(|writer| {
                let writer = writer.clone();
                let node = node_handle.clone();
                async move {
                    let result = writer.flush().await;
                    if let Err(e) = result {
                        self.event_hub.error_log_with_node_handle(
                            None,
                            node,
                            format!("Failed to flush feature writer: {e}"),
                        );
                    }
                }
            });
            futures::future::join_all(futures).await;
        });
        self.send_non_op(ExecutorOperation::Terminate { ctx })?;
        self.event_hub.info_log_with_node_handle(
            None,
            self.owner.clone(),
            format!(
                "Node terminated successfully with node handle: {:?}",
                self.owner.id,
            ),
        );
        Ok(())
    }

    pub fn owner(&self) -> &NodeHandle {
        &self.owner
    }

    pub fn new(
        owner: NodeHandle,
        feature_writers: HashMap<FeatureWriterKey, Vec<Box<dyn FeatureWriter>>>,
        senders: Vec<SenderWithPortMapping>,
        runtime: Arc<Handle>,
        event_hub: EventHub,
    ) -> Self {
        Self {
            owner,
            feature_writers,
            senders,
            runtime,
            event_hub,
        }
    }
}

impl ChannelManager {
    fn node_id(&self) -> String {
        self.owner.id.clone().into_inner()
    }

    fn send(&self, ctx: ExecutorContext) {
        let feature_id = ctx.feature.id;
        let port = ctx.port.clone();
        let node_id = self.owner.id.clone().into_inner();
        self.send_op(ctx).unwrap_or_else(|e| {
            panic!(
                "Failed to send operation: node_id = {node_id:?}, feature_id = {feature_id:?}, port = {port:?}, error = {e:?}"
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct NoopChannelForwarder {
    pub send_features: Arc<Mutex<Vec<Feature>>>,
    pub send_ports: Arc<Mutex<Vec<Port>>>,
}

impl Default for NoopChannelForwarder {
    fn default() -> Self {
        Self {
            send_features: Arc::new(Mutex::new(Vec::new())),
            send_ports: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl NoopChannelForwarder {
    pub fn node_id(&self) -> String {
        "noop".to_string()
    }

    pub fn send(&self, ctx: ExecutorContext) {
        let mut send_features = self.send_features.lock().unwrap();
        send_features.push(ctx.feature.clone());
        let mut send_ports = self.send_ports.lock().unwrap();
        send_ports.push(ctx.port.clone());
    }
}

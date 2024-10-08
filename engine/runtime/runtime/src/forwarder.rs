use std::collections::HashMap;
use std::sync::Arc;

use crossbeam::channel::Sender;
use tokio::runtime::Handle;

use crate::channels::ProcessorChannelForwarder;
use crate::error_manager::ErrorManager;
use crate::errors::ExecutionError;
use crate::event::Event;
use crate::executor_operation::{ExecutorContext, ExecutorOperation, NodeContext};
use crate::feature_store::FeatureWriter;
use crate::node::{NodeHandle, Port};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ChannelManager {
    owner: NodeHandle,
    feature_writers: HashMap<Port, Box<dyn FeatureWriter>>,
    senders: Vec<SenderWithPortMapping>,
    error_manager: Arc<ErrorManager>,
    runtime: Arc<Handle>,
    #[allow(dead_code)]
    event_sender: tokio::sync::broadcast::Sender<Event>,
}

impl ChannelManager {
    #[inline]
    pub fn send_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        if let Some(writer) = self.feature_writers.get(&ctx.port) {
            let edge_id = writer.edge_id();
            let feature_id = ctx.feature.id;
            let mut writer = writer.clone();
            let feature = ctx.feature.clone();
            self.runtime.spawn(async move {
                let _ = writer.write(&feature).await;
            });
            let _ = self.event_sender.send(Event::EdgePassThrough {
                feature_id,
                edge_id,
            });
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
        let writers = self.feature_writers.values().cloned().collect::<Vec<_>>();
        self.runtime.block_on(async {
            for writer in writers {
                let result = writer.flush().await;
                if let Err(e) = result {
                    self.error_manager.report(e.into());
                }
            }
        });
        self.send_non_op(ExecutorOperation::Terminate { ctx })
    }

    pub fn owner(&self) -> &NodeHandle {
        &self.owner
    }

    pub fn new(
        owner: NodeHandle,
        feature_writers: HashMap<Port, Box<dyn FeatureWriter>>,
        senders: Vec<SenderWithPortMapping>,
        error_manager: Arc<ErrorManager>,
        runtime: Arc<Handle>,
        event_sender: tokio::sync::broadcast::Sender<Event>,
    ) -> Self {
        Self {
            owner,
            feature_writers,
            senders,
            error_manager,
            runtime,
            event_sender,
        }
    }
}

impl ProcessorChannelForwarder for ChannelManager {
    fn send(&mut self, ctx: ExecutorContext) {
        self.send_op(ctx)
            .unwrap_or_else(|e| panic!("Failed to send operation: {e}"))
    }
}

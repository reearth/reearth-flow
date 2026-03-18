use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam::channel::Sender;

/// Timeout for individual channel send operations.
/// If a downstream channel stays full for this long, the send fails rather
/// than blocking forever (which previously caused 6-hour batch timeouts).
const CHANNEL_SEND_TIMEOUT: Duration = Duration::from_secs(300);

use crate::cache::executor_cache_subdir;
use reearth_flow_types::Feature;
use tokio::runtime::Handle;

use crate::errors::ExecutionError;
use crate::event::{Event, EventHub};
use crate::executor_operation::{Context, ExecutorContext, ExecutorOperation, NodeContext};
use crate::feature_store::{FeatureWriter, FeatureWriterKey};

use crate::node::{NodeHandle, Port};

/// Opens a JSONL (or zstd-compressed JSONL) file for line-by-line reading.
///
/// If the path ends with `.zst`, the file is transparently decompressed using
/// zstd streaming decoding with multi-frame support (for files written across
/// multiple flush operations). Otherwise a plain buffered reader is returned.
pub(crate) fn open_jsonl_reader(path: &Path) -> std::io::Result<Box<dyn BufRead>> {
    let file = std::fs::File::open(path)?;
    if path.extension().and_then(|e| e.to_str()) == Some("zst") {
        let decoder = zstd::Decoder::new(file)?;
        Ok(Box::new(BufReader::new(decoder)))
    } else {
        Ok(Box::new(BufReader::new(file)))
    }
}

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

    /// Sends a file-backed operation to downstream processors.
    ///
    /// The file at `path` will be moved and cleaned up by the engine core.
    pub fn send_file(&self, path: PathBuf, port: Port, context: Context) {
        match self {
            ProcessorChannelForwarder::ChannelManager(channel_manager) => {
                channel_manager.send_file(path, port, context)
            }
            ProcessorChannelForwarder::Noop(noop) => noop.send_file(path, port),
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

    pub fn wait_until_downstream_empty(&self, max_wait: std::time::Duration) -> bool {
        match self {
            ProcessorChannelForwarder::ChannelManager(cm) => {
                cm.wait_until_downstream_empty(max_wait)
            }
            ProcessorChannelForwarder::Noop(_) => true,
        }
    }

    /// Reset the send counter to zero.
    /// Call this before a section where you want to measure sends (e.g., before finish()).
    pub fn reset_send_count(&self) {
        match self {
            ProcessorChannelForwarder::ChannelManager(cm) => cm.reset_send_count(),
            ProcessorChannelForwarder::Noop(_) => {}
        }
    }

    /// Get the number of features sent since the last reset.
    pub fn get_send_count(&self) -> u64 {
        match self {
            ProcessorChannelForwarder::ChannelManager(cm) => cm.get_send_count(),
            ProcessorChannelForwarder::Noop(_) => 0,
        }
    }

    /// Get the executor ID for cache isolation
    pub fn executor_id(&self) -> uuid::Uuid {
        match self {
            ProcessorChannelForwarder::ChannelManager(cm) => cm.executor_id(),
            ProcessorChannelForwarder::Noop(_) => uuid::Uuid::nil(),
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
                self.sender
                    .send_timeout(ExecutorOperation::Op { ctx }, CHANNEL_SEND_TIMEOUT)?;
            }
            ctx.port = last_port.clone();
            self.sender
                .send_timeout(ExecutorOperation::Op { ctx }, CHANNEL_SEND_TIMEOUT)?;
        }
        Ok(())
    }

    pub fn send_file_backed_op(
        &self,
        path: &Path,
        port: &Port,
        context: &Context,
    ) -> Result<(), ExecutionError> {
        let Some(ports) = self.port_mapping.get(port) else {
            return Ok(());
        };

        if let Some((last_port, rest_ports)) = ports.split_last() {
            for p in rest_ports {
                self.sender.send_timeout(
                    ExecutorOperation::FileBackedOp {
                        path: path.to_path_buf(),
                        port: p.clone(),
                        context: context.clone(),
                    },
                    CHANNEL_SEND_TIMEOUT,
                )?;
            }
            self.sender.send_timeout(
                ExecutorOperation::FileBackedOp {
                    path: path.to_path_buf(),
                    port: last_port.clone(),
                    context: context.clone(),
                },
                CHANNEL_SEND_TIMEOUT,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChannelManager {
    owner: NodeHandle,
    feature_writers: HashMap<FeatureWriterKey, Vec<Box<dyn FeatureWriter>>>,
    senders: Vec<SenderWithPortMapping>,
    runtime: Arc<Handle>,
    event_hub: EventHub,
    /// Counter for features sent since the last reset - used for measuring sends in a specific phase
    send_count: Arc<AtomicU64>,
    /// Unique identifier for this workflow execution, used for cache isolation
    executor_id: uuid::Uuid,
}

impl Clone for ChannelManager {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            feature_writers: self.feature_writers.clone(),
            senders: self.senders.clone(),
            runtime: self.runtime.clone(),
            event_hub: self.event_hub.clone(),
            send_count: self.send_count.clone(),
            executor_id: self.executor_id,
        }
    }
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
                sender
                    .sender
                    .send_timeout(op.clone(), CHANNEL_SEND_TIMEOUT)?;
            }
            last_sender.sender.send_timeout(op, CHANNEL_SEND_TIMEOUT)?;
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
        // Flush writers on a dedicated thread to avoid blocking the tokio
        // runtime (concurrent block_on calls from many processor nodes can
        // starve the timer driver, preventing tokio::time::timeout from
        // ever firing).
        let runtime = Arc::clone(&self.runtime);
        let event_hub = self.event_hub.clone();
        let (done_tx, done_rx) = crossbeam::channel::bounded::<()>(1);
        std::thread::Builder::new()
            .name("flush-writers".to_string())
            .spawn(move || {
                runtime.block_on(async {
                    let futures = all_writers.iter().map(|writer| {
                        let writer = writer.clone();
                        let node = node_handle.clone();
                        let event_hub = event_hub.clone();
                        async move {
                            let result =
                                tokio::time::timeout(CHANNEL_SEND_TIMEOUT, writer.flush()).await;
                            match result {
                                Ok(Err(e)) => {
                                    event_hub.error_log_with_node_handle(
                                        None,
                                        node,
                                        format!("Failed to flush feature writer: {e}"),
                                    );
                                }
                                Err(_elapsed) => {
                                    tracing::warn!(
                                        node_id = ?node.id,
                                        "Feature writer flush timed out after {:?}",
                                        CHANNEL_SEND_TIMEOUT,
                                    );
                                }
                                Ok(Ok(())) => {}
                            }
                        }
                    });
                    futures::future::join_all(futures).await;
                });
                let _ = done_tx.send(());
            })
            .map_err(|e| ExecutionError::CannotSpawnWorkerThread(e))?;
        // Sync wait with a hard OS-level timeout — no dependency on tokio.
        if done_rx.recv_timeout(CHANNEL_SEND_TIMEOUT).is_err() {
            tracing::warn!(
                node_id = ?self.owner.id,
                "send_terminate: writer flush timed out after {:?}, proceeding",
                CHANNEL_SEND_TIMEOUT,
            );
        }
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

    pub fn are_downstream_channels_empty(&self) -> bool {
        self.senders.iter().all(|s| s.sender.is_empty())
    }

    /// Collect `(index, len)` for every non-empty downstream channel.
    fn downstream_channel_lengths(&self) -> Vec<(usize, usize)> {
        self.senders
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let len = s.sender.len();
                if len > 0 {
                    Some((i, len))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Wait for all downstream channels to drain, with a hard timeout.
    ///
    /// Logs progress every 30 s so silent hangs are visible in logs.
    /// After `max_wait` the function returns `false` (timed-out) instead
    /// of spinning forever; the caller can then proceed with a forced
    /// shutdown rather than blocking indefinitely.
    pub fn wait_until_downstream_empty(&self, max_wait: std::time::Duration) -> bool {
        let start = std::time::Instant::now();
        let log_interval = std::time::Duration::from_secs(30);
        let mut next_log = log_interval;

        while !self.are_downstream_channels_empty() {
            if start.elapsed() >= max_wait {
                let pending = self.downstream_channel_lengths();
                tracing::warn!(
                    node_id = ?self.owner.id,
                    ?pending,
                    elapsed = ?start.elapsed(),
                    "wait_until_downstream_empty timed out, proceeding with shutdown",
                );
                return false;
            }
            if start.elapsed() >= next_log {
                let pending = self.downstream_channel_lengths();
                tracing::info!(
                    node_id = ?self.owner.id,
                    ?pending,
                    elapsed = ?start.elapsed(),
                    "wait_until_downstream_empty still waiting",
                );
                next_log += log_interval;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        true
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
        executor_id: uuid::Uuid,
    ) -> Self {
        Self {
            owner,
            feature_writers,
            senders,
            runtime,
            event_hub,
            send_count: Arc::new(AtomicU64::new(0)),
            executor_id,
        }
    }

    /// Reset the send counter to zero.
    /// Call this before a section where you want to measure sends (e.g., before finish()).
    pub fn reset_send_count(&self) {
        self.send_count.store(0, Ordering::SeqCst);
    }

    /// Get the number of features sent since the last reset.
    pub fn get_send_count(&self) -> u64 {
        self.send_count.load(Ordering::SeqCst)
    }

    /// Increment the send counter.
    fn increment_send_count(&self, count: u64) {
        self.send_count.fetch_add(count, Ordering::SeqCst);
    }

    /// Get the executor ID for cache isolation
    pub fn executor_id(&self) -> uuid::Uuid {
        self.executor_id
    }
}

/// Returns the channel buffer directory path within the executor-specific cache.
fn channel_buffer_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "channel-buffers")
}

impl ChannelManager {
    fn node_id(&self) -> String {
        self.owner.id.clone().into_inner()
    }

    /// Sends a file-backed operation to downstream processors.
    ///
    /// The file at `path` will be moved and cleaned up by the engine core.
    fn send_file(&self, path: PathBuf, port: Port, context: Context) {
        // Count features in the file for debugging
        let feature_count = open_jsonl_reader(&path)
            .map(|r| {
                r.lines()
                    .filter(|l| l.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
                    .count()
            })
            .unwrap_or(0);
        self.increment_send_count(feature_count as u64);

        // Write features to FeatureWriter for observability
        self.write_file_features_to_store(&path, &port);

        // Move the file to executor-specific cache directory to prevent deletion when
        // the source processor's temp directory is cleaned up by Drop
        let cache_dir = channel_buffer_dir(self.executor_id);
        std::fs::create_dir_all(&cache_dir).unwrap_or_else(|e| {
            panic!("Failed to create channel buffer directory {cache_dir:?}: {e}")
        });

        let is_zst = path.extension().and_then(|e| e.to_str()) == Some("zst");
        let file_name = if is_zst {
            let stem = path
                .file_stem()
                .and_then(|s| Path::new(s).file_stem())
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            format!("{}-{}.jsonl.zst", uuid::Uuid::new_v4(), stem)
        } else {
            format!(
                "{}-{}.jsonl",
                uuid::Uuid::new_v4(),
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output")
            )
        };
        let new_path = cache_dir.join(file_name);

        std::fs::rename(&path, &new_path).unwrap_or_else(|e| {
            panic!("Failed to move file from {:?} to {:?}: {e}", path, new_path)
        });

        // Send FileBackedOp through channels with the new path
        let node_id = self.owner.id.clone().into_inner();
        if let Some((last_sender, senders)) = self.senders.split_last() {
            for sender in senders {
                sender
                    .send_file_backed_op(&new_path, &port, &context)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Failed to send file-backed operation: node_id = {node_id:?}, path = {new_path:?}, error = {e:?}",
                        )
                    });
            }
            last_sender
                .send_file_backed_op(&new_path, &port, &context)
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to send file-backed operation: node_id = {node_id:?}, path = {new_path:?}, error = {e:?}",
                    )
                });
        }
    }

    fn write_file_features_to_store(&self, path: &Path, port: &Port) {
        let reader = match open_jsonl_reader(path) {
            Ok(r) => r,
            Err(_) => return,
        };
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if line.is_empty() {
                continue;
            }
            let feature: Feature = match serde_json::from_str(&line) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let sender_ports: HashMap<Port, Vec<Port>> = {
                let mut sender_port = HashMap::new();
                for sender in &self.senders {
                    for (p, ports) in &sender.port_mapping {
                        let entry = sender_port.entry(p.clone()).or_insert_with(Vec::new);
                        for port_item in ports {
                            if !entry.contains(port_item) {
                                entry.push(port_item.clone());
                            }
                        }
                    }
                }
                sender_port
            };

            if let Some(sender_ports) = sender_ports.get(port) {
                for mapped_port in sender_ports {
                    if let Some(writers) = self
                        .feature_writers
                        .get(&FeatureWriterKey(port.clone(), mapped_port.clone()))
                    {
                        for writer in writers {
                            let edge_id = writer.edge_id();
                            let feature_id = feature.id;
                            let mut writer = writer.clone();
                            let feature = feature.clone();
                            let event_hub = self.event_hub.clone();
                            let node_handle = self.owner.clone();

                            self.event_hub.send(Event::EdgePassThrough {
                                feature_id,
                                edge_id: edge_id.clone(),
                            });

                            self.runtime.block_on(async move {
                                let result = writer.write(&feature).await;
                                if let Err(e) = result {
                                    event_hub.error_log_with_node_handle(
                                        None,
                                        node_handle,
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
        }
    }

    fn send(&self, ctx: ExecutorContext) {
        let feature_id = ctx.feature.id;
        let port = ctx.port.clone();
        let node_id = self.owner.id.clone().into_inner();
        self.send_op(ctx).unwrap_or_else(|e| {
            panic!(
                "Failed to send operation: node_id = {node_id:?}, feature_id = {feature_id:?}, port = {port:?}, error = {e:?}"
            )
        });
        self.increment_send_count(1);
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

    pub fn send_file(&self, path: PathBuf, port: Port) {
        let reader = match open_jsonl_reader(&path) {
            Ok(r) => r,
            Err(_) => return,
        };
        let mut send_features = self.send_features.lock().unwrap();
        let mut send_ports = self.send_ports.lock().unwrap();
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if line.is_empty() {
                continue;
            }
            let feature: Feature = match serde_json::from_str(&line) {
                Ok(f) => f,
                Err(_) => continue,
            };
            send_features.push(feature);
            send_ports.push(port.clone());
        }
    }
}

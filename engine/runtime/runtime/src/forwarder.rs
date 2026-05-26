use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam::channel::Sender;

use crate::cache::executor_cache_subdir;
use reearth_flow_types::Feature;
use tokio::runtime::Handle;

use crate::errors::ExecutionError;
use crate::event::EventHub;
use crate::executor_operation::{Context, ExecutorContext, ExecutorOperation, NodeContext};
use crate::feature_store::FeatureWriter;

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

    /// Enable spill mode: send() will try_send and spill to disk on full channels.
    pub fn enable_spill_mode(&self) {
        if let ProcessorChannelForwarder::ChannelManager(cm) = self {
            cm.enable_spill_mode();
        }
    }

    /// Flush spill files accumulated during finish() as FileBackedOps.
    pub fn flush_spill_files(&self, context: &Context) {
        if let ProcessorChannelForwarder::ChannelManager(cm) = self {
            cm.flush_spill_files(context);
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
                self.sender.send(ExecutorOperation::FileBackedOp {
                    path: path.to_path_buf(),
                    port: p.clone(),
                    context: context.clone(),
                })?;
            }
            self.sender.send(ExecutorOperation::FileBackedOp {
                path: path.to_path_buf(),
                port: last_port.clone(),
                context: context.clone(),
            })?;
        }
        Ok(())
    }
}

/// Per-sender spill buffer for features that couldn't be sent during finish().
#[derive(Debug)]
struct SpillFile {
    writer: BufWriter<std::fs::File>,
    path: PathBuf,
    count: usize,
}

#[derive(Debug)]
pub struct ChannelManager {
    owner: NodeHandle,
    /// Port-based feature writers: one writer per output port (for port-based intermediate data).
    port_writers: HashMap<Port, Box<dyn FeatureWriter>>,
    senders: Vec<SenderWithPortMapping>,
    runtime: Arc<Handle>,
    event_hub: EventHub,
    /// Counter for features sent since the last reset - used for measuring sends in a specific phase
    send_count: Arc<AtomicU64>,
    /// Unique identifier for this workflow execution, used for cache isolation
    executor_id: uuid::Uuid,
    /// When true, send() uses try_send and spills to disk on full channels.
    spill_mode: AtomicBool,
    /// Spill files keyed by (sender_index, port).
    spill_files: Mutex<HashMap<(usize, Port), SpillFile>>,
}

impl Clone for ChannelManager {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            port_writers: self.port_writers.clone(),
            senders: self.senders.clone(),
            runtime: self.runtime.clone(),
            event_hub: self.event_hub.clone(),
            send_count: self.send_count.clone(),
            executor_id: self.executor_id,
            spill_mode: AtomicBool::new(self.spill_mode.load(Ordering::Relaxed)),
            spill_files: Mutex::new(HashMap::new()),
        }
    }
}

impl ChannelManager {
    /// Write a feature to the port-based writer for the given port, if one exists.
    fn write_to_port(&self, port: &Port, feature: &Feature) {
        if let Some(port_writer) = self.port_writers.get(port) {
            let mut writer = port_writer.clone();
            let feature = feature.clone();
            let event_hub = self.event_hub.clone();
            let node_handle = self.owner.clone();
            self.runtime.block_on(async move {
                if let Err(e) = writer.write(&feature).await {
                    event_hub.error_log_with_node_handle(
                        None,
                        node_handle,
                        format!("Failed to write feature to port writer: {e}"),
                    );
                }
            });
        }
    }

    #[inline]
    pub fn send_op(&self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        self.write_to_port(&ctx.port, &ctx.feature);

        if self.spill_mode.load(Ordering::Relaxed) {
            return self.send_op_or_spill(ctx);
        }

        if let Some((last_sender, senders)) = self.senders.split_last() {
            for sender in senders {
                sender.send_op(ctx.clone())?;
            }
            last_sender.send_op(ctx)?;
        }
        Ok(())
    }

    /// Try to send to each downstream channel; spill to disk if full.
    ///
    /// Each sender has one channel shared across all port mappings. If
    /// try_send fails (channel full) for a sender, the entire feature is
    /// spilled to disk for that sender. The spill file is later flushed
    /// via send_file_backed_op which resolves the full port mapping,
    /// avoiding partial sends and duplicates.
    fn send_op_or_spill(&self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        for (idx, sender) in self.senders.iter().enumerate() {
            let Some(ports) = sender.port_mapping.get(&ctx.port) else {
                continue;
            };
            // Probe with a single try_send to check channel capacity.
            // If full, spill the feature and skip all ports for this sender.
            let first_port = match ports.first() {
                Some(p) => p,
                None => continue,
            };
            let probe = ExecutorOperation::Op {
                ctx: ExecutorContext {
                    port: first_port.clone(),
                    ..ctx.clone()
                },
            };
            match sender.sender.try_send(probe) {
                Ok(()) => {
                    // Channel has space. Send remaining mapped ports normally.
                    // First port already sent via try_send above.
                    if let Some((_, rest)) = ports.split_first() {
                        for port in rest {
                            let op = ExecutorOperation::Op {
                                ctx: ExecutorContext {
                                    port: port.clone(),
                                    ..ctx.clone()
                                },
                            };
                            // These may block but the channel just had space
                            sender.sender.send(op)?;
                        }
                    }
                }
                Err(crossbeam::channel::TrySendError::Full(_)) => {
                    self.spill_feature(idx, &ctx.port, &ctx.feature);
                }
                Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
                    return Err(ExecutionError::CannotSendToChannel(
                        "channel disconnected during spill".to_string(),
                    ));
                }
            }
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
        let all_writers: Vec<_> = self.port_writers.values().cloned().collect();
        let node_handle = self.owner.clone();
        let event_hub = self.event_hub.clone();
        self.runtime.block_on(async {
            let futures = all_writers.iter().map(|writer| {
                let writer = writer.clone();
                let node = node_handle.clone();
                let event_hub = event_hub.clone();
                async move {
                    if let Err(e) = writer.flush().await {
                        event_hub.error_log_with_node_handle(
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
        port_writers: HashMap<Port, Box<dyn FeatureWriter>>,
        senders: Vec<SenderWithPortMapping>,
        runtime: Arc<Handle>,
        event_hub: EventHub,
        executor_id: uuid::Uuid,
    ) -> Self {
        Self {
            owner,
            port_writers,
            senders,
            runtime,
            event_hub,
            send_count: Arc::new(AtomicU64::new(0)),
            executor_id,
            spill_mode: AtomicBool::new(false),
            spill_files: Mutex::new(HashMap::new()),
        }
    }

    /// Enable spill mode: send() will use try_send and spill to disk on full channels.
    pub fn enable_spill_mode(&self) {
        self.spill_mode.store(true, Ordering::Relaxed);
    }

    /// Spill a feature to a JSONL file for the given sender/port.
    fn spill_feature(&self, sender_idx: usize, port: &Port, feature: &Feature) {
        let mut spills = self.spill_files.lock().unwrap();
        let key = (sender_idx, port.clone());
        let spill = spills.entry(key).or_insert_with(|| {
            let dir = executor_cache_subdir(self.executor_id, "finish-spill");
            if let Err(e) = std::fs::create_dir_all(&dir) {
                tracing::error!(node_id = ?self.owner.id, ?e, "Failed to create spill directory");
            }
            let path = dir.join(format!(
                "{}-{}-{}.jsonl",
                self.owner.id,
                sender_idx,
                uuid::Uuid::new_v4()
            ));
            match std::fs::File::create(&path) {
                Ok(file) => SpillFile {
                    writer: BufWriter::new(file),
                    path,
                    count: 0,
                },
                Err(e) => {
                    tracing::error!(node_id = ?self.owner.id, ?e, "Failed to create spill file");
                    // Create a writer to /dev/null as fallback — features will be lost
                    // but the process won't crash.
                    let devnull = std::fs::File::create("/dev/null").unwrap();
                    SpillFile {
                        writer: BufWriter::new(devnull),
                        path,
                        count: 0,
                    }
                }
            }
        });
        match serde_json::to_string(feature) {
            Ok(json) => {
                if let Err(e) = writeln!(spill.writer, "{}", json) {
                    tracing::error!(
                        node_id = ?self.owner.id,
                        ?e,
                        "Failed to write feature to spill file"
                    );
                } else {
                    spill.count += 1;
                }
            }
            Err(e) => {
                tracing::error!(
                    node_id = ?self.owner.id,
                    ?e,
                    "Failed to serialize feature for spill"
                );
            }
        }
    }

    /// Flush all spill files by sending them as FileBackedOps through the channels.
    pub fn flush_spill_files(&self, context: &Context) {
        let mut spills = self.spill_files.lock().unwrap();
        for ((sender_idx, port), mut spill) in spills.drain() {
            if spill.count == 0 {
                continue;
            }
            if let Err(e) = spill.writer.flush() {
                tracing::error!(
                    node_id = ?self.owner.id, ?e,
                    "Failed to flush spill file writer"
                );
            }
            drop(spill.writer);

            tracing::info!(
                node_id = ?self.owner.id,
                sender_idx,
                ?port,
                count = spill.count,
                "Flushing spill file with {} features",
                spill.count,
            );

            let cache_dir = channel_buffer_dir(self.executor_id);
            if let Err(e) = std::fs::create_dir_all(&cache_dir) {
                tracing::error!(
                    node_id = ?self.owner.id, ?e,
                    "Failed to create channel buffer directory for spill flush"
                );
                continue;
            }
            let dest = cache_dir.join(spill.path.file_name().unwrap_or_default());
            if let Err(e) = std::fs::rename(&spill.path, &dest) {
                tracing::error!(
                    node_id = ?self.owner.id, ?e,
                    src = ?spill.path, dst = ?dest,
                    "Failed to move spill file to channel buffer"
                );
                continue;
            }
            if let Some(sender) = self.senders.get(sender_idx) {
                if let Err(e) = sender.send_file_backed_op(&dest, &port, context) {
                    tracing::error!(
                        node_id = ?self.owner.id, ?e,
                        "Failed to send spill FileBackedOp"
                    );
                }
            }
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

            self.write_to_port(port, &feature);
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

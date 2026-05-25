use std::path::PathBuf;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use tracing::{error_span, info_span};

use crate::{
    event::EventHub,
    kvs::KvStore,
    node::{Port, DEFAULT_PORT},
};

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ExecutorOperation {
    Op {
        ctx: ExecutorContext,
    },
    FileBackedOp {
        path: PathBuf,
        port: Port,
        context: Context,
    },
    Terminate {
        ctx: NodeContext,
    },
}

#[derive(Debug, Clone)]
pub struct Context {
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI. Tests using
    /// `NodeContext::default()` get `file:///` (permissive) — see `Default` impl.
    pub output_path: Uri,
}

impl From<ExecutorContext> for Context {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            output_path: ctx.output_path,
        }
    }
}

impl From<NodeContext> for Context {
    fn from(ctx: NodeContext) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            output_path: ctx.output_path,
        }
    }
}

impl Context {
    pub fn new(
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        output_path: Uri,
    ) -> Self {
        Self {
            expr_engine,
            storage_resolver,
            kv_store,
            event_hub,
            output_path,
        }
    }

    pub fn as_executor_context(&self, feature: Feature, port: Port) -> ExecutorContext {
        ExecutorContext {
            feature,
            port,
            expr_engine: self.expr_engine.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            output_path: self.output_path.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorContext {
    pub feature: Feature,
    pub port: Port,
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI. Tests using
    /// `NodeContext::default()` get `file:///` (permissive) — see `Default` impl.
    pub output_path: Uri,
}

impl ExecutorContext {
    pub fn new(
        feature: Feature,
        port: Port,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        output_path: Uri,
    ) -> Self {
        Self {
            feature,
            port,
            expr_engine,
            storage_resolver,
            kv_store,
            event_hub,
            output_path,
        }
    }

    pub fn as_context(&self) -> Context {
        Context {
            expr_engine: self.expr_engine.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            output_path: self.output_path.clone(),
        }
    }

    pub fn new_with_feature_and_port(&self, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            expr_engine: Arc::clone(&self.expr_engine),
            storage_resolver: Arc::clone(&self.storage_resolver),
            kv_store: Arc::clone(&self.kv_store),
            event_hub: self.event_hub.clone(),
            output_path: self.output_path.clone(),
        }
    }

    pub fn new_with_node_context_feature_and_port(
        ctx: &NodeContext,
        feature: Feature,
        port: Port,
    ) -> Self {
        Self {
            feature,
            port,
            expr_engine: Arc::clone(&ctx.expr_engine),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            kv_store: Arc::clone(&ctx.kv_store),
            event_hub: ctx.event_hub.clone(),
            output_path: ctx.output_path.clone(),
        }
    }

    pub fn new_with_context_feature_and_port(ctx: &Context, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            expr_engine: Arc::clone(&ctx.expr_engine),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            kv_store: Arc::clone(&ctx.kv_store),
            event_hub: ctx.event_hub.clone(),
            output_path: ctx.output_path.clone(),
        }
    }

    pub fn new_with_default_port(
        feature: Feature,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        output_path: Uri,
    ) -> Self {
        Self {
            feature,
            port: DEFAULT_PORT.clone(),
            expr_engine,
            storage_resolver,
            kv_store,
            event_hub,
            output_path,
        }
    }

    pub fn info_span(&self) -> tracing::Span {
        info_span!("action")
    }

    pub fn error_span(&self) -> tracing::Span {
        error_span!("action")
    }
}

#[derive(Debug, Clone)]
pub struct NodeContext {
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI. Tests using
    /// `NodeContext::default()` get `file:///` (permissive) — see `Default` impl.
    pub output_path: Uri,
}

impl From<Context> for NodeContext {
    fn from(ctx: Context) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            output_path: ctx.output_path,
        }
    }
}

impl From<ExecutorContext> for NodeContext {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            output_path: ctx.output_path,
        }
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            expr_engine: Arc::new(Engine::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            kv_store: Arc::new(crate::kvs::create_kv_store()),
            event_hub: EventHub::new(30),
            // Permissive sentinel: `file:///` accepts any file:// URI.
            // Only used by tests; production constructs the context explicitly.
            output_path: std::str::FromStr::from_str("file:///")
                .expect("'file:///' is always a valid URI"),
        }
    }
}

impl NodeContext {
    pub fn new(
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        output_path: Uri,
    ) -> Self {
        Self {
            expr_engine,
            storage_resolver,
            kv_store,
            event_hub,
            output_path,
        }
    }

    pub fn info_span(&self) -> tracing::Span {
        info_span!("action")
    }

    pub fn error_span(&self) -> tracing::Span {
        error_span!("action")
    }

    pub fn as_context(&self) -> Context {
        Context {
            expr_engine: self.expr_engine.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            output_path: self.output_path.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorOptions {
    pub channel_buffer_sz: usize,
    pub event_hub_capacity: usize,
    pub thread_pool_size: usize,
    pub feature_flush_threshold: usize,
    /// Per-job sandbox root for sink writes, wired from the worker's resolved
    /// `workerArtifactPath`. CLI callers set this too (Task 5). Tests and legacy
    /// callers that do not set it via the builder will get the permissive
    /// `file:///` sentinel through `ExecutorOptions::default()`.
    pub output_path: Uri,
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            channel_buffer_sz: 256,
            event_hub_capacity: 8192,
            thread_pool_size: 30,
            feature_flush_threshold: 512,
            // Permissive sentinel — same as NodeContext::default().
            // Production callers must override this with the real artifact URI.
            output_path: std::str::FromStr::from_str("file:///")
                .expect("'file:///' is always a valid URI"),
        }
    }
}

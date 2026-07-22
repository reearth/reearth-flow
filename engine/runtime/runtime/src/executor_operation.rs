use std::path::PathBuf;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use tracing::{error_span, info_span};

use crate::{
    event::EventHub,
    kvs::KvStore,
    node::{Port, FEATURES_PORT},
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
    pub env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI; production
    /// entrypoints (`Runner::run_with_sandbox_root`) reject the `file:///`
    /// sentinel. Tests using `NodeContext::default()` get `file:///`, which
    /// `sandbox::ensure_under` treats as "no sandbox" for any candidate scheme.
    pub sandbox_root: Uri,
}

impl From<ExecutorContext> for Context {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
        }
    }
}

impl From<NodeContext> for Context {
    fn from(ctx: NodeContext) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
        }
    }
}

impl Context {
    pub fn new(
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
        }
    }

    pub fn as_executor_context(&self, feature: Feature, port: Port) -> ExecutorContext {
        ExecutorContext {
            feature,
            port,
            env_vars: self.env_vars.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorContext {
    pub feature: Feature,
    pub port: Port,
    pub env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI; production
    /// entrypoints (`Runner::run_with_sandbox_root`) reject the `file:///`
    /// sentinel. Tests using `NodeContext::default()` get `file:///`, which
    /// `sandbox::ensure_under` treats as "no sandbox" for any candidate scheme.
    pub sandbox_root: Uri,
}

impl ExecutorContext {
    pub fn new(
        feature: Feature,
        port: Port,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            feature,
            port,
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
        }
    }

    pub fn as_context(&self) -> Context {
        Context {
            env_vars: self.env_vars.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
        }
    }

    pub fn new_with_feature_and_port(&self, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            env_vars: Arc::clone(&self.env_vars),
            storage_resolver: Arc::clone(&self.storage_resolver),
            kv_store: Arc::clone(&self.kv_store),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
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
            env_vars: Arc::clone(&ctx.env_vars),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            kv_store: Arc::clone(&ctx.kv_store),
            event_hub: ctx.event_hub.clone(),
            sandbox_root: ctx.sandbox_root.clone(),
        }
    }

    pub fn new_with_context_feature_and_port(ctx: &Context, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            env_vars: Arc::clone(&ctx.env_vars),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            kv_store: Arc::clone(&ctx.kv_store),
            event_hub: ctx.event_hub.clone(),
            sandbox_root: ctx.sandbox_root.clone(),
        }
    }

    pub fn new_with_features_port(
        feature: Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            feature,
            port: FEATURES_PORT.clone(),
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
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
    pub env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI; production
    /// entrypoints (`Runner::run_with_sandbox_root`) reject the `file:///`
    /// sentinel. Tests using `NodeContext::default()` get `file:///`, which
    /// `sandbox::ensure_under` treats as "no sandbox" for any candidate scheme.
    pub sandbox_root: Uri,
}

impl From<Context> for NodeContext {
    fn from(ctx: Context) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
        }
    }
}

impl From<ExecutorContext> for NodeContext {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
        }
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            env_vars: Arc::new(serde_json::Map::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            kv_store: Arc::new(crate::kvs::create_kv_store()),
            event_hub: EventHub::new(30),
            // Permissive sentinel: `file:///` is treated by
            // `sandbox::ensure_under` as "no sandbox" — any candidate URI,
            // regardless of scheme, passes. Only used by tests / the legacy
            // `Runner::run` path; production entrypoints reject this value.
            sandbox_root: std::str::FromStr::from_str("file:///")
                .expect("'file:///' is always a valid URI"),
        }
    }
}

impl NodeContext {
    pub fn new(
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
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
            env_vars: self.env_vars.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
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
    pub sandbox_root: Uri,
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
            sandbox_root: std::str::FromStr::from_str("file:///")
                .expect("'file:///' is always a valid URI"),
        }
    }
}

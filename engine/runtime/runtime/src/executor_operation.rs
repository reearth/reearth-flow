use std::sync::Arc;

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use tracing::{error_span, info_span};

use crate::{
    kvs::KvStore,
    node::{Port, DEFAULT_PORT},
};

#[derive(Clone, Debug)]
pub enum ExecutorOperation {
    Op { ctx: ExecutorContext },
    Terminate { ctx: NodeContext },
}

#[derive(Debug, Clone)]
pub struct Context {
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub logger: Arc<LoggerFactory>,
    pub kv_store: Arc<dyn KvStore>,
}

impl From<ExecutorContext> for Context {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            logger: ctx.logger,
            kv_store: ctx.kv_store,
        }
    }
}

impl From<NodeContext> for Context {
    fn from(ctx: NodeContext) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            logger: ctx.logger,
            kv_store: ctx.kv_store,
        }
    }
}

impl Context {
    pub fn new(
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: Arc<LoggerFactory>,
        kv_store: Arc<dyn KvStore>,
    ) -> Self {
        Self {
            expr_engine,
            storage_resolver,
            logger,
            kv_store,
        }
    }

    pub fn as_executor_context(&self, feature: Feature, port: Port) -> ExecutorContext {
        ExecutorContext {
            feature,
            port,
            expr_engine: self.expr_engine.clone(),
            storage_resolver: self.storage_resolver.clone(),
            logger: self.logger.clone(),
            kv_store: self.kv_store.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorContext {
    pub feature: Feature,
    pub port: Port,
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub logger: Arc<LoggerFactory>,
    pub kv_store: Arc<dyn KvStore>,
}

impl From<Context> for ExecutorContext {
    fn from(ctx: Context) -> Self {
        Self {
            feature: Feature::default(),
            port: DEFAULT_PORT.clone(),
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            logger: ctx.logger,
            kv_store: ctx.kv_store,
        }
    }
}

impl Default for ExecutorContext {
    fn default() -> Self {
        Self {
            feature: Feature::default(),
            port: DEFAULT_PORT.clone(),
            expr_engine: Arc::new(Engine::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            logger: Arc::new(LoggerFactory::new(
                reearth_flow_action_log::ActionLogger::root(
                    reearth_flow_action_log::Discard,
                    reearth_flow_action_log::o!(),
                ),
                Uri::for_test("ram:///log/").path(),
            )),
            kv_store: Arc::new(crate::kvs::create_kv_store()),
        }
    }
}

impl ExecutorContext {
    pub fn new(
        feature: Feature,
        port: Port,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: Arc<LoggerFactory>,
        kv_store: Arc<dyn KvStore>,
    ) -> Self {
        Self {
            feature,
            port,
            expr_engine,
            storage_resolver,
            logger,
            kv_store,
        }
    }

    pub fn as_context(&self) -> Context {
        Context {
            expr_engine: self.expr_engine.clone(),
            storage_resolver: self.storage_resolver.clone(),
            logger: self.logger.clone(),
            kv_store: self.kv_store.clone(),
        }
    }

    pub fn new_with_feature_and_port(&self, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            expr_engine: Arc::clone(&self.expr_engine),
            storage_resolver: Arc::clone(&self.storage_resolver),
            logger: Arc::clone(&self.logger),
            kv_store: Arc::clone(&self.kv_store),
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
            logger: Arc::clone(&ctx.logger),
            kv_store: Arc::clone(&ctx.kv_store),
        }
    }

    pub fn new_with_context_feature_and_port(ctx: &Context, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            expr_engine: Arc::clone(&ctx.expr_engine),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            logger: Arc::clone(&ctx.logger),
            kv_store: Arc::clone(&ctx.kv_store),
        }
    }

    pub fn new_with_default_port(
        feature: Feature,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: Arc<LoggerFactory>,
        kv_store: Arc<dyn KvStore>,
    ) -> Self {
        Self {
            feature,
            port: DEFAULT_PORT.clone(),
            expr_engine,
            storage_resolver,
            logger,
            kv_store,
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
    pub logger: Arc<LoggerFactory>,
    pub kv_store: Arc<dyn KvStore>,
}

impl From<Context> for NodeContext {
    fn from(ctx: Context) -> Self {
        Self {
            expr_engine: ctx.expr_engine,
            storage_resolver: ctx.storage_resolver,
            logger: ctx.logger,
            kv_store: ctx.kv_store,
        }
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            expr_engine: Arc::new(Engine::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            logger: Arc::new(LoggerFactory::new(
                reearth_flow_action_log::ActionLogger::root(
                    reearth_flow_action_log::Discard,
                    reearth_flow_action_log::o!(),
                ),
                Uri::for_test("ram:///log/").path(),
            )),
            kv_store: Arc::new(crate::kvs::create_kv_store()),
        }
    }
}

impl NodeContext {
    pub fn new(
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: Arc<LoggerFactory>,
        kv_store: Arc<dyn KvStore>,
    ) -> Self {
        Self {
            expr_engine,
            storage_resolver,
            logger,
            kv_store,
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
            logger: self.logger.clone(),
            kv_store: self.kv_store.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorOptions {
    pub channel_buffer_sz: usize,
    pub event_hub_capacity: usize,
    pub error_threshold: Option<u32>,
    pub thread_pool_size: usize,
    pub feature_flush_threshold: usize,
}

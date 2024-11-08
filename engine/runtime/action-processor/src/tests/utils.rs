use std::sync::Arc;

use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    event::EventHub,
    executor_operation::ExecutorContext,
    kvs,
    node::{Port, DEFAULT_PORT},
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;

#[derive(Debug, Clone)]
pub(crate) struct MockProcessorChannelForwarder {
    pub(crate) send_feature: Feature,
    pub(crate) send_port: Port,
}

impl Default for MockProcessorChannelForwarder {
    fn default() -> Self {
        Self {
            send_feature: Feature::default(),
            send_port: DEFAULT_PORT.clone(),
        }
    }
}

impl ProcessorChannelForwarder for MockProcessorChannelForwarder {
    fn send(&mut self, ctx: ExecutorContext) {
        self.send_feature = ctx.feature;
        self.send_port = ctx.port;
    }
}

pub(crate) fn create_default_execute_context(feature: &Feature) -> ExecutorContext {
    ExecutorContext::new(
        feature.clone(),
        DEFAULT_PORT.clone(),
        Arc::new(Engine::new()),
        Arc::new(StorageResolver::new()),
        Arc::new(kvs::create_kv_store()),
        EventHub::new(30),
    )
}

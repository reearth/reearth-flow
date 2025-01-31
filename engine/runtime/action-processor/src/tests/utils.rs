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
use tokio::runtime::Handle;

#[derive(Debug, Clone, Default)]
pub(crate) struct MockProcessorChannelForwarder {
    pub(crate) send_features: Vec<Feature>,
    pub(crate) send_ports: Vec<Port>,
}

impl ProcessorChannelForwarder for MockProcessorChannelForwarder {
    fn node_id(&self) -> String {
        "mock".to_string()
    }

    fn send(&mut self, ctx: ExecutorContext) {
        self.send_features.push(ctx.feature);
        self.send_ports.push(ctx.port);
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
        Arc::new(Handle::current()),
    )
}

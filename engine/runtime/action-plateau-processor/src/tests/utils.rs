use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    kvs::create_kv_store,
    node::DEFAULT_PORT,
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use std::sync::Arc;

pub(crate) fn create_default_execute_context(feature: Feature) -> ExecutorContext {
    ExecutorContext::new(
        feature,
        DEFAULT_PORT.clone(),
        Arc::new(Engine::new()),
        Arc::new(StorageResolver::new()),
        Arc::new(create_kv_store()),
        EventHub::new(30),
    )
}

pub(crate) fn create_default_node_context() -> NodeContext {
    let expr_engine = Arc::new(Engine::new());
    let storage_resolver = Arc::new(StorageResolver::new());
    let kv_store = Arc::new(create_kv_store());
    let event_hub = EventHub::new(1024);
    NodeContext::new(expr_engine, storage_resolver, kv_store, event_hub)
}

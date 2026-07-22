use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    kvs::create_kv_store,
    node::FEATURES_PORT,
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use std::sync::Arc;

pub(crate) fn create_default_execute_context(feature: Feature) -> ExecutorContext {
    ExecutorContext::new(
        feature,
        FEATURES_PORT.clone(),
        Arc::new(serde_json::Map::new()),
        Arc::new(StorageResolver::new()),
        Arc::new(create_kv_store()),
        EventHub::new(30),
        Uri::for_test("file:///"),
    )
}

pub(crate) fn create_default_node_context() -> NodeContext {
    let env_vars = Arc::new(serde_json::Map::new());
    let storage_resolver = Arc::new(StorageResolver::new());
    let kv_store = Arc::new(create_kv_store());
    let event_hub = EventHub::new(1024);
    NodeContext::new(
        env_vars,
        storage_resolver,
        kv_store,
        event_hub,
        Uri::for_test("file:///"),
    )
}

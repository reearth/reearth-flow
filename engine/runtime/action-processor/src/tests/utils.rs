use std::sync::Arc;

use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_runtime::{
    event::EventHub, executor_operation::ExecutorContext, kvs, node::DEFAULT_PORT,
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;

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

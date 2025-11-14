#[cfg(test)]
mod tests {
    use crate::executor_operation::{Context, ExecutorContext, ExecutorOperation, NodeContext};
    use crate::event::EventHub;
    use crate::kvs::create_kv_store;
    use crate::node::{Port, DEFAULT_PORT};
    use reearth_flow_eval_expr::engine::Engine;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};
    use std::sync::Arc;

    #[test]
    fn test_context_new() {
        let expr_engine = Arc::new(Engine::new());
        let storage_resolver = Arc::new(StorageResolver::new());
        let kv_store = Arc::new(create_kv_store());
        let event_hub = EventHub::new(30);
        
        let context = Context::new(
            expr_engine.clone(),
            storage_resolver.clone(),
            kv_store.clone(),
            event_hub.clone(),
        );
        
        assert!(Arc::ptr_eq(&context.expr_engine, &expr_engine));
        assert!(Arc::ptr_eq(&context.storage_resolver, &storage_resolver));
    }

    #[test]
    fn test_context_as_executor_context() {
        let context = Context::new(
            Arc::new(Engine::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(create_kv_store()),
            EventHub::new(30),
        );
        
        let feature = Feature::new();
        let port = DEFAULT_PORT.clone();
        
        let exec_ctx = context.as_executor_context(feature.clone(), port.clone());
        
        assert_eq!(exec_ctx.feature.id, feature.id);
        assert_eq!(exec_ctx.port, port);
    }

    #[test]
    fn test_executor_context_default() {
        let ctx = ExecutorContext::default();
        
        assert!(ctx.feature.attributes.is_empty());
        assert_eq!(ctx.port, DEFAULT_PORT.clone());
    }

    #[test]
    fn test_executor_context_new() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("test"),
            AttributeValue::String("value".to_string()),
        );
        
        let port = Port::new("custom");
        let ctx = ExecutorContext::new(
            feature.clone(),
            port.clone(),
            Arc::new(Engine::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(create_kv_store()),
            EventHub::new(30),
        );
        
        assert_eq!(ctx.feature.id, feature.id);
        assert_eq!(ctx.port, port);
    }

    #[test]
    fn test_executor_context_as_context() {
        let exec_ctx = ExecutorContext::default();
        let context = exec_ctx.as_context();
        
        assert!(Arc::ptr_eq(&context.expr_engine, &exec_ctx.expr_engine));
        assert!(Arc::ptr_eq(&context.storage_resolver, &exec_ctx.storage_resolver));
    }

    #[test]
    fn test_executor_context_new_with_feature_and_port() {
        let ctx1 = ExecutorContext::default();
        
        let mut new_feature = Feature::new();
        new_feature.attributes.insert(
            Attribute::new("key"),
            AttributeValue::String("value".to_string()),
        );
        let new_port = Port::new("output");
        
        let ctx2 = ctx1.new_with_feature_and_port(new_feature.clone(), new_port.clone());
        
        assert_eq!(ctx2.feature.id, new_feature.id);
        assert_eq!(ctx2.port, new_port);
        assert!(Arc::ptr_eq(&ctx1.expr_engine, &ctx2.expr_engine));
    }

    #[test]
    fn test_executor_context_new_with_node_context_feature_and_port() {
        let node_ctx = NodeContext::default();
        let feature = Feature::new();
        let port = Port::new("test");
        
        let exec_ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &node_ctx,
            feature.clone(),
            port.clone(),
        );
        
        assert_eq!(exec_ctx.feature.id, feature.id);
        assert_eq!(exec_ctx.port, port);
    }

    #[test]
    fn test_node_context_default() {
        let ctx = NodeContext::default();
        
        assert!(!Arc::ptr_eq(&ctx.expr_engine, &Arc::new(Engine::new())));
    }

    #[test]
    fn test_node_context_new() {
        let expr_engine = Arc::new(Engine::new());
        let storage_resolver = Arc::new(StorageResolver::new());
        let kv_store = Arc::new(create_kv_store());
        let event_hub = EventHub::new(30);
        
        let ctx = NodeContext::new(
            expr_engine.clone(),
            storage_resolver.clone(),
            kv_store.clone(),
            event_hub.clone(),
        );
        
        assert!(Arc::ptr_eq(&ctx.expr_engine, &expr_engine));
        assert!(Arc::ptr_eq(&ctx.storage_resolver, &storage_resolver));
    }

    #[test]
    fn test_node_context_as_context() {
        let node_ctx = NodeContext::default();
        let context = node_ctx.as_context();
        
        assert!(Arc::ptr_eq(&context.expr_engine, &node_ctx.expr_engine));
        assert!(Arc::ptr_eq(&context.storage_resolver, &node_ctx.storage_resolver));
    }

    #[test]
    fn test_executor_context_from_context() {
        let context = Context::new(
            Arc::new(Engine::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(create_kv_store()),
            EventHub::new(30),
        );
        
        let exec_ctx = ExecutorContext::from(context.clone());
        
        assert!(Arc::ptr_eq(&exec_ctx.expr_engine, &context.expr_engine));
        assert_eq!(exec_ctx.port, DEFAULT_PORT.clone());
    }

    #[test]
    fn test_node_context_from_context() {
        let context = Context::new(
            Arc::new(Engine::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(create_kv_store()),
            EventHub::new(30),
        );
        
        let node_ctx = NodeContext::from(context.clone());
        
        assert!(Arc::ptr_eq(&node_ctx.expr_engine, &context.expr_engine));
    }

    #[test]
    fn test_node_context_from_executor_context() {
        let exec_ctx = ExecutorContext::default();
        let node_ctx = NodeContext::from(exec_ctx.clone());
        
        assert!(Arc::ptr_eq(&node_ctx.expr_engine, &exec_ctx.expr_engine));
    }

    #[test]
    fn test_context_from_executor_context() {
        let exec_ctx = ExecutorContext::default();
        let context = Context::from(exec_ctx.clone());
        
        assert!(Arc::ptr_eq(&context.expr_engine, &exec_ctx.expr_engine));
    }

    #[test]
    fn test_context_from_node_context() {
        let node_ctx = NodeContext::default();
        let context = Context::from(node_ctx.clone());
        
        assert!(Arc::ptr_eq(&context.expr_engine, &node_ctx.expr_engine));
    }

    #[test]
    fn test_executor_operation_op() {
        let exec_ctx = ExecutorContext::default();
        let op = ExecutorOperation::Op { ctx: exec_ctx.clone() };
        
        match op {
            ExecutorOperation::Op { ctx } => {
                assert_eq!(ctx.feature.id, exec_ctx.feature.id);
            }
            _ => panic!("Expected Op variant"),
        }
    }

    #[test]
    fn test_executor_operation_terminate() {
        let node_ctx = NodeContext::default();
        let op = ExecutorOperation::Terminate { ctx: node_ctx.clone() };
        
        match op {
            ExecutorOperation::Terminate { ctx } => {
                assert!(Arc::ptr_eq(&ctx.expr_engine, &node_ctx.expr_engine));
            }
            _ => panic!("Expected Terminate variant"),
        }
    }

    #[test]
    fn test_executor_context_with_custom_feature() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("name"),
            AttributeValue::String("test_feature".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("count"),
            AttributeValue::Number(serde_json::Number::from(42)),
        );
        
        let ctx = ExecutorContext::new(
            feature.clone(),
            DEFAULT_PORT.clone(),
            Arc::new(Engine::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(create_kv_store()),
            EventHub::new(30),
        );
        
        assert_eq!(ctx.feature.attributes.len(), 2);
        assert_eq!(
            ctx.feature.get(&"name"),
            Some(&AttributeValue::String("test_feature".to_string()))
        );
    }

    #[test]
    fn test_executor_context_new_with_default_port() {
        let feature = Feature::new();
        let ctx = ExecutorContext::new_with_default_port(
            feature.clone(),
            Arc::new(Engine::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(create_kv_store()),
            EventHub::new(30),
        );
        
        assert_eq!(ctx.port, DEFAULT_PORT.clone());
        assert_eq!(ctx.feature.id, feature.id);
    }

    #[test]
    fn test_executor_context_info_span() {
        let ctx = ExecutorContext::default();
        let span = ctx.info_span();
        
        assert_eq!(span.metadata().map(|m| m.name()), Some("action"));
    }

    #[test]
    fn test_executor_context_error_span() {
        let ctx = ExecutorContext::default();
        let span = ctx.error_span();
        
        assert_eq!(span.metadata().map(|m| m.name()), Some("action"));
    }
}


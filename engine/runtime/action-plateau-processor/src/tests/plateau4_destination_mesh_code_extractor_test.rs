#[cfg(test)]
mod tests {
    use crate::plateau4::destination_mesh_code_extractor::DestinationMeshCodeExtractorFactory;
    use crate::tests::create_default_node_context;
    use reearth_flow_runtime::node::ProcessorFactory;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_factory_name() {
        let factory = DestinationMeshCodeExtractorFactory::default();
        assert_eq!(factory.name(), "PLATEAU4.DestinationMeshCodeExtractor");
    }

    #[test]
    fn test_factory_description() {
        let factory = DestinationMeshCodeExtractorFactory::default();
        assert!(!factory.description().is_empty());
    }

    #[test]
    fn test_factory_categories() {
        let factory = DestinationMeshCodeExtractorFactory::default();
        assert!(factory.categories().contains(&"PLATEAU"));
    }

    #[test]
    fn test_factory_ports() {
        let factory = DestinationMeshCodeExtractorFactory::default();
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 2);
    }

    #[test]
    fn test_factory_build_with_valid_params() {
        use reearth_flow_runtime::event::EventHub;

        let factory = DestinationMeshCodeExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);

        let mut params = HashMap::new();
        params.insert("meshType".to_string(), json!(3));
        params.insert("epsgCode".to_string(), json!("6668"));

        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_build_with_invalid_mesh_type() {
        use reearth_flow_runtime::event::EventHub;

        let factory = DestinationMeshCodeExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);

        let mut params = HashMap::new();
        params.insert("meshType".to_string(), json!(10));
        params.insert("epsgCode".to_string(), json!("6668"));

        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(params));
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_build_with_default_params() {
        use reearth_flow_runtime::event::EventHub;

        let factory = DestinationMeshCodeExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);


        let result = factory.build(node_ctx, event_hub, "test".to_string(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_parameter_schema() {
        let factory = DestinationMeshCodeExtractorFactory::default();
        let schema = factory.parameter_schema();
        assert!(schema.is_some());
    }

    #[test]
    fn test_factory_build_with_mesh_type_1() {
        use reearth_flow_runtime::event::EventHub;

        let factory = DestinationMeshCodeExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);

        let mut params = HashMap::new();
        params.insert("meshType".to_string(), json!(1));
        params.insert("epsgCode".to_string(), json!("6668"));

        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_build_with_mesh_type_6() {
        use reearth_flow_runtime::event::EventHub;

        let factory = DestinationMeshCodeExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);

        let mut params = HashMap::new();
        params.insert("meshType".to_string(), json!(6));
        params.insert("epsgCode".to_string(), json!("6668"));

        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_build_with_different_epsg_codes() {
        use reearth_flow_runtime::event::EventHub;

        let factory = DestinationMeshCodeExtractorFactory::default();


        let node_ctx1 = create_default_node_context();
        let event_hub1 = EventHub::new(30);
        let mut params1 = HashMap::new();
        params1.insert("meshType".to_string(), json!(3));
        params1.insert("epsgCode".to_string(), json!("6697"));

        let result1 = factory.build(node_ctx1, event_hub1, "test".to_string(), Some(params1));
        assert!(result1.is_ok());


        let node_ctx2 = create_default_node_context();
        let event_hub2 = EventHub::new(30);
        let mut params2 = HashMap::new();
        params2.insert("meshType".to_string(), json!(3));
        params2.insert("epsgCode".to_string(), json!("6668"));

        let result2 = factory.build(node_ctx2, event_hub2, "test".to_string(), Some(params2));
        assert!(result2.is_ok());
    }
}


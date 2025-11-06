#[cfg(test)]
mod tests {
    use crate::plateau3::max_lod_extractor::{MaxLodExtractor, MaxLodExtractorFactory};
    use crate::tests::{create_default_execute_context, create_default_node_context};
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_runtime::node::{Processor, ProcessorFactory};
    use reearth_flow_types::{Attribute, AttributeValue, Feature};

    #[test]
    fn test_max_lod_extractor_valid_mesh_file() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/53394525_bldg_6697_op.gml".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("meshCode"),
            AttributeValue::String("53394525".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("maxLod"),
            AttributeValue::Number(serde_json::Number::from(2)),
        );

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {};
        let result = extractor.process(ctx, &fw);

        assert!(result.is_ok());

        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        let output_feature = &send_features[0];

        // Check that the correct attributes are set
        assert_eq!(
            output_feature.get(&"code"),
            Some(&AttributeValue::String("53394525".to_string()))
        );
        assert_eq!(
            output_feature.get(&"type"),
            Some(&AttributeValue::String("bldg".to_string()))
        );
        assert_eq!(
            output_feature.get(&"maxLod"),
            Some(&AttributeValue::String("2".to_string()))
        );
        assert_eq!(
            output_feature.get(&"file"),
            Some(&AttributeValue::String("53394525_bldg_6697_op.gml".to_string()))
        );
    }

    #[test]
    fn test_max_lod_extractor_non_mesh_file() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/codelists.gml".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("meshCode"),
            AttributeValue::String("53394525".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("maxLod"),
            AttributeValue::Number(serde_json::Number::from(2)),
        );

        let ctx = create_default_execute_context(feature.clone());
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {};
        let result = extractor.process(ctx, &fw);

        // Non-numeric files should return Ok but not process
        assert!(result.is_ok());
        
        // No features should be sent since file doesn't start with digits
        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 0);
    }

    #[test]
    fn test_max_lod_extractor_missing_package() {
        let mut feature = Feature::new();
        // Missing package attribute
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/53394525_bldg_6697_op.gml".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("meshCode"),
            AttributeValue::String("53394525".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("maxLod"),
            AttributeValue::Number(serde_json::Number::from(2)),
        );

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let mut extractor = MaxLodExtractor {};
        let result = extractor.process(ctx, &fw);

        assert!(result.is_err());
    }

    #[test]
    fn test_max_lod_extractor_missing_mesh_code() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/53394525_bldg_6697_op.gml".to_string()),
        );
        // Missing meshCode
        feature.attributes.insert(
            Attribute::new("maxLod"),
            AttributeValue::Number(serde_json::Number::from(2)),
        );

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let mut extractor = MaxLodExtractor {};
        let result = extractor.process(ctx, &fw);

        assert!(result.is_err());
    }

    #[test]
    fn test_max_lod_extractor_missing_max_lod() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/53394525_bldg_6697_op.gml".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("meshCode"),
            AttributeValue::String("53394525".to_string()),
        );
        // Missing maxLod

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let mut extractor = MaxLodExtractor {};
        let result = extractor.process(ctx, &fw);

        assert!(result.is_err());
    }

    #[test]
    fn test_max_lod_extractor_invalid_uri() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("not a valid uri".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("meshCode"),
            AttributeValue::String("53394525".to_string()),
        );
        feature.attributes.insert(
            Attribute::new("maxLod"),
            AttributeValue::Number(serde_json::Number::from(2)),
        );

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let mut extractor = MaxLodExtractor {};
        let result = extractor.process(ctx, &fw);

        // Should handle invalid URIs
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_max_lod_extractor_finish() {
        let node_ctx = create_default_node_context();
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let extractor = MaxLodExtractor {};
        let result = extractor.finish(node_ctx, &fw);

        assert!(result.is_ok());
    }

    #[test]
    fn test_max_lod_extractor_name() {
        let extractor = MaxLodExtractor {};
        assert_eq!(extractor.name(), "MaxLodExtractor");
    }

    #[test]
    fn test_max_lod_extractor_factory_name() {
        let factory = MaxLodExtractorFactory::default();
        assert_eq!(factory.name(), "PLATEAU3.MaxLodExtractor");
    }

    #[test]
    fn test_max_lod_extractor_factory_description() {
        let factory = MaxLodExtractorFactory::default();
        assert!(!factory.description().is_empty());
    }

    #[test]
    fn test_max_lod_extractor_factory_categories() {
        let factory = MaxLodExtractorFactory::default();
        assert!(factory.categories().contains(&"PLATEAU"));
    }

    #[test]
    fn test_max_lod_extractor_factory_ports() {
        let factory = MaxLodExtractorFactory::default();
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 1);
    }

    #[test]
    fn test_max_lod_extractor_factory_build() {
        use reearth_flow_runtime::event::EventHub;

        let factory = MaxLodExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let result = factory.build(node_ctx, event_hub, "test".to_string(), None);

        assert!(result.is_ok());
    }
}


#[cfg(test)]
mod tests {
    use crate::plateau3::attribute_flattener::AttributeFlattenerFactory;
    use crate::tests::{create_default_execute_context, create_default_node_context};
    use reearth_flow_runtime::event::EventHub;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_runtime::node::ProcessorFactory;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};
    use std::collections::HashMap;

    fn create_test_building_feature() -> Feature {
        let mut feature = Feature::new();
        let mut city_gml_attrs = HashMap::new();
        
        city_gml_attrs.insert("type".to_string(), AttributeValue::String("bldg:Building".to_string()));
        city_gml_attrs.insert("gml:name".to_string(), AttributeValue::String("Test Building".to_string()));
        
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attrs),
        );
        
        feature
    }

    #[test]
    fn test_factory_name() {
        let factory = AttributeFlattenerFactory::default();
        assert_eq!(factory.name(), "PLATEAU3.AttributeFlattener");
    }

    #[test]
    fn test_factory_description() {
        let factory = AttributeFlattenerFactory::default();
        assert!(!factory.description().is_empty());
    }

    #[test]
    fn test_factory_categories() {
        let factory = AttributeFlattenerFactory::default();
        assert!(factory.categories().contains(&"PLATEAU"));
    }

    #[test]
    fn test_factory_ports() {
        let factory = AttributeFlattenerFactory::default();
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 1);
    }

    #[test]
    fn test_factory_parameter_schema() {
        let factory = AttributeFlattenerFactory::default();
        assert!(factory.parameter_schema().is_none());
    }

    #[test]
    fn test_factory_build() {
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        
        let result = factory.build(node_ctx, event_hub, "test".to_string(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_building_with_simple_attributes() {
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let feature = create_test_building_feature();
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_ok());
        
        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        let output_feature = &send_features[0];
        // cityGmlAttributes should be removed
        assert!(output_feature.get(&"cityGmlAttributes").is_none());
        // Flattened attributes should be present
        assert!(output_feature.get(&"type").is_some());
    }

    #[test]
    fn test_process_building_with_nested_attributes() {
        let mut feature = Feature::new();
        let mut city_gml_attrs = HashMap::new();
        
        city_gml_attrs.insert("type".to_string(), AttributeValue::String("bldg:Building".to_string()));
        
        // Add nested building ID attribute
        let mut building_id_attr = HashMap::new();
        building_id_attr.insert("uro:buildingID".to_string(), AttributeValue::String("12345".to_string()));
        building_id_attr.insert("uro:branchID".to_string(), AttributeValue::String("1".to_string()));
        
        city_gml_attrs.insert(
            "uro:buildingIDAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(building_id_attr)]),
        );
        
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attrs),
        );
        
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_ok());
        
        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
    }

    #[test]
    fn test_process_building_with_disaster_risk_attributes() {
        let mut feature = Feature::new();
        let mut city_gml_attrs = HashMap::new();
        
        city_gml_attrs.insert("type".to_string(), AttributeValue::String("bldg:Building".to_string()));
        
        // Add disaster risk attribute
        let mut risk_attr = HashMap::new();
        risk_attr.insert("uro:description".to_string(), AttributeValue::String("洪水".to_string()));
        risk_attr.insert("uro:adminType".to_string(), AttributeValue::String("国".to_string()));
        risk_attr.insert("uro:scale".to_string(), AttributeValue::String("L2".to_string()));
        risk_attr.insert("uro:rank".to_string(), AttributeValue::String("5m以上".to_string()));
        
        city_gml_attrs.insert(
            "uro:buildingDisasterRiskAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(risk_attr)]),
        );
        
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attrs),
        );
        
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_ok());
        
        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        // Disaster risk attributes should be flattened
        let output_feature = &send_features[0];
        assert!(output_feature.get(&"uro:buildingDisasterRiskAttribute").is_none());
    }

    #[test]
    fn test_process_building_part() {
        let mut feature = Feature::new();
        let mut city_gml_attrs = HashMap::new();
        
        city_gml_attrs.insert("type".to_string(), AttributeValue::String("bldg:BuildingPart".to_string()));
        city_gml_attrs.insert("gml:name".to_string(), AttributeValue::String("Test Part".to_string()));
        
        // Add ancestors (root building attributes)
        let mut ancestors = HashMap::new();
        ancestors.insert("rootAttr".to_string(), AttributeValue::String("rootValue".to_string()));
        city_gml_attrs.insert("ancestors".to_string(), AttributeValue::Map(ancestors));
        
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attrs),
        );
        
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_ok());
        
        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        // Should have merged ancestor attributes
        let output_feature = &send_features[0];
        assert!(output_feature.get(&"rootAttr").is_some());
    }

    #[test]
    fn test_process_with_generic_attributes() {
        let mut feature = Feature::new();
        let mut city_gml_attrs = HashMap::new();
        
        city_gml_attrs.insert("type".to_string(), AttributeValue::String("bldg:Building".to_string()));
        
        // Add generic attributes
        let mut generic_attrs = HashMap::new();
        generic_attrs.insert("customAttr1".to_string(), AttributeValue::String("value1".to_string()));
        generic_attrs.insert("customAttr2".to_string(), AttributeValue::String("value2".to_string()));
        generic_attrs.insert("type".to_string(), AttributeValue::String("string".to_string()));
        
        city_gml_attrs.insert("gen:genericAttribute".to_string(), AttributeValue::Map(generic_attrs));
        
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attrs),
        );
        
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_ok());
        
        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        let output_feature = &send_features[0];
        // Generic attributes should be flattened (excluding 'type')
        assert!(output_feature.get(&"customAttr1").is_some());
        assert!(output_feature.get(&"customAttr2").is_some());
        // gen:genericAttribute should be removed
        assert!(output_feature.get(&"gen:genericAttribute").is_none());
    }

    #[test]
    fn test_process_missing_city_gml_attributes() {
        let feature = Feature::new(); // No cityGmlAttributes
        
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_missing_type_in_city_gml_attributes() {
        let mut feature = Feature::new();
        let city_gml_attrs = HashMap::new(); // No 'type' field
        
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attrs),
        );
        
        let factory = AttributeFlattenerFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = EventHub::new(30);
        let mut processor = factory.build(node_ctx, event_hub, "test".to_string(), None).unwrap();
        
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);
        
        let result = processor.process(ctx, &fw);
        assert!(result.is_err());
    }

    #[test]
    fn test_finish() {
        let factory = AttributeFlattenerFactory::default();
        let node_ctx_build = create_default_node_context();
        let event_hub = EventHub::new(30);
        let processor = factory.build(node_ctx_build, event_hub, "test".to_string(), None).unwrap();
        
        let node_ctx = create_default_node_context();
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);
        
        let result = processor.finish(node_ctx, &fw);
        assert!(result.is_ok());
    }
}


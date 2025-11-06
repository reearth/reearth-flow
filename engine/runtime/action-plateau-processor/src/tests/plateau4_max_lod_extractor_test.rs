#[cfg(test)]
mod tests {
    use crate::plateau4::max_lod_extractor::{MaxLodExtractor, MaxLodExtractorFactory};
    use crate::tests::{create_default_execute_context, create_default_node_context};
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_runtime::node::{Processor, ProcessorFactory};
    use reearth_flow_types::lod::LodMask;
    use reearth_flow_types::metadata::Metadata;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};
    use std::collections::HashMap;

    #[test]
    fn test_max_lod_extractor_with_single_feature() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building.gml".to_string()),
        );
        let mut lod = LodMask::default();
        lod.add_lod(2);
        feature.metadata = Metadata {
            lod: Some(lod),
            ..Default::default()
        };

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };

        let result = extractor.process(ctx, &fw);
        assert!(result.is_ok());


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 0);
    }

    #[test]
    fn test_max_lod_extractor_finish_flushes_buffer() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building.gml".to_string()),
        );
        let mut lod = LodMask::default();
        lod.add_lod(2);
        feature.metadata = Metadata {
            lod: Some(lod),
            ..Default::default()
        };

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };

        extractor.process(ctx, &fw).unwrap();


        let node_ctx = create_default_node_context();
        let result = extractor.finish(node_ctx, &fw);
        assert!(result.is_ok());


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        let output_feature = &send_features[0];
        assert_eq!(
            output_feature.get(&"maxLod"),
            Some(&AttributeValue::Number(serde_json::Number::from(2)))
        );
    }

    #[test]
    fn test_max_lod_extractor_multiple_features_same_file() {
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };


        let mut feature1 = Feature::new();
        feature1.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building.gml".to_string()),
        );
        let mut lod1 = LodMask::default();
        lod1.add_lod(1);
        feature1.metadata = Metadata {
            lod: Some(lod1),
            ..Default::default()
        };
        let ctx1 = create_default_execute_context(feature1);
        extractor.process(ctx1, &fw).unwrap();


        let mut feature2 = Feature::new();
        feature2.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building.gml".to_string()),
        );
        let mut lod2 = LodMask::default();
        lod2.add_lod(3);
        feature2.metadata = Metadata {
            lod: Some(lod2),
            ..Default::default()
        };
        let ctx2 = create_default_execute_context(feature2);
        extractor.process(ctx2, &fw).unwrap();


        let node_ctx = create_default_node_context();
        extractor.finish(node_ctx, &fw).unwrap();


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        let output_feature = &send_features[0];
        assert_eq!(
            output_feature.get(&"maxLod"),
            Some(&AttributeValue::Number(serde_json::Number::from(3)))
        );
    }

    #[test]
    fn test_max_lod_extractor_different_files_flush() {
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };


        let mut feature1 = Feature::new();
        feature1.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building1.gml".to_string()),
        );
        let mut lod1 = LodMask::default();
        lod1.add_lod(2);
        feature1.metadata = Metadata {
            lod: Some(lod1),
            ..Default::default()
        };
        let ctx1 = create_default_execute_context(feature1);
        extractor.process(ctx1, &fw).unwrap();


        let mut feature2 = Feature::new();
        feature2.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building2.gml".to_string()),
        );
        let mut lod2 = LodMask::default();
        lod2.add_lod(1);
        feature2.metadata = Metadata {
            lod: Some(lod2),
            ..Default::default()
        };
        let ctx2 = create_default_execute_context(feature2);
        extractor.process(ctx2, &fw).unwrap();


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
    }

    #[test]
    fn test_max_lod_extractor_missing_city_gml_path() {
        let feature = Feature::new();
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };

        let result = extractor.process(ctx, &fw);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_lod_extractor_missing_lod_metadata() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building.gml".to_string()),
        );


        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };

        let result = extractor.process(ctx, &fw);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_lod_extractor_with_range_lod() {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String("file:///path/to/building.gml".to_string()),
        );
        let mut lod = LodMask::default();
        lod.add_lod(1);
        lod.add_lod(2);
        lod.add_lod(3);
        feature.metadata = Metadata {
            lod: Some(lod),
            ..Default::default()
        };

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };

        extractor.process(ctx, &fw).unwrap();


        let node_ctx = create_default_node_context();
        extractor.finish(node_ctx, &fw).unwrap();


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
        
        let output_feature = &send_features[0];
        assert_eq!(
            output_feature.get(&"maxLod"),
            Some(&AttributeValue::Number(serde_json::Number::from(3)))
        );
    }

    #[test]
    fn test_max_lod_extractor_name() {
        let extractor = MaxLodExtractor {
            city_gml_path_attribute: Attribute::new("cityGmlPath"),
            max_lod_attribute: Attribute::new("maxLod"),
            buffer: HashMap::new(),
        };
        assert_eq!(extractor.name(), "MaxLodExtractor");
    }

    #[test]
    fn test_max_lod_extractor_factory_name() {
        let factory = MaxLodExtractorFactory::default();
        assert_eq!(factory.name(), "PLATEAU4.MaxLodExtractor");
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
}


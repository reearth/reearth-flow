#[cfg(test)]
mod tests {
    use crate::plateau3::building_usage_attribute_validator::{
        BuildingUsageAttributeValidator, BuildingUsageAttributeValidatorFactory, CODE_ERROR_PORT,
        L_BLDG_ERROR_PORT,
    };
    use crate::tests::{create_default_execute_context, create_default_node_context};
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_runtime::node::{Processor, ProcessorFactory};
    use reearth_flow_types::{Attribute, AttributeValue, Feature};
    use std::collections::HashMap;

    fn create_test_feature_with_usage_attributes(
        usage_attributes: HashMap<String, AttributeValue>,
    ) -> Feature {
        let mut feature = Feature::new();
        let mut city_gml_attributes = HashMap::new();
        for (key, value) in usage_attributes {
            city_gml_attributes.insert(key, value);
        }
        feature.attributes.insert(
            Attribute::new("cityGmlAttributes"),
            AttributeValue::Map(city_gml_attributes),
        );
        feature
    }

    #[test]
    fn test_validator_with_missing_required_attribute() {
        let mut usage_map = HashMap::new();

        usage_map.insert(
            "uro:majorUsage2".to_string(),
            AttributeValue::String("residential".to_string()),
        );
        

        let mut detail_attr = HashMap::new();
        detail_attr.insert(
            "uro:surveyYear".to_string(),
            AttributeValue::String("2023".to_string()),
        );
        usage_map.insert(
            "uro:buildingDetailAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(detail_attr)]),
        );

        let feature = create_test_feature_with_usage_attributes(usage_map);
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok());


        let send_ports = noop_forwarder.send_ports.lock().unwrap();
        assert!(send_ports.contains(&L_BLDG_ERROR_PORT.clone()));
    }

    #[test]
    fn test_validator_with_all_required_attributes() {
        let mut usage_map = HashMap::new();
        usage_map.insert(
            "uro:majorUsage2".to_string(),
            AttributeValue::String("residential".to_string()),
        );
        usage_map.insert(
            "uro:majorUsage".to_string(),
            AttributeValue::String("commercial".to_string()),
        );

        let feature = create_test_feature_with_usage_attributes(usage_map);
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok());


        let send_ports = noop_forwarder.send_ports.lock().unwrap();
        assert_eq!(send_ports.len(), 1);
    }

    #[test]
    fn test_validator_with_major_city_code_error() {
        let mut usage_map = HashMap::new();
        
        let mut building_id_attr = HashMap::new();
        building_id_attr.insert(
            "uro:city".to_string(),
            AttributeValue::String("札幌市".to_string()),
        );
        usage_map.insert(
            "uro:buildingIDAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(building_id_attr)]),
        );

        let feature = create_test_feature_with_usage_attributes(usage_map);
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());


        let mut city_name_to_code = HashMap::new();
        city_name_to_code.insert("札幌市".to_string(), "01100".to_string());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code,
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok());


        let send_ports = noop_forwarder.send_ports.lock().unwrap();
        assert!(send_ports.contains(&CODE_ERROR_PORT.clone()));
    }

    #[test]
    fn test_validator_with_non_major_city_code() {
        let mut usage_map = HashMap::new();
        
        let mut building_id_attr = HashMap::new();
        building_id_attr.insert(
            "uro:city".to_string(),
            AttributeValue::String("筑波市".to_string()),
        );
        usage_map.insert(
            "uro:buildingIDAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(building_id_attr)]),
        );

        let feature = create_test_feature_with_usage_attributes(usage_map);
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());


        let mut city_name_to_code = HashMap::new();
        city_name_to_code.insert("筑波市".to_string(), "08220".to_string());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code,
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok());


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert_eq!(send_features.len(), 1);
    }

    #[test]
    fn test_validator_missing_city_gml_attributes() {
        let feature = Feature::new();
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_name() {
        let validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };
        assert_eq!(validator.name(), "BuildingUsageAttributeValidator");
    }

    #[test]
    fn test_validator_finish() {
        let validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };
        let node_ctx = create_default_node_context();
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let result = validator.finish(node_ctx, &fw);
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory_name() {
        let factory = BuildingUsageAttributeValidatorFactory::default();
        assert_eq!(factory.name(), "PLATEAU3.BuildingUsageAttributeValidator");
    }

    #[test]
    fn test_factory_description() {
        let factory = BuildingUsageAttributeValidatorFactory::default();
        assert!(!factory.description().is_empty());
    }

    #[test]
    fn test_factory_categories() {
        let factory = BuildingUsageAttributeValidatorFactory::default();
        assert!(factory.categories().contains(&"PLATEAU"));
    }

    #[test]
    fn test_factory_ports() {
        let factory = BuildingUsageAttributeValidatorFactory::default();
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 3);
    }

    #[test]
    fn test_validator_with_city_not_in_codelist() {
        let mut usage_map = HashMap::new();
        
        let mut building_id_attr = HashMap::new();
        building_id_attr.insert(
            "uro:city".to_string(),
            AttributeValue::String("未知の市".to_string()),
        );
        usage_map.insert(
            "uro:buildingIDAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(building_id_attr)]),
        );

        let feature = create_test_feature_with_usage_attributes(usage_map);
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok());


        let send_ports = noop_forwarder.send_ports.lock().unwrap();
        assert!(send_ports.contains(&CODE_ERROR_PORT.clone()));
    }

    #[test]
    fn test_validator_multiple_missing_attributes() {
        let mut usage_map = HashMap::new();

        usage_map.insert(
            "uro:detailedUsage2".to_string(),
            AttributeValue::String("type1".to_string()),
        );
        usage_map.insert(
            "uro:secondFloorUsage".to_string(),
            AttributeValue::String("type2".to_string()),
        );
        
        let mut detail_attr = HashMap::new();
        detail_attr.insert(
            "uro:surveyYear".to_string(),
            AttributeValue::String("2023".to_string()),
        );
        usage_map.insert(
            "uro:buildingDetailAttribute".to_string(),
            AttributeValue::Array(vec![AttributeValue::Map(detail_attr)]),
        );

        let feature = create_test_feature_with_usage_attributes(usage_map);
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        let mut validator = BuildingUsageAttributeValidator {
            city_name_to_code: HashMap::new(),
        };

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok());


        let send_features = noop_forwarder.send_features.lock().unwrap();
        assert!(!send_features.is_empty());
        
        let output_feature = &send_features[0];
        let errors = output_feature.get(&"errors");
        

        if let Some(AttributeValue::Array(error_array)) = errors {
            assert!(error_array.len() >= 2);
        }
    }
}


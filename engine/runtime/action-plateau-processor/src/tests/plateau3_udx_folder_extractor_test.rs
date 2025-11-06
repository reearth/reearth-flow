#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, path::Path};

    use once_cell::sync::Lazy;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_runtime::node::{Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT};
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
    use serde_json::json;

    use crate::plateau3::udx_folder_extractor::{
        mapper, UdxFolderExtractorFactory, UdxFolderExtractorParam,
    };
    use crate::tests::{create_default_execute_context, create_default_node_context};

    static CITY_GML_PATH_EXPR: Lazy<Expr> = Lazy::new(|| Expr::new("cityGmlPath".to_string()));

    fn build_factory_processor(with: HashMap<String, serde_json::Value>) -> Box<dyn Processor> {
        let factory = UdxFolderExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = reearth_flow_runtime::event::EventHub::new(30);
        factory
            .build(node_ctx, event_hub, "test".to_string(), Some(with))
            .expect("processor should build")
    }

    fn create_feature(path: &str, extra: HashMap<&str, AttributeValue>) -> Feature {
        let mut feature = Feature::new();
        feature.attributes.insert(
            Attribute::new("cityGmlPath"),
            AttributeValue::String(path.to_string()),
        );
        for (k, v) in extra {
            feature.attributes.insert(Attribute::new(k), v);
        }
        feature
    }

    fn create_temp_dir() -> (tempfile::TempDir, String) {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let path = tmp.path().to_string_lossy().to_string();
        (tmp, format!("file://{path}"))
    }

    #[test]
    fn test_factory_metadata_matches_definition() {
        let factory = UdxFolderExtractorFactory::default();
        assert_eq!(factory.name(), "PLATEAU3.UDXFolderExtractor");
        assert!(factory.description().contains("UDX folders"));
        assert!(factory.categories().contains(&"PLATEAU"));
        assert_eq!(factory.get_input_ports(), vec![DEFAULT_PORT.clone()]);
        assert_eq!(factory.get_output_ports(), vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]);
        assert!(factory.parameter_schema().is_none());
    }

    #[test]
    fn test_factory_requires_with_parameter() {
        let factory = UdxFolderExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = reearth_flow_runtime::event::EventHub::new(30);
        let result = factory.build(node_ctx, event_hub, "test".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_build_success_with_all_params() {
        let mut with = HashMap::new();
        with.insert("cityGmlPath".to_string(), json!(CITY_GML_PATH_EXPR.as_ref()));
        with.insert("codelistsPath".to_string(), json!("file:///tmp/codelists"));
        with.insert("schemasPath".to_string(), json!("file:///tmp/schemas"));

        let factory = UdxFolderExtractorFactory::default();
        let node_ctx = create_default_node_context();
        let event_hub = reearth_flow_runtime::event::EventHub::new(30);
        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(with));
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_valid_three_level_path_goes_to_default_port() {
        let mut with = HashMap::new();
        with.insert("cityGmlPath".to_string(), json!(CITY_GML_PATH_EXPR.as_ref()));

        let mut processor = build_factory_processor(with);

        let feature = create_feature(
            "file:///root/bldg/1234/5678/file.gml",
            HashMap::new(),
        );
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        processor.process(ctx, &fw).expect("processing succeeds");

        let ports = noop_forwarder.send_ports.lock().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], DEFAULT_PORT.clone());

        let features = noop_forwarder.send_features.lock().unwrap();
        let output = features.first().expect("feature emitted");
        assert_eq!(
            output.get(&"udxDirs"),
            Some(&AttributeValue::String("bldg/1234/5678".to_string()))
        );
        assert_eq!(
            output.get(&"package"),
            Some(&AttributeValue::String("bldg".to_string()))
        );
    }

    #[test]
    fn test_process_non_plateau_package_routed_to_rejected_port() {
        let mut with = HashMap::new();
        with.insert("cityGmlPath".to_string(), json!(CITY_GML_PATH_EXPR.as_ref()));

        let mut processor = build_factory_processor(with);

        let feature = create_feature(
            "file:///root/unknown/1234/5678/file.gml",
            HashMap::new(),
        );
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        processor.process(ctx, &fw).expect("processing succeeds");

        let ports = noop_forwarder.send_ports.lock().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
    }

    #[test]
    fn test_process_missing_city_gml_attribute_returns_error() {
        let mut with = HashMap::new();
        with.insert("cityGmlPath".to_string(), json!(CITY_GML_PATH_EXPR.as_ref()));

        let mut processor = build_factory_processor(with);

        let feature = Feature::new();
        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder);

        let result = processor.process(ctx, &fw);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapper_shapes_three_level_path() {
        let feature = create_feature(
            "file:///root/bldg/1234/5678/file.gml",
            HashMap::new(),
        );
        let engine = create_default_node_context().expr_engine;
        let expr = engine.compile("cityGmlPath").unwrap();

        let response = mapper(
            &feature,
            &None,
            &expr,
            engine,
            StorageResolver::new().into(),
            &None,
            &None,
        )
        .expect("mapping succeeds");

        assert_eq!(response.root, "root");
        assert_eq!(response.package, "bldg");
        assert_eq!(response.admin, "1234");
        assert_eq!(response.area, "5678");
        assert_eq!(response.udx_dirs, "bldg/1234/5678");
    }

    #[test]
    fn test_mapper_handles_two_level_path() {
        let feature = create_feature("file:///root/bldg/5678/file.gml", HashMap::new());
        let engine = create_default_node_context().expr_engine;
        let expr = engine.compile("cityGmlPath").unwrap();

        let response = mapper(
            &feature,
            &None,
            &expr,
            engine,
            StorageResolver::new().into(),
            &None,
            &None,
        )
        .expect("mapping succeeds");

        assert_eq!(response.package, "bldg");
        assert!(response.admin.is_empty());
        assert_eq!(response.area, "5678");
        assert_eq!(response.udx_dirs, "bldg/5678");
    }

    #[test]
    fn test_mapper_handles_single_level_path() {
        let feature = create_feature("file:///root/bldg/file.gml", HashMap::new());
        let engine = create_default_node_context().expr_engine;
        let expr = engine.compile("cityGmlPath").unwrap();

        let response = mapper(
            &feature,
            &None,
            &expr,
            engine,
            StorageResolver::new().into(),
            &None,
            &None,
        )
        .expect("mapping succeeds");

        assert_eq!(response.package, "bldg");
        assert!(response.admin.is_empty());
        assert!(response.area.is_empty());
        assert_eq!(response.udx_dirs, "bldg");
    }

    #[test]
    fn test_mapper_invalid_uri_returns_error() {
        let feature = create_feature("not-uri", HashMap::new());
        let engine = create_default_node_context().expr_engine;
        let expr = engine.compile("cityGmlPath").unwrap();

        let result = mapper(
            &feature,
            &None,
            &expr,
            engine,
            StorageResolver::new().into(),
            &None,
            &None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_process_copies_codelists_and_schemas_when_missing() {
        let (tmp_root, tmp_root_uri) = create_temp_dir();
        let codelists_path = Path::new(tmp_root.path()).join("codelists");
        let schemas_path = Path::new(tmp_root.path()).join("schemas");
        fs::create_dir_all(codelists_path.join("existing")).unwrap();
        fs::create_dir_all(schemas_path.join("existing")).unwrap();

        let mut with = HashMap::new();
        with.insert("cityGmlPath".to_string(), json!(CITY_GML_PATH_EXPR.as_ref()));
        with.insert("codelistsPath".to_string(), json!(tmp_root_uri.clone()));
        with.insert("schemasPath".to_string(), json!(tmp_root_uri.clone()));

        let mut processor = build_factory_processor(with);

        let feature = create_feature(
            format!("{tmp_root_uri}/bldg/admin/area/file.gml").as_str(),
            HashMap::new(),
        );

        let ctx = create_default_execute_context(feature);
        let noop_forwarder = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop_forwarder.clone());

        processor.process(ctx, &fw).expect("processing succeeds");

        let codelists_dst = Path::new(tmp_root.path()).join("bldg").join("codelists");
        let schemas_dst = Path::new(tmp_root.path()).join("bldg").join("schemas");
        assert!(codelists_dst.exists());
        assert!(schemas_dst.exists());
    }
}


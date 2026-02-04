use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use once_cell::sync::Lazy;
use reearth_flow_common::{
    uri::{Uri, PROTOCOL_SEPARATOR},
    xml::{self, XmlDocument, XmlRoNamespace},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde_json::Value;

use super::errors::{Result, XmlProcessorError};
use super::namespace::recursive_check_namespace;
use super::types::{
    SchemaStore, ValidationResult, ValidationType, XmlInputType, XmlValidatorParam,
};

static SUCCESS_PORT: Lazy<Port> = Lazy::new(|| Port::new("success"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

#[derive(Debug, Clone, Default)]
pub struct XmlValidatorFactory;

impl ProcessorFactory for XmlValidatorFactory {
    fn name(&self) -> &str {
        "XMLValidator"
    }

    fn description(&self) -> &str {
        "Validates XML documents against XSD schemas with success/failure routing"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(XmlValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![SUCCESS_PORT.clone(), FAILED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: XmlValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                XmlProcessorError::ValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                XmlProcessorError::ValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(XmlProcessorError::ValidatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        };
        Ok(Box::new(process))
    }
}

#[derive(Clone)]
pub struct XmlValidator {
    params: XmlValidatorParam,
    schema_store: Arc<parking_lot::RwLock<SchemaStore>>,
}

impl Debug for XmlValidator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("XmlValidator")
            .field("params", &self.params)
            .finish()
    }
}

impl Processor for XmlValidator {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match self.params.validation_type {
            ValidationType::Syntax => self.validate_syntax_only(ctx, fw)?,
            ValidationType::SyntaxAndNamespace => self.validate_syntax_and_namespace(ctx, fw)?,
            ValidationType::SyntaxAndSchema => self.validate_syntax_and_schema(ctx, fw)?,
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "XMLValidator"
    }
}

impl XmlValidator {
    fn validate_syntax_only(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;
        let xml_content = self.get_xml_content(&ctx, feature)?;

        let Ok(document) = xml::parse(xml_content) else {
            Self::send_syntax_error(&ctx, fw, feature);
            return Ok(());
        };

        let Ok(_) = xml::get_root_node(&document) else {
            Self::send_syntax_error(&ctx, fw, feature);
            return Ok(());
        };

        fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
        Ok(())
    }

    fn validate_syntax_and_namespace(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;
        let xml_content = self.get_xml_content(&ctx, feature)?;

        let document = match xml::parse(xml_content) {
            Ok(doc) => doc,
            Err(_) => {
                Self::send_syntax_error(&ctx, fw, feature);
                return Ok(());
            }
        };

        match Self::check_namespace(&document) {
            Ok(result) => {
                if result.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.insert(
                        "xmlError",
                        AttributeValue::Array(
                            result
                                .into_iter()
                                .map(|r| AttributeValue::Map(r.into()))
                                .collect::<Vec<_>>(),
                        ),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                }
            }
            Err(_) => {
                Self::send_syntax_error(&ctx, fw, feature);
            }
        }

        Ok(())
    }

    fn validate_syntax_and_schema(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let feature = &ctx.feature;
        // Get XML as bytes for streaming validation (more memory efficient)
        let xml_bytes = self.get_xml_content_bytes(&ctx, feature)?;

        match self.check_schema_streaming(feature, &xml_bytes) {
            Ok(result) => {
                if result.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.insert(
                        "xmlError",
                        AttributeValue::Array(
                            result
                                .into_iter()
                                .map(|r| AttributeValue::Map(r.into()))
                                .collect::<Vec<_>>(),
                        ),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                }
            }
            Err(e) => {
                let mut feature = feature.clone();
                feature.insert(
                    "xmlError",
                    AttributeValue::Array(vec![AttributeValue::Map(
                        ValidationResult::new(
                            "SchemaError",
                            &format!("Schema validation failed: {e}"),
                        )
                        .into(),
                    )]),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
            }
        }

        Ok(())
    }

    fn get_xml_content(&self, ctx: &ExecutorContext, feature: &Feature) -> Result<String> {
        match self.params.input_type {
            XmlInputType::File => {
                let uri = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("Required Uri".to_string()))?;
                let uri = match uri {
                    AttributeValue::String(s) => Uri::from_str(s)
                        .map_err(|_| XmlProcessorError::Validator("Invalid URI".to_string()))?,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                let storage = ctx
                    .storage_resolver
                    .resolve(&uri)
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                let content = storage
                    .get_sync(uri.path().as_path())
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                String::from_utf8(content.to_vec())
                    .map_err(|_| XmlProcessorError::Validator("Invalid UTF-8".to_string()))
            }
            XmlInputType::Text => {
                let content = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("No Attribute".to_string()))?;
                let content = match content {
                    AttributeValue::String(s) => s,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                Ok(content.to_string())
            }
        }
    }

    /// Get XML content as bytes for streaming validation (more memory efficient)
    fn get_xml_content_bytes(&self, ctx: &ExecutorContext, feature: &Feature) -> Result<Vec<u8>> {
        match self.params.input_type {
            XmlInputType::File => {
                let uri = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("Required Uri".to_string()))?;
                let uri = match uri {
                    AttributeValue::String(s) => Uri::from_str(s)
                        .map_err(|_| XmlProcessorError::Validator("Invalid URI".to_string()))?,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                let storage = ctx
                    .storage_resolver
                    .resolve(&uri)
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                let content = storage
                    .get_sync(uri.path().as_path())
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;
                Ok(content.to_vec())
            }
            XmlInputType::Text => {
                let content = feature
                    .attributes
                    .get(&self.params.attribute)
                    .ok_or(XmlProcessorError::Validator("No Attribute".to_string()))?;
                let content = match content {
                    AttributeValue::String(s) => s,
                    _ => {
                        return Err(XmlProcessorError::Validator(
                            "Invalid Attribute".to_string(),
                        ))
                    }
                };
                Ok(content.as_bytes().to_vec())
            }
        }
    }

    fn send_syntax_error(ctx: &ExecutorContext, fw: &ProcessorChannelForwarder, feature: &Feature) {
        let mut feature = feature.clone();
        feature.insert(
            "xmlError",
            AttributeValue::Array(vec![AttributeValue::Map(
                ValidationResult::new("SyntaxError", "Invalid document structure").into(),
            )]),
        );
        fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
    }

    fn check_namespace(document: &XmlDocument) -> Result<Vec<ValidationResult>> {
        let root_node = match xml::get_root_readonly_node(document) {
            Ok(node) => node,
            Err(e) => {
                return Err(XmlProcessorError::Validator(format!(
                    "Failed to get root node: {e}"
                )));
            }
        };

        let namespaces: Vec<XmlRoNamespace> = root_node
            .get_namespace_declarations()
            .into_iter()
            .map(|ns| ns.into())
            .collect::<Vec<_>>();

        Ok(recursive_check_namespace(root_node, &namespaces))
    }

    /// Streaming schema validation using StreamValidator
    /// Memory efficient - does NOT load the entire DOM tree
    fn check_schema_streaming(
        &self,
        feature: &Feature,
        xml_bytes: &[u8],
    ) -> Result<Vec<ValidationResult>> {
        // Parse schema locations from XML (minimal parsing)
        let doc = xml::parse(xml_bytes)
            .map_err(|e| XmlProcessorError::Validator(format!("XML parse error: {e:?}")))?;
        let schema_locations = xml::parse_schema_locations(&doc)
            .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;

        #[cfg(test)]
        println!(
            "[check_schema_streaming] Parsed schema locations: {:?}",
            schema_locations
        );
        tracing::debug!("Parsed schema locations: {:?}", schema_locations);

        if schema_locations.is_empty() {
            return Ok(Vec::new());
        }

        // Get or compile schema (cached for reuse)
        if !self.schema_store.read().contains_key(&schema_locations) {
            let compiled_schema =
                self.get_or_compile_schema_streaming(&schema_locations, feature)?;
            self.schema_store
                .write()
                .insert(schema_locations.clone(), compiled_schema);
        }

        // Use cached compiled schema for streaming validation
        let store = self.schema_store.read();
        let compiled_schema = store.get(&schema_locations).unwrap();

        // Perform streaming validation using StreamValidator
        let result = xml::validate_xml_streaming(xml_bytes, compiled_schema, 100)
            .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;

        #[cfg(test)]
        println!(
            "[check_schema_streaming] Validation result: {} errors",
            result.len()
        );
        #[cfg(test)]
        for err in &result {
            println!("[check_schema_streaming]   Error: {:?}", err);
        }

        // Convert validation errors
        let validation_results = result
            .into_iter()
            .map(|err| {
                ValidationResult::new_with_line_and_col(
                    "SchemaError",
                    &err.message,
                    err.line().map(|l| l as i32),
                    err.column().map(|c| c as i32),
                )
            })
            .collect::<Vec<_>>();

        // Remove duplicates
        let set: HashSet<_> = validation_results.into_iter().collect();
        Ok(set.into_iter().collect())
    }

    /// Compile schema for streaming validation (StreamValidator)
    fn get_or_compile_schema_streaming(
        &self,
        schema_locations: &[(String, String)],
        feature: &Feature,
    ) -> Result<Arc<xml::CompiledSchema>> {
        // Resolve schema locations to actual URLs/paths
        let resolved_locations: Vec<(String, String)> = schema_locations
            .iter()
            .filter_map(|(ns, location)| {
                self.resolve_schema_target(location, feature)
                    .filter(|t| !t.is_empty())
                    .map(|target| (ns.clone(), target))
            })
            .collect();

        if resolved_locations.is_empty() {
            return Err(XmlProcessorError::Validator(format!(
                "Failed to resolve any schema locations from: {:?}",
                schema_locations
            )));
        }

        #[cfg(test)]
        println!(
            "[get_or_compile_schema_streaming] resolved_locations: {:?}",
            resolved_locations
        );
        tracing::debug!("Resolved schema locations: {:?}", resolved_locations);

        // Get base directory for relative schema resolution
        let base_dir = self.get_schema_base_dir(feature);

        #[cfg(test)]
        println!("[get_or_compile_schema_streaming] base_dir: {:?}", base_dir);

        // Compile schema for streaming validation
        xml::compile_schema_for_streaming(&resolved_locations, base_dir.as_deref())
            .map_err(|e| XmlProcessorError::Validator(format!("Failed to compile schema: {e:?}")))
    }

    /// Get the base directory for schema resolution.
    /// This is used by fastxml's DefaultFetcher to resolve relative schema paths.
    fn get_schema_base_dir(&self, feature: &Feature) -> Option<PathBuf> {
        // First try dirSchemas attribute (for PLATEAU CityGML)
        if let Some(dir_schemas) = Self::get_dir_schemas(feature) {
            if let Some(path) = dir_schemas.path().to_str() {
                let path_buf = PathBuf::from(path);
                if path_buf.exists() {
                    return Some(path_buf);
                }
            }
        }

        // Fall back to XML file's parent directory
        self.get_xml_base_url(feature).and_then(|uri| {
            uri.path().to_str().map(PathBuf::from).and_then(
                |p| {
                    if p.exists() {
                        Some(p)
                    } else {
                        None
                    }
                },
            )
        })
    }

    fn resolve_schema_target(&self, location: &str, feature: &Feature) -> Option<String> {
        if !location.contains(PROTOCOL_SEPARATOR) && !location.starts_with('/') {
            // location is relative

            // Check if the location contains "schemas/" and we have dir_schemas attribute
            // This handles PLATEAU CityGML files where schemas are in a separate directory
            if let Some(schemas_relative) = Self::extract_schemas_relative_path(location) {
                if let Some(dir_schemas) = Self::get_dir_schemas(feature) {
                    let joined = dir_schemas.join(&schemas_relative).ok()?;
                    return Some(joined.path().to_str()?.to_string());
                }
            }

            // Fallback: resolve against the XML base URL
            let base_path = self.get_xml_base_url(feature)?;
            let joined = base_path.join(Path::new(location)).ok()?;
            Some(joined.path().to_str().unwrap().to_string())
        } else {
            // location is absolute or has a protocol, use it as is
            Some(location.to_string())
        }
    }

    /// Extract the relative path after "schemas/" from a schema location
    /// e.g., "../../schemas/iur/uro/3.1/urbanObject.xsd" -> "iur/uro/3.1/urbanObject.xsd"
    fn extract_schemas_relative_path(location: &str) -> Option<String> {
        const SCHEMAS_DIR: &str = "schemas/";
        location
            .find(SCHEMAS_DIR)
            .map(|idx| location[idx + SCHEMAS_DIR.len()..].to_string())
    }

    /// Get the dirSchemas attribute from a feature if it exists
    /// Note: UDXFolderExtractor uses camelCase for attribute names
    fn get_dir_schemas(feature: &Feature) -> Option<Uri> {
        feature
            .attributes
            .get(&Attribute::new("dirSchemas"))
            .and_then(|v| {
                if let AttributeValue::String(s) = v {
                    Uri::from_str(s).ok()
                } else {
                    None
                }
            })
    }

    fn get_xml_base_url(&self, feature: &Feature) -> Option<Uri> {
        match self.params.input_type {
            XmlInputType::File => feature
                .attributes
                .get(&self.params.attribute)
                .and_then(|v| {
                    if let AttributeValue::String(s) = v {
                        match Uri::from_str(s) {
                            Ok(uri) => {
                                if uri.is_dir() {
                                    Some(uri)
                                } else {
                                    uri.parent()
                                }
                            }
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                }),
            XmlInputType::Text => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils;
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry};

    fn create_xml_validator(validation_type: ValidationType) -> XmlValidator {
        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_content"),
            input_type: XmlInputType::Text,
            validation_type,
        };

        XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    fn create_feature_with_xml(xml_content: &str) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("xml_content"),
            AttributeValue::String(xml_content.to_string()),
        );

        Feature::new_with_attributes_and_geometry(attributes, Geometry::new(), Default::default())
    }

    fn run_validator_test(
        xml_content: &str,
        validation_type: ValidationType,
    ) -> (Port, Vec<Feature>) {
        let feature = create_feature_with_xml(xml_content);
        let mut validator = create_xml_validator(validation_type);

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                (send_ports[0].clone(), send_features.clone())
            }
            _ => panic!("Expected Noop forwarder for testing"),
        }
    }

    #[test]
    fn test_xml_validator_syntax_validation() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element>test</element>
</root>"#;

        let (port, _features) = run_validator_test(xml_content, ValidationType::Syntax);
        assert_eq!(port, *SUCCESS_PORT, "Should output to success port");
    }

    #[test]
    fn test_xml_validator_invalid_syntax() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element>test</element>
    <unclosed_tag>
</root>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::Syntax);
        assert_eq!(
            port, *FAILED_PORT,
            "Should output to failed port for invalid XML"
        );

        // Verify error information is present
        match features[0].attributes.get(&Attribute::new("xmlError")) {
            Some(AttributeValue::Array(errors)) => {
                assert!(
                    !errors.is_empty(),
                    "Should have validation error information"
                );
                match errors.first() {
                    Some(AttributeValue::Map(error_map)) => {
                        assert!(
                            error_map.contains_key("errorType"),
                            "Should have error type"
                        );
                        assert!(
                            error_map.contains_key("message"),
                            "Should have error message"
                        );
                    }
                    _ => panic!("Expected error map in first array element"),
                }
            }
            _ => panic!("Should have xmlError attribute with validation errors"),
        }
    }

    #[test]
    fn test_xml_validator_malformed_xml() {
        let xml_content = r#"This is not XML at all!
<random>unclosed tag
&invalid;entity;
<>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::Syntax);
        assert_eq!(
            port, *FAILED_PORT,
            "Should output to failed port for malformed XML"
        );

        // Verify error information is present
        match features[0].attributes.get(&Attribute::new("xmlError")) {
            Some(AttributeValue::Array(errors)) => {
                assert!(
                    !errors.is_empty(),
                    "Should have validation error information"
                );
                match errors.first() {
                    Some(AttributeValue::Map(error_map)) => {
                        match error_map.get("errorType") {
                            Some(AttributeValue::String(error_type)) => {
                                assert_eq!(error_type, "SyntaxError", "Should be syntax error");
                            }
                            _ => panic!("Expected errorType to be a string"),
                        }
                        match error_map.get("message") {
                            Some(AttributeValue::String(message)) => {
                                assert_eq!(
                                    message, "Invalid document structure",
                                    "Should have proper error message"
                                );
                            }
                            _ => panic!("Expected message to be a string"),
                        }
                    }
                    _ => panic!("Expected error map in first array element"),
                }
            }
            _ => panic!("Should have xmlError attribute with validation errors"),
        }
    }

    #[test]
    fn test_xml_validator_missing_local_schema() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/schema ./test-schema.xsd">
    <element>test</element>
</root>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);

        // Since local schema file doesn't exist, this should fail
        assert_eq!(
            port, *FAILED_PORT,
            "Should output to failed port for missing schema"
        );

        // Verify error information is present
        match features[0].attributes.get(&Attribute::new("xmlError")) {
            Some(AttributeValue::Array(errors)) => {
                assert!(
                    !errors.is_empty(),
                    "Should have validation error information"
                );
            }
            _ => panic!("Should have xmlError attribute with validation errors"),
        }
    }

    #[test]
    fn test_xml_validator_in_async_context() {
        // Verify that lazy initialization prevents panic when creating reqwest::blocking::Client in async context
        use reearth_flow_eval_expr::engine::Engine;
        use reearth_flow_runtime::{
            event::EventHub,
            executor_operation::{ExecutorContext, NodeContext},
            forwarder::{NoopChannelForwarder, ProcessorChannelForwarder},
            kvs::create_kv_store,
            node::ProcessorFactory,
        };
        use reearth_flow_storage::resolve::StorageResolver;

        // Create a runtime to simulate the actual execution environment
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // Test that XmlValidator can be created and executed within runtime.block_on
        runtime.block_on(async {
            let factory = XmlValidatorFactory {};

            // Create required dependencies for NodeContext
            let expr_engine = Arc::new(Engine::new());
            let storage_resolver = Arc::new(StorageResolver::new());
            let kv_store = Arc::new(create_kv_store());
            let event_hub = EventHub::new(1024);

            let ctx = NodeContext::new(
                expr_engine.clone(),
                storage_resolver.clone(),
                kv_store.clone(),
                event_hub.clone(),
            );

            let mut with = HashMap::new();
            with.insert(
                "attribute".to_string(),
                serde_json::Value::String("xml_content".to_string()),
            );
            with.insert(
                "inputType".to_string(),
                serde_json::Value::String("text".to_string()),
            );
            with.insert(
                "validationType".to_string(),
                serde_json::Value::String("syntaxAndSchema".to_string()),
            );

            // This should not panic with our lazy initialization
            let result = factory.build(ctx, event_hub, "xmlValidator".to_string(), Some(with));

            assert!(
                result.is_ok(),
                "Should be able to create XmlValidator in async context"
            );

            let mut processor = result.unwrap();

            // Create a feature with XML content that includes HTTPS schema reference
            let mut feature = Feature::new_with_attributes(IndexMap::new());
            feature.insert(
                Attribute::new("xml_content"),
                AttributeValue::String(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
                    <note xmlns="http://example.com/note"
                          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                          xsi:schemaLocation="http://example.com/note https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/runtime/action-processor/fixtures/xml/simple_note_schema.xsd">
                      <to>Tove</to>
                      <from>Jani</from>
                      <heading>Reminder</heading>
                      <body>Don't forget me this weekend!</body>
                    </note>"#.to_string()
                )
            );

            // Execute the processor in spawn_blocking to simulate real runtime behavior
            let handle = tokio::task::spawn_blocking(move || {
                let exec_ctx = ExecutorContext::new(
                    feature,
                    DEFAULT_PORT.clone(),
                    expr_engine,
                    storage_resolver,
                    kv_store,
                    EventHub::new(1024),
                );
                let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

                // This should not panic - HTTP client creation happens here in blocking context
                let result = processor.process(exec_ctx, &fw);

                (result, fw)
            });

            let (process_result, fw) = handle.await.unwrap();

            assert!(process_result.is_ok(), "Processing should complete without panic");

            // Check that we received output
            match &fw {
                ProcessorChannelForwarder::Noop(noop_fw) => {
                    let send_ports = noop_fw.send_ports.lock().unwrap();
                    assert!(!send_ports.is_empty(), "Should have sent output");

                    // The output should be on either success or failed port
                    assert!(
                        send_ports[0] == *SUCCESS_PORT || send_ports[0] == *FAILED_PORT,
                        "Should output to success or failed port"
                    );
                }
                _ => panic!("Expected Noop forwarder for testing"),
            }
        });
    }
}

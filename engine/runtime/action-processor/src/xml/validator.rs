use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::Instant,
};

use fastxml::schema::fetcher::{DefaultFetcher, FileCachingFetcher, SchemaFetcher};
use fastxml::schema::types::CompiledSchema;
use fastxml::schema::xsd::{compile_schemas, register_builtin_types, SchemaResolver, XsdSchema};
use once_cell::sync::Lazy;
use reearth_flow_common::{
    uri::Uri,
    xml::{self, XmlDocument, XmlRoNamespace},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, Feature};
use serde_json::Value;

use super::errors::{Result, XmlProcessorError};
use super::namespace::recursive_check_namespace;
use super::types::{ValidationResult, ValidationType, XmlInputType, XmlValidatorParam};

/// Get current process RSS (Resident Set Size) in MB using sysinfo.
fn current_rss_mb() -> f64 {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let pid = Pid::from_u32(std::process::id());
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    sys.process(pid)
        .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
        .unwrap_or(0.0)
}

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
            schema_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        };
        Ok(Box::new(process))
    }
}

#[derive(Clone)]
pub struct XmlValidator {
    params: XmlValidatorParam,
    /// Cache of compiled schemas keyed by sorted schema locations.
    /// Shared across clones (threads) via Arc so that the ~27s compilation
    /// cost is paid only once per unique set of schema locations.
    schema_cache: Arc<parking_lot::RwLock<HashMap<String, Arc<CompiledSchema>>>>,
}

/// Build a deterministic cache key from schema locations.
fn schema_cache_key(locations: &[(String, String)]) -> String {
    let mut parts: Vec<String> = locations
        .iter()
        .map(|(ns, loc)| format!("{ns}={loc}"))
        .collect();
    parts.sort();
    parts.join("|")
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
        let total_start = Instant::now();
        let feature = &ctx.feature;
        let xml_content = self.get_xml_content(&ctx, feature)?;

        tracing::info!(
            "[PERF] XMLValidator::validate_syntax_and_namespace START | xml_size={} bytes",
            xml_content.len()
        );

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

        tracing::info!(
            "[PERF] XMLValidator::validate_syntax_and_namespace TOTAL | elapsed={}ms",
            total_start.elapsed().as_millis()
        );

        Ok(())
    }

    fn validate_syntax_and_schema(
        &self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<()> {
        let total_start = Instant::now();
        let feature = &ctx.feature;
        // Get XML as bytes for streaming validation (more memory efficient)
        let xml_bytes = self.get_xml_content_bytes(&ctx, feature)?;

        tracing::info!(
            "[PERF] XMLValidator::validate_syntax_and_schema START | xml_size={} bytes",
            xml_bytes.len()
        );

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

        tracing::info!(
            "[PERF] XMLValidator::validate_syntax_and_schema TOTAL | elapsed={}ms",
            total_start.elapsed().as_millis()
        );

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
        let start = Instant::now();
        let result = match self.params.input_type {
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
        };
        let bytes = result?;
        tracing::info!(
            "[PERF] XMLValidator::get_xml_content_bytes | elapsed={}ms | size={} bytes",
            start.elapsed().as_millis(),
            bytes.len()
        );
        Ok(bytes)
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

    /// Streaming schema validation with per-step performance profiling.
    /// Decomposes the validation into individual steps to measure:
    /// 1. Schema location extraction (XML parse + attribute read)
    /// 2. Schema fetch + resolve + compile (with caching fetcher)
    /// 3. Streaming validation
    fn check_schema_streaming(
        &self,
        feature: &Feature,
        xml_bytes: &[u8],
    ) -> Result<Vec<ValidationResult>> {
        use fastxml::schema::validator::StreamValidator;

        let total_start = Instant::now();
        let rss_start = current_rss_mb();
        tracing::info!(
            "[PERF] XMLValidator::check_schema_streaming START | xml_size={} bytes | rss={:.1} MB",
            xml_bytes.len(),
            rss_start
        );

        // Determine base directory for relative schema resolution
        let base_dir = self.get_xml_base_url(feature).and_then(|uri| {
            uri.path()
                .to_str()
                .map(PathBuf::from)
                .filter(|p| p.exists())
        });

        // --- Step 1: Extract schema locations ---
        let step1_start = Instant::now();
        let document = xml::parse(xml_bytes).map_err(|e| {
            XmlProcessorError::Validator(format!("Failed to parse XML for schema locations: {e}"))
        })?;
        let schema_locations = xml::parse_schema_locations(&document).map_err(|e| {
            XmlProcessorError::Validator(format!("Failed to extract schema locations: {e}"))
        })?;
        // Drop the document to free DOM memory before proceeding
        drop(document);
        let rss_after_locations = current_rss_mb();
        tracing::info!(
            "[PERF] XMLValidator::check_schema_streaming schema_locations extracted | elapsed={}ms | locations={} | rss={:.1} MB",
            step1_start.elapsed().as_millis(),
            schema_locations.len(),
            rss_after_locations
        );

        if schema_locations.is_empty() {
            tracing::info!(
                "[PERF] XMLValidator::check_schema_streaming TOTAL (no schema) | elapsed={}ms | rss_delta={:+.1} MB",
                total_start.elapsed().as_millis(),
                current_rss_mb() - rss_start
            );
            return Ok(Vec::new());
        }

        // --- Step 2: Fetch + resolve + compile schemas (with cache) ---
        let step2_start = Instant::now();
        let cache_key = schema_cache_key(&schema_locations);

        // Check cache first
        let cached = {
            let cache = self.schema_cache.read();
            cache.get(&cache_key).cloned()
        };

        let compiled = if let Some(compiled) = cached {
            let rss_after_cache = current_rss_mb();
            tracing::info!(
                "[PERF] XMLValidator::check_schema_streaming schema cache HIT | elapsed={}ms | rss={:.1} MB",
                step2_start.elapsed().as_millis(),
                rss_after_cache
            );
            compiled
        } else {
            // Cache miss - compile from scratch
            let inner = match &base_dir {
                Some(dir) => DefaultFetcher::with_base_dir(dir),
                None => DefaultFetcher::new(),
            };
            let fetcher = FileCachingFetcher::new(inner).map_err(|e| {
                XmlProcessorError::Validator(format!("Failed to create caching fetcher: {e:?}"))
            })?;

            let mut all_schemas: Vec<XsdSchema> = Vec::new();
            for (_namespace, location) in &schema_locations {
                let fetch_result = fetcher.fetch(location).map_err(|e| {
                    XmlProcessorError::Validator(format!(
                        "Failed to fetch schema {}: {e:?}",
                        location
                    ))
                })?;

                let base_uri = &fetch_result.final_url;
                let mut resolver = SchemaResolver::new(&fetcher);
                let schemas = resolver
                    .resolve_all(&fetch_result.content, base_uri)
                    .map_err(|e| {
                        XmlProcessorError::Validator(format!(
                            "Failed to resolve schema imports for {}: {e:?}",
                            location
                        ))
                    })?;
                all_schemas.extend(schemas);
            }

            let mut compiled = compile_schemas(all_schemas).map_err(|e| {
                XmlProcessorError::Validator(format!("Failed to compile schemas: {e:?}"))
            })?;
            register_builtin_types(&mut compiled);
            let compiled = Arc::new(compiled);

            // Store in cache
            {
                let mut cache = self.schema_cache.write();
                cache.insert(cache_key, Arc::clone(&compiled));
            }

            let rss_after_compile = current_rss_mb();
            tracing::info!(
                "[PERF] XMLValidator::check_schema_streaming schema compiled (cache MISS) | elapsed={}ms | rss={:.1} MB",
                step2_start.elapsed().as_millis(),
                rss_after_compile
            );
            compiled
        };

        // --- Step 3: Streaming validation ---
        let step3_start = Instant::now();
        let reader = std::io::BufReader::new(std::io::Cursor::new(xml_bytes));
        let validator = StreamValidator::new(Arc::clone(&compiled));
        let errors = validator.validate(reader).map_err(|e| {
            XmlProcessorError::Validator(format!("Streaming validation failed: {e:?}"))
        })?;

        let rss_after_validate = current_rss_mb();
        tracing::info!(
            "[PERF] XMLValidator::check_schema_streaming validation done | elapsed={}ms | errors={} | rss={:.1} MB",
            step3_start.elapsed().as_millis(),
            errors.len(),
            rss_after_validate
        );

        // Convert errors to ValidationResult and deduplicate
        let validation_results: HashSet<_> = errors
            .into_iter()
            .map(|err| {
                ValidationResult::new_with_line_and_col(
                    "SchemaError",
                    &err.message,
                    err.line().map(|l| l as i32),
                    err.column().map(|c| c as i32),
                )
            })
            .collect();

        let rss_end = current_rss_mb();
        tracing::info!(
            "[PERF] XMLValidator::check_schema_streaming TOTAL | elapsed={}ms | rss_delta={:+.1} MB",
            total_start.elapsed().as_millis(),
            rss_end - rss_start
        );

        Ok(validation_results.into_iter().collect())
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
    use std::sync::Arc;

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
            schema_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
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

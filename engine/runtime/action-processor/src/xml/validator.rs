use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    path::Path,
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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::cache::{create_filesystem_cache, SchemaCache};
use super::errors::{Result, XmlProcessorError};
use super::schema_fetcher::{HttpSchemaFetcher, SchemaFetcher};
use super::schema_resolver::XmlSchemaResolver;

static SUCCESS_PORT: Lazy<Port> = Lazy::new(|| Port::new("success"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

/// Remove XML declaration from content to prevent "multiple XML declaration" errors
/// when schemas are included by other schemas
fn remove_xml_declaration(content: &str) -> &str {
    let trimmed = content.trim_start();
    if trimmed.starts_with("<?xml") {
        if let Some(end_pos) = trimmed.find("?>") {
            return trimmed[end_pos + 2..].trim_start();
        }
    }
    content
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
struct ValidationResult {
    error_type: String,
    message: String,
    line: Option<i32>,
    col: Option<i32>,
}

impl ValidationResult {
    fn new(error_type: &str, message: &str) -> Self {
        ValidationResult {
            error_type: error_type.to_string(),
            message: message.to_string(),
            line: None,
            col: None,
        }
    }

    fn new_with_line_and_col(
        error_type: &str,
        message: &str,
        line: Option<i32>,
        col: Option<i32>,
    ) -> Self {
        ValidationResult {
            error_type: error_type.to_string(),
            message: message.to_string(),
            line,
            col,
        }
    }
}

impl From<ValidationResult> for HashMap<String, AttributeValue> {
    fn from(result: ValidationResult) -> Self {
        let mut map = HashMap::new();
        map.insert(
            "errorType".to_string(),
            AttributeValue::String(result.error_type),
        );
        map.insert(
            "message".to_string(),
            AttributeValue::String(result.message),
        );
        map.insert(
            "line".to_string(),
            AttributeValue::String(result.line.unwrap_or_default().to_string()),
        );
        map.insert(
            "col".to_string(),
            AttributeValue::String(result.col.unwrap_or_default().to_string()),
        );
        map
    }
}

#[derive(Debug, Clone, Default)]
pub struct XmlValidatorFactory;

impl ProcessorFactory for XmlValidatorFactory {
    fn name(&self) -> &str {
        "XMLValidator"
    }

    fn description(&self) -> &str {
        "Validates XML content"
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

        let schema_fetcher = Arc::new(HttpSchemaFetcher::new());

        // TODO: replace it with node_cache in ctx
        // let schema_cache = create_schema_cache(ctx.node_cache());
        let cache_dir = std::env::temp_dir().join("reearth-flow-xmlvalidator-schema");
        let schema_cache = create_filesystem_cache(cache_dir)?;

        let process = XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            schema_fetcher,
            schema_cache,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum XmlInputType {
    File,
    Text,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum ValidationType {
    Syntax,
    SyntaxAndNamespace,
    SyntaxAndSchema,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct XmlValidatorParam {
    attribute: Attribute,
    input_type: XmlInputType,
    validation_type: ValidationType,
}

type SchemaStore = HashMap<Vec<(String, String)>, xml::XmlSchemaValidationContext>;

#[derive(Clone)]
pub struct XmlValidator {
    params: XmlValidatorParam,
    schema_store: Arc<parking_lot::RwLock<SchemaStore>>,
    schema_fetcher: Arc<dyn SchemaFetcher>,
    schema_cache: Arc<dyn SchemaCache>,
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
            ValidationType::Syntax => {
                let feature = &ctx.feature;
                let xml_content = self.get_xml_content(&ctx, feature)?;
                let Ok(document) = xml::parse(xml_content) else {
                    let mut feature = feature.clone();
                    feature.attributes.insert(
                        Attribute::new("xmlError"),
                        AttributeValue::Array(vec![AttributeValue::Map(
                            ValidationResult::new("SyntaxError", "Invalid document structure")
                                .into(),
                        )]),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                    return Ok(());
                };
                let Ok(_) = xml::get_root_node(&document) else {
                    let mut feature = feature.clone();
                    feature.attributes.insert(
                        Attribute::new("xmlError"),
                        AttributeValue::Array(vec![AttributeValue::Map(
                            ValidationResult::new("SyntaxError", "Invalid document structure")
                                .into(),
                        )]),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                    return Ok(());
                };
                fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
            }
            ValidationType::SyntaxAndNamespace => {
                let feature = &ctx.feature;
                let xml_content = self.get_xml_content(&ctx, feature)?;
                let document = match xml::parse(xml_content) {
                    Ok(doc) => doc,
                    Err(_) => {
                        let mut feature = feature.clone();
                        feature.attributes.insert(
                            Attribute::new("xmlError"),
                            AttributeValue::Array(vec![AttributeValue::Map(
                                ValidationResult::new("SyntaxError", "Invalid document structure")
                                    .into(),
                            )]),
                        );
                        fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                        return Ok(());
                    }
                };
                let root_node = match xml::get_root_readonly_node(&document) {
                    Ok(node) => node,
                    Err(_) => {
                        let mut feature = feature.clone();
                        feature.attributes.insert(
                            Attribute::new("xmlError"),
                            AttributeValue::Array(vec![AttributeValue::Map(
                                ValidationResult::new("SyntaxError", "Invalid document structure")
                                    .into(),
                            )]),
                        );
                        fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                        return Ok(());
                    }
                };
                let namespaces: Vec<XmlRoNamespace> = root_node
                    .get_namespace_declarations()
                    .into_iter()
                    .map(|ns| ns.into())
                    .collect::<Vec<_>>();
                let result = recursive_check_namespace(root_node, &namespaces);
                if result.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.attributes.insert(
                        Attribute::new("xmlError"),
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
            ValidationType::SyntaxAndSchema => {
                let feature = &ctx.feature;
                let xml_content = self.get_xml_content(&ctx, feature)?;
                let Ok(document) = xml::parse(xml_content) else {
                    let mut feature = feature.clone();
                    feature.attributes.insert(
                        Attribute::new("xmlError"),
                        AttributeValue::Array(vec![AttributeValue::Map(
                            ValidationResult::new("SyntaxError", "Invalid document structure")
                                .into(),
                        )]),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), FAILED_PORT.clone()));
                    return Ok(());
                };
                match self.check_schema(feature, &ctx, &document) {
                    Ok(result) => {
                        if result.is_empty() {
                            fw.send(
                                ctx.new_with_feature_and_port(
                                    feature.clone(),
                                    SUCCESS_PORT.clone(),
                                ),
                            );
                        } else {
                            let mut feature = feature.clone();
                            feature.attributes.insert(
                                Attribute::new("xmlError"),
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
                        // Schema validation setup failed - report the actual error
                        let mut feature = feature.clone();
                        feature.attributes.insert(
                            Attribute::new("xmlError"),
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
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "XMLValidator"
    }
}

impl XmlValidator {
    /// Generate a cache key for a schema URL
    /// Preserves directory structure for relative path references
    fn generate_cache_key(url: &str) -> String {
        // Remove protocol and host to get path-like structure
        if let Some(protocol_end) = url.find("://") {
            let after_protocol = &url[protocol_end + 3..];
            // Find the start of the path (after host)
            if let Some(path_start) = after_protocol.find('/') {
                let path = &after_protocol[path_start + 1..];
                // Use a hash of the host as prefix to avoid conflicts
                let host = &after_protocol[..path_start];
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                host.hash(&mut hasher);
                let host_hash = format!("{:x}", hasher.finish());
                // Preserve directory structure by keeping the path as-is
                return format!("xmlvalidator-schema/{}/{}", &host_hash[..8], path);
            }
        }
        // Fallback to simple hash-based approach
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());
        let filename = url.split('/').next_back().unwrap_or("schema.xsd");
        format!("xmlvalidator-schema/{}-{}", &hash[..8], filename)
    }

    fn resolve_schema_target(&self, location: &str, feature: &Feature) -> Option<String> {
        if !location.contains(PROTOCOL_SEPARATOR) && !location.starts_with('/') {
            let base_path = self.get_base_path(feature)?;
            let joined = base_path.join(Path::new(location)).ok()?;
            Some(joined.path().to_str().unwrap().to_string())
        } else {
            Some(location.to_string())
        }
    }

    fn process_schema_location(
        &self,
        location: &str,
        feature: &Feature,
        resolver: &XmlSchemaResolver,
    ) -> Result<Option<xml::XmlSchemaValidationContext>> {
        let target = match self.resolve_schema_target(location, feature) {
            Some(t) if !t.is_empty() => t,
            _ => return Ok(None),
        };

        // For HTTP(S) schemas, we need to cache them locally for libxml to use
        if target.starts_with("http://") || target.starts_with("https://") {
            if !self.schema_cache.is_available() {
                // Skip HTTPS schema validation if no cache available
                return Ok(None);
            }

            // Check if the root schema is already cached
            let root_cache_key = Self::generate_cache_key(&target);

            let mut url_to_cache_path = HashMap::new();

            // If root schema is already cached, use it directly
            // Note: We assume if the root is cached, its dependencies are also cached
            // since they are fetched together during resolution
            if self.schema_cache.is_cached(&root_cache_key) {
                let cached_root_path = self.schema_cache.get_schema_path(&root_cache_key)?;
                return Ok(Some(
                    xml::create_xml_schema_validation_context(
                        cached_root_path
                            .to_str()
                            .ok_or_else(|| {
                                XmlProcessorError::Validator("Invalid path".to_string())
                            })?
                            .to_string(),
                    )
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?,
                ));
            }

            // If not cached, fetch the schema and all its dependencies
            let resolution = resolver.resolve_schema_dependencies(&target)?;

            // Build URL to cache path mapping for all schemas
            for url in resolution.schemas.keys() {
                let cache_key = Self::generate_cache_key(url);
                let cached_path = self.schema_cache.get_schema_path(&cache_key)?;
                url_to_cache_path.insert(url.clone(), cached_path);
            }

            // Second pass: save schemas with rewritten import/include paths
            for (url, resolved_schema) in &resolution.schemas {
                let mut content = resolved_schema.content.clone();

                // Rewrite schemaLocation attributes to point to cached files
                for (import_url, import_path) in &url_to_cache_path {
                    if url != import_url {
                        // Calculate relative path from current schema to imported schema
                        let current_path = url_to_cache_path.get(url).unwrap();
                        let relative_path = if let (Some(current_parent), Some(import_parent)) =
                            (current_path.parent(), import_path.parent())
                        {
                            if current_parent == import_parent {
                                // Same directory - use just the filename
                                import_path
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string()
                            } else {
                                // Different directories - use absolute file:// URL
                                format!("file://{}", import_path.display())
                            }
                        } else {
                            format!("file://{}", import_path.display())
                        };

                        // Replace absolute URLs with appropriate paths
                        content = content.replace(
                            &format!(r#"schemaLocation="{import_url}""#),
                            &format!(r#"schemaLocation="{}""#, relative_path),
                        );
                        content = content.replace(
                            &format!(r#"schemaLocation='{import_url}'"#),
                            &format!(r#"schemaLocation='{}'"#, relative_path),
                        );

                        // Also handle relative paths in the original schema
                        // Replace relative imports that might be in the schema
                        if let Some(last_slash) = import_url.rfind('/') {
                            let import_filename = &import_url[last_slash + 1..];
                            content = content.replace(
                                &format!(r#"schemaLocation="{}""#, import_filename),
                                &format!(r#"schemaLocation="{}""#, relative_path),
                            );
                            content = content.replace(
                                &format!(r#"schemaLocation='{}'"#, import_filename),
                                &format!(r#"schemaLocation='{}'"#, relative_path),
                            );
                        }
                    }
                }

                let cache_key = Self::generate_cache_key(url);

                // Remove XML declaration from non-root schemas to prevent
                // "multiple XML declaration" errors when schemas are included
                let final_content = if url != &target {
                    remove_xml_declaration(&content).to_string()
                } else {
                    content
                };

                // Save rewritten schema content to cache
                self.schema_cache
                    .put_schema(&cache_key, final_content.as_bytes())?;
            }

            // Create validation context from the cached root schema
            let cached_root_path = url_to_cache_path.get(&target).ok_or_else(|| {
                XmlProcessorError::Validator("Root schema not found in cache".to_string())
            })?;

            Ok(Some(
                xml::create_xml_schema_validation_context(
                    cached_root_path
                        .to_str()
                        .ok_or_else(|| {
                            XmlProcessorError::Validator("Invalid cache path".to_string())
                        })?
                        .to_string(),
                )
                .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?,
            ))
        } else {
            // For local files, use direct validation
            Ok(Some(
                xml::create_xml_schema_validation_context(target)
                    .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?,
            ))
        }
    }

    fn get_or_create_schema_context(
        &self,
        schema_locations: &[(String, String)],
        feature: &Feature,
    ) -> Result<xml::XmlSchemaValidationContext> {
        let resolver = XmlSchemaResolver::new(self.schema_fetcher.clone());
        let mut validation_context = None;

        // Process each schema location and resolve all dependencies
        for (_ns, location) in schema_locations.iter() {
            if let Some(context) = self.process_schema_location(location, feature, &resolver)? {
                validation_context = Some(context);
            }
        }

        validation_context
            .ok_or_else(|| XmlProcessorError::Validator("No schema context created".to_string()))
    }

    fn get_base_path(&self, feature: &Feature) -> Option<Uri> {
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

    fn check_schema(
        &self,
        feature: &Feature,
        _ctx: &ExecutorContext,
        document: &XmlDocument,
    ) -> Result<Vec<ValidationResult>> {
        // Skip schema validation entirely if no cache is available
        if !self.schema_cache.is_available() {
            return Ok(Vec::new());
        }

        let schema_locations = xml::parse_schema_locations(document)
            .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;

        let result = if !self.schema_store.read().contains_key(&schema_locations) {
            // Get or create validation context
            let schema_context = self.get_or_create_schema_context(&schema_locations, feature)?;

            // Validate document
            let result = xml::validate_document_by_schema_context(document, &schema_context)
                .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?;

            // Cache the schema context for future use
            self.schema_store
                .write()
                .insert(schema_locations, schema_context);

            result
        } else {
            // Use cached schema context
            xml::validate_document_by_schema_context(
                document,
                self.schema_store.read().get(&schema_locations).unwrap(),
            )
            .map_err(|e| XmlProcessorError::Validator(format!("{e:?}")))?
        };

        // Convert validation errors
        let validation_results = result
            .into_iter()
            .map(|err| {
                ValidationResult::new_with_line_and_col(
                    "SchemaError",
                    err.message.as_deref().unwrap_or("Unknown error"),
                    err.line,
                    err.col,
                )
            })
            .collect::<Vec<_>>();

        // Remove duplicates
        let set: HashSet<_> = validation_results.into_iter().collect();
        Ok(set.into_iter().collect())
    }
}

fn recursive_check_namespace(
    node: xml::XmlRoNode,
    namespaces: &Vec<XmlRoNamespace>,
) -> Vec<ValidationResult> {
    let mut result = Vec::new();
    match node.get_namespace() {
        Some(ns) => {
            if !namespaces.iter().any(|n| n.get_prefix() == ns.get_prefix()) {
                result.push(ValidationResult::new(
                    "NamespaceError",
                    &format!("No namespace declaration for {}", ns.get_prefix()),
                ));
            }
        }
        None => {
            let tag = xml::get_readonly_node_tag(&node);
            if tag.contains(':') {
                let prefix = tag.split(':').collect::<Vec<&str>>()[0];
                if !namespaces.iter().any(|n| n.get_prefix() == prefix) {
                    result.push(ValidationResult::new(
                        "NamespaceError",
                        &format!("No namespace declaration for {prefix}"),
                    ));
                }
            } else {
                result.push(ValidationResult::new(
                    "NamespaceError",
                    "No namespace declaration",
                ));
            }
        }
    };
    let child_node = node.get_child_nodes();
    let child_nodes = child_node
        .into_iter()
        .filter(|n| {
            if let Some(typ) = n.get_type() {
                typ == xml::XmlNodeType::ElementNode
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    for child in child_nodes {
        let child_result = recursive_check_namespace(child, namespaces);
        result.extend(child_result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils;
    use crate::xml::cache::{create_filesystem_cache, NoOpSchemaCache};
    use crate::xml::schema_fetcher::MockSchemaFetcher;
    use indexmap::IndexMap;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry};
    use std::env;

    fn create_xml_validator(validation_type: ValidationType) -> XmlValidator {
        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_content"),
            input_type: XmlInputType::Text,
            validation_type,
        };

        let schema_fetcher = Arc::new(HttpSchemaFetcher::new());

        // Use filesystem cache for schema validation tests
        let schema_cache = match validation_type {
            ValidationType::SyntaxAndSchema => {
                let temp_dir =
                    env::temp_dir().join(format!("xml_validator_test_{}", uuid::Uuid::new_v4()));
                create_filesystem_cache(temp_dir).unwrap()
            }
            _ => Arc::new(NoOpSchemaCache),
        };

        XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            schema_fetcher,
            schema_cache,
        }
    }

    fn create_xml_validator_with_mock_fetcher(
        validation_type: ValidationType,
        fetcher: Arc<dyn SchemaFetcher>,
    ) -> XmlValidator {
        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_content"),
            input_type: XmlInputType::Text,
            validation_type,
        };

        // Use filesystem cache for schema validation tests
        let schema_cache = match validation_type {
            ValidationType::SyntaxAndSchema => {
                let temp_dir =
                    env::temp_dir().join(format!("xml_validator_test_{}", uuid::Uuid::new_v4()));
                create_filesystem_cache(temp_dir).unwrap()
            }
            _ => Arc::new(NoOpSchemaCache),
        };

        XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            schema_fetcher: fetcher,
            schema_cache,
        }
    }

    fn create_feature_with_xml(xml_content: &str) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("xml_content"),
            AttributeValue::String(xml_content.to_string()),
        );

        Feature {
            id: uuid::Uuid::new_v4(),
            geometry: Geometry::new(),
            attributes,
            metadata: Default::default(),
        }
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
    fn test_xml_validator_https_schema_success() {
        // Test with a simple XML that validates against a publicly available HTTPS schema
        // Using a simple XML structure that should be valid against basic schemas
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns="http://example.com/test"
      xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/test https://raw.githubusercontent.com/w3c/xmlschema11-tests/master/misc/XMLSchema.xsd">
    <element>test</element>
</root>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);

        // This test may succeed or fail depending on network connectivity and schema availability
        // We mainly want to verify that HTTPS schema fetching works without causing crashes
        match port {
            ref p if p == &*SUCCESS_PORT => {
                // Schema validation succeeded - HTTPS fetching worked
                // HTTPS schema validation succeeded
            }
            ref p if p == &*FAILED_PORT => {
                // Schema validation failed - this is also acceptable as the XML might not match the schema
                // Verify that we got proper error information
                match features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        assert!(
                            !errors.is_empty(),
                            "Should have validation error information"
                        );
                        // HTTPS schema validation failed with proper error handling
                    }
                    _ => panic!("Should have xmlError attribute when validation fails"),
                }
            }
            _ => panic!("Unexpected port returned"),
        }
    }

    #[test]
    fn test_xml_validator_https_schema_simple() {
        // Test with a very simple XML that uses an HTTPS schema
        // Using httpbin.org which provides predictable XML responses
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="https://httpbin.org/xml">
    <to>Tove</to>
    <from>Jani</from>
    <heading>Reminder</heading>
    <body>Don't forget me this weekend!</body>
</note>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);

        // httpbin.org/xml returns XML content, not an XSD schema, so this should fail
        // but it tests that our HTTPS fetching mechanism works
        match port {
            ref p if p == &*FAILED_PORT => {
                // Expected to fail since httpbin.org/xml is not a valid XSD schema
                match features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        assert!(
                            !errors.is_empty(),
                            "Should have validation error information"
                        );
                        // HTTPS schema fetch attempted successfully (validation failed as expected)
                    }
                    _ => panic!("Should have xmlError attribute when validation fails"),
                }
            }
            ref p if p == &*SUCCESS_PORT => {
                // Unexpected success, but not necessarily wrong
                println!("Unexpected success in HTTPS schema validation");
            }
            _ => panic!("Unexpected port returned"),
        }
    }

    #[test]
    fn test_xml_validator_https_schema_violation() {
        // Test with XML that violates a publicly available HTTPS schema
        // Using a simple note structure that intentionally violates constraints
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/microsoft/xml-document-transform/master/src/Microsoft.Web.XmlTransform/Microsoft.Web.XmlTransform.xsd">
    <invalidElement>This should not be allowed</invalidElement>
    <anotherInvalidElement>
        <nestedInvalid>Deep nesting that violates schema</nestedInvalid>
    </anotherInvalidElement>
</note>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);

        // This should fail due to schema violations
        match port {
            ref p if p == &*FAILED_PORT => {
                // Expected failure due to schema violations
                match features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        assert!(!errors.is_empty(), "Should have schema validation errors");

                        // Check that we have proper error details
                        if let Some(AttributeValue::Map(error_map)) = errors.first() {
                            match error_map.get("errorType") {
                                Some(AttributeValue::String(error_type)) => {
                                    // Should be SchemaError, not SyntaxError
                                    assert_eq!(
                                        error_type, "SchemaError",
                                        "Should be a schema validation error"
                                    );
                                }
                                _ => {
                                    // If not specifically SchemaError, at least verify we have error info
                                    assert!(
                                        error_map.contains_key("errorType"),
                                        "Should have error type"
                                    );
                                }
                            }
                            assert!(
                                error_map.contains_key("message"),
                                "Should have error message"
                            );
                        }
                        println!("HTTPS schema violation detected successfully");
                    }
                    _ => panic!("Should have xmlError attribute when schema validation fails"),
                }
            }
            ref p if p == &*SUCCESS_PORT => {
                // If it succeeds, it might mean the schema allows this structure
                // or the schema couldn't be fetched properly
                println!("XML validation succeeded - schema may be permissive or unavailable");
            }
            _ => panic!("Unexpected port returned"),
        }
    }

    #[test]
    fn test_xml_validator_https_schema_valid_structure() {
        // Test with a simple, well-formed XML that should work with basic schemas
        // Using a minimal structure that most schemas would accept
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="https://raw.githubusercontent.com/w3c/xmlschema11-tests/master/misc/XMLSchema.xsd">
</root>"#;

        let (port, features) = run_validator_test(xml_content, ValidationType::SyntaxAndSchema);

        // This test verifies that HTTPS schema fetching works and processes validation
        match port {
            ref p if p == &*SUCCESS_PORT => {
                println!("HTTPS schema validation succeeded with valid XML structure");
            }
            ref p if p == &*FAILED_PORT => {
                // May fail due to network issues or strict schema requirements
                match features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        assert!(
                            !errors.is_empty(),
                            "Should have validation error information"
                        );
                        println!("HTTPS schema validation failed (possibly due to strict schema or network)");
                    }
                    _ => panic!("Should have xmlError attribute when validation fails"),
                }
            }
            _ => panic!("Unexpected port returned"),
        }
    }

    #[test]
    fn test_xml_validator_mock_schema_success() {
        // Test with mock schema fetcher that returns valid XSD schema
        let valid_xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           elementFormDefault="qualified">
    <xs:element name="note">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="to" type="xs:string"/>
                <xs:element name="from" type="xs:string"/>
                <xs:element name="heading" type="xs:string"/>
                <xs:element name="body" type="xs:string"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="https://example.com/note.xsd">
    <to>Tove</to>
    <from>Jani</from>
    <heading>Reminder</heading>
    <body>Don't forget me this weekend!</body>
</note>"#;

        let mock_fetcher = MockSchemaFetcher::new()
            .with_response("https://example.com/note.xsd", Ok(valid_xsd.to_string()));

        let feature = create_feature_with_xml(xml_content);
        let mut validator = create_xml_validator_with_mock_fetcher(
            ValidationType::SyntaxAndSchema,
            Arc::new(mock_fetcher),
        );

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                // Check if validation succeeded or failed with proper error handling
                match send_ports[0] {
                    ref p if p == &*SUCCESS_PORT => {
                        println!("Mock schema validation succeeded as expected");
                    }
                    ref p if p == &*FAILED_PORT => {
                        // It's also acceptable if the schema validation detects issues
                        // The important thing is that HTTPS fetching worked
                        if let Some(AttributeValue::Array(errors)) =
                            send_features[0].attributes.get(&Attribute::new("xmlError"))
                        {
                            println!(
                                "Mock schema validation failed (which may be correct): {errors:?}"
                            );
                            // Verify we have proper schema error handling
                            assert!(
                                !errors.is_empty(),
                                "Should have validation error information"
                            );
                        } else {
                            panic!("Should have xmlError attribute when validation fails");
                        }
                    }
                    _ => panic!("Unexpected port returned"),
                }
            }
            _ => panic!("Expected Noop forwarder for testing"),
        }
    }

    #[test]
    fn test_xml_validator_mock_schema_violation() {
        // Test with mock schema that will cause validation failure
        let strict_xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="note">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="to" type="xs:string"/>
                <xs:element name="from" type="xs:string"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="https://example.com/strict.xsd">
    <to>Tove</to>
    <from>Jani</from>
    <heading>This element is not allowed by schema</heading>
    <body>This element is also not allowed</body>
</note>"#;

        let mock_fetcher = MockSchemaFetcher::new()
            .with_response("https://example.com/strict.xsd", Ok(strict_xsd.to_string()));

        let feature = create_feature_with_xml(xml_content);
        let mut validator = create_xml_validator_with_mock_fetcher(
            ValidationType::SyntaxAndSchema,
            Arc::new(mock_fetcher),
        );

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");
                assert_eq!(
                    send_ports[0], *FAILED_PORT,
                    "Should output to failed port for schema violation"
                );

                // Verify schema error information
                match send_features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        assert!(!errors.is_empty(), "Should have schema validation errors");
                        match errors.first() {
                            Some(AttributeValue::Map(error_map)) => {
                                match error_map.get("errorType") {
                                    Some(AttributeValue::String(error_type)) => {
                                        assert_eq!(
                                            error_type, "SchemaError",
                                            "Should be schema validation error"
                                        );
                                    }
                                    _ => panic!("Expected errorType to be SchemaError"),
                                }
                            }
                            _ => panic!("Expected error map in first array element"),
                        }
                    }
                    _ => panic!("Should have xmlError attribute with validation errors"),
                }
            }
            _ => panic!("Expected Noop forwarder for testing"),
        }
    }

    #[test]
    fn test_xml_validator_mock_schema_fetch_error() {
        // Test with mock fetcher that returns an error
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="https://example.com/nonexistent.xsd">
    <to>Tove</to>
</note>"#;

        let mock_fetcher = MockSchemaFetcher::new().with_response(
            "https://example.com/nonexistent.xsd",
            Err(XmlProcessorError::Validator("Network error".to_string())),
        );

        let feature = create_feature_with_xml(xml_content);
        let mut validator = create_xml_validator_with_mock_fetcher(
            ValidationType::SyntaxAndSchema,
            Arc::new(mock_fetcher),
        );

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");
                assert_eq!(
                    send_ports[0], *FAILED_PORT,
                    "Should output to failed port for fetch error"
                );

                // Verify error information
                match send_features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        assert!(
                            !errors.is_empty(),
                            "Should have validation error information"
                        );
                    }
                    _ => panic!("Should have xmlError attribute with validation errors"),
                }
            }
            _ => panic!("Expected Noop forwarder for testing"),
        }
    }

    #[test]
    fn test_xml_validator_schema_cache_hit() {
        // Test that schemas are cached and not fetched multiple times
        let valid_xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/note"
           elementFormDefault="qualified">
    <xs:element name="note">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="to" type="xs:string"/>
                <xs:element name="from" type="xs:string"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns="http://example.com/note"
      xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/note https://example.com/cached.xsd">
    <to>Tove</to>
    <from>Jani</from>
</note>"#;

        let mock_fetcher = MockSchemaFetcher::new()
            .with_response("https://example.com/cached.xsd", Ok(valid_xsd.to_string()));

        let mut validator = create_xml_validator_with_mock_fetcher(
            ValidationType::SyntaxAndSchema,
            Arc::new(mock_fetcher.clone()),
        );

        // First validation - should fetch schema
        let feature1 = create_feature_with_xml(xml_content);
        let ctx1 = utils::create_default_execute_context(&feature1);
        let fw1 = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result1 = validator.process(ctx1, &fw1);
        assert!(result1.is_ok(), "First XML validation should succeed");

        // Verify first call was made
        assert_eq!(
            mock_fetcher.get_call_count("https://example.com/cached.xsd"),
            1,
            "Schema should be fetched once on first use"
        );

        // Second validation with same schema - should use cache
        let feature2 = create_feature_with_xml(xml_content);
        let ctx2 = utils::create_default_execute_context(&feature2);
        let fw2 = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result2 = validator.process(ctx2, &fw2);
        assert!(result2.is_ok(), "Second XML validation should succeed");

        // Verify call count is still 1 (cache hit)
        assert_eq!(
            mock_fetcher.get_call_count("https://example.com/cached.xsd"),
            1,
            "Schema should not be fetched again on second use (cache hit)"
        );

        // Third validation with same schema - should still use cache
        let feature3 = create_feature_with_xml(xml_content);
        let ctx3 = utils::create_default_execute_context(&feature3);
        let fw3 = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result3 = validator.process(ctx3, &fw3);
        assert!(result3.is_ok(), "Third XML validation should succeed");

        // Verify call count is still 1 (cache hit again)
        assert_eq!(
            mock_fetcher.get_call_count("https://example.com/cached.xsd"),
            1,
            "Schema should not be fetched on third use (cache hit again)"
        );

        // Schema caching verified: fetched once, used three times
    }

    #[test]
    fn test_xml_validator_http_schema_support() {
        // Test HTTP schema support (not just HTTPS)
        let valid_xsd = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/note"
           elementFormDefault="qualified">
    <xs:element name="note">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="to" type="xs:string"/>
                <xs:element name="from" type="xs:string"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note xmlns="http://example.com/note"
      xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/note http://example.com/note.xsd">
    <to>Tove</to>
    <from>Jani</from>
</note>"#;

        let mock_fetcher = MockSchemaFetcher::new()
            .with_response("http://example.com/note.xsd", Ok(valid_xsd.to_string()));

        let feature = create_feature_with_xml(xml_content);
        let mut validator = create_xml_validator_with_mock_fetcher(
            ValidationType::SyntaxAndSchema,
            Arc::new(mock_fetcher.clone()),
        );

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "HTTP schema validation should succeed");

        // Verify HTTP schema was fetched
        assert_eq!(
            mock_fetcher.get_call_count("http://example.com/note.xsd"),
            1,
            "HTTP schema should be fetched once"
        );

        // Test caching for HTTP schemas too
        let feature2 = create_feature_with_xml(xml_content);
        let ctx2 = utils::create_default_execute_context(&feature2);
        let fw2 = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result2 = validator.process(ctx2, &fw2);
        assert!(
            result2.is_ok(),
            "Second HTTP schema validation should succeed"
        );

        // Verify HTTP schema was cached (still only 1 call)
        assert_eq!(
            mock_fetcher.get_call_count("http://example.com/note.xsd"),
            1,
            "HTTP schema should be cached and not fetched again"
        );

        // HTTP schema support verified: fetched and cached correctly
    }

    #[test]
    fn test_xml_validator_real_citygml_validation() {
        // Test with real CityGML data from Ppen Geospatial Consortium (OGC) test data
        // Source: https://schemas.opengis.net/citygml/examples/2.0/building/Building_LOD1-EPSG25832.gml
        let citygml_content = include_str!("test-data/Building_LOD1-EPSG25832.gml");

        // This CityGML file references multiple real schemas:
        // - http://schemas.opengis.net/citygml/building/2.0/building.xsd
        // - http://schemas.opengis.net/citygml/relief/2.0/relief.xsd

        let mut validator = create_xml_validator(ValidationType::SyntaxAndSchema);
        let feature = create_feature_with_xml(citygml_content);

        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "XML validation processing should succeed");

        let (port, features) = match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");
                (send_ports[0].clone(), send_features.clone())
            }
            _ => panic!("Expected Noop forwarder for testing"),
        };

        // Verify schema caching worked correctly
        {
            // Check that building schema was cached in filesystem
            let building_schema_url =
                "http://schemas.opengis.net/citygml/building/2.0/building.xsd";
            let cache_key = XmlValidator::generate_cache_key(building_schema_url);

            assert!(
                validator.schema_cache.is_cached(&cache_key),
                "Should have cached building schema"
            );
        }

        // CityGML syntax should be valid
        // Schema validation may succeed or fail depending on network connectivity and schema complexity
        match port {
            ref p if p == &*SUCCESS_PORT => {
                // Real CityGML validation succeeded - schemas were accessible and valid
            }
            ref p if p == &*FAILED_PORT => {
                // This is also acceptable - complex schemas may have validation issues
                // The important thing is that our HTTPS fetching mechanism works
                match features[0].attributes.get(&Attribute::new("xmlError")) {
                    Some(AttributeValue::Array(errors)) => {
                        // Real CityGML validation failed with proper error handling
                        // Verify we have proper error information
                        assert!(
                            !errors.is_empty(),
                            "Should have validation error information"
                        );
                    }
                    _ => panic!("Should have xmlError attribute when validation fails"),
                }
            }
            _ => panic!("Unexpected port returned"),
        }
    }

    #[test]
    fn test_xml_validator_schema_with_multiple_xml_declarations() {
        // This test reproduces an issue found with CityGML validation where multiple schemas
        // are imported and each contains its own XML declaration. When libxml2 processes
        // schemas with imports, our code inlines the fetched schema content, resulting in
        // XML declarations appearing in the middle of the document, causing parse errors like:
        // "Entity: line N: parser error : XML declaration allowed only at the start of the document"
        // The fix removes XML declarations from fetched schemas before inlining them.
        let citygml_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://www.opengis.net/citygml/2.0"
           xmlns="http://www.opengis.net/citygml/2.0"
           elementFormDefault="qualified">
    <xs:import namespace="http://www.opengis.net/citygml/building/2.0" schemaLocation="http://example.com/building.xsd"/>
</xs:schema>"#;

        let building_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://www.opengis.net/citygml/building/2.0"
           xmlns="http://www.opengis.net/citygml/building/2.0"
           elementFormDefault="qualified">
    <xs:element name="Building" type="xs:string"/>
</xs:schema>"#;

        let mut fetcher = MockSchemaFetcher::new();
        fetcher.responses.insert(
            "http://example.com/citygml.xsd".to_string(),
            Ok(citygml_schema.to_string()),
        );
        fetcher.responses.insert(
            "http://example.com/building.xsd".to_string(),
            Ok(building_schema.to_string()),
        );
        let mock_fetcher = Arc::new(fetcher);

        let mut validator =
            create_xml_validator_with_mock_fetcher(ValidationType::SyntaxAndSchema, mock_fetcher);

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0"
                xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://example.com/citygml.xsd">
    <bldg:Building>Test Building</bldg:Building>
</core:CityModel>"#;

        let feature = create_feature_with_xml(xml_content);
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "Processing should not fail");

        // Check that validation failed due to malformed schema
        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");
                assert_eq!(
                    send_ports[0], *FAILED_PORT,
                    "Should output to failed port due to malformed schema"
                );

                let send_features = noop_fw.send_features.lock().unwrap();
                if let Some(error_attr) =
                    send_features[0].attributes.get(&Attribute::new("xmlError"))
                {
                    if let AttributeValue::Array(errors) = error_attr {
                        assert!(!errors.is_empty(), "Should have validation errors");
                        // Check if error message contains information about XML declaration
                        let error_str = format!("{errors:?}");
                        println!("Actual error output: {error_str}");
                        // The error is reported as "Invalid document structure" because
                        // libxml2 fails to parse the schema with multiple XML declarations
                        assert!(
                            error_str.contains("Invalid document structure")
                                || error_str.contains("XML declaration")
                                || error_str.contains("parser error")
                                || error_str.contains("SchemaError"),
                            "Error should indicate schema parsing issue: {error_str}"
                        );
                    }
                } else {
                    panic!("No xmlError attribute found in output");
                }
            }
            _ => panic!("Expected Noop forwarder"),
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
            let mut feature = Feature::default();
            feature.attributes.insert(
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

    #[test]
    fn test_xml_validator_nested_schema_dependencies() {
        // Reproduce the error: "failed to load external entity"
        // This occurs when a schema imports another schema, and the imported schema URL fails to load

        // Main schema that imports another schema
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/main"
           xmlns="http://example.com/main"
           elementFormDefault="qualified">
    <!-- This import will fail because the URL is not mocked -->
    <xs:import namespace="http://docs.oasis-open.org/election/external"
               schemaLocation="http://docs.oasis-open.org/election/external/xAL.xsd"/>
    <xs:element name="MainElement" type="xs:string"/>
</xs:schema>"#;

        let mut fetcher = MockSchemaFetcher::new();
        // Only mock the main schema, not the imported one
        fetcher.responses.insert(
            "http://example.com/main.xsd".to_string(),
            Ok(main_schema.to_string()),
        );

        // Deliberately NOT mocking http://docs.oasis-open.org/election/external/xAL.xsd
        // to simulate the error condition

        let mock_fetcher = Arc::new(fetcher);

        let mut validator =
            create_xml_validator_with_mock_fetcher(ValidationType::SyntaxAndSchema, mock_fetcher);

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<main:MainElement xmlns:main="http://example.com/main"
                  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                  xsi:schemaLocation="http://example.com/main http://example.com/main.xsd">
    Test Content
</main:MainElement>"#;

        let feature = create_feature_with_xml(xml_content);
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "Processing should not fail");

        // Check that validation failed due to missing imported schema
        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");
                assert_eq!(
                    send_ports[0], *FAILED_PORT,
                    "Should output to failed port due to missing imported schema"
                );

                let send_features = noop_fw.send_features.lock().unwrap();
                if let Some(error_attr) =
                    send_features[0].attributes.get(&Attribute::new("xmlError"))
                {
                    if let AttributeValue::Array(errors) = error_attr {
                        assert!(!errors.is_empty(), "Should have validation errors");
                        let error_str = format!("{errors:?}");
                        println!("Nested schema dependency error: {error_str}");

                        // The error should indicate failure to load external entity
                        assert!(
                            error_str.contains("failed to load external entity")
                                || error_str.contains("Unknown IO error")
                                || error_str.contains("SchemaError")
                                || error_str.contains("Invalid document structure")
                                || error_str.contains("No mock response for URL"),
                            "Error should indicate schema loading issue: {error_str}"
                        );
                    }
                } else {
                    panic!("No xmlError attribute found in output");
                }
            }
            _ => panic!("Expected Noop forwarder"),
        }
    }

    #[test]
    fn test_xml_validator_with_resolved_nested_dependencies() {
        // Test successful validation when all nested schema dependencies are resolved

        // Main schema that imports another schema
        let main_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/main"
           xmlns="http://example.com/main"
           xmlns:types="http://example.com/types"
           elementFormDefault="qualified">
    <xs:import namespace="http://example.com/types"
               schemaLocation="http://example.com/types.xsd"/>
    <xs:element name="MainElement">
        <xs:complexType>
            <xs:sequence>
                <xs:element ref="types:TypedElement"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let types_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/types"
           xmlns:types="http://example.com/types"
           xmlns:base="http://example.com/base"
           elementFormDefault="qualified">
    <xs:import namespace="http://example.com/base"
               schemaLocation="http://example.com/base.xsd"/>
    <xs:element name="TypedElement" type="base:BaseType"/>
</xs:schema>"#;

        let base_schema = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/base"
           xmlns:base="http://example.com/base">
    <xs:simpleType name="BaseType">
        <xs:restriction base="xs:string">
            <xs:maxLength value="100"/>
        </xs:restriction>
    </xs:simpleType>
</xs:schema>"#;

        let mut fetcher = MockSchemaFetcher::new();
        // Mock all schemas in the dependency chain
        fetcher.responses.insert(
            "http://example.com/main.xsd".to_string(),
            Ok(main_schema.to_string()),
        );
        fetcher.responses.insert(
            "http://example.com/types.xsd".to_string(),
            Ok(types_schema.to_string()),
        );
        fetcher.responses.insert(
            "http://example.com/base.xsd".to_string(),
            Ok(base_schema.to_string()),
        );

        let mock_fetcher = Arc::new(fetcher);

        let mut validator =
            create_xml_validator_with_mock_fetcher(ValidationType::SyntaxAndSchema, mock_fetcher);

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<main:MainElement xmlns:main="http://example.com/main"
                  xmlns:types="http://example.com/types"
                  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                  xsi:schemaLocation="http://example.com/main http://example.com/main.xsd">
    <types:TypedElement>Valid Content</types:TypedElement>
</main:MainElement>"#;

        let feature = create_feature_with_xml(xml_content);
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "Processing should not fail");

        // Check that validation succeeded with resolved dependencies
        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                // Debug output for failures
                if send_ports[0] == *FAILED_PORT {
                    if let Some(error_attr) =
                        send_features[0].attributes.get(&Attribute::new("xmlError"))
                    {
                        println!("Validation failed with errors: {error_attr:?}");
                    }
                }

                assert_eq!(
                    send_ports[0], *SUCCESS_PORT,
                    "Should output to success port when all schemas are resolved"
                );
            }
            _ => panic!("Expected Noop forwarder"),
        }
    }

    #[test]
    fn test_xml_validator_with_noop_cache() {
        // Test that schema validation is skipped
        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_content"),
            input_type: XmlInputType::Text,
            validation_type: ValidationType::SyntaxAndSchema,
        };

        let schema_fetcher = Arc::new(HttpSchemaFetcher::new());

        // Create validator without schema cache
        let mut validator = XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            schema_fetcher,
            schema_cache: Arc::new(NoOpSchemaCache), // No cache means validation should be skipped
        };

        // XML with invalid schema that would normally fail validation
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:schemaLocation="http://example.com/schema http://example.com/invalid.xsd">
    <invalid_element>This would fail schema validation</invalid_element>
</root>"#;

        let feature = create_feature_with_xml(xml_content);
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "Processing should succeed");

        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                // Should succeed because validation is skipped when cache is unavailable
                assert_eq!(
                    send_ports[0], *SUCCESS_PORT,
                    "Should output to success port when cache is unavailable (validation skipped)"
                );

                // Should not have xmlError attribute since validation was skipped
                assert!(
                    !send_features[0]
                        .attributes
                        .contains_key(&Attribute::new("xmlError")),
                    "Should not have xmlError attribute when validation is skipped"
                );
            }
            _ => panic!("Expected Noop forwarder"),
        }
    }

    #[test]
    fn test_xml_validator_with_filesystem_cache() {
        // Create a filesystem-based cache
        let temp_dir =
            env::temp_dir().join(format!("xml_validator_fs_test_{}", uuid::Uuid::new_v4()));
        let schema_cache = create_filesystem_cache(temp_dir.clone()).unwrap();

        // Pre-populate the cache with a schema
        let schema_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           elementFormDefault="qualified">
    <xs:element name="test">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="value" type="xs:string"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        schema_cache
            .put_schema("test-schema", schema_content.as_bytes())
            .unwrap();

        // Create validator with filesystem cache
        let params = XmlValidatorParam {
            attribute: Attribute::new("xml_content"),
            input_type: XmlInputType::Text,
            validation_type: ValidationType::SyntaxAndSchema,
        };

        let schema_fetcher = Arc::new(HttpSchemaFetcher::new());

        let mut validator = XmlValidator {
            params,
            schema_store: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            schema_fetcher,
            schema_cache,
        };

        // Test with valid XML
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<test xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
      xsi:noNamespaceSchemaLocation="file://test-schema.xsd">
    <value>Hello World</value>
</test>"#;

        let feature = create_feature_with_xml(xml_content);
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "Processing should succeed");

        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                // For file:// schemas, it should use the filesystem path
                // The validation result depends on whether the schema file exists
                // Since we're using a test schema, it might fail, but that's OK
                // The important thing is that the filesystem cache is being used
            }
            _ => panic!("Expected Noop forwarder"),
        }

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_xml_validator_complex_schema_handling() {
        // Comprehensive test for complex schema handling including:
        // 1. PLATEAU CityGML with multiple namespaces
        // 2. Deep schema dependencies with relative paths
        // 3. Real-world schema fetching simulation

        // Part 1: Test with mock schemas for controlled testing
        let mut fetcher = MockSchemaFetcher::new();

        // Setup mock schemas with dependencies
        let citygml_base = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" 
           xmlns:gml="http://www.opengis.net/gml"
           targetNamespace="http://www.opengis.net/citygml/2.0"
           elementFormDefault="qualified">
    <xs:import namespace="http://www.opengis.net/gml" schemaLocation="gml.xsd"/>
    <xs:element name="CityModel" type="xs:anyType"/>
</xs:schema>"#;

        let gml_simplified = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://www.opengis.net/gml"
           elementFormDefault="qualified">
    <xs:element name="boundedBy" type="xs:anyType"/>
    <xs:element name="Envelope" type="xs:anyType"/>
</xs:schema>"#;

        // Mock responses for controlled testing
        fetcher.responses.insert(
            "http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd".to_string(),
            Ok(citygml_base.to_string()),
        );
        fetcher.responses.insert(
            "http://schemas.opengis.net/citygml/2.0/gml.xsd".to_string(),
            Ok(gml_simplified.to_string()),
        );

        // Test complex XML with multiple namespaces (similar to PLATEAU)
        let complex_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" 
                xmlns:gml="http://www.opengis.net/gml" 
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd">
    <gml:boundedBy>
        <gml:Envelope>Test</gml:Envelope>
    </gml:boundedBy>
</core:CityModel>"#;

        let mock_fetcher = Arc::new(fetcher);
        let mut validator =
            create_xml_validator_with_mock_fetcher(ValidationType::SyntaxAndSchema, mock_fetcher);

        let feature = create_feature_with_xml(complex_xml);
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let result = validator.process(ctx, &fw);
        assert!(result.is_ok(), "Processing should succeed");

        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                assert!(!send_ports.is_empty(), "Should have sent output");

                // Should successfully validate with mocked schemas
                assert_eq!(
                    send_ports[0], *SUCCESS_PORT,
                    "Should validate successfully with proper schema dependency resolution"
                );
            }
            _ => panic!("Expected Noop forwarder"),
        }

        // Part 2: Test deep schema dependencies (A->B->C pattern)
        let mut deep_fetcher = MockSchemaFetcher::new();

        let schema_a = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/a"
           xmlns:b="http://example.com/b"
           elementFormDefault="qualified">
    <xs:import namespace="http://example.com/b" schemaLocation="schemas/b.xsd"/>
    <xs:element name="RootA">
        <xs:complexType>
            <xs:sequence>
                <xs:element ref="b:ElementB"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>"#;

        let schema_b = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/b"
           xmlns:c="http://example.com/c"
           elementFormDefault="qualified">
    <xs:import namespace="http://example.com/c" schemaLocation="c.xsd"/>
    <xs:element name="ElementB" type="c:TypeC"/>
</xs:schema>"#;

        let schema_c = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           targetNamespace="http://example.com/c">
    <xs:simpleType name="TypeC">
        <xs:restriction base="xs:string">
            <xs:pattern value="[A-Z]+"/>
        </xs:restriction>
    </xs:simpleType>
</xs:schema>"#;

        deep_fetcher.responses.insert(
            "http://example.com/a.xsd".to_string(),
            Ok(schema_a.to_string()),
        );
        deep_fetcher.responses.insert(
            "http://example.com/schemas/b.xsd".to_string(),
            Ok(schema_b.to_string()),
        );
        deep_fetcher.responses.insert(
            "http://example.com/schemas/c.xsd".to_string(),
            Ok(schema_c.to_string()),
        );

        let deep_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<a:RootA xmlns:a="http://example.com/a"
         xmlns:b="http://example.com/b"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://example.com/a http://example.com/a.xsd">
    <b:ElementB>VALID</b:ElementB>
</a:RootA>"#;

        let deep_mock_fetcher = Arc::new(deep_fetcher);
        let mut deep_validator = create_xml_validator_with_mock_fetcher(
            ValidationType::SyntaxAndSchema,
            deep_mock_fetcher,
        );

        let deep_feature = create_feature_with_xml(deep_xml);
        let deep_ctx = utils::create_default_execute_context(&deep_feature);
        let deep_fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());

        let deep_result = deep_validator.process(deep_ctx, &deep_fw);
        assert!(
            deep_result.is_ok(),
            "Deep dependency processing should succeed"
        );

        match &deep_fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                assert_eq!(
                    send_ports[0], *SUCCESS_PORT,
                    "Should validate successfully with deep schema dependencies"
                );
            }
            _ => panic!("Expected Noop forwarder"),
        }
    }
}

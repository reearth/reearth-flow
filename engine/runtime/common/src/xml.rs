use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::marker::PhantomData;
use std::sync::Arc;

use fastxml::error::StructuredError;
use fastxml::schema::fetcher::{DefaultFetcher, SchemaFetcher};
use fastxml::schema::validator::{
    StreamValidator, XmlSchemaValidationContext as FastXmlValidationContext,
};
use fastxml::schema::xsd::{
    compile_schemas, parse_xsd_multiple, parse_xsd_with_imports, register_builtin_types,
    SchemaResolver, XsdSchema,
};
use fastxml::{
    create_context as fastxml_create_context, create_safe_context as fastxml_create_safe_context,
    evaluate as fastxml_evaluate, parse as fastxml_parse, NodeType,
};
use tracing::{debug, warn};

use crate::str::to_hash;
use crate::uri::Uri;

// Re-export types for compatibility
pub use fastxml::error::StructuredError as XmlStructuredError;
pub use fastxml::xpath::XPathResult as XmlXpathValue;
pub use fastxml::Namespace as XmlNamespace;
pub use fastxml::NodeType as XmlNodeType;
pub use fastxml::XmlDocument;
pub use fastxml::XmlNode;
pub use fastxml::XmlRoNode;

// Context type alias - fastxml provides this directly
pub type XmlContext = fastxml::xpath::XmlContext;

pub struct XmlSchemaValidationContext {
    inner: parking_lot::RwLock<FastXmlValidationContext>,
    _marker: PhantomData<*mut ()>,
}

unsafe impl Send for XmlSchemaValidationContext {}
unsafe impl Sync for XmlSchemaValidationContext {}

pub struct XmlSafeContext {
    inner: fastxml::xpath::XmlSafeContext,
    _marker: PhantomData<*mut ()>,
}

unsafe impl Send for XmlSafeContext {}
unsafe impl Sync for XmlSafeContext {}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct XmlRoNamespace {
    pub prefix: String,
    pub href: String,
}

impl From<XmlNamespace> for XmlRoNamespace {
    fn from(ns: XmlNamespace) -> Self {
        Self {
            prefix: ns.get_prefix().to_string(),
            href: ns.get_href().to_string(),
        }
    }
}

impl XmlRoNamespace {
    pub fn new(prefix: String, href: String) -> Self {
        Self { prefix, href }
    }

    pub fn get_prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn get_href(&self) -> String {
        self.href.clone()
    }
}

pub fn parse<T: AsRef<[u8]>>(xml: T) -> crate::Result<XmlDocument> {
    fastxml_parse(xml.as_ref()).map_err(|e| crate::Error::Xml(format!("{e}")))
}

pub fn evaluate<T: AsRef<str>>(document: &XmlDocument, xpath: T) -> crate::Result<XmlXpathValue> {
    fastxml_evaluate(document, xpath.as_ref())
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))
}

pub fn create_context(document: &XmlDocument) -> crate::Result<XmlContext> {
    let mut context = fastxml_create_context(document)
        .map_err(|_| crate::Error::Xml("Failed to initialize xpath context".to_string()))?;
    let root = document
        .get_root_element()
        .map_err(|e| crate::Error::Xml(format!("{e}")))?;

    let ns_decls = root.get_namespace_declarations();
    debug!(
        "xml::create_context: Root element '{}' has {} namespace declarations",
        root.get_name(),
        ns_decls.len()
    );

    for ns in ns_decls.iter() {
        debug!(
            "xml::create_context: Registering namespace - prefix='{}', href='{}'",
            ns.get_prefix(),
            ns.get_href()
        );
        context
            .register_namespace(ns.get_prefix(), ns.get_href())
            .map_err(|_| crate::Error::Xml("Failed to register namespace".to_string()))?;
    }

    if ns_decls.is_empty() {
        warn!(
            "xml::create_context: No namespace declarations found on root element '{}'. XPath queries with namespace prefixes may fail.",
            root.get_name()
        );
    }

    Ok(context)
}

pub fn create_safe_context(document: &XmlDocument) -> crate::Result<XmlSafeContext> {
    let context = fastxml_create_safe_context(document)
        .map_err(|_| crate::Error::Xml("Failed to initialize xpath context".to_string()))?;
    let root = document
        .get_root_element()
        .map_err(|e| crate::Error::Xml(format!("{e}")))?;

    let ns_decls = root.get_namespace_declarations();
    debug!(
        "xml::create_safe_context: Root element '{}' has {} namespace declarations",
        root.get_name(),
        ns_decls.len()
    );

    for ns in ns_decls.iter() {
        debug!(
            "xml::create_safe_context: Registering namespace - prefix='{}', href='{}'",
            ns.get_prefix(),
            ns.get_href()
        );
        context
            .register_namespace(ns.get_prefix(), ns.get_href())
            .map_err(|_| crate::Error::Xml("Failed to register namespace".to_string()))?;
    }

    if ns_decls.is_empty() {
        warn!(
            "xml::create_safe_context: No namespace declarations found on root element '{}'. XPath queries with namespace prefixes may fail.",
            root.get_name()
        );
    }

    Ok(XmlSafeContext {
        inner: context,
        _marker: PhantomData,
    })
}

pub fn collect_text_values(xpath_value: &XmlXpathValue) -> Vec<String> {
    fastxml::collect_text_values(xpath_value)
}

/// Convert XPathResult to serde_json::Value
pub fn xpath_value_to_json(value: &XmlXpathValue) -> serde_json::Value {
    match value {
        XmlXpathValue::Boolean(b) => serde_json::Value::Bool(*b),
        XmlXpathValue::Number(n) => {
            // Use serde_json::Number::from_f64 to avoid panicking on non-finite floats (NaN/±inf).
            // Fall back to Null when the number cannot be represented in JSON.
            if let Some(num) = serde_json::Number::from_f64(*n) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::Null
            }
        }
        XmlXpathValue::String(s) => serde_json::Value::String(s.clone()),
        XmlXpathValue::Nodes(nodes) => {
            let values: Vec<serde_json::Value> = nodes
                .iter()
                .map(|node| {
                    // For nodeset, collect text content
                    node.get_content()
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null)
                })
                .collect();
            if values.len() == 1 {
                values.into_iter().next().unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Array(values)
            }
        }
    }
}

/// Get nodes from XPathResult as a Vec
/// This is a helper to provide the same API as the old libxml
pub trait XPathResultExt {
    fn get_nodes_as_vec(&self) -> Vec<XmlNode>;
    fn get_readonly_nodes_as_vec(&self) -> Vec<XmlRoNode>;
}

impl XPathResultExt for XmlXpathValue {
    fn get_nodes_as_vec(&self) -> Vec<XmlNode> {
        match self {
            XmlXpathValue::Nodes(nodes) => nodes.clone(),
            _ => Vec::new(),
        }
    }

    fn get_readonly_nodes_as_vec(&self) -> Vec<XmlRoNode> {
        match self {
            XmlXpathValue::Nodes(nodes) => {
                nodes.iter().cloned().map(XmlRoNode::from_node).collect()
            }
            _ => Vec::new(),
        }
    }
}

/// Convert XPathResult to a display string
pub fn xpath_value_to_string(value: &XmlXpathValue) -> String {
    match value {
        XmlXpathValue::Boolean(b) => b.to_string(),
        XmlXpathValue::Number(n) => n.to_string(),
        XmlXpathValue::String(s) => s.clone(),
        XmlXpathValue::Nodes(nodes) => {
            let texts: Vec<String> = nodes.iter().filter_map(|node| node.get_content()).collect();
            texts.join("")
        }
    }
}

pub fn collect_text_value(xpath_value: &XmlXpathValue) -> String {
    let v = collect_text_values(xpath_value);
    if v.is_empty() {
        "".to_string()
    } else {
        v[0].to_string()
    }
}

pub fn get_node_prefix(node: &XmlNode) -> String {
    match node.get_namespace() {
        Some(ns) => ns.get_prefix().to_string(),
        None => "".to_string(),
    }
}

pub fn get_readonly_node_prefix(node: &XmlRoNode) -> String {
    match node.get_namespace() {
        Some(ns) => ns.get_prefix().to_string(),
        None => "".to_string(),
    }
}

pub fn get_node_tag(node: &XmlNode) -> String {
    // Use qname() to get the full qualified name including prefix,
    // which preserves the original prefix even for undeclared namespaces.
    node.qname().to_string()
}

pub fn get_node_id(uri: &Uri, node: &XmlNode) -> String {
    node.get_attributes()
        .get("id")
        .map(|id| id.to_string())
        .unwrap_or_else(|| {
            let tag = get_node_tag(node);
            let mut key_values = node
                .get_attributes()
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>();
            key_values.sort();
            to_hash(format!("{}:{}[{}]", uri, tag, key_values.join(",")).as_str())
        })
}

pub fn get_readonly_node_tag(node: &XmlRoNode) -> String {
    // Use qname() to get the full qualified name including prefix,
    // which preserves the original prefix even for undeclared namespaces.
    node.qname().to_string()
}

pub fn get_root_node(document: &XmlDocument) -> crate::Result<XmlNode> {
    document
        .get_root_element()
        .map_err(|e| crate::Error::Xml(format!("{e}")))
}

pub fn get_root_readonly_node(document: &XmlDocument) -> crate::Result<XmlRoNode> {
    document
        .get_root_element_ro()
        .map_err(|e| crate::Error::Xml(format!("{e}")))
}

pub fn node_to_xml_string(document: &XmlDocument, node: &mut XmlNode) -> crate::Result<String> {
    fastxml::node_to_xml_string(document, node).map_err(|e| crate::Error::Xml(format!("{e}")))
}

pub fn readonly_node_to_xml_string(
    document: &XmlDocument,
    node: &XmlRoNode,
) -> crate::Result<String> {
    fastxml::readonly_node_to_xml_string(document, node)
        .map_err(|e| crate::Error::Xml(format!("{e}")))
}

pub fn parse_schema_locations(document: &XmlDocument) -> crate::Result<Vec<(String, String)>> {
    let root = get_root_node(document)?;
    let mut schema_locations = Vec::new();
    let mut namespaces = HashMap::new();
    root.get_namespace_declarations().iter().for_each(|ns| {
        namespaces.insert(ns.get_href(), ns.get_prefix());
    });
    for (key, value) in root.get_attributes().iter() {
        if key == "schemaLocation" {
            schema_locations = value.split_whitespace().map(|s| s.to_string()).collect();
        } else if key == "noNamespaceSchemaLocation" {
            // Handle xsi:noNamespaceSchemaLocation - use empty string for namespace
            return Ok(vec![("".to_string(), value.to_string())]);
        }
    }

    let mut result = Vec::<(String, String)>::new();
    for i in (0..schema_locations.len()).step_by(2) {
        if i + 1 < schema_locations.len() {
            result.push((
                schema_locations[i].to_string(),
                schema_locations[i + 1].to_string(),
            ));
        }
    }
    Ok(result)
}

pub fn create_xml_schema_validation_context(
    schema_location: String,
) -> crate::Result<XmlSchemaValidationContext> {
    let ctx = fastxml::create_xml_schema_validation_context(schema_location)
        .map_err(|e| crate::Error::Xml(format!("Failed to parse schema: {e:?}")))?;
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

pub fn create_xml_schema_validation_context_from_buffer(
    schema: &[u8],
) -> crate::Result<XmlSchemaValidationContext> {
    let ctx = fastxml::create_xml_schema_validation_context_from_buffer(schema)
        .map_err(|e| crate::Error::Xml(format!("Failed to parse schema: {e:?}")))?;
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

/// Create schema validation context from multiple schema sources.
/// This is useful when you have a wrapper schema that includes/imports other schemas.
#[allow(dead_code)]
pub fn create_xml_schema_validation_context_from_multiple(
    schemas: &[(&str, &[u8])],
) -> crate::Result<XmlSchemaValidationContext> {
    let compiled = parse_xsd_multiple(schemas)
        .map_err(|e| crate::Error::Xml(format!("Failed to parse schemas: {e:?}")))?;
    let ctx = FastXmlValidationContext::new(compiled);
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

/// Create schema validation context with automatic import resolution.
///
/// This function uses fastxml's built-in schema fetcher and resolver to:
/// 1. Fetch the schema from the given URI (HTTP or local file)
/// 2. Recursively resolve all xs:import and xs:include dependencies
/// 3. Compile all schemas into a single validation context
///
/// # Arguments
/// * `schema_uri` - The URI of the main schema (HTTP URL or local file path)
/// * `base_dir` - Optional base directory for resolving relative paths
///
/// # Returns
/// A compiled schema validation context with all dependencies resolved
pub fn create_validation_context_with_imports(
    schema_uri: &str,
    base_dir: Option<&std::path::Path>,
) -> crate::Result<XmlSchemaValidationContext> {
    // Create fetcher with optional base directory
    let fetcher = match base_dir {
        Some(dir) => DefaultFetcher::with_base_dir(dir),
        None => DefaultFetcher::new(),
    };

    // Fetch the main schema
    let fetch_result = fetcher
        .fetch(schema_uri)
        .map_err(|e| crate::Error::Xml(format!("Failed to fetch schema {}: {e:?}", schema_uri)))?;

    // Use the final URL (after redirects) as the base URI
    let base_uri = &fetch_result.final_url;

    // Parse with import resolution
    let compiled =
        parse_xsd_with_imports(&fetch_result.content, base_uri, &fetcher).map_err(|e| {
            crate::Error::Xml(format!(
                "Failed to parse schema with imports {}: {e:?}",
                schema_uri
            ))
        })?;

    let ctx = FastXmlValidationContext::new(compiled);
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

/// Create schema validation context for multiple schemas with automatic import resolution.
///
/// This function processes multiple schema locations and combines them into a single
/// validation context. Each schema and its dependencies are resolved automatically.
///
/// # Arguments
/// * `schema_locations` - List of (namespace, location) pairs from xsi:schemaLocation
/// * `base_dir` - Optional base directory for resolving relative paths
///
/// # Returns
/// A compiled schema validation context with all schemas and dependencies resolved
pub fn create_validation_context_for_schema_locations(
    schema_locations: &[(String, String)],
    base_dir: Option<&std::path::Path>,
) -> crate::Result<XmlSchemaValidationContext> {
    let compiled = compile_schemas_from_locations(schema_locations, base_dir)?;
    let ctx = FastXmlValidationContext::new(compiled);
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

pub fn validate_document_by_schema(
    document: &XmlDocument,
    schema_location: String,
) -> crate::Result<Vec<StructuredError>> {
    let xsd_validator = create_xml_schema_validation_context(schema_location)?;
    validate_document_by_schema_context(document, &xsd_validator)
}

pub fn validate_document_by_schema_context(
    document: &XmlDocument,
    xsd_validator: &XmlSchemaValidationContext,
) -> crate::Result<Vec<StructuredError>> {
    match xsd_validator.inner.read().validate(document) {
        Ok(errors) => Ok(errors),
        Err(e) => Err(crate::Error::Xml(format!("Validation error: {e:?}"))),
    }
}

// =============================================================================
// Streaming Validation (StreamValidator) - Memory efficient
// =============================================================================

pub use fastxml::schema::types::CompiledSchema;

/// Compile schemas for streaming validation using StreamValidator.
/// This returns an Arc<CompiledSchema> that can be reused across multiple validations.
pub fn compile_schema_for_streaming(
    schema_locations: &[(String, String)],
    base_dir: Option<&std::path::Path>,
) -> crate::Result<Arc<CompiledSchema>> {
    let compiled = compile_schemas_from_locations(schema_locations, base_dir)?;
    Ok(Arc::new(compiled))
}

/// Fetch, resolve, and compile XSD schemas from schema location pairs.
fn compile_schemas_from_locations(
    schema_locations: &[(String, String)],
    base_dir: Option<&std::path::Path>,
) -> crate::Result<CompiledSchema> {
    if schema_locations.is_empty() {
        return Err(crate::Error::Xml(
            "No schema locations provided".to_string(),
        ));
    }

    let fetcher = match base_dir {
        Some(dir) => DefaultFetcher::with_base_dir(dir),
        None => DefaultFetcher::new(),
    };

    let mut all_schemas: Vec<XsdSchema> = Vec::new();

    for (_namespace, location) in schema_locations {
        let fetch_result = fetcher.fetch(location).map_err(|e| {
            crate::Error::Xml(format!("Failed to fetch schema {}: {e:?}", location))
        })?;

        let base_uri = &fetch_result.final_url;
        let mut resolver = SchemaResolver::new(&fetcher);
        let schemas = resolver
            .resolve_all(&fetch_result.content, base_uri)
            .map_err(|e| {
                crate::Error::Xml(format!(
                    "Failed to resolve schema imports for {}: {e:?}",
                    location
                ))
            })?;

        all_schemas.extend(schemas);
    }

    let mut compiled = compile_schemas(all_schemas)
        .map_err(|e| crate::Error::Xml(format!("Failed to compile schemas: {e:?}")))?;
    register_builtin_types(&mut compiled);

    Ok(compiled)
}

/// Validate XML using StreamValidator (streaming, memory-efficient).
/// This function does NOT parse the XML into a DOM tree, significantly reducing memory usage.
///
/// # Arguments
/// * `xml_bytes` - The XML content as bytes
/// * `schema` - Compiled schema to validate against
/// * `max_errors` - Maximum number of errors to collect (0 for unlimited)
///
/// # Returns
/// Vector of validation errors, or error if validation setup failed
pub fn validate_xml_streaming(
    xml_bytes: &[u8],
    schema: &Arc<CompiledSchema>,
    max_errors: usize,
) -> crate::Result<Vec<StructuredError>> {
    let reader = BufReader::new(Cursor::new(xml_bytes));
    let mut validator = StreamValidator::new(Arc::clone(schema));

    if max_errors > 0 {
        validator = validator.with_max_errors(max_errors);
    }

    validator
        .validate(reader)
        .map_err(|e| crate::Error::Xml(format!("Streaming validation failed: {e:?}")))
}

/// Validate XML by automatically extracting schema location from the document.
/// This is a convenience function that combines schema extraction and streaming validation.
///
/// # Arguments
/// * `xml_bytes` - The XML content as bytes
/// * `base_dir` - Optional base directory for resolving relative schema paths
/// * `max_errors` - Maximum number of errors to collect (0 for unlimited)
pub fn validate_xml_streaming_from_schema_location(
    xml_bytes: &[u8],
    base_dir: Option<&std::path::Path>,
    max_errors: usize,
) -> crate::Result<Vec<StructuredError>> {
    // First, parse just enough to get schema locations (quick-xml based, not full DOM)
    let doc = fastxml_parse(xml_bytes).map_err(|e| crate::Error::Xml(format!("{e}")))?;
    let locations = parse_schema_locations(&doc)?;

    if locations.is_empty() {
        // No schema location - return empty (valid)
        return Ok(Vec::new());
    }

    let schema = compile_schema_for_streaming(&locations, base_dir)?;
    validate_xml_streaming(xml_bytes, &schema, max_errors)
}

pub fn find_nodes_by_xpath(
    ctx: &XmlContext,
    xpath: &str,
    node: &XmlNode,
) -> crate::Result<Vec<XmlNode>> {
    let result = ctx
        .evaluate_from(xpath, node)
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))?;
    let result = result
        .into_nodes()
        .into_iter()
        .filter(|node| node.get_type() == NodeType::Element)
        .collect::<Vec<_>>();
    Ok(result)
}

pub fn find_readonly_nodes_by_xpath(
    ctx: &XmlContext,
    xpath: &str,
    node: &XmlRoNode,
) -> crate::Result<Vec<XmlRoNode>> {
    let result = fastxml::find_readonly_nodes_by_xpath(ctx, xpath, node)
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))?;
    let result = result
        .into_iter()
        .filter(|node| node.get_type() == NodeType::Element)
        .collect::<Vec<_>>();
    Ok(result)
}

pub fn find_safe_readonly_nodes_by_xpath(
    ctx: &XmlSafeContext,
    xpath: &str,
    node: &XmlRoNode,
) -> crate::Result<Vec<XmlRoNode>> {
    let result = fastxml::find_safe_readonly_nodes_by_xpath(&ctx.inner, xpath, node)
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))?;
    let result = result
        .into_iter()
        .filter(|node| node.get_type() == NodeType::Element)
        .collect::<Vec<_>>();
    Ok(result)
}

pub fn find_readonly_nodes_in_elements(
    ctx: &XmlContext,
    node: &XmlRoNode,
    elements_to_match: &[&str],
) -> crate::Result<Vec<XmlRoNode>> {
    let elements_to_match = elements_to_match
        .iter()
        .map(|element| format!("name()='{element}'"))
        .collect::<Vec<_>>();
    let elements_to_match_query = elements_to_match.join(" or ");
    let elements_to_match_query = format!("({elements_to_match_query})");
    let xpath = format!("//*[{elements_to_match_query}]");
    let nodes = find_readonly_nodes_by_xpath(ctx, &xpath, node)
        .map_err(|e| crate::Error::Xml(format!("Failed to evaluate xpath with {e}")))?;
    Ok(nodes)
}

pub fn find_safe_readonly_nodes_in_elements(
    ctx: &XmlSafeContext,
    node: &XmlRoNode,
    elements_to_match: &[&str],
) -> crate::Result<Vec<XmlRoNode>> {
    let elements_to_match = elements_to_match
        .iter()
        .map(|element| format!("name()='{element}'"))
        .collect::<Vec<_>>();
    let elements_to_match_query = elements_to_match.join(" or ");
    let elements_to_match_query = format!("({elements_to_match_query})");
    let xpath = format!("//*[{elements_to_match_query}]");
    let nodes = find_safe_readonly_nodes_by_xpath(ctx, &xpath, node)
        .map_err(|e| crate::Error::Xml(format!("Failed to evaluate xpath with {e}")))?;
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let xml = r#"<root><element>Test</element></root>"#;
        let document = parse(xml).unwrap();
        let serialized = fastxml::serialize::document_to_xml_string(&document).unwrap();
        // fastxml serializes without trailing newlines
        assert_eq!(
            serialized,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root><element>Test</element></root>"
        );
    }

    #[test]
    fn test_evaluate() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <gml:Dictionary xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:gml="http://www.opengis.net/gml" gml:id="Agreement_class">
            <gml:name>Agreement_class</gml:name>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id1">
                    <gml:description>building agreement</gml:description>
                    <gml:name>1010</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id2">
                    <gml:description>green space agreement</gml:description>
                    <gml:name>1020</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id3">
                    <gml:description>landscape agreement</gml:description>
                    <gml:name>1030</gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
            <gml:dictionaryEntry>
                <gml:Definition gml:id="id4">
                    <gml:description>development permit</gml:description>
                    <gml:name>1040<gml:hoge>hogehoge</gml:hoge></gml:name>
                </gml:Definition>
            </gml:dictionaryEntry>
        </gml:Dictionary>
                "#;
        let document = parse(xml).unwrap();
        let value = evaluate(
            &document,
            "/gml:Dictionary/gml:dictionaryEntry/gml:Definition/gml:name/text()",
        )
        .unwrap();
        assert_eq!(
            collect_text_values(&value),
            vec!["1010", "1020", "1030", "1040"]
        );

        let document = parse(xml).unwrap();
        let value = evaluate(&document, "/gml:Dictionary").unwrap();
        let values = value.get_nodes_as_vec();
        let root = values.first().unwrap();
        let ctx = create_context(&document).unwrap();
        let result = ctx
            .evaluate_from("//*[name()='gml:Definition']", root)
            .unwrap();
        let result = result.get_nodes_as_vec();
        for node in result {
            let tag = get_node_tag(&node);
            println!(
                "gml id: {:?}",
                node.get_attribute_ns("id", "http://www.opengis.net/gml")
            );
            println!("tag: {tag}");
        }
    }

    #[test]
    fn test_parse_schema_locations() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="https://www.geospatial.jp/iur/uro/3.0 ../../schemas/iur/uro/3.0/urbanObject.xsd http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/landuse/2.0 http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd http://www.opengis.net/citygml/generics/2.0 http://schemas.opengis.net/citygml/generics/2.0/generics.xsd http://www.opengis.net/citygml/cityobjectgroup/2.0 http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd http://www.opengis.net/citygml/appearance/2.0 http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd">
            <gml:boundedBy>
                    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
                            <gml:lowerCorner>35.79152909951733 139.85598420153815 0</gml:lowerCorner>
                            <gml:upperCorner>35.79167257523232 139.85619582799433 7.121</gml:upperCorner>
                    </gml:Envelope>
            </gml:boundedBy>
        </core:CityModel>
"#;
        let document = parse(xml).unwrap();
        let result = parse_schema_locations(&document).unwrap();
        assert_eq!(
            result,
            vec![
                (
                    "https://www.geospatial.jp/iur/uro/3.0".to_string(),
                    "../../schemas/iur/uro/3.0/urbanObject.xsd".to_string()
                ),
                (
                    "http://www.opengis.net/citygml/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd".to_string()
                ),
                (
                    "http://www.opengis.net/citygml/landuse/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd".to_string()
                ),
                (
                    "http://www.opengis.net/citygml/building/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/building/2.0/building.xsd".to_string()
                ),
                (
                    "http://www.opengis.net/citygml/transportation/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd"
                        .to_string()
                ),
                (
                    "http://www.opengis.net/citygml/generics/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/generics/2.0/generics.xsd".to_string()
                ),
                (
                    "http://www.opengis.net/citygml/cityobjectgroup/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd"
                        .to_string()
                ),
                (
                    "http://www.opengis.net/gml".to_string(),
                    "http://schemas.opengis.net/gml/3.1.1/base/gml.xsd".to_string()
                ),
                (
                    "http://www.opengis.net/citygml/appearance/2.0".to_string(),
                    "http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd".to_string()
                )
            ]
        )
    }

    #[test]
    fn test_find_readonly_nodes_by_xpath() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <bldg:Building xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" gml:id="bldg_6797d950-4b05-4a26-982c-ba3f4b221f6b">
                                <core:creationDate>2025-03-21</core:creationDate>
                                <bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
                                <bldg:usage codeSpace="../../codelists/Building_usage.xml">461</bldg:usage>
                                <bldg:measuredHeight uom="m">4.6</bldg:measuredHeight>
                                <uro:buildingIDAttribute>
                                        <uro:BuildingIDAttribute>
                                                <uro:buildingID>12206-bldg-51914</uro:buildingID>
                                                <uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
                                                <uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12206</uro:city>
                                        </uro:BuildingIDAttribute>
                                </uro:buildingIDAttribute>
                        </bldg:Building>
        "#;
        let document = parse(xml).unwrap();
        let ctx = create_context(&document).unwrap();
        let root = get_root_readonly_node(&document).unwrap();
        let result = find_readonly_nodes_by_xpath(
            &ctx,
            ".//uro:buildingIDAttribute/uro:BuildingIDAttribute/uro:buildingID",
            &root,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_namespace_uri_xpath_function() {
        // Test that namespace-uri() and local-name() XPath functions work
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>36.6470041354812 137.05268308385453 0</gml:lowerCorner>
            <gml:upperCorner>36.647798243275254 137.0537094956814 105.03314</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <core:cityObjectMember>
        <bldg:Building gml:id="bldg_test">
            <core:creationDate>2025-03-21</core:creationDate>
        </bldg:Building>
    </core:cityObjectMember>
</core:CityModel>"#;

        let document = parse(xml).unwrap();
        let ctx = create_context(&document).unwrap();
        let root = get_root_readonly_node(&document).unwrap();

        // Test using namespace-uri() and local-name() XPath functions (supported in fastxml)
        let result = find_readonly_nodes_by_xpath(
            &ctx,
            ".//*[namespace-uri()='http://www.opengis.net/gml' and local-name()='Envelope']",
            &root,
        );
        println!("Result for namespace-uri() query: {:?}", result);
        assert!(
            result.is_ok(),
            "namespace-uri() XPath query failed: {:?}",
            result.err()
        );
        let nodes = result.unwrap();
        println!("Found {} nodes with namespace-uri() query", nodes.len());
        assert_eq!(
            nodes.len(),
            1,
            "Expected 1 Envelope node with namespace-uri(), found {}",
            nodes.len()
        );

        // Also test the prefixed version for comparison
        let result_prefixed = find_readonly_nodes_by_xpath(&ctx, ".//gml:Envelope", &root);
        assert!(result_prefixed.is_ok(), "Prefixed XPath query failed");
        assert_eq!(
            result_prefixed.unwrap().len(),
            1,
            "Expected 1 Envelope node with prefix"
        );

        // Test get_child_nodes and get_readonly_node_tag
        let envelope = &nodes[0];
        let children = envelope.get_child_nodes();
        println!("Envelope has {} children", children.len());
        for child in children.iter() {
            let tag = get_readonly_node_tag(child);
            let node_type = child.get_type();
            let ns_info = child
                .get_namespace()
                .map(|n| format!("{}={}", n.get_prefix(), n.get_href()));
            println!(
                "  Child: tag='{}', type={:?}, namespace={:?}",
                tag, node_type, ns_info
            );
        }

        // In fastxml, get_namespace() works correctly for child elements
        let lower_corner = children
            .iter()
            .find(|&n| get_readonly_node_tag(n) == "gml:lowerCorner");
        assert!(
            lower_corner.is_some(),
            "Should find gml:lowerCorner in Envelope children"
        );

        // Test get_attribute_ns for namespaced attributes
        let building = find_readonly_nodes_by_xpath(&ctx, ".//bldg:Building", &root).unwrap();
        assert_eq!(building.len(), 1, "Should find bldg:Building");
        let building_node = &building[0];

        // Test gml:id attribute - should work in fastxml
        let gml_id = building_node.get_attribute_ns("id", "http://www.opengis.net/gml");
        println!("gml:id via get_attribute_ns: {:?}", gml_id);
        assert_eq!(
            gml_id,
            Some("bldg_test".to_string()),
            "get_attribute_ns should return gml:id value"
        );

        // Also try regular get_attribute - returns local name as key in fastxml
        let attrs = building_node.get_attributes();
        println!("All attributes: {:?}", attrs);
        assert!(
            attrs.contains_key("id"),
            "Attributes should contain 'id' key"
        );

        // Test that root node has proper namespace
        let root_name = root.get_name();
        let root_tag = get_readonly_node_tag(&root);
        println!("Root name (get_name): {}", root_name);
        println!("Root tag (get_readonly_node_tag): {}", root_tag);
        let root_ns = root.get_namespace();
        println!(
            "Root namespace: {:?}",
            root_ns
                .as_ref()
                .map(|n| format!("{}={}", n.get_prefix(), n.get_href()))
        );

        // In fastxml, get_namespace() returns correct namespace,
        // so get_readonly_node_tag returns tag with prefix
        assert_eq!(
            root_tag, "core:CityModel",
            "Root tag should have namespace prefix in fastxml"
        );
        assert!(root_ns.is_some(), "Root should have namespace");

        // Test building node namespace
        let building_name = building_node.get_name();
        let building_tag = get_readonly_node_tag(building_node);
        println!("Building name (get_name): {}", building_name);
        println!("Building tag (get_readonly_node_tag): {}", building_tag);
        let building_ns = building_node.get_namespace();
        println!(
            "Building namespace: {:?}",
            building_ns
                .as_ref()
                .map(|n| format!("{}={}", n.get_prefix(), n.get_href()))
        );
        assert_eq!(
            building_tag, "bldg:Building",
            "Building tag should have namespace prefix in fastxml"
        );
        assert!(building_ns.is_some(), "Building should have namespace");
    }
}

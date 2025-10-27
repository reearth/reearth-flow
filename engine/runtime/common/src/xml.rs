use std::collections::HashMap;
use std::marker::PhantomData;

use libxml::error::StructuredError;
use libxml::parser::{Parser, ParserOptions};
use libxml::schemas::SchemaValidationContext;
use libxml::tree::document;
use libxml::xpath::Context;
use tracing::{debug, warn};

use crate::str::to_hash;
use crate::uri::Uri;

pub type XmlDocument = document::Document;
pub type XmlXpathValue = libxml::xpath::Object;
pub type XmlContext = libxml::xpath::Context;
pub type XmlNode = libxml::tree::Node;
pub type XmlRoNode = libxml::readonly::RoNode;
pub type XmlNamespace = libxml::tree::Namespace;
pub type XmlNodeType = libxml::tree::NodeType;
pub type XmlSchemaParserContext = libxml::schemas::SchemaParserContext;

pub struct XmlSchemaValidationContext {
    inner: parking_lot::RwLock<SchemaValidationContext>,
    _marker: PhantomData<*mut ()>,
}

unsafe impl Send for XmlSchemaValidationContext {}
unsafe impl Sync for XmlSchemaValidationContext {}

pub struct XmlSafeContext {
    inner: parking_lot::RwLock<Context>,
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
            prefix: ns.get_prefix(),
            href: ns.get_href(),
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
    let parser = Parser::default();
    parser
        .parse_string_with_options(
            xml,
            ParserOptions {
                recover: false,
                no_def_dtd: true,
                no_error: true,
                no_warning: false,
                pedantic: true,
                no_blanks: false,
                no_net: false,
                no_implied: false,
                compact: true,
                ignore_enc: false,
                encoding: None,
                huge: false,
            },
        )
        .map_err(|e| crate::Error::Xml(format!("{e}")))
}

pub fn evaluate<T: AsRef<str>>(document: &XmlDocument, xpath: T) -> crate::Result<XmlXpathValue> {
    let context = create_context(document)?;
    context
        .evaluate(xpath.as_ref())
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))
}

pub fn create_context(document: &XmlDocument) -> crate::Result<XmlContext> {
    let context = Context::new(document)
        .map_err(|_| crate::Error::Xml("Failed to initialize xpath context".to_string()))?;
    let root = document
        .get_root_element()
        .ok_or(crate::Error::Xml("No root element".to_string()))?;

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
            .register_namespace(ns.get_prefix().as_str(), ns.get_href().as_str())
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
    let context = Context::new(document)
        .map_err(|_| crate::Error::Xml("Failed to initialize xpath context".to_string()))?;
    let root = document
        .get_root_element()
        .ok_or(crate::Error::Xml("No root element".to_string()))?;

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
            .register_namespace(ns.get_prefix().as_str(), ns.get_href().as_str())
            .map_err(|_| crate::Error::Xml("Failed to register namespace".to_string()))?;
    }

    if ns_decls.is_empty() {
        warn!(
            "xml::create_safe_context: No namespace declarations found on root element '{}'. XPath queries with namespace prefixes may fail.",
            root.get_name()
        );
    }

    Ok(XmlSafeContext {
        inner: parking_lot::RwLock::new(context),
        _marker: PhantomData,
    })
}

pub fn collect_text_values(xpath_value: &XmlXpathValue) -> Vec<String> {
    xpath_value
        .get_nodes_as_vec()
        .iter()
        .map(|node| node.get_content())
        .collect()
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
        Some(ns) => ns.get_prefix(),
        None => "".to_string(),
    }
}

pub fn get_readonly_node_prefix(node: &XmlRoNode) -> String {
    match node.get_namespace() {
        Some(ns) => ns.get_prefix(),
        None => "".to_string(),
    }
}

pub fn get_node_tag(node: &XmlNode) -> String {
    match node.get_namespace() {
        Some(ns) => format!("{}:{}", ns.get_prefix(), node.get_name()).to_string(),
        None => node.get_name(),
    }
}

pub fn get_node_id(uri: &Uri, node: &XmlNode) -> String {
    node.get_attributes()
        .get("id")
        .map(|id| id.to_string())
        .unwrap_or_else(|| {
            let tag = get_node_tag(node);
            let mut key_values = node
                .get_properties()
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>();
            key_values.sort();
            to_hash(format!("{}:{}[{}]", uri, tag, key_values.join(",")).as_str())
        })
}

pub fn get_readonly_node_tag(node: &XmlRoNode) -> String {
    match node.get_namespace() {
        Some(ns) => format!("{}:{}", ns.get_prefix(), node.get_name()).to_string(),
        None => node.get_name(),
    }
}

pub fn get_root_node(document: &XmlDocument) -> crate::Result<XmlNode> {
    document
        .get_root_element()
        .ok_or(crate::Error::Xml("No root element".to_string()))
}

pub fn get_root_readonly_node(document: &XmlDocument) -> crate::Result<XmlRoNode> {
    document
        .get_root_readonly()
        .ok_or(crate::Error::Xml("No root element".to_string()))
}

pub fn node_to_xml_string(document: &XmlDocument, node: &mut XmlNode) -> crate::Result<String> {
    let doc =
        parse(document.node_to_string(node)).map_err(|e| crate::Error::Xml(format!("{e}")))?;
    Ok(doc.to_string())
}

pub fn readonly_node_to_xml_string(
    document: &XmlDocument,
    node: &XmlRoNode,
) -> crate::Result<String> {
    let doc =
        parse(document.ronode_to_string(node)).map_err(|e| crate::Error::Xml(format!("{e}")))?;
    Ok(doc.to_string())
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
    let mut xsd_parser = XmlSchemaParserContext::from_file(schema_location.as_str());
    let ctx = SchemaValidationContext::from_parser(&mut xsd_parser)
        .map_err(|e| crate::Error::Xml(format!("Failed to parse schema: {e:?}")))?;
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

pub fn create_xml_schema_validation_context_from_buffer(
    schema: &[u8],
) -> crate::Result<XmlSchemaValidationContext> {
    let mut xsd_parser = XmlSchemaParserContext::from_buffer(schema);
    let ctx = SchemaValidationContext::from_parser(&mut xsd_parser)
        .map_err(|e| crate::Error::Xml(format!("Failed to parse schema: {e:?}")))?;
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
    match xsd_validator.inner.write().validate_document(document) {
        Ok(_) => Ok(vec![]),
        Err(e) => Ok(e),
    }
}

pub fn validate_node_by_schema_context(
    node: &XmlNode,
    xsd_validator: &XmlSchemaValidationContext,
) -> crate::Result<Vec<StructuredError>> {
    match xsd_validator.inner.write().validate_node(node) {
        Ok(_) => Ok(vec![]),
        Err(e) => Ok(e),
    }
}

pub fn find_nodes_by_xpath(
    ctx: &XmlContext,
    xpath: &str,
    node: &XmlNode,
) -> crate::Result<Vec<XmlNode>> {
    let result = ctx
        .node_evaluate(xpath, node)
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))?;
    let result = result
        .get_nodes_as_vec()
        .into_iter()
        .filter(|node| {
            if let Some(node_type) = node.get_type() {
                node_type == XmlNodeType::ElementNode
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    Ok(result)
}

pub fn find_readonly_nodes_by_xpath(
    ctx: &XmlContext,
    xpath: &str,
    node: &XmlRoNode,
) -> crate::Result<Vec<XmlRoNode>> {
    let result = ctx
        .node_evaluate_readonly(xpath, *node)
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))?;
    let result = result
        .get_readonly_nodes_as_vec()
        .into_iter()
        .filter(|node| {
            if let Some(node_type) = node.get_type() {
                node_type == XmlNodeType::ElementNode
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    Ok(result)
}

pub fn find_safe_readonly_nodes_by_xpath(
    ctx: &XmlSafeContext,
    xpath: &str,
    node: &XmlRoNode,
) -> crate::Result<Vec<XmlRoNode>> {
    let result = ctx
        .inner
        .read()
        .node_evaluate_readonly(xpath, *node)
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))?;
    let result = result
        .get_readonly_nodes_as_vec()
        .into_iter()
        .filter(|node| {
            if let Some(node_type) = node.get_type() {
                node_type == XmlNodeType::ElementNode
            } else {
                false
            }
        })
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
        assert_eq!(
            document.to_string(),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root><element>Test</element></root>\n"
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
            .node_evaluate("//*[name()='gml:Definition']", root)
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
                                <bldg:lod0RoofEdge>
                                        <gml:MultiSurface>
                                                <gml:surfaceMember>
                                                        <gml:Polygon>
                                                                <gml:exterior>
                                                                        <gml:LinearRing>
                                                                                <gml:posList>35.37504090912612 139.8985740681761 0 35.37501086437582 139.89859169739964 0 35.37501767875842 139.8986090041047 0 35.37504772351121 139.89859137488614 0 35.37504090912612 139.8985740681761 0</gml:posList>
                                                                        </gml:LinearRing>
                                                                </gml:exterior>
                                                        </gml:Polygon>
                                                </gml:surfaceMember>
                                        </gml:MultiSurface>
                                </bldg:lod0RoofEdge>
                                <bldg:lod1Solid>
                                        <gml:Solid>
                                                <gml:exterior>
                                                        <gml:CompositeSurface>
                                                                <gml:surfaceMember>
                                                                        <gml:Polygon>
                                                                                <gml:exterior>
                                                                                        <gml:LinearRing>
                                                                                                <gml:posList>35.37504090912612 139.8985740681761 3.721 35.37504772351121 139.89859137488614 3.721 35.37501767875842 139.8986090041047 3.721 35.37501086437582 139.89859169739964 3.721 35.37504090912612 139.8985740681761 3.721</gml:posList>
                                                                                        </gml:LinearRing>
                                                                                </gml:exterior>
                                                                        </gml:Polygon>
                                                                </gml:surfaceMember>
                                                                <gml:surfaceMember>
                                                                        <gml:Polygon>
                                                                                <gml:exterior>
                                                                                        <gml:LinearRing>
                                                                                                <gml:posList>35.37504090912612 139.8985740681761 3.721 35.37501086437582 139.89859169739964 3.721 35.37501086437582 139.89859169739964 8.186 35.37504090912612 139.8985740681761 8.186 35.37504090912612 139.8985740681761 3.721</gml:posList>
                                                                                        </gml:LinearRing>
                                                                                </gml:exterior>
                                                                        </gml:Polygon>
                                                                </gml:surfaceMember>
                                                                <gml:surfaceMember>
                                                                        <gml:Polygon>
                                                                                <gml:exterior>
                                                                                        <gml:LinearRing>
                                                                                                <gml:posList>35.37501086437582 139.89859169739964 3.721 35.37501767875842 139.8986090041047 3.721 35.37501767875842 139.8986090041047 8.186 35.37501086437582 139.89859169739964 8.186 35.37501086437582 139.89859169739964 3.721</gml:posList>
                                                                                        </gml:LinearRing>
                                                                                </gml:exterior>
                                                                        </gml:Polygon>
                                                                </gml:surfaceMember>
                                                                <gml:surfaceMember>
                                                                        <gml:Polygon>
                                                                                <gml:exterior>
                                                                                        <gml:LinearRing>
                                                                                                <gml:posList>35.37501767875842 139.8986090041047 3.721 35.37504772351121 139.89859137488614 3.721 35.37504772351121 139.89859137488614 8.186 35.37501767875842 139.8986090041047 8.186 35.37501767875842 139.8986090041047 3.721</gml:posList>
                                                                                        </gml:LinearRing>
                                                                                </gml:exterior>
                                                                        </gml:Polygon>
                                                                </gml:surfaceMember>
                                                                <gml:surfaceMember>
                                                                        <gml:Polygon>
                                                                                <gml:exterior>
                                                                                        <gml:LinearRing>
                                                                                                <gml:posList>35.37504772351121 139.89859137488614 3.721 35.37504090912612 139.8985740681761 3.721 35.37504090912612 139.8985740681761 8.186 35.37504772351121 139.89859137488614 8.186 35.37504772351121 139.89859137488614 3.721</gml:posList>
                                                                                        </gml:LinearRing>
                                                                                </gml:exterior>
                                                                        </gml:Polygon>
                                                                </gml:surfaceMember>
                                                                <gml:surfaceMember>
                                                                        <gml:Polygon>
                                                                                <gml:exterior>
                                                                                        <gml:LinearRing>
                                                                                                <gml:posList>35.37504090912612 139.8985740681761 8.186 35.37501086437582 139.89859169739964 8.186 35.37501767875842 139.8986090041047 8.186 35.37504772351121 139.89859137488614 8.186 35.37504090912612 139.8985740681761 8.186</gml:posList>
                                                                                        </gml:LinearRing>
                                                                                </gml:exterior>
                                                                        </gml:Polygon>
                                                                </gml:surfaceMember>
                                                        </gml:CompositeSurface>
                                                </gml:exterior>
                                        </gml:Solid>
                                </bldg:lod1Solid>
                                <uro:bldgDataQualityAttribute>
                                        <uro:DataQualityAttribute>
                                                <uro:geometrySrcDescLod0 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod0>
                                                <uro:geometrySrcDescLod1 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">000</uro:geometrySrcDescLod1>
                                                <uro:geometrySrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_geometrySrcDesc.xml">999</uro:geometrySrcDescLod2>
                                                <uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">000</uro:thematicSrcDesc>
                                                <uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">201</uro:thematicSrcDesc>
                                                <uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">400</uro:thematicSrcDesc>
                                                <uro:thematicSrcDesc codeSpace="../../codelists/DataQualityAttribute_thematicSrcDesc.xml">700</uro:thematicSrcDesc>
                                                <uro:appearanceSrcDescLod2 codeSpace="../../codelists/DataQualityAttribute_appearanceSrcDesc.xml">99</uro:appearanceSrcDescLod2>
                                                <uro:lod1HeightType codeSpace="../../codelists/DataQualityAttribute_lod1HeightType.xml">2</uro:lod1HeightType>
                                                <uro:publicSurveyDataQualityAttribute>
                                                        <uro:PublicSurveyDataQualityAttribute>
                                                                <uro:srcScaleLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod0>
                                                                <uro:srcScaleLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml">1</uro:srcScaleLod1>
                                                                <uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">003</uro:publicSurveySrcDescLod0>
                                                                <uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</uro:publicSurveySrcDescLod0>
                                                                <uro:publicSurveySrcDescLod0 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod0>
                                                                <uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">003</uro:publicSurveySrcDescLod1>
                                                                <uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">012</uro:publicSurveySrcDescLod1>
                                                                <uro:publicSurveySrcDescLod1 codeSpace="../../codelists/PublicSurveyDataQualityAttribute_geometrySrcDesc.xml">023</uro:publicSurveySrcDescLod1>
                                                        </uro:PublicSurveyDataQualityAttribute>
                                                </uro:publicSurveyDataQualityAttribute>
                                        </uro:DataQualityAttribute>
                                </uro:bldgDataQualityAttribute>
                                <uro:bldgDisasterRiskAttribute>
                                        <uro:TsunamiRiskAttribute>
                                                <uro:description codeSpace="../../codelists/TsunamiRiskAttribute_description.xml">1</uro:description>
                                                <uro:rank codeSpace="../../codelists/TsunamiRiskAttribute_rank.xml">1</uro:rank>
                                                <uro:depth uom="m">0.147</uro:depth>
                                        </uro:TsunamiRiskAttribute>
                                </uro:bldgDisasterRiskAttribute>
                                <uro:buildingDetailAttribute>
                                        <uro:BuildingDetailAttribute>
                                                <uro:surveyYear>2021</uro:surveyYear>
                                        </uro:BuildingDetailAttribute>
                                </uro:buildingDetailAttribute>
                                <uro:buildingIDAttribute>
                                        <uro:BuildingIDAttribute>
                                                <uro:buildingID>12206-bldg-51914</uro:buildingID>
                                                <uro:prefecture codeSpace="../../codelists/Common_localPublicAuthorities.xml">12</uro:prefecture>
                                                <uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">12206</uro:city>
                                        </uro:BuildingIDAttribute>
                                </uro:buildingIDAttribute>
                                <uro:largeCustomerFacilityAttribute>
                                        <uro:LargeCustomerFacilityAttribute>
                                                <uro:surveyYear>2021</uro:surveyYear>
                                        </uro:LargeCustomerFacilityAttribute>
                                </uro:largeCustomerFacilityAttribute>
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
}

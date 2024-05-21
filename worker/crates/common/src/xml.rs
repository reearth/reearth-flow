use std::collections::HashSet;
use std::marker::PhantomData;

use libxml::parser::{Parser, ParserOptions};
use libxml::schemas::SchemaValidationContext;
use libxml::tree::document;
use libxml::xpath::Context;

pub type XmlDocument = document::Document;
pub type XmlXpathValue = libxml::xpath::Object;
pub type XmlContext = libxml::xpath::Context;
pub type XmlNode = libxml::tree::Node;
pub type XmlNamespace = libxml::tree::Namespace;
pub type XmlNodeType = libxml::tree::NodeType;
pub type XmlSchemaParserContext = libxml::schemas::SchemaParserContext;

pub struct XmlSchemaValidationContext {
    inner: parking_lot::RwLock<SchemaValidationContext>,
    _marker: PhantomData<*mut ()>,
}

unsafe impl Send for XmlSchemaValidationContext {}
unsafe impl Sync for XmlSchemaValidationContext {}

pub fn parse<T: AsRef<[u8]>>(xml: T) -> crate::Result<XmlDocument> {
    let parser = Parser::default();
    parser
        .parse_string_with_options(
            xml,
            ParserOptions {
                recover: true,
                no_def_dtd: true,
                no_error: true,
                no_warning: false,
                pedantic: false,
                no_blanks: false,
                no_net: false,
                no_implied: false,
                compact: true,
                ignore_enc: false,
                encoding: None,
            },
        )
        .map_err(|e| crate::Error::Xml(format!("{}", e)))
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
    for ns in root.get_namespace_declarations().iter() {
        context
            .register_namespace(ns.get_prefix().as_str(), ns.get_href().as_str())
            .map_err(|_| crate::Error::Xml("Failed to register namespace".to_string()))?;
    }
    Ok(context)
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

pub fn get_node_tag(node: &XmlNode) -> String {
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

pub fn node_to_xml_string(document: &XmlDocument, node: &mut XmlNode) -> crate::Result<String> {
    let doc =
        parse(document.node_to_string(node)).map_err(|e| crate::Error::Xml(format!("{}", e)))?;
    Ok(doc.to_string())
}

pub fn parse_schema_locations(document: &XmlDocument) -> crate::Result<HashSet<String>> {
    let root = get_root_node(document)?;
    let mut schema_locations = Vec::new();
    for (key, value) in root.get_attributes().iter() {
        if key == "schemaLocation" {
            schema_locations = value.split_whitespace().map(|s| s.to_string()).collect();
        }
    }
    Ok(HashSet::from_iter(
        schema_locations
            .into_iter()
            .enumerate()
            .filter(|&(i, _)| i % 2 == 1)
            .map(|(_, v)| v)
            .collect::<Vec<_>>(),
    ))
}

pub fn create_xml_schema_validation_context(
    schema_location: String,
) -> crate::Result<XmlSchemaValidationContext> {
    let mut xsd_parser = XmlSchemaParserContext::from_file(schema_location.as_str());
    let ctx = SchemaValidationContext::from_parser(&mut xsd_parser)
        .map_err(|e| crate::Error::Xml(format!("Failed to parse schema: {:?}", e)))?;
    Ok(XmlSchemaValidationContext {
        inner: parking_lot::RwLock::new(ctx),
        _marker: PhantomData,
    })
}

pub fn validate_document_by_schema(
    document: &XmlDocument,
    schema_location: String,
) -> crate::Result<bool> {
    let mut xsd_validator = create_xml_schema_validation_context(schema_location)?;
    validate_document_by_schema_context(document, &mut xsd_validator)
}

pub fn validate_document_by_schema_context(
    document: &XmlDocument,
    xsd_validator: &mut XmlSchemaValidationContext,
) -> crate::Result<bool> {
    match xsd_validator.inner.write().validate_document(document) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
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
            .node_evaluate("//*[(name()='gml:description' or name()='gml:name')]", root)
            .unwrap();
        let result = result.get_nodes_as_vec();
        for node in result {
            let tag = get_node_tag(&node);
            println!("tag: {}", tag);
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
            HashSet::from_iter(vec![
                "../../schemas/iur/uro/3.0/urbanObject.xsd".to_string(),
                "http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd".to_string(),
                "http://schemas.opengis.net/citygml/landuse/2.0/landUse.xsd".to_string(),
                "http://schemas.opengis.net/citygml/building/2.0/building.xsd".to_string(),
                "http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd"
                    .to_string(),
                "http://schemas.opengis.net/citygml/generics/2.0/generics.xsd".to_string(),
                "http://schemas.opengis.net/citygml/cityobjectgroup/2.0/cityObjectGroup.xsd"
                    .to_string(),
                "http://schemas.opengis.net/gml/3.1.1/base/gml.xsd".to_string(),
                "http://schemas.opengis.net/citygml/appearance/2.0/appearance.xsd".to_string(),
            ])
        )
    }
}

use libxml::parser::{Parser, ParserOptions};
use libxml::tree::document;
use libxml::xpath::Context;

pub type XmlDocument = document::Document;
pub type XmlXpathValue = libxml::xpath::Object;
pub type XmlContext = libxml::xpath::Context;
pub type XmlNode = libxml::tree::Node;
pub type XmlNamespace = libxml::tree::Namespace;
pub type XmlNodeType = libxml::tree::NodeType;

pub fn parse<T: AsRef<[u8]>>(xml: T) -> crate::Result<XmlDocument> {
    let parser = Parser::default();
    parser
        .parse_string_with_options(
            xml,
            ParserOptions {
                recover: true,
                no_def_dtd: true,
                no_error: false,
                no_warning: false,
                pedantic: false,
                no_blanks: false,
                no_net: false,
                no_implied: false,
                compact: false,
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

pub fn get_node_tag(node: &XmlNode) -> String {
    match node.get_namespace() {
        Some(ns) => format!("{}:{}", ns.get_prefix(), node.get_name()).to_string(),
        None => node.get_name(),
    }
}

pub fn node_to_xml_string(document: &XmlDocument, node: &mut XmlNode) -> crate::Result<String> {
    let doc =
        parse(document.node_to_string(node)).map_err(|e| crate::Error::Xml(format!("{}", e)))?;
    Ok(doc.to_string())
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
                    <gml:name>1040</gml:name>
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
            .node_evaluate("./gml:dictionaryEntry/gml:Definition", root)
            .unwrap();
        let result = result.get_nodes_as_vec();
        let node = result.first().unwrap();
        let attribute_node = node.get_attribute_node("id").unwrap();
        assert_eq!(attribute_node.get_content(), "id1");
    }
}

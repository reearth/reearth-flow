use libxml::parser::{Parser, ParserOptions};
use libxml::tree::document;
use libxml::xpath::Context;

pub type XmlDocument = document::Document;
pub type XmlXpathValue = libxml::xpath::Object;

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
    context
        .evaluate(xpath.as_ref())
        .map_err(|_| crate::Error::Xml("Failed to evaluate xpath".to_string()))
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
        let xml = r#"<root><element>Test</element></root>"#;
        let document = parse(xml).unwrap();
        let value = evaluate(&document, "//element/text()").unwrap();
        assert_eq!(value.to_string(), "Test");
    }
}

use anyhow::anyhow;
use sxd_document::dom::Document;
use sxd_document::parser;
use sxd_document::Package;
use sxd_xpath::{context::Context, Factory, Value};

pub fn parse<T: AsRef<str>>(xml: T) -> anyhow::Result<Package> {
    parser::parse(xml.as_ref()).map_err(|e| anyhow::anyhow!(e))
}

pub fn evaluate<'d, T: AsRef<str>>(
    package: &'d Document<'d>,
    xpath: T,
) -> anyhow::Result<Value<'d>> {
    let evaluator = XPathEvaluator::new();
    evaluator.evaluate(package, xpath.as_ref())
}

struct XPathEvaluator<'d> {
    context: Context<'d>,
    factory: Factory,
}

impl<'d> XPathEvaluator<'d> {
    fn new() -> XPathEvaluator<'d> {
        let context = Context::new();
        XPathEvaluator {
            context,
            factory: Factory::new(),
        }
    }

    fn evaluate(&self, doc: &'d Document<'d>, xpath: &str) -> anyhow::Result<Value<'d>> {
        let root = doc.root();
        let xpath = self.factory.build(xpath)?;
        let xpath = xpath.ok_or(anyhow!("Failed to build xpath"))?;
        xpath
            .evaluate(&self.context, root)
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let xml = r#"<root><element>Test</element></root>"#;
        let package = parse(xml).unwrap();
        assert_eq!(package.as_document().root().children().len(), 1);
    }

    #[test]
    fn test_evaluate() {
        let xml = r#"<root><element>Test</element></root>"#;
        let package = parse(xml).unwrap();
        let document = package.as_document();
        let value = evaluate(&document, "//element/text()").unwrap();
        assert_eq!(value.string(), "Test");
    }
}

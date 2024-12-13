use roxmltree::Node;

use crate::parser::{
    node_parser::parse_node,
    types::RsFile,
    utils::target_namespace,
    xsd_elements::{ElementType, XsdNode},
};

pub fn parse_schema<'input>(schema: &Node<'_, 'input>) -> RsFile<'input> {
    let mut xsd_namespaces = schema
        .namespaces()
        .filter(|namespace| namespace.uri() == "http://www.w3.org/2001/XMLSchema");

    RsFile {
        name: "".into(),
        namespace: None,
        target_ns: target_namespace(schema).cloned(),
        xsd_ns: xsd_namespaces
            .clone()
            .find(|namespace| namespace.name().is_some())
            .or_else(|| xsd_namespaces.next())
            .cloned(),
        types: schema
            .children()
            .filter(|n| {
                n.is_element()
                    && n.xsd_type() != ElementType::Annotation
                    && n.xsd_type() != ElementType::AttributeGroup
            })
            .map(|node| parse_node(&node, schema))
            .collect(),
        attribute_groups: schema
            .children()
            .filter(|n| n.is_element() && n.xsd_type() == ElementType::AttributeGroup)
            .map(|node| parse_node(&node, schema))
            .collect(),
    }
}

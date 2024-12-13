use roxmltree::Node;

use crate::parser::{
    node_parser::parse_node,
    types::RsEntity,
    utils::get_documentation,
    xsd_elements::{ElementType, XsdNode},
};

pub fn parse_simple_type(node: &Node, parent: &Node) -> RsEntity {
    let name = node.attr_name();

    assert_eq!(
        parent.xsd_type() == ElementType::Schema,
        name.is_some(),
        "Name required if the simpleType element is a child of the schema element, and not allowed at other times"
    );

    let content = node
        .children()
        .filter(|n| n.is_element() && n.xsd_type() != ElementType::Annotation)
        .last()
        .expect(
            "Simple types must be defined in one of the following ways: [Union, List, Restriction]",
        );

    let mut content_type = parse_node(&content, node);

    if let Some(n) = name {
        content_type.set_name(n);
    }
    content_type.set_comment(get_documentation(node));
    content_type
}

use roxmltree::Node;

use crate::parser::{
    node_parser::parse_node,
    types::{Alias, RsEntity, Struct, StructField},
    utils::get_documentation,
    xsd_elements::{ElementType, XsdNode},
};

pub fn parse_attribute_group(node: &Node, parent: &Node) -> RsEntity {
    if parent.xsd_type() == ElementType::Schema {
        return parse_global_attribute_group(node);
    }

    let reference = node
        .attr_ref()
        .expect("Non-global attributeGroups must be references.")
        .to_string();

    RsEntity::Alias(Alias {
        name: reference.to_string(),
        original: reference,
        comment: get_documentation(node),
        ..Default::default()
    })
}

fn parse_global_attribute_group(node: &Node) -> RsEntity {
    let name = node
        .attr_name()
        .unwrap_or_else(|| panic!("Name attribute required. {:?}", node));

    let fields = attributes_to_fields(node);

    RsEntity::Struct(Struct {
        name: name.to_string(),
        fields: std::cell::RefCell::new(fields),
        ..Default::default()
    })
}

pub fn attributes_to_fields(node: &Node) -> Vec<StructField> {
    node.children()
        .filter(|n| {
            n.xsd_type() == ElementType::Attribute || n.xsd_type() == ElementType::AnyAttribute
        })
        .map(|n| match parse_node(&n, node) {
            RsEntity::StructField(sf) => sf,
            _ => unreachable!("Invalid attribute parsing: {:?}", n),
        })
        .collect()
}

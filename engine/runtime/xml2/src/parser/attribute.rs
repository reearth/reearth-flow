use roxmltree::Node;

use crate::parser::{
    node_parser::parse_node,
    types::{Alias, RsEntity, Struct, StructField, StructFieldSource, TypeModifier},
    utils::get_documentation,
    xsd_elements::{ElementType, UseType, XsdNode},
};

pub fn parse_attribute(node: &Node, parent: &Node) -> RsEntity {
    if parent.xsd_type() == ElementType::Schema {
        return parse_global_attribute(node);
    }

    let name = node
        .attr_name()
        .or_else(|| node.attr_ref())
        .expect("All attributes have name or ref")
        .to_string();

    let type_name = node
        .attr_type()
        .or_else(|| node.attr_ref())
        .unwrap_or("String")
        .to_string();

    let type_modifier = match node.attr_use() {
        UseType::Optional => TypeModifier::Option,
        UseType::Prohibited => TypeModifier::Empty,
        UseType::Required => TypeModifier::None,
    };

    RsEntity::StructField(StructField {
        type_name,
        comment: get_documentation(node),
        subtypes: vec![],
        name,
        source: StructFieldSource::Attribute,
        type_modifiers: vec![type_modifier],
    })
}

fn parse_global_attribute(node: &Node) -> RsEntity {
    if let Some(reference) = node.attr_ref() {
        return RsEntity::Alias(Alias {
            name: reference.to_string(),
            original: reference.to_string(),
            comment: get_documentation(node),
            ..Default::default()
        });
    }

    let name = node
        .attr_name()
        .unwrap_or_else(|| panic!("Name attribute required. {:?}", node));

    if let Some(ty) = node.attr_type() {
        return RsEntity::Alias(Alias {
            name: name.to_string(),
            original: ty.to_string(),
            comment: get_documentation(node),
            ..Default::default()
        });
    }

    if let Some(content) = node
        .children()
        .filter(|n| n.is_element() && n.xsd_type() == ElementType::SimpleType)
        .last()
    {
        let mut entity = parse_node(&content, node);
        entity.set_name(name);
        return entity;
    }

    RsEntity::Struct(Struct {
        name: name.to_string(),
        ..Default::default()
    })
}

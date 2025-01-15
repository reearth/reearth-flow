use std::cell::RefCell;

use roxmltree::Node;

use crate::parser::{
    constants::attribute,
    node_parser::parse_node,
    types::{Enum, EnumCase, EnumSource, RsEntity, Struct},
    utils::{attributes_to_fields, enum_to_field, get_documentation, get_parent_name},
    xsd_elements::{ElementType, XsdNode},
};

pub fn parse_union(union: &Node) -> RsEntity {
    let mut cases = union
        .attribute(attribute::MEMBER_TYPES)
        .map(create_enum_cases)
        .unwrap_or_default();

    let subtypes = union
        .children()
        .filter(|e| e.is_element() && e.xsd_type() == ElementType::SimpleType)
        .enumerate()
        .map(|st| enum_subtype_from_node(&st.1, union, st.0))
        .collect::<Vec<RsEntity>>();

    cases.append(
        &mut subtypes
            .iter()
            .enumerate()
            .map(|val| EnumCase {
                name: format!("EnumCase_{}", val.0),
                type_name: Some(val.1.name().to_string()),
                source: EnumSource::Union,
                ..Default::default()
            })
            .collect(),
    );

    let mut union_enum = Enum {
        cases,
        subtypes,
        comment: get_documentation(union),
        type_name: "String".into(),
        source: EnumSource::Union,
        ..Default::default()
    };

    let mut fields = attributes_to_fields(union);

    if fields.is_empty() {
        RsEntity::Enum(union_enum)
    } else {
        union_enum.name = format!("{}Choice", get_parent_name(union));
        fields.push(enum_to_field(union_enum));
        RsEntity::Struct(Struct {
            fields: RefCell::new(fields),
            ..Default::default()
        })
    }
}

fn create_enum_cases(member_types: &str) -> Vec<EnumCase> {
    member_types
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|mt| EnumCase {
            name: mt.to_string(),
            type_name: Some(mt.to_string()),
            source: EnumSource::Union,
            ..Default::default()
        })
        .collect()
}

fn enum_subtype_from_node(node: &Node, parent: &Node, index: usize) -> RsEntity {
    let mut entity = parse_node(node, parent);
    entity.set_name(format!("EnumCaseType_{}", index).as_str());
    entity
}

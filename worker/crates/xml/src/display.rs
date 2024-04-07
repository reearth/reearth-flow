use std::fmt::{Formatter, Result as FmtResult};

use crate::{
    convert::{
        as_attribute, as_character_data, as_document_decl, as_document_fragment, as_document_type,
        as_element, as_entity, as_entity_reference, as_notation, as_processing_instruction,
        RefAttribute, RefCharacterData, RefDocumentDecl, RefDocumentFragment, RefDocumentType,
        RefElement, RefEntity, RefEntityReference, RefNotation, RefProcessingInstruction,
    },
    node::RefNode,
    syntax::*,
    traits::NodeType,
    Result,
};

#[inline]
pub(crate) fn fmt_element(
    element: RefElement<'_>,
    target_tags: &[String],
    exclude_tags: &[String],
) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    let is_target = (!target_tags.is_empty()
        && target_tags.contains(&element.node_name().to_string()))
        || (exclude_tags.is_empty() && !exclude_tags.contains(&element.node_name().to_string()));
    if is_target {
        result.push(format!(
            "{}{}",
            XML_ELEMENT_START_START,
            element.node_name()
        ));
        for attr in element.attributes().values() {
            let attribute = fmt_attribute(as_attribute(attr).unwrap())?;
            result.extend(attribute);
        }
        result.push(XML_ELEMENT_START_END.to_string());
    }
    for child in element.child_nodes() {
        let child_node = recursive_fmt_node(&child, target_tags, exclude_tags)?;
        result.extend(child_node);
    }
    if is_target {
        result.push(format!(
            "{}{}{}",
            XML_ELEMENT_END_START,
            element.node_name(),
            XML_ELEMENT_END_END
        ));
    }
    Ok(result)
}

#[inline]
pub(crate) fn fmt_attribute(attribute: RefAttribute<'_>) -> Result<Vec<String>> {
    Ok(vec![format!(
        "{}=\"{}\"",
        attribute.node_name(),
        attribute.value().unwrap_or_default()
    )])
}

#[inline]
pub(crate) fn fmt_text(character_data: RefCharacterData<'_>) -> Result<Vec<String>> {
    match character_data.data() {
        None => Ok(vec![]),
        Some(data) => Ok(vec![format!("{}", data)]),
    }
}

#[inline]
pub(crate) fn fmt_cdata(character_data: RefCharacterData<'_>) -> Result<Vec<String>> {
    match character_data.data() {
        None => Ok(vec![]),
        Some(data) => Ok(vec![format!(
            "{} {} {}",
            XML_CDATA_START, data, XML_CDATA_END
        )]),
    }
}

#[inline]
pub(crate) fn fmt_processing_instruction(pi: RefProcessingInstruction<'_>) -> Result<Vec<String>> {
    match pi.data() {
        None => Ok(vec![format!(
            "{}{}{}",
            XML_PI_START,
            pi.target(),
            XML_PI_END
        )]),
        Some(data) => Ok(vec![format!(
            "{}{} {}{}",
            XML_PI_START,
            pi.target(),
            data,
            XML_PI_END
        )]),
    }
}

#[inline]
pub(crate) fn fmt_comment(character_data: RefCharacterData<'_>) -> Result<Vec<String>> {
    match character_data.data() {
        None => Ok(vec![]),
        Some(data) => Ok(vec![format!(
            "{}{}{}",
            XML_COMMENT_START, data, XML_COMMENT_END
        )]),
    }
}

#[inline]
pub(crate) fn fmt_document(
    document: RefDocumentDecl<'_>,
    target_tags: &[String],
    exclude_tags: &[String],
) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    if let Some(xml_declaration) = &document.xml_declaration() {
        result.push(format!("{}", xml_declaration));
    }

    if let Some(doc_type) = &document.doc_type() {
        let child_node = recursive_fmt_node(doc_type, target_tags, exclude_tags)?;
        result.extend(child_node);
    }
    for child in document.child_nodes() {
        let child_node = recursive_fmt_node(&child, target_tags, exclude_tags)?;
        result.extend(child_node);
    }
    Ok(result)
}

#[inline]
pub(crate) fn fmt_document_type(doc_type: RefDocumentType<'_>) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    result.push(format!("{} {}", XML_DOCTYPE_START, doc_type.node_name()));
    if let Some(id) = &doc_type.public_id() {
        result.push(format!(" {} \"{}\"", XML_DOCTYPE_PUBLIC, id));
    }
    if let Some(id) = &doc_type.system_id() {
        result.push(format!(" {} \"{}\"", XML_DOCTYPE_SYSTEM, id));
    }
    if (doc_type.entities().len() + doc_type.notations().len() > 0)
        || doc_type.internal_subset().is_some()
    {
        result.push(XML_DOCTYPE_ENTITY_START.to_string());
        for (_, entity) in doc_type.entities() {
            result.push(format!("{}", entity));
        }
        for (_, notation) in doc_type.notations() {
            result.push(format!("{}", notation));
        }
        if let Some(internal_subset) = doc_type.internal_subset() {
            result.push(internal_subset.to_string());
        }
        result.push(XML_DOCTYPE_ENTITY_END.to_string());
    }
    result.push(XML_DOCTYPE_END.to_string());
    Ok(result)
}

#[inline]
pub(crate) fn fmt_document_fragment(fragment: RefDocumentFragment<'_>) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    result.push(format!("{}{} ", XML_CDATA_START, fragment.node_name()));
    for child in fragment.child_nodes() {
        result.push(format!("{}", child));
    }
    result.push(XML_CDATA_END.to_string());
    Ok(result)
}

#[inline]
pub(crate) fn fmt_entity(entity: RefEntity<'_>) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    result.push(format!("{} {}", XML_ENTITY_START, entity.node_name()));
    if entity.public_id().is_none() && entity.system_id().is_none() {
        result.push(format!(" \"{}\"", entity.node_value().unwrap_or_default()));
    } else if let Some(public_id) = entity.public_id() {
        result.push(format!(" {} \"{}\"", XML_DOCTYPE_PUBLIC, public_id));
        if let Some(system_id) = entity.system_id() {
            result.push(format!(" \"{}\"", system_id));
        }
    } else if let Some(system_id) = entity.system_id() {
        result.push(format!(" {} \"{}\"", XML_DOCTYPE_SYSTEM, system_id));
    }
    if let Some(entity_name) = entity.notation_name() {
        result.push(format!(" {}", entity_name));
    }
    result.push(XML_ENTITY_END.to_string());
    Ok(result)
}

#[inline]
pub(crate) fn fmt_entity_reference(entity_ref: RefEntityReference<'_>) -> Result<Vec<String>> {
    Ok(vec![format!(
        "{}{}{}",
        XML_ENTITYREF_START,
        entity_ref.node_name(),
        XML_ENTITYREF_END
    )])
}

#[inline]
pub(crate) fn fmt_notation(notation: RefNotation<'_>) -> Result<Vec<String>> {
    let mut result = Vec::<String>::new();
    result.push(format!("{} {}", XML_NOTATION_START, notation.node_name()));
    if let Some(public_id) = notation.public_id() {
        result.push(format!(" {} \"{}\"", XML_DOCTYPE_PUBLIC, public_id));
        if let Some(system_id) = notation.system_id() {
            result.push(format!(" \"{}\"", system_id));
        }
    } else if let Some(system_id) = notation.system_id() {
        result.push(format!(" {} \"{}\"", XML_DOCTYPE_SYSTEM, system_id));
    }
    result.push(XML_NOTATION_END.to_string());
    Ok(result)
}

#[inline]
pub(crate) fn recursive_fmt_node(
    node: &RefNode,
    target_tags: &[String],
    exclude_tags: &[String],
) -> Result<Vec<String>> {
    match node.borrow().node_type {
        NodeType::Element => fmt_element(as_element(node).unwrap(), target_tags, exclude_tags),
        NodeType::Attribute => fmt_attribute(as_attribute(node).unwrap()),
        NodeType::Text => fmt_text(as_character_data(node).unwrap()),
        NodeType::CData => fmt_cdata(as_character_data(node).unwrap()),
        NodeType::ProcessingInstruction => {
            fmt_processing_instruction(as_processing_instruction(node).unwrap())
        }
        NodeType::Comment => fmt_comment(as_character_data(node).unwrap()),
        NodeType::Document => {
            fmt_document(as_document_decl(node).unwrap(), target_tags, exclude_tags)
        }
        NodeType::DocumentType => fmt_document_type(as_document_type(node).unwrap()),
        NodeType::DocumentFragment => fmt_document_fragment(as_document_fragment(node).unwrap()),
        NodeType::Entity => fmt_entity(as_entity(node).unwrap()),
        NodeType::EntityReference => fmt_entity_reference(as_entity_reference(node).unwrap()),
        NodeType::Notation => fmt_notation(as_notation(node).unwrap()),
    }
}

pub(crate) fn fmt_node(
    node: &RefNode,
    target_tags: &[String],
    exclude_tags: &[String],
) -> Result<String> {
    let result = recursive_fmt_node(node, target_tags, exclude_tags)?;
    Ok(result.join(""))
}

pub(crate) fn debug_element(element: RefElement<'_>, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", element.node_name())
}

pub(crate) fn debug_attribute(attribute: RefAttribute<'_>, f: &mut Formatter<'_>) -> FmtResult {
    write!(
        f,
        "{}=\"{}\"",
        attribute.node_name(),
        attribute.value().unwrap_or_default()
    )
}

pub(crate) fn debug_text(character_data: RefCharacterData<'_>, f: &mut Formatter<'_>) -> FmtResult {
    match character_data.data() {
        None => Ok(()),
        Some(data) => write!(f, "{}", data),
    }
}

pub(crate) fn debug_cdata(
    character_data: RefCharacterData<'_>,
    f: &mut Formatter<'_>,
) -> FmtResult {
    match character_data.data() {
        None => Ok(()),
        Some(data) => write!(f, "{}{}{}", XML_CDATA_START, data, XML_CDATA_END),
    }
}

pub(crate) fn debug_processing_instruction(
    pi: RefProcessingInstruction<'_>,
    f: &mut Formatter<'_>,
) -> FmtResult {
    match pi.data() {
        None => write!(f, "{}{}{}", XML_PI_START, pi.target(), XML_PI_END),
        Some(data) => write!(f, "{}{} {}{}", XML_PI_START, pi.target(), data, XML_PI_END),
    }
}

pub(crate) fn debug_comment(
    character_data: RefCharacterData<'_>,
    f: &mut Formatter<'_>,
) -> FmtResult {
    match character_data.data() {
        None => Ok(()),
        Some(data) => write!(f, "{}{}{}", XML_COMMENT_START, data, XML_COMMENT_END),
    }
}

pub(crate) fn debug_document(document: RefDocumentDecl<'_>, f: &mut Formatter<'_>) -> FmtResult {
    if let Some(xml_declaration) = &document.xml_declaration() {
        write!(f, "{}", xml_declaration)?;
    }
    if let Some(doc_type) = &document.doc_type() {
        write!(f, "{}", doc_type)?;
    }
    Ok(())
}

pub(crate) fn debug_document_type(
    doc_type: RefDocumentType<'_>,
    f: &mut Formatter<'_>,
) -> FmtResult {
    write!(f, "{} {}", XML_DOCTYPE_START, doc_type.node_name())?;
    if let Some(id) = &doc_type.public_id() {
        write!(f, " {} \"{}\"", XML_DOCTYPE_PUBLIC, id)?;
    }
    if let Some(id) = &doc_type.system_id() {
        write!(f, " {} \"{}\"", XML_DOCTYPE_SYSTEM, id)?;
    }
    if (doc_type.entities().len() + doc_type.notations().len() > 0)
        || doc_type.internal_subset().is_some()
    {
        write!(f, "{}", XML_DOCTYPE_ENTITY_START)?;
        for (_, entity) in doc_type.entities() {
            write!(f, "{}", entity)?;
        }
        for (_, notation) in doc_type.notations() {
            write!(f, "{}", notation)?;
        }
        if let Some(internal_subset) = doc_type.internal_subset() {
            write!(f, "{}", internal_subset)?;
        }
        write!(f, "{}", XML_DOCTYPE_ENTITY_END)?;
    }
    write!(f, "{}", XML_DOCTYPE_END)
}

pub(crate) fn debug_document_fragment(
    fragment: RefDocumentFragment<'_>,
    f: &mut Formatter<'_>,
) -> FmtResult {
    write!(f, "{}{} ", XML_CDATA_START, fragment.node_name())?;
    for child in fragment.child_nodes() {
        write!(f, "{}", child)?;
    }
    write!(f, "{}", XML_CDATA_END)
}

pub(crate) fn debug_entity(entity: RefEntity<'_>, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{} {}", XML_ENTITY_START, entity.node_name())?;
    if entity.public_id().is_none() && entity.system_id().is_none() {
        write!(f, " \"{}\"", entity.node_value().unwrap_or_default())?;
    } else if let Some(public_id) = entity.public_id() {
        write!(f, " {} \"{}\"", XML_DOCTYPE_PUBLIC, public_id)?;
        if let Some(system_id) = entity.system_id() {
            write!(f, " \"{}\"", system_id)?;
        }
    } else if let Some(system_id) = entity.system_id() {
        write!(f, " {} \"{}\"", XML_DOCTYPE_SYSTEM, system_id)?;
    }
    if let Some(entity_name) = entity.notation_name() {
        write!(f, " {}", entity_name)?;
    }
    write!(f, "{}", XML_ENTITY_END)
}

pub(crate) fn debug_entity_reference(
    entity_ref: RefEntityReference<'_>,
    f: &mut Formatter<'_>,
) -> FmtResult {
    write!(
        f,
        "{}{}{}",
        XML_ENTITYREF_START,
        entity_ref.node_name(),
        XML_ENTITYREF_END
    )
}

pub(crate) fn debug_notation(notation: RefNotation<'_>, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{} {}", XML_NOTATION_START, notation.node_name())?;
    if let Some(public_id) = notation.public_id() {
        write!(f, " {} \"{}\"", XML_DOCTYPE_PUBLIC, public_id)?;
        if let Some(system_id) = notation.system_id() {
            write!(f, " \"{}\"", system_id)?;
        }
    } else if let Some(system_id) = notation.system_id() {
        write!(f, " {} \"{}\"", XML_DOCTYPE_SYSTEM, system_id)?;
    }
    write!(f, "{}", XML_NOTATION_END)
}

pub(crate) fn debug_node(node: &RefNode, f: &mut Formatter<'_>) -> FmtResult {
    match node.borrow().node_type {
        NodeType::Element => debug_element(as_element(node).unwrap(), f),
        NodeType::Attribute => debug_attribute(as_attribute(node).unwrap(), f),
        NodeType::Text => debug_text(as_character_data(node).unwrap(), f),
        NodeType::CData => debug_cdata(as_character_data(node).unwrap(), f),
        NodeType::ProcessingInstruction => {
            debug_processing_instruction(as_processing_instruction(node).unwrap(), f)
        }
        NodeType::Comment => debug_comment(as_character_data(node).unwrap(), f),
        NodeType::Document => debug_document(as_document_decl(node).unwrap(), f),
        NodeType::DocumentType => debug_document_type(as_document_type(node).unwrap(), f),
        NodeType::DocumentFragment => {
            debug_document_fragment(as_document_fragment(node).unwrap(), f)
        }
        NodeType::Entity => debug_entity(as_entity(node).unwrap(), f),
        NodeType::EntityReference => debug_entity_reference(as_entity_reference(node).unwrap(), f),
        NodeType::Notation => debug_notation(as_notation(node).unwrap(), f),
    }
}

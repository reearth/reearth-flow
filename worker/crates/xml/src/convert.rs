#![allow(dead_code)]
use crate::error::Error;
use crate::namespace::MutRefNamespaced;
use crate::node::*;
use crate::traits::*;
use crate::Result;

#[macro_export]
macro_rules! make_ref_type {
    ($ref_t:ident, $trait_t:ident) => {
        /// **Ref** type for dynamic trait cast
        pub type $ref_t<'a> = &'a dyn $trait_t<NodeRef = RefNode>;
    };
    ($ref_t:ident, $mut_t:ident, $trait_t:ident) => {
        /// **Ref** type for dynamic trait cast
        pub type $ref_t<'a> = &'a dyn $trait_t<NodeRef = RefNode>;
        /// Mutable **Ref** type for mutable dynamic trait cast
        pub type $mut_t<'a> = &'a mut dyn $trait_t<NodeRef = RefNode>;
    };
}

#[macro_export]
macro_rules! make_is_as_functions {
    ($is_f:ident, $is_t:expr, $as_f:ident, $as_t:ident) => {
        ///
        /// Determines if the specified node is of the correct node type.
        ///
        #[inline]
        pub fn $is_f(ref_node: &RefNode) -> bool {
            ref_node.borrow().node_type == $is_t
        }

        ///
        /// Safely _cast_ the specified `RefNode` into a **Ref** type.
        ///
        #[inline]
        pub fn $as_f(ref_node: &RefNode) -> Result<$as_t<'_>> {
            if ref_node.borrow().node_type == $is_t {
                Ok(ref_node as $as_t<'_>)
            } else {
                Err(Error::InvalidState)
            }
        }
    };
    ($is_f:ident, $is_t:expr, $as_f:ident, $as_t:ident, $as_mut_f:ident, $as_mut_t:ident) => {
        ///
        /// Determines if the specified node is of the correct node type.
        ///
        #[inline]
        pub fn $is_f(ref_node: &RefNode) -> bool {
            ref_node.borrow().node_type == $is_t
        }

        ///
        /// Safely _cast_ the specified `RefNode` into a **Ref** type.
        ///
        #[inline]
        pub fn $as_f(ref_node: &RefNode) -> Result<$as_t<'_>> {
            if ref_node.borrow().node_type == $is_t {
                Ok(ref_node as $as_t<'_>)
            } else {
                Err(Error::InvalidState)
            }
        }

        ///
        /// Safely _cast_ the specified `RefNode` into a mutable **Ref** type.
        ///
        #[inline]
        pub fn $as_mut_f(ref_node: &mut RefNode) -> Result<$as_mut_t<'_>> {
            if ref_node.borrow().node_type == $is_t {
                Ok(ref_node as $as_mut_t<'_>)
            } else {
                Err(Error::InvalidState)
            }
        }
    };
}

make_ref_type!(RefAttribute, MutRefAttribute, Attribute);

make_ref_type!(RefCDataSection, MutRefCDataSection, CDataSection);

make_ref_type!(RefCharacterData, MutRefCharacterData, CharacterData);

make_ref_type!(RefComment, MutRefComment, Comment);

make_ref_type!(RefDocument, MutRefDocument, Document);

make_ref_type!(
    RefDocumentFragment,
    MutRefDocumentFragment,
    DocumentFragment
);

make_ref_type!(RefDocumentType, MutRefDocumentType, DocumentType);

make_ref_type!(RefElement, MutRefElement, Element);

make_ref_type!(RefEntity, MutRefEntity, Entity);

make_ref_type!(RefEntityReference, MutRefEntityReference, EntityReference);

make_ref_type!(RefNotation, MutRefNotation, Notation);

make_ref_type!(
    RefProcessingInstruction,
    MutRefProcessingInstruction,
    ProcessingInstruction
);

make_ref_type!(RefText, MutRefText, Text);

make_is_as_functions!(
    is_attribute,
    NodeType::Attribute,
    as_attribute,
    RefAttribute,
    as_attribute_mut,
    MutRefAttribute
);

make_is_as_functions!(
    is_element,
    NodeType::Element,
    as_element,
    RefElement,
    as_element_mut,
    MutRefElement
);

///
/// Determines if the specified node is a type of `CharacterData`.
///
#[inline]
pub fn is_character_data(ref_node: &RefNode) -> bool {
    matches!(
        ref_node.borrow().node_type,
        NodeType::CData | NodeType::Comment | NodeType::Text
    )
}

#[inline]
pub fn as_character_data(ref_node: &RefNode) -> Result<RefCharacterData<'_>> {
    match ref_node.borrow().node_type {
        NodeType::CData | NodeType::Comment | NodeType::Text => {
            Ok(ref_node as RefCharacterData<'_>)
        }
        _ => Err(Error::InvalidState),
    }
}

#[inline]
pub fn as_character_data_mut(ref_node: &mut RefNode) -> Result<MutRefCharacterData<'_>> {
    let node_type = { &ref_node.borrow().node_type.clone() };
    match node_type {
        NodeType::CData | NodeType::Comment | NodeType::Text => {
            Ok(ref_node as MutRefCharacterData<'_>)
        }
        _ => Err(Error::InvalidState),
    }
}

make_is_as_functions!(
    is_text,
    NodeType::Text,
    as_text,
    RefText,
    as_text_mut,
    MutRefText
);

make_is_as_functions!(
    is_cdata_section,
    NodeType::CData,
    as_cdata_section,
    RefCDataSection,
    as_cdata_section_mut,
    MutRefCDataSection
);

make_is_as_functions!(
    is_entity_reference,
    NodeType::EntityReference,
    as_entity_reference,
    RefEntityReference,
    as_entity_reference_mut,
    MutRefEntityReference
);

make_is_as_functions!(
    is_entity_,
    NodeType::Entity,
    as_entity,
    RefEntity,
    as_entity_mut,
    MutRefEntity
);

make_is_as_functions!(
    is_processing_instruction,
    NodeType::ProcessingInstruction,
    as_processing_instruction,
    RefProcessingInstruction,
    as_processing_instruction_mut,
    MutRefProcessingInstruction
);

make_is_as_functions!(
    is_comment,
    NodeType::Comment,
    as_comment,
    RefComment,
    as_comment_mut,
    MutRefComment
);

make_is_as_functions!(
    is_document,
    NodeType::Document,
    as_document,
    RefDocument,
    as_document_mut,
    MutRefDocument
);

make_is_as_functions!(
    is_document_type,
    NodeType::DocumentType,
    as_document_type,
    RefDocumentType,
    as_document_type_mut,
    MutRefDocumentType
);

make_is_as_functions!(
    is_document_fragment,
    NodeType::DocumentFragment,
    as_document_fragment,
    RefDocumentFragment,
    as_document_fragment_mut,
    MutRefDocumentFragment
);

make_is_as_functions!(
    is_notation,
    NodeType::Notation,
    as_notation,
    RefNotation,
    as_notation_mut,
    MutRefNotation
);

make_is_as_functions!(
    is_element_namespaced,
    NodeType::Element,
    as_element_namespaced,
    RefNamespaced
);

make_ref_type!(RefDocumentDecl, MutRefDocumentDecl, DocumentDecl);

make_ref_type!(RefNamespaced, Namespaced);

make_is_as_functions!(
    is_document_decl,
    NodeType::Document,
    as_document_decl,
    RefDocumentDecl,
    as_document_decl_mut,
    MutRefDocumentDecl
);

#[inline]
pub(crate) fn as_element_namespaced_mut(ref_node: &mut RefNode) -> Result<MutRefNamespaced<'_>> {
    if ref_node.borrow().node_type == NodeType::Element {
        Ok(ref_node as MutRefNamespaced<'_>)
    } else {
        Err(Error::InvalidState)
    }
}

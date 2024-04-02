use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use crate::decl::XmlDecl;
use crate::dom_impl::get_implementation;
use crate::mutex::{ArcRWLock, WeakRwLock};
use crate::name::Name;
use crate::options::ProcessingOptions;
use crate::traits::{DOMImplementation, NodeType};

pub type RefNode = ArcRWLock<NodeImpl>;
pub(crate) type WeakRefNode = WeakRwLock<NodeImpl>;

#[derive(Clone, Debug)]
pub(crate) enum Extension {
    None,
    Attribute {
        owner_element: Option<WeakRefNode>,
    },
    Document {
        implementation: &'static dyn DOMImplementation<NodeRef = RefNode>,
        document_type: Option<RefNode>,
        xml_declaration: Option<XmlDecl>,
        id_map: HashMap<String, WeakRefNode>,
        options: ProcessingOptions,
    },
    DocumentType {
        entities: HashMap<Name, RefNode>,
        notations: HashMap<Name, RefNode>,
        public_id: Option<String>,
        system_id: Option<String>,
        internal_subset: Option<String>,
    },
    Element {
        attributes: HashMap<Name, RefNode>,
        namespaces: HashMap<Option<String>, String>,
    },
    #[allow(dead_code)]
    Entity {
        public_id: Option<String>,
        system_id: Option<String>,
        notation_name: Option<String>,
    },
    #[allow(dead_code)]
    Notation {
        public_id: Option<String>,
        system_id: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct NodeImpl {
    pub(crate) node_type: NodeType,
    pub(crate) name: Name,
    pub(crate) value: Option<String>,
    pub(crate) parent_node: Option<WeakRefNode>,
    pub(crate) owner_document: Option<WeakRefNode>,
    pub(crate) child_nodes: Vec<RefNode>,
    pub(crate) extension: Extension,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Debug for &'static dyn DOMImplementation<NodeRef = RefNode> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DOMImplementation")
    }
}

// ------------------------------------------------------------------------------------------------

impl NodeImpl {
    pub(crate) fn new_element(owner_document: WeakRefNode, name: Name) -> Self {
        Self {
            node_type: NodeType::Element,
            name,
            value: None,
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::Element {
                attributes: Default::default(),
                namespaces: Default::default(),
            },
        }
    }
    pub(crate) fn new_attribute(
        owner_document: WeakRefNode,
        name: Name,
        value: Option<&str>,
    ) -> Self {
        let children = if let Some(value) = value {
            vec![RefNode::new(Self::new_text(owner_document.clone(), value))]
        } else {
            Vec::new()
        };
        Self {
            node_type: NodeType::Attribute,
            name,
            value: None,
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: children,
            extension: Extension::Attribute {
                owner_element: None,
            },
        }
    }
    pub(crate) fn new_text(owner_document: WeakRefNode, data: &str) -> Self {
        Self {
            node_type: NodeType::Text,
            name: Name::for_text(),
            value: Some(data.to_string()),
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::None,
        }
    }
    pub(crate) fn new_cdata(owner_document: WeakRefNode, data: &str) -> Self {
        Self {
            node_type: NodeType::CData,
            name: Name::for_cdata(),
            value: Some(data.to_string()),
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::None,
        }
    }
    pub(crate) fn new_processing_instruction(
        owner_document: WeakRefNode,
        target: Name,
        data: Option<&str>,
    ) -> Self {
        Self {
            node_type: NodeType::ProcessingInstruction,
            name: target,
            value: data.map(String::from),
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::None,
        }
    }
    pub(crate) fn new_comment(owner_document: WeakRefNode, data: &str) -> Self {
        Self {
            node_type: NodeType::Comment,
            name: Name::for_comment(),
            value: Some(data.to_string()),
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::None,
        }
    }
    pub(crate) fn new_document(doc_type: Option<RefNode>, options: ProcessingOptions) -> Self {
        Self {
            node_type: NodeType::Document,
            name: Name::for_document(),
            value: None,
            parent_node: None,
            owner_document: None,
            child_nodes: vec![],
            extension: Extension::Document {
                implementation: get_implementation(),
                document_type: doc_type,
                id_map: Default::default(),
                xml_declaration: None,
                options,
            },
        }
    }
    pub(crate) fn new_document_fragment(owner_document: WeakRefNode) -> Self {
        Self {
            node_type: NodeType::DocumentFragment,
            name: Name::for_document_fragment(),
            value: None,
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::None,
        }
    }
    pub(crate) fn new_document_type(
        owner_document: Option<WeakRefNode>,
        name: Name,
        public_id: Option<&str>,
        system_id: Option<&str>,
    ) -> Self {
        Self {
            node_type: NodeType::DocumentType,
            name,
            value: None,
            parent_node: owner_document.clone(),
            owner_document,
            child_nodes: vec![],
            extension: Extension::DocumentType {
                entities: Default::default(),
                notations: Default::default(),
                public_id: public_id.map(String::from),
                system_id: system_id.map(String::from),
                internal_subset: None,
            },
        }
    }

    pub(crate) fn new_entity_reference(owner_document: WeakRefNode, name: Name) -> Self {
        Self {
            node_type: NodeType::EntityReference,
            name,
            value: None,
            parent_node: None,
            owner_document: Some(owner_document),
            child_nodes: vec![],
            extension: Extension::None,
        }
    }

    pub(crate) fn clone_node(&self, deep: bool) -> Self {
        let extension = match &self.extension {
            Extension::None => Extension::None,
            Extension::Attribute { owner_element } => Extension::Attribute {
                owner_element: owner_element.clone(),
            },
            Extension::Document {
                implementation,
                document_type,
                id_map,
                xml_declaration,
                options,
            } => Extension::Document {
                #[allow(suspicious_double_ref_op)]
                implementation: implementation.clone(),
                document_type: document_type.clone(),
                id_map: id_map.clone(),
                xml_declaration: xml_declaration.clone(),
                options: options.clone(),
            },
            Extension::DocumentType {
                entities,
                notations,
                public_id,
                system_id,
                internal_subset,
            } => Extension::DocumentType {
                entities: entities.clone(),
                notations: notations.clone(),
                public_id: public_id.clone(),
                system_id: system_id.clone(),
                internal_subset: internal_subset.clone(),
            },
            Extension::Element {
                attributes,
                namespaces,
            } => Extension::Element {
                attributes: attributes.clone(),
                namespaces: namespaces.clone(),
            },
            entity @ Extension::Entity { .. } => entity.clone(),
            notation @ Extension::Notation { .. } => notation.clone(),
        };
        Self {
            node_type: self.node_type.clone(),
            name: self.name.clone(),
            value: self.value.clone(),
            parent_node: None,
            owner_document: self.owner_document.clone(),
            child_nodes: if deep {
                self.child_nodes
                    .iter()
                    .map(|node| ArcRWLock::new(node.borrow().clone_node(deep)))
                    .collect()
            } else {
                vec![]
            },
            extension,
        }
    }
}

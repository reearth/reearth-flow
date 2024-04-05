use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::convert::{
    as_attribute, as_character_data_mut, as_document, as_document_mut, as_element, is_attribute,
    is_document, is_document_fragment, is_element, is_text,
};
use crate::decl::XmlDecl;
use crate::dom_impl::{get_implementation, Implementation};
use crate::error::Error;
use crate::name::Name;
use crate::namespace::as_element_namespaced_mut;
use crate::node::{Extension, NodeImpl, RefNode, WeakRefNode};
use crate::options::ProcessingOptions;
use crate::text;
use crate::traits::{
    Attribute, CDataSection, CharacterData, Comment, DOMImplementation, Document, DocumentDecl,
    DocumentFragment, DocumentType, Element, Entity, EntityReference, Node, NodeType, Notation,
    ProcessingInstruction, Text,
};
use crate::Result;
use crate::{display, syntax::*};

const MSG_INVALID_EXTENSION: &str = "This node's extension does not match it's node type.";

macro_rules! unwrap_extension_field {
    ($node:expr, $variant:ident, $field:ident) => {{
        let ref_self = $node.borrow();
        if let Extension::$variant { $field, .. } = &ref_self.extension {
            $field.clone()
        } else {
            Default::default()
        }
    }};
    ($node:expr, $variant:ident, $field:ident, $closure_fn:expr) => {{
        let ref_self = $node.borrow();
        if let Extension::$variant { $field, .. } = &ref_self.extension {
            #[allow(clippy::redundant_closure_call)]
            $closure_fn($field)
        } else {
            Default::default()
        }
    }};
    ($node:expr, $variant:ident, $field:ident, $some_closure:expr) => {{
        let ref_self = $node.borrow();
        if let Extension::$variant { $field, .. } = &ref_self.extension {
            match $field {
                None => Default::default(),
                Some(value) => $some_closure(value),
            }
        } else {
            Default::default()
        }
    }};
    ($node:expr, $variant:ident, $field:ident, $none_closure:expr, $some_closure:expr) => {{
        let ref_self = $node.borrow();
        if let Extension::$variant { $field, .. } = &ref_self.extension {
            match $field {
                None => $none_closure(),
                Some(value) => $some_closure(value),
            }
        } else {
            Default::default()
        }
    }};
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Attribute for RefNode {
    fn value(&self) -> Option<String> {
        if self.has_child_nodes() {
            let mut result = String::new();
            for child_node in self.child_nodes() {
                if child_node.node_type() == NodeType::EntityReference {
                    if let Some(value) = child_node.node_value() {
                        result.push_str(&value);
                    }
                } else if child_node.node_type() == NodeType::Text {
                    let ref_node = child_node.borrow();
                    if let Some(data) = &ref_node.value {
                        result.push_str(data);
                    }
                }
            }
            let normalized = text::normalize_attribute_value(&result, self, false);
            Some(text::escape(&normalized))
        } else {
            None
        }
    }
    fn set_value(&mut self, value: &str) -> Result<()> {
        self.unset_value()?;
        let document_node = self.owner_document().unwrap();
        let document = as_document(&document_node).unwrap();
        let _safe_to_ignore = self.append_child(document.create_text_node(value))?;
        Ok(())
    }
    fn unset_value(&mut self) -> Result<()> {
        let mut mut_self = self.borrow_mut();
        mut_self.child_nodes.clear();
        Ok(())
    }
    fn owner_element(&self) -> Option<Self::NodeRef> {
        unwrap_extension_field!(self, Attribute, owner_element, |owner_element: &Option<
            WeakRefNode,
        >| {
            match owner_element {
                None => None,
                Some(weak_ref) => weak_ref.clone().upgrade(),
            }
        })
    }
}

impl CDataSection for RefNode {}

impl CharacterData for RefNode {
    fn substring_data(&self, offset: usize, count: usize) -> Result<String> {
        if offset + count == offset {
            return Ok(String::new());
        }
        let ref_self = self.borrow();
        match &ref_self.value {
            None => Err(Error::IndexSize),
            Some(data) => {
                if offset >= data.len() {
                    Err(Error::IndexSize)
                } else if offset + count >= data.len() {
                    Ok(data[offset..].to_string())
                } else {
                    Ok(data[offset..offset + count].to_string())
                }
            }
        }
    }

    fn append_data(&mut self, new_data: &str) -> Result<()> {
        if new_data.is_empty() {
            return Ok(());
        }
        let mut mut_self = self.borrow_mut();
        match &mut_self.value {
            None => mut_self.value = Some(new_data.to_string()),
            Some(old_data) => mut_self.value = Some(format!("{}{}", old_data, new_data)),
        }
        Ok(())
    }

    fn insert_data(&mut self, offset: usize, new_data: &str) -> Result<()> {
        if new_data.is_empty() {
            return Ok(());
        }
        self.replace_data(offset, 0, new_data)
    }

    fn delete_data(&mut self, offset: usize, count: usize) -> Result<()> {
        if offset + count == offset {
            return Ok(());
        }
        const NOTHING: &str = "";
        self.replace_data(offset, count, NOTHING)
    }

    fn replace_data(&mut self, offset: usize, count: usize, replace_data: &str) -> Result<()> {
        let mut mut_self = self.borrow_mut();
        match &mut_self.value {
            None => {
                if offset + count != 0 {
                    Err(Error::IndexSize)
                } else {
                    mut_self.value = Some(replace_data.to_string());
                    Ok(())
                }
            }
            Some(old_data) => {
                if offset >= old_data.len() {
                    Err(Error::IndexSize)
                } else {
                    let mut new_data = old_data.clone();
                    if offset + count >= old_data.len() {
                        new_data.replace_range(offset.., replace_data);
                    } else {
                        new_data.replace_range(offset..offset + count, replace_data);
                    }
                    mut_self.value = Some(new_data);
                    Ok(())
                }
            }
        }
    }
}

impl Comment for RefNode {}

impl Document for RefNode {
    fn doc_type(&self) -> Option<RefNode> {
        unwrap_extension_field!(self, Document, document_type)
    }

    fn document_element(&self) -> Option<RefNode> {
        self.child_nodes().first().cloned()
    }

    fn implementation(&self) -> &dyn DOMImplementation<NodeRef = RefNode> {
        let ref_self = self.borrow();
        if let Extension::Document { implementation, .. } = &ref_self.extension {
            *implementation
        } else {
            panic!("{}", MSG_INVALID_EXTENSION);
        }
    }

    fn create_attribute(&self, name: &str) -> Result<RefNode> {
        let name = Name::from_str(name)?;
        let node_impl = NodeImpl::new_attribute(self.clone().downgrade(), name, None);
        Ok(RefNode::new(node_impl))
    }

    fn create_attribute_with(&self, name: &str, value: &str) -> Result<RefNode> {
        let name = Name::from_str(name)?;
        let node_impl = NodeImpl::new_attribute(self.clone().downgrade(), name, Some(value));
        Ok(RefNode::new(node_impl))
    }

    fn create_attribute_ns(&self, namespace_uri: &str, qualified_name: &str) -> Result<RefNode> {
        let name = Name::new_ns(namespace_uri, qualified_name)?;
        let node_impl = NodeImpl::new_attribute(self.clone().downgrade(), name, None);
        Ok(RefNode::new(node_impl))
    }

    fn create_cdata_section(&self, data: &str) -> Result<RefNode> {
        let node_impl = NodeImpl::new_cdata(self.clone().downgrade(), data);
        Ok(RefNode::new(node_impl))
    }

    fn create_document_fragment(&self) -> Result<RefNode> {
        let node_impl = NodeImpl::new_document_fragment(self.clone().downgrade());
        Ok(RefNode::new(node_impl))
    }

    fn create_entity_reference(&self, name: &str) -> Result<RefNode> {
        let name = Name::from_str(name)?;
        let node_impl = NodeImpl::new_entity_reference(self.clone().downgrade(), name);
        Ok(RefNode::new(node_impl))
    }

    fn create_comment(&self, data: &str) -> RefNode {
        let node_impl = NodeImpl::new_comment(self.clone().downgrade(), data);
        RefNode::new(node_impl)
    }

    fn create_element(&self, tag_name: &str) -> Result<RefNode> {
        let name = Name::from_str(tag_name)?;
        let node_impl = NodeImpl::new_element(self.clone().downgrade(), name);
        Ok(RefNode::new(node_impl))
    }

    fn create_element_ns(&self, namespace_uri: &str, qualified_name: &str) -> Result<RefNode> {
        let name = Name::new_ns(namespace_uri, qualified_name)?;
        let node_impl = NodeImpl::new_element(self.clone().downgrade(), name);
        Ok(RefNode::new(node_impl))
    }

    fn create_processing_instruction(&self, target: &str, data: Option<&str>) -> Result<RefNode> {
        if target.to_ascii_lowercase() == XML_PI_RESERVED {
            return Err(Error::Syntax(
                "Invalid processing instruction target".to_string(),
            ));
        }
        let target = Name::from_str(target)?;
        let node_impl =
            NodeImpl::new_processing_instruction(self.clone().downgrade(), target, data);
        Ok(RefNode::new(node_impl))
    }

    fn create_text_node(&self, data: &str) -> RefNode {
        let node_impl = NodeImpl::new_text(self.clone().downgrade(), data);
        RefNode::new(node_impl)
    }

    fn get_element_by_id(&self, id: &str) -> Option<RefNode> {
        let ref_self = self.borrow();
        if let Extension::Document { id_map, .. } = &ref_self.extension {
            match id_map.get(&id.to_string()) {
                None => None,
                Some(weak_ref) => weak_ref.clone().upgrade(),
            }
        } else {
            None
        }
    }

    fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<RefNode> {
        if let Some(root_element) = self.document_element() {
            Element::get_elements_by_tag_name(&root_element, tag_name)
        } else {
            Vec::default()
        }
    }

    fn get_elements_by_tag_name_ns(&self, namespace_uri: &str, local_name: &str) -> Vec<RefNode> {
        if let Some(root_element) = self.document_element() {
            Element::get_elements_by_tag_name_ns(&root_element, namespace_uri, local_name)
        } else {
            Vec::default()
        }
    }
}

impl DocumentFragment for RefNode {}

impl DocumentType for RefNode {
    fn entities(&self) -> HashMap<Name, Self::NodeRef, RandomState> {
        unwrap_extension_field!(self, DocumentType, entities)
    }

    fn notations(&self) -> HashMap<Name, Self::NodeRef, RandomState> {
        unwrap_extension_field!(self, DocumentType, notations)
    }

    fn public_id(&self) -> Option<String> {
        unwrap_extension_field!(self, DocumentType, public_id)
    }

    fn system_id(&self) -> Option<String> {
        unwrap_extension_field!(self, DocumentType, system_id)
    }

    fn internal_subset(&self) -> Option<String> {
        unwrap_extension_field!(self, DocumentType, internal_subset)
    }
}

// ------------------------------------------------------------------------------------------------

impl DOMImplementation for Implementation {
    type NodeRef = RefNode;

    fn create_document(
        &self,
        namespace_uri: Option<&str>,
        qualified_name: Option<&str>,
        doc_type: Option<RefNode>,
    ) -> Result<RefNode> {
        let mut options = ProcessingOptions::new();
        options.set_add_namespaces();
        create_document_with_options(namespace_uri, qualified_name, doc_type, options)
    }

    fn create_document_type(
        &self,
        qualified_name: &str,
        public_id: Option<&str>,
        system_id: Option<&str>,
    ) -> Result<RefNode> {
        let name = Name::from_str(qualified_name)?;
        let node_impl = NodeImpl::new_document_type(None, name, public_id, system_id);
        Ok(RefNode::new(node_impl))
    }

    fn has_feature(&self, feature: &str, version: &str) -> bool {
        (feature == XML_FEATURE_CORE || feature == XML_FEATURE_XML)
            && (version == XML_FEATURE_V1 || version == XML_FEATURE_V2)
    }

    fn create_document_with_options(
        &self,
        namespace_uri: Option<&str>,
        qualified_name: Option<&str>,
        doc_type: Option<Self::NodeRef>,
        options: ProcessingOptions,
    ) -> Result<Self::NodeRef> {
        create_document_with_options(namespace_uri, qualified_name, doc_type, options)
    }
}

impl Element for RefNode {
    fn to_xml(&self) -> Result<String> {
        if self.borrow().node_type == NodeType::Element {
            let element = as_element(self).unwrap();
            if let Some(_prefix) = element.prefix() {
                let namespaces = element.get_attributes_ns();
                let mut new_node = self.clone_node(true).unwrap();
                new_node.remove_owner_document();
                let mut document = get_implementation()
                    .create_document(None, None, None)
                    .unwrap();
                let result = document.set_xml_declaration(XmlDecl::default());
                if result.is_err() {
                    return Err(Error::Malformed("Invalid XML declaration".to_string()));
                }
                document.append_child(new_node.clone()).unwrap();
                for (_, v) in namespaces {
                    let mut namespace = v.clone_node(true).unwrap();
                    namespace.remove_owner_document();
                    namespace.borrow_mut().owner_document = Some(document.clone().downgrade());
                    let result = new_node.set_attribute_node_ns(namespace);
                    if result.is_err() {
                        return Err(Error::Malformed("Invalid namespace attribute".to_string()));
                    }
                }
                Ok(format!("{}", document))
            } else {
                Ok(format!("{}", self))
            }
        } else {
            Err(Error::Malformed("Invalid element".to_string()))
        }
    }

    fn get_attributes(&self) -> HashMap<Name, RefNode> {
        let result = HashMap::new();
        if is_element(self) {
            let ref_self = self.borrow();
            if let Extension::Element { attributes, .. } = &ref_self.extension {
                attributes.clone()
            } else {
                result
            }
        } else {
            result
        }
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        match self.get_attribute_node(name) {
            None => None,
            Some(attribute_node) => match as_attribute(&attribute_node) {
                Ok(attribute) => attribute.value(),
                Err(_) => None,
            },
        }
    }

    fn set_attribute(&mut self, name: &str, value: &str) -> Result<()> {
        let attr_name = Name::from_str(name)?;
        let attr_node = {
            let ref_self = &self.borrow_mut();
            let document = ref_self.owner_document.as_ref().unwrap();
            NodeImpl::new_attribute(document.clone(), attr_name, Some(value))
        };
        self.set_attribute_node(RefNode::new(attr_node)).map(|_| ())
    }

    fn remove_attribute(&mut self, name: &str) -> Result<()> {
        match self.get_attribute_node(name) {
            None => Ok(()),
            Some(attribute_node) => self.remove_attribute_node(attribute_node).map(|_| ()),
        }
    }

    fn get_attribute_node(&self, name: &str) -> Option<RefNode> {
        if is_element(self) {
            match Name::from_str(name) {
                Ok(name) => {
                    let ref_self = self.borrow();
                    if let Extension::Element { attributes, .. } = &ref_self.extension {
                        let node_name = name.to_string();
                        attributes
                            .iter()
                            .find(|(name, _)| name.to_string() == node_name)
                            .map(|(_, node)| node.clone())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }

    fn set_attribute_node(&mut self, new_attribute: RefNode) -> Result<RefNode> {
        if is_element(self) && is_attribute(&new_attribute) {
            check_same_document(self, &new_attribute)?;
            {
                let mut mut_child = new_attribute.borrow_mut();
                if let Extension::Attribute { owner_element, .. } = &mut mut_child.extension {
                    *owner_element = Some(self.clone().downgrade())
                } else {
                    panic!("{}", MSG_INVALID_EXTENSION);
                }
            }

            let name: Name = new_attribute.node_name();
            if name.is_namespace_attribute() {
                let attribute = as_attribute(&new_attribute).unwrap();
                let namespace_uri = attribute.value().unwrap();

                let as_namespaced = as_element_namespaced_mut(self).unwrap();
                let _ignore = match &name.prefix() {
                    None => as_namespaced.insert_mapping(None, &namespace_uri),
                    Some(_) => {
                        as_namespaced.insert_mapping(Some(name.local_name()), &namespace_uri)
                    }
                }?;
            }

            let mut mut_self = self.borrow_mut();
            if let Extension::Element { attributes, .. } = &mut mut_self.extension {
                let _safe_to_ignore =
                    attributes.insert(new_attribute.node_name(), new_attribute.clone());
                {
                    let attribute = as_attribute(&new_attribute).unwrap();
                    let document = attribute.owner_document().unwrap();
                    let mut mut_document = document.borrow_mut();
                    let lax = if let Extension::Document { options, .. } = &mut_document.extension {
                        options.has_assume_ids()
                    } else {
                        false
                    };
                    if name.is_id_attribute(lax) {
                        //
                        // Update the document ID mapping
                        //
                        if let Extension::Document { id_map, .. } = &mut mut_document.extension {
                            let id_value = attribute.value().unwrap();
                            if id_map.contains_key(&id_value) {
                                return Err(Error::Syntax("Duplicate ID value".to_string()));
                            }
                            let _safe_to_ignore = id_map.insert(id_value, self.clone().downgrade());
                        }
                    }
                }
                Ok(new_attribute)
            } else {
                Err(Error::Syntax("Invalid extension".to_string()))
            }
        } else {
            Err(Error::InvalidState)
        }
    }

    fn remove_attribute_node(&mut self, old_attribute: RefNode) -> Result<RefNode> {
        if is_element(self) {
            let mut mut_self = self.borrow_mut();
            if let Extension::Element { attributes, .. } = &mut mut_self.extension {
                let _safe_to_ignore = attributes.remove(&old_attribute.node_name());
                let mut_old = old_attribute.clone();
                let mut mut_old = mut_old.borrow_mut();
                mut_old.parent_node = None;
                Ok(old_attribute)
            } else {
                Err(Error::Syntax("Invalid extension".to_string()))
            }
        } else {
            Err(Error::InvalidState)
        }
    }

    fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<RefNode> {
        let mut results = Vec::default();
        if is_element(self) {
            let tag_name = tag_name.to_string();
            let ref_self = self.borrow();
            if tag_name_match(&ref_self.name.to_string(), &tag_name) {
                results.push(self.clone());
            }
            for child_node in &ref_self.child_nodes {
                if let Ok(ref_child) = as_element(child_node) {
                    results.extend(ref_child.get_elements_by_tag_name(&tag_name));
                }
            }
        }
        results
    }

    fn get_attribute_ns(&self, namespace_uri: &str, local_name: &str) -> Option<String> {
        match self.get_attribute_node_ns(namespace_uri, local_name) {
            None => None,
            Some(attribute_node) => match as_attribute(&attribute_node) {
                Ok(attribute) => attribute.value(),
                Err(_) => None,
            },
        }
    }

    fn set_attribute_ns(
        &mut self,
        namespace_uri: &str,
        qualified_name: &str,
        value: &str,
    ) -> Result<()> {
        let attr_name = Name::new_ns(namespace_uri, qualified_name)?;
        let attr_node = {
            let ref_self = &self.borrow_mut();
            let document = ref_self.owner_document.as_ref().unwrap();
            NodeImpl::new_attribute(document.clone(), attr_name, Some(value))
        };
        self.set_attribute_node(RefNode::new(attr_node)).map(|_| ())
    }

    fn remove_attribute_ns(&mut self, namespace_uri: &str, local_name: &str) -> Result<()> {
        match self.get_attribute_node_ns(namespace_uri, local_name) {
            None => Ok(()),
            Some(attribute_node) => self.remove_attribute_node(attribute_node).map(|_| ()),
        }
    }

    fn get_attributes_ns(&self) -> HashMap<Name, RefNode> {
        let mut result = HashMap::new();
        if is_element(self) {
            let ref_self = self.borrow();
            if let Extension::Element { attributes, .. } = &ref_self.extension {
                let ns = attributes
                    .iter()
                    .filter(|(name, _)| name.is_namespace_attribute())
                    .map(|(name, node)| (name.clone(), node.clone()))
                    .collect::<HashMap<_, _>>();
                result.extend(ns);
                let ref_self = self.borrow();
                match &ref_self.parent_node {
                    None => result,
                    Some(parent) => {
                        let parent = parent.clone();
                        match parent.upgrade() {
                            None => result,
                            Some(parent_node) => {
                                let parents = parent_node.get_attributes_ns();
                                result.extend(parents);
                                result
                            }
                        }
                    }
                }
            } else {
                HashMap::new()
            }
        } else {
            result
        }
    }

    fn get_attribute_node_ns(&self, namespace_uri: &str, local_name: &str) -> Option<RefNode> {
        if is_element(self) {
            match Name::new_ns(namespace_uri, local_name) {
                Ok(_) => {
                    let ref_self = self.borrow();
                    if let Extension::Element { attributes, .. } = &ref_self.extension {
                        let namespace_uri = &Some(namespace_uri.to_string());
                        let local_name = &local_name.to_string();
                        attributes
                            .iter()
                            .find(|(name, _)| {
                                name.namespace_uri() == namespace_uri
                                    && name.local_name() == local_name
                            })
                            .map(|(_, node)| node.clone())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }

    fn set_attribute_node_ns(&mut self, new_attribute: RefNode) -> Result<RefNode> {
        self.set_attribute_node(new_attribute)
    }

    fn get_elements_by_tag_name_ns(&self, namespace_uri: &str, local_name: &str) -> Vec<RefNode> {
        let mut results = Vec::default();
        if is_element(self) {
            let namespace_uri = namespace_uri.to_string();
            let local_name = local_name.to_string();
            let ref_self = self.borrow();
            if namespaced_name_match(
                match ref_self.name.namespace_uri() {
                    None => None,
                    Some(s) => Some(s.as_str()),
                },
                ref_self.name.local_name(),
                &namespace_uri,
                &local_name,
            ) {
                results.push(self.clone());
            }
            for child_node in &ref_self.child_nodes {
                if let Ok(ref_child) = as_element(child_node) {
                    results
                        .extend(ref_child.get_elements_by_tag_name_ns(&namespace_uri, &local_name));
                }
            }
        }
        results
    }

    fn has_attribute(&self, name: &str) -> bool {
        if is_element(self) {
            match Name::from_str(name) {
                Ok(name) => {
                    let ref_self = self.borrow();
                    if let Extension::Element { attributes, .. } = &ref_self.extension {
                        attributes.keys().any(|n| n.to_string() == name.to_string())
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn has_attribute_ns(&self, namespace_uri: &str, local_name: &str) -> bool {
        if is_element(self) {
            match Name::new_ns(namespace_uri, local_name) {
                Ok(name) => {
                    let ref_self = self.borrow();
                    if let Extension::Element { attributes, .. } = &ref_self.extension {
                        attributes.keys().any(|n| {
                            n.namespace_uri() == name.namespace_uri()
                                && n.local_name() == name.local_name()
                        })
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl Entity for RefNode {
    fn public_id(&self) -> Option<String> {
        unwrap_extension_field!(self, Entity, public_id)
    }

    fn system_id(&self) -> Option<String> {
        unwrap_extension_field!(self, Entity, system_id)
    }

    fn notation_name(&self) -> Option<String> {
        unwrap_extension_field!(self, Entity, notation_name)
    }
}

// ------------------------------------------------------------------------------------------------

impl EntityReference for RefNode {}

// ------------------------------------------------------------------------------------------------

impl Node for RefNode {
    type NodeRef = RefNode;

    fn node_name(&self) -> Name {
        let ref_self = self.borrow();
        ref_self.name.clone()
    }

    fn node_value(&self) -> Option<String> {
        let ref_self = self.borrow();
        ref_self.value.clone()
    }

    fn set_node_value(&mut self, value: &str) -> Result<()> {
        let mut mut_self = self.borrow_mut();
        mut_self.value = Some(value.to_string());
        Ok(())
    }

    fn unset_node_value(&mut self) -> Result<()> {
        let mut mut_self = self.borrow_mut();
        mut_self.value = None;
        Ok(())
    }

    fn node_type(&self) -> NodeType {
        let ref_self = self.borrow();
        ref_self.node_type.clone()
    }

    fn parent_node(&self) -> Option<RefNode> {
        if is_attribute(self) {
            return None;
        }
        let ref_self = self.borrow();
        match &ref_self.parent_node {
            None => None,
            Some(node) => node.clone().upgrade(),
        }
    }

    fn child_nodes(&self) -> Vec<RefNode> {
        let ref_self = self.borrow();
        ref_self.child_nodes.clone()
    }

    fn first_child(&self) -> Option<RefNode> {
        let ref_self = self.borrow();
        ref_self.child_nodes.first().cloned()
    }

    fn last_child(&self) -> Option<RefNode> {
        let ref_self = self.borrow();
        ref_self.child_nodes.last().cloned()
    }

    fn previous_sibling(&self) -> Option<RefNode> {
        if is_attribute(self) {
            return None;
        }
        let ref_self = self.borrow();
        match &ref_self.parent_node {
            None => None,
            Some(parent_node) => {
                let parent_node = parent_node.clone();
                let parent_node = parent_node.upgrade()?;
                let ref_parent = parent_node.borrow();
                match ref_parent
                    .child_nodes
                    .iter()
                    .position(|child| child == self)
                {
                    None => None,
                    Some(index) => {
                        if index == 0 {
                            None
                        } else {
                            let sibling = ref_parent.child_nodes.get(index - 1);
                            sibling.cloned()
                        }
                    }
                }
            }
        }
    }

    fn next_sibling(&self) -> Option<RefNode> {
        if is_attribute(self) {
            return None;
        }
        let ref_self = self.borrow();
        match &ref_self.parent_node {
            None => None,
            Some(parent_node) => {
                let parent_node = parent_node.clone();
                let parent_node = parent_node.upgrade()?;
                let ref_parent = parent_node.borrow();
                match ref_parent
                    .child_nodes
                    .iter()
                    .position(|child| child == self)
                {
                    None => None,
                    Some(index) => {
                        let sibling = ref_parent.child_nodes.get(index + 1);
                        sibling.cloned()
                    }
                }
            }
        }
    }

    fn attributes(&self) -> HashMap<Name, RefNode, RandomState> {
        if is_element(self) {
            unwrap_extension_field!(self, Element, attributes)
        } else {
            HashMap::default()
        }
    }

    fn owner_document(&self) -> Option<RefNode> {
        let ref_self = self.borrow();
        match &ref_self.owner_document {
            None => None,
            Some(node) => node.clone().upgrade(),
        }
    }

    fn insert_before(&mut self, new_child: RefNode, ref_child: Option<RefNode>) -> Result<RefNode> {
        fn insert_or_append(
            parent_node: &mut RefNode,
            new_child: &RefNode,
            insert_position: Option<usize>,
        ) {
            let mut mut_parent = parent_node.borrow_mut();
            let new_child = new_child.clone();
            match insert_position {
                None => mut_parent.child_nodes.push(new_child),
                Some(position) => mut_parent.child_nodes.insert(position, new_child),
            }
        }

        if !is_child_allowed(self, &new_child) {
            return Err(Error::HierarchyRequest);
        }

        //
        // Special case for Document only.
        //
        if is_document(self)
            && is_element(&new_child)
            && self
                .child_nodes()
                .iter()
                .any(|n| n.node_type() == NodeType::Element)
        {
            return Err(Error::HierarchyRequest);
        }

        //
        // Find the index in `child_nodes` of the `ref_child`.
        //
        let insert_position = match ref_child {
            None => None,
            Some(ref_child) => match self
                .borrow()
                .child_nodes
                .iter()
                .position(|child| child == &ref_child)
            {
                None => {
                    return Err(Error::NotFound);
                }
                position => position,
            },
        };

        check_same_document(self, &new_child)?;

        match new_child.parent_node() {
            None => (),
            Some(mut parent_node) => {
                let _safe_to_ignore = parent_node.remove_child(new_child.clone())?;
            }
        }

        //
        // update new child with references from self
        //
        {
            let ref_self = self.borrow();
            let mut mut_child = new_child.borrow_mut();
            mut_child.parent_node = Some(self.to_owned().downgrade());
            if is_document(self) {
                mut_child.owner_document = Some(self.clone().downgrade());
            } else {
                mut_child.owner_document = ref_self.owner_document.clone();
            }
        }

        //
        // Special case
        //
        if is_document_fragment(&new_child) {
            for (index, child) in new_child.child_nodes().iter().enumerate() {
                match insert_position {
                    None => insert_or_append(self, child, None),
                    Some(position) => insert_or_append(self, child, Some(position + index)),
                }
            }
        } else {
            insert_or_append(self, &new_child, insert_position)
        }

        Ok(new_child)
    }

    fn replace_child(&mut self, new_child: RefNode, old_child: RefNode) -> Result<RefNode> {
        if !is_child_allowed(self, &new_child) {
            return Err(Error::HierarchyRequest);
        }
        let exists = {
            let ref_self = self.borrow();
            ref_self.child_nodes.contains(&old_child.clone())
        };
        if exists {
            let next_node = old_child.next_sibling();
            let removed = self.remove_child(old_child)?;
            let _safe_to_ignore = self.insert_before(new_child, next_node)?;
            Ok(removed)
        } else {
            Err(Error::NotFound)
        }
    }

    fn remove_child(&mut self, old_child: Self::NodeRef) -> Result<Self::NodeRef> {
        let position = {
            let ref_self = self.borrow();
            ref_self
                .child_nodes
                .iter()
                .position(|child| child == &old_child)
        };
        match position {
            None => Err(Error::NotFound),
            Some(position) => {
                let removed = {
                    let mut mut_self = self.borrow_mut();
                    mut_self.child_nodes.remove(position)
                };
                let mut mut_removed = removed.borrow_mut();
                mut_removed.parent_node = None;
                Ok(removed.clone())
            }
        }
    }

    fn append_child(&mut self, new_child: RefNode) -> Result<RefNode> {
        self.insert_before(new_child, None)
    }

    fn has_child_nodes(&self) -> bool {
        !self.child_nodes().is_empty()
    }

    fn clone_node(&self, deep: bool) -> Option<RefNode> {
        let ref_self = self.borrow();
        let new_node = ref_self.clone_node(deep);
        Some(RefNode::new(new_node))
    }

    fn normalize(&mut self) {
        for child_node in self.child_nodes() {
            if is_text(&child_node) {
                if CharacterData::length(&child_node) == 0 {
                    if self.remove_child(child_node).is_err() {
                        panic!("Could not remove unnecessary text node");
                    }
                } else if let Some(last_child_node) = child_node.previous_sibling() {
                    let last_child_node = &mut last_child_node.clone();
                    if is_text(last_child_node) {
                        if last_child_node
                            .append_data(&child_node.node_value().unwrap())
                            .is_err()
                        {
                            panic!("Could not merge text nodes");
                        }
                        if self.remove_child(child_node).is_err() {
                            panic!("Could not remove unnecessary text node");
                        }
                    }
                }
            }
        }
    }

    fn is_supported(&self, feature: &str, version: &str) -> bool {
        get_implementation().has_feature(feature, version)
    }

    fn has_attributes(&self) -> bool {
        !self.attributes().is_empty()
    }

    fn remove_owner_document(&mut self) {
        self.borrow_mut().owner_document = None;
    }
}

// ------------------------------------------------------------------------------------------------

impl Notation for RefNode {
    fn public_id(&self) -> Option<String> {
        unwrap_extension_field!(self, Notation, public_id)
    }

    fn system_id(&self) -> Option<String> {
        unwrap_extension_field!(self, Notation, system_id)
    }
}

impl ProcessingInstruction for RefNode {}

impl Text for RefNode {
    fn split(&mut self, offset: usize) -> Result<RefNode> {
        let new_data = {
            let text = as_character_data_mut(self)?;
            let length = text.length();
            if offset >= length {
                String::new()
            } else {
                let count = length - offset;
                let new_data = text.substring_data(offset, count)?;
                text.delete_data(offset, count)?;
                new_data
            }
        };

        let new_node = {
            let mut_self = self.borrow_mut();
            match mut_self.node_type {
                NodeType::Text => {
                    let document = mut_self.owner_document.as_ref().unwrap();
                    Ok(NodeImpl::new_text(document.clone(), &new_data))
                }
                NodeType::CData => {
                    let document = mut_self.owner_document.as_ref().unwrap();
                    Ok(NodeImpl::new_cdata(document.clone(), &new_data))
                }
                _ => Err(Error::Syntax),
            }
        };
        match new_node {
            Ok(new_node) => {
                let new_node = RefNode::new(new_node);
                if let Some(mut parent) = self.parent_node() {
                    let _safe_to_ignore =
                        parent.insert_before(new_node.clone(), Some(self.clone()))?;
                }
                Ok(new_node)
            }
            Err(_) => Err(Error::Syntax("Invalid node type".to_string())),
        }
    }
}

impl Display for RefNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        display::fmt_node(self, f)
    }
}

impl Debug for RefNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        display::debug_node(self, f)
    }
}

const WILD_CARD: &str = "*";

fn tag_name_match(test: &str, against: &str) -> bool {
    (test == against) || test == WILD_CARD || against == WILD_CARD
}

fn namespaced_name_match(
    test_ns: Option<&str>,
    test_local: &str,
    against_ns: &str,
    against_local: &str,
) -> bool {
    match test_ns {
        None => {
            against_ns == WILD_CARD
                && ((test_local == against_local)
                    || test_local == WILD_CARD
                    || against_local == WILD_CARD)
        }
        Some(test_ns) => {
            ((test_ns == against_ns) || test_ns == WILD_CARD || against_ns == WILD_CARD)
                && ((test_local == against_local)
                    || test_local == WILD_CARD
                    || against_local == WILD_CARD)
        }
    }
}

//
// CHECK: Raise `Error::WrongDocument` if `newChild` was created from a different
// document than the one that created this node.
//
fn check_same_document(self_node: &RefNode, new_child: &RefNode) -> Result<()> {
    {
        if self_node.node_type() == NodeType::Document {
            let child_document = &new_child.borrow().owner_document;
            if !match child_document {
                None => true,
                Some(child_document) => {
                    let child_document = child_document.clone().upgrade().unwrap();
                    self_node == &child_document
                }
            } {
                return Err(Error::WrongDocument);
            }
        } else {
            let self_document = &self_node.borrow().owner_document;
            let child_document = &new_child.borrow().owner_document;
            if !match (self_document, child_document) {
                (None, None) => true,
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (Some(self_document), Some(child_document)) => {
                    let self_document = self_document.clone().upgrade().unwrap();
                    let child_document = child_document.clone().upgrade().unwrap();
                    self_document == child_document
                }
            } {
                return Err(Error::WrongDocument);
            }
        }
    }
    Ok(())
}

fn is_child_allowed(parent: &RefNode, child: &RefNode) -> bool {
    let self_node_type = { &parent.borrow().node_type };
    let child_node_type = { &child.borrow().node_type };
    match self_node_type {
        NodeType::Element => matches!(
            child_node_type,
            NodeType::Element
                | NodeType::Text
                | NodeType::Comment
                | NodeType::ProcessingInstruction
                | NodeType::CData
                | NodeType::EntityReference
        ),
        NodeType::Attribute => {
            matches!(child_node_type, NodeType::Text | NodeType::EntityReference)
        }
        NodeType::Text => false,
        NodeType::CData => false,
        NodeType::EntityReference => matches!(
            child_node_type,
            NodeType::Element
                | NodeType::Text
                | NodeType::Comment
                | NodeType::ProcessingInstruction
                | NodeType::CData
                | NodeType::EntityReference
        ),
        NodeType::Entity => matches!(
            child_node_type,
            NodeType::Element
                | NodeType::Text
                | NodeType::Comment
                | NodeType::ProcessingInstruction
                | NodeType::CData
                | NodeType::EntityReference
        ),
        NodeType::ProcessingInstruction => false,
        NodeType::Comment => false,
        NodeType::Document => matches!(
            child_node_type,
            NodeType::Element | NodeType::Comment | NodeType::ProcessingInstruction
        ),
        NodeType::DocumentType => false,
        NodeType::DocumentFragment => matches!(
            child_node_type,
            NodeType::Element
                | NodeType::Text
                | NodeType::Comment
                | NodeType::ProcessingInstruction
                | NodeType::CData
                | NodeType::EntityReference
        ),
        NodeType::Notation => false,
    }
}

pub(crate) fn create_document_with_options(
    namespace_uri: Option<&str>,
    qualified_name: Option<&str>,
    doc_type: Option<RefNode>,
    options: ProcessingOptions,
) -> Result<RefNode> {
    let node_impl = NodeImpl::new_document(doc_type, options);
    let mut document_node = RefNode::new(node_impl);

    //
    // If specified, create a new root element
    //
    let element: Option<RefNode> = {
        let ref_document = as_document(&document_node)?;
        match (namespace_uri, qualified_name) {
            (Some(namespace_uri), Some(qualified_name)) => {
                Some(ref_document.create_element_ns(namespace_uri, qualified_name)?)
            }
            (None, Some(qualified_name)) => Some(ref_document.create_element(qualified_name)?),
            (Some(_), None) => None,
            (None, None) => None,
        }
    };

    if let Some(element_node) = element {
        let document = as_document_mut(&mut document_node)?;
        let _safe_to_ignore = document.append_child(element_node)?;
    }

    Ok(document_node)
}

impl DocumentDecl for RefNode {
    fn xml_declaration(&self) -> Option<XmlDecl> {
        let ref_self = self.borrow();
        if let Extension::Document {
            xml_declaration, ..
        } = &ref_self.extension
        {
            xml_declaration.clone()
        } else {
            None
        }
    }

    fn set_xml_declaration(&mut self, xml_decl: XmlDecl) -> Result<()> {
        let mut mut_self = self.borrow_mut();
        if let Extension::Document {
            xml_declaration, ..
        } = &mut mut_self.extension
        {
            *xml_declaration = Some(xml_decl);
            Ok(())
        } else {
            Err(Error::InvalidState)
        }
    }
}

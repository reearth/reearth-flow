use std::collections::HashMap;

use crate::decl::XmlDecl;
use crate::name::Name;
use crate::namespace::NamespacePrefix;
use crate::options::ProcessingOptions;
use crate::Result;

use crate::text;

pub trait Attribute: Node {
    fn value(&self) -> Option<String>;

    fn set_value(&mut self, value: &str) -> Result<()>;

    fn unset_value(&mut self) -> Result<()>;

    fn specified(&self) -> bool {
        true
    }

    fn owner_element(&self) -> Option<Self::NodeRef>;
}

pub trait CDataSection: Text {}

pub trait CharacterData: Node {
    fn length(&self) -> usize {
        match self.data() {
            None => 0,
            Some(s) => s.len(),
        }
    }

    fn data(&self) -> Option<String> {
        let node_type = self.node_type();
        match (
            Node::node_value(self),
            node_type == NodeType::Text || node_type == NodeType::Comment,
        ) {
            (None, _) => None,
            (v @ Some(_), false) => v,
            (Some(value), true) => Some(text::escape(&value)),
        }
    }

    fn set_data(&mut self, data: &str) -> Result<()> {
        Node::set_node_value(self, data)
    }

    fn unset_data(&mut self) -> Result<()> {
        Node::unset_node_value(self)
    }

    fn substring_data(&self, offset: usize, count: usize) -> Result<String>;

    fn append_data(&mut self, data: &str) -> Result<()>;

    fn insert_data(&mut self, offset: usize, data: &str) -> Result<()>;

    fn delete_data(&mut self, offset: usize, count: usize) -> Result<()>;

    fn replace_data(&mut self, offset: usize, count: usize, data: &str) -> Result<()>;
}

pub trait Comment: CharacterData {}

///
pub trait Document: Node {
    fn doc_type(&self) -> Option<Self::NodeRef>;

    fn document_element(&self) -> Option<Self::NodeRef>;

    fn implementation(&self) -> &dyn DOMImplementation<NodeRef = Self::NodeRef>;

    fn create_attribute(&self, name: &str) -> Result<Self::NodeRef>;

    fn create_attribute_with(&self, name: &str, value: &str) -> Result<Self::NodeRef>;

    fn create_attribute_ns(
        &self,
        namespace_uri: &str,
        qualified_name: &str,
    ) -> Result<Self::NodeRef>;

    fn create_cdata_section(&self, data: &str) -> Result<Self::NodeRef>;

    fn create_document_fragment(&self) -> Result<Self::NodeRef>;

    fn create_entity_reference(&self, name: &str) -> Result<Self::NodeRef>;

    fn create_comment(&self, data: &str) -> Self::NodeRef;

    fn create_element(&self, tag_name: &str) -> Result<Self::NodeRef>;

    fn create_element_ns(&self, namespace_uri: &str, qualified_name: &str)
        -> Result<Self::NodeRef>;

    fn create_processing_instruction(
        &self,
        target: &str,
        data: Option<&str>,
    ) -> Result<Self::NodeRef>;

    fn create_text_node(&self, data: &str) -> Self::NodeRef;

    fn get_element_by_id(&self, id: &str) -> Option<Self::NodeRef>;

    fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<Self::NodeRef>;

    fn get_elements_by_tag_name_ns(
        &self,
        namespace_uri: &str,
        local_name: &str,
    ) -> Vec<Self::NodeRef>;
}

pub trait DocumentFragment: Node {}

pub trait DocumentType: Node {
    fn entities(&self) -> HashMap<Name, Self::NodeRef>;
    fn notations(&self) -> HashMap<Name, Self::NodeRef>;
    /// The public identifier of the external subset.
    fn public_id(&self) -> Option<String>;
    /// The system identifier of the external subset.
    fn system_id(&self) -> Option<String>;

    fn internal_subset(&self) -> Option<String>;
}

pub trait DOMImplementation {
    type NodeRef;

    fn create_document(
        &self,
        namespace_uri: Option<&str>,
        qualified_name: Option<&str>,
        doc_type: Option<Self::NodeRef>,
    ) -> Result<Self::NodeRef>;

    fn create_document_type(
        &self,
        qualified_name: &str,
        public_id: Option<&str>,
        system_id: Option<&str>,
    ) -> Result<Self::NodeRef>;

    fn has_feature(&self, feature: &str, version: &str) -> bool;

    fn create_document_with_options(
        &self,
        namespace_uri: Option<&str>,
        qualified_name: Option<&str>,
        doc_type: Option<Self::NodeRef>,
        options: ProcessingOptions,
    ) -> Result<Self::NodeRef>;
}

pub trait Element: Node {
    fn tag_name(&self) -> String {
        Node::node_name(self).to_string()
    }

    fn get_attribute(&self, name: &str) -> Option<String>;

    fn set_attribute(&mut self, name: &str, value: &str) -> Result<()>;

    fn remove_attribute(&mut self, _name: &str) -> Result<()>;

    fn get_attribute_node(&self, name: &str) -> Option<Self::NodeRef>;

    fn set_attribute_node(&mut self, new_attribute: Self::NodeRef) -> Result<Self::NodeRef>;

    fn remove_attribute_node(&mut self, _old_attribute: Self::NodeRef) -> Result<Self::NodeRef>;

    fn get_elements_by_tag_name(&self, _tag_name: &str) -> Vec<Self::NodeRef>;

    fn get_attribute_ns(&self, _namespace_uri: &str, _local_name: &str) -> Option<String>;

    fn set_attribute_ns(
        &mut self,
        namespace_uri: &str,
        qualified_name: &str,
        value: &str,
    ) -> Result<()>;

    fn remove_attribute_ns(&mut self, _namespace_uri: &str, _local_name: &str) -> Result<()>;

    fn get_attribute_node_ns(
        &self,
        _namespace_uri: &str,
        _local_name: &str,
    ) -> Option<Self::NodeRef>;

    fn set_attribute_node_ns(&mut self, _new_attribute: Self::NodeRef) -> Result<Self::NodeRef>;

    fn get_elements_by_tag_name_ns(
        &self,
        _namespace_uri: &str,
        _local_name: &str,
    ) -> Vec<Self::NodeRef>;

    fn has_attribute(&self, name: &str) -> bool;

    fn has_attribute_ns(&self, namespace_uri: &str, local_name: &str) -> bool;
}

pub trait Entity: Node {
    fn public_id(&self) -> Option<String>;

    fn system_id(&self) -> Option<String>;

    fn notation_name(&self) -> Option<String>;
}

pub trait EntityReference: Node {}

pub trait Node {
    type NodeRef;

    fn node_name(&self) -> Name;

    fn node_value(&self) -> Option<String>;

    fn set_node_value(&mut self, value: &str) -> Result<()>;

    fn unset_node_value(&mut self) -> Result<()>;

    fn node_type(&self) -> NodeType;

    fn parent_node(&self) -> Option<Self::NodeRef>;

    fn child_nodes(&self) -> Vec<Self::NodeRef>;

    fn first_child(&self) -> Option<Self::NodeRef>;

    fn last_child(&self) -> Option<Self::NodeRef>;

    fn previous_sibling(&self) -> Option<Self::NodeRef>;

    fn next_sibling(&self) -> Option<Self::NodeRef>;

    fn attributes(&self) -> HashMap<Name, Self::NodeRef>;

    fn owner_document(&self) -> Option<Self::NodeRef>;

    fn insert_before(
        &mut self,
        new_child: Self::NodeRef,
        ref_child: Option<Self::NodeRef>,
    ) -> Result<Self::NodeRef>;

    fn replace_child(
        &mut self,
        new_child: Self::NodeRef,
        old_child: Self::NodeRef,
    ) -> Result<Self::NodeRef>;

    fn remove_child(&mut self, old_child: Self::NodeRef) -> Result<Self::NodeRef>;

    fn append_child(&mut self, new_child: Self::NodeRef) -> Result<Self::NodeRef>;

    fn has_child_nodes(&self) -> bool;

    fn clone_node(&self, deep: bool) -> Option<Self::NodeRef>;

    fn normalize(&mut self);

    fn is_supported(&self, feature: &str, version: &str) -> bool;

    fn has_attributes(&self) -> bool;

    fn namespace_uri(&self) -> Option<String> {
        self.node_name().namespace_uri
    }

    fn local_name(&self) -> String {
        self.node_name().local_name
    }

    fn prefix(&self) -> Option<String> {
        self.node_name().prefix
    }
}

pub trait Notation: Node {
    fn public_id(&self) -> Option<String>;
    fn system_id(&self) -> Option<String>;
}

pub trait ProcessingInstruction: Node {
    fn length(&self) -> usize {
        match self.data() {
            None => 0,
            Some(s) => s.len(),
        }
    }

    fn data(&self) -> Option<String> {
        Node::node_value(self)
    }

    fn set_data(&mut self, data: &str) -> Result<()> {
        Node::set_node_value(self, data)
    }

    fn unset_data(&mut self) -> Result<()> {
        Node::unset_node_value(self)
    }

    fn target(&self) -> String {
        Node::node_name(self).to_string()
    }
}

pub trait Text: CharacterData {
    fn split(&mut self, offset: usize) -> Result<Self::NodeRef>;
}

// ------------------------------------------------------------------------------------------------

///
/// This corresponds to the DOM `NodeType` set of constants.
///
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum NodeType {
    /// The node is an [`Element`](trait.Element.html)
    Element = 1,
    /// The node is an [`Attribute`](trait.Attribute.html)
    Attribute,
    /// The node is a [`Text`](trait.Text.html)
    Text,
    /// The node is a [`CDataSection`](trait.CDataSection.html)
    CData,
    /// The node is an `EntityReference`
    EntityReference,
    /// The node is an `Entity`
    Entity,
    /// The node is a [`ProcessingInstruction`](trait.ProcessingInstruction.html)
    ProcessingInstruction,
    /// The node is a [`Comment`](trait.Comment.html)
    Comment,
    /// The node is a [`Document`](trait.Document.html)
    Document,
    /// The node is a [`DocumentType`](trait.DocumentType.html)
    DocumentType,
    /// The node is a `DocumentFragment`
    DocumentFragment,
    /// The node is a `Notation`
    Notation,
}

pub trait DocumentDecl: Document {
    ///
    /// Retrieve the current XML declaration, if set.
    ///
    fn xml_declaration(&self) -> Option<XmlDecl>;
    ///
    /// Set the current XML declaration for this document.
    ///
    /// Note that it is not possible to unset (set to `None`) this value.
    ///
    fn set_xml_declaration(&mut self, xml_decl: XmlDecl) -> Result<()>;
}

pub trait Namespaced: Element {
    fn contains_mapping(&self, prefix: Option<&str>) -> bool;

    fn get_namespace(&self, prefix: Option<&str>) -> Option<String>;

    fn resolve_namespace(&self, prefix: Option<&str>) -> Option<String>;

    fn contains_mapped_namespace(&self, namespace_uri: &str) -> bool;

    fn get_prefix(&self, namespace_uri: &str) -> NamespacePrefix;

    fn resolve_prefix(&self, namespace_uri: &str) -> NamespacePrefix;
}

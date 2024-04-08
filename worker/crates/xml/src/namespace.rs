/*!
This module provides support types for the [`Namespaced`](trait.Namespaced.html) trait.
*/

use crate::error::Error;
use crate::node::{Extension, RefNode};
use crate::traits::{Namespaced, NodeType};
use crate::Result;

pub type MutRefNamespaced<'a> = &'a mut dyn MutNamespaced<NodeRef = RefNode>;

#[inline]
pub(crate) fn as_element_namespaced_mut(ref_node: &mut RefNode) -> Result<MutRefNamespaced<'_>> {
    if ref_node.borrow().node_type == NodeType::Element {
        Ok(ref_node as MutRefNamespaced<'_>)
    } else {
        Err(Error::InvalidState)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NamespacePrefix {
    /// No mapping was discovered.
    None,
    /// A mapping was discovered for the default (no-value) prefix.
    Default,
    /// A mapping was discovered for the named prefix.
    Some(String),
}

pub trait MutNamespaced: Namespaced {
    fn insert_mapping(
        &mut self,
        prefix: Option<&str>,
        namespace_uri: &str,
    ) -> Result<Option<String>>;
    fn remove_mapping(&mut self, prefix: Option<&str>) -> Result<Option<String>>;
}

impl NamespacePrefix {
    #[allow(dead_code)]
    pub fn new(prefix: &str) -> Self {
        Self::Some(prefix.to_string())
    }

    pub fn is_none(&self) -> bool {
        matches!(self, NamespacePrefix::None)
    }
}

impl Namespaced for RefNode {
    fn contains_mapping(&self, prefix: Option<&str>) -> bool {
        let ref_self = self.borrow();
        if ref_self.node_type == NodeType::Element {
            if let Extension::Element { namespaces, .. } = &ref_self.extension {
                namespaces.contains_key(&prefix.map(String::from))
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_namespace(&self, prefix: Option<&str>) -> Option<String> {
        let ref_self = self.borrow();
        if ref_self.node_type == NodeType::Element {
            if let Extension::Element { namespaces, .. } = &ref_self.extension {
                let value = namespaces.get(&prefix.map(String::from));
                value.map(String::to_string)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn resolve_namespace(&self, prefix: Option<&str>) -> Option<String> {
        match self.get_namespace(prefix) {
            None => {
                let ref_self = self.borrow();
                match &ref_self.parent_node {
                    None => None,
                    Some(parent) => {
                        let parent = parent.clone();
                        match parent.upgrade() {
                            None => None,
                            Some(parent_node) => parent_node.resolve_namespace(prefix),
                        }
                    }
                }
            }
            found => found,
        }
    }

    fn contains_mapped_namespace(&self, namespace_uri: &str) -> bool {
        !self.get_prefix(namespace_uri).is_none()
    }

    fn get_prefix(&self, namespace_uri: &str) -> NamespacePrefix {
        let ref_self = self.borrow();
        if ref_self.node_type == NodeType::Element {
            if let Extension::Element { namespaces, .. } = &ref_self.extension {
                let ns = namespace_uri.to_string();
                let value = namespaces.iter().find(|(_, v)| **v == ns);
                match value {
                    None => NamespacePrefix::None,
                    Some((Some(k), _)) => NamespacePrefix::Some(k.clone()),
                    Some((None, _)) => NamespacePrefix::Default,
                }
            } else {
                NamespacePrefix::None
            }
        } else {
            NamespacePrefix::None
        }
    }

    fn resolve_prefix(&self, namespace_uri: &str) -> NamespacePrefix {
        match self.get_prefix(namespace_uri) {
            NamespacePrefix::None => {
                let ref_self = self.borrow();
                match &ref_self.parent_node {
                    None => NamespacePrefix::None,
                    Some(parent) => {
                        let parent = parent.clone();
                        let parent_node = parent.upgrade().unwrap();
                        parent_node.resolve_prefix(namespace_uri)
                    }
                }
            }
            found => found,
        }
    }
}

impl MutNamespaced for RefNode {
    fn insert_mapping(
        &mut self,
        prefix: Option<&str>,
        namespace_uri: &str,
    ) -> Result<Option<String>> {
        let mut mut_self = self.borrow_mut();
        if mut_self.node_type == NodeType::Element {
            if let Extension::Element { namespaces, .. } = &mut mut_self.extension {
                Ok(namespaces.insert(prefix.map(String::from), namespace_uri.to_string()))
            } else {
                Err(Error::InvalidState)
            }
        } else {
            Err(Error::InvalidState)
        }
    }

    fn remove_mapping(&mut self, prefix: Option<&str>) -> Result<Option<String>> {
        let mut mut_self = self.borrow_mut();
        if mut_self.node_type == NodeType::Element {
            if let Extension::Element { namespaces, .. } = &mut mut_self.extension {
                Ok(namespaces.remove(&prefix.map(String::from)))
            } else {
                Err(Error::InvalidState)
            }
        } else {
            Err(Error::InvalidState)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        convert::{as_document_mut, as_element_mut, as_element_namespaced, RefNamespaced},
        dom_impl::get_implementation,
        options::ProcessingOptions,
    };

    use super::*;

    const HTML: &str = "http://www.w3.org/1999/xhtml";
    const XSD: &str = "http://www.w3.org/2001/XMLSchema";
    const XSLT: &str = "http://www.w3.org/1999/XSL/Transform";
    const EX: &str = "http://example.org/xmlns/example";

    fn make_document_node() -> RefNode {
        let mut options = ProcessingOptions::new();
        options.set_add_namespaces();

        let implementation = get_implementation();
        implementation
            .create_document_with_options(Some("http://example.org/"), Some("root"), None, options)
            .unwrap()
    }

    fn make_node(document: &mut RefNode, name: &str) -> RefNode {
        let document = as_document_mut(document).unwrap();
        let element = document.create_element(name).unwrap();
        let mut document_element = document.document_element().unwrap();
        let document_element = as_element_mut(&mut document_element).unwrap();
        let _ = document_element.append_child(element.clone());
        element
    }

    #[test]
    fn test_empty_element() {
        let mut document = make_document_node();
        let ref_node = make_node(&mut document, "element");
        let namespaced = as_element_namespaced(&ref_node).unwrap();

        // prefix
        assert!(!namespaced.contains_mapping(None));
        assert!(!namespaced.contains_mapping(Some("s")));
        assert!(namespaced.get_namespace(None).is_none());
        assert!(namespaced.get_namespace(Some("s")).is_none());
        assert!(namespaced.resolve_namespace(None).is_none());
        assert!(namespaced.resolve_namespace(Some("s")).is_none());

        // namespace
        assert!(!namespaced.contains_mapped_namespace(HTML));
        assert!(namespaced.get_prefix(HTML).is_none());
        assert!(namespaced.resolve_prefix(HTML).is_none());
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_this_element_string_prefix() {
        let mut document = make_document_node();
        let mut ref_node = make_node(&mut document, "element");
        let namespaced = &mut ref_node as MutRefNamespaced<'_>;

        namespaced.insert_mapping(Some("xsd"), XSD);

        // prefix
        let ns_result = Some(XSD.to_string());

        assert!(!namespaced.contains_mapping(None));
        assert!(namespaced.contains_mapping(Some("xsd")));
        assert_eq!(namespaced.get_namespace(None), None);
        assert_eq!(namespaced.get_namespace(Some("xsd")), ns_result);
        assert_eq!(namespaced.resolve_namespace(None), None);
        assert_eq!(namespaced.resolve_namespace(Some("xsd")), ns_result);

        // namespace
        let prefix_result = NamespacePrefix::new("xsd");

        assert!(!namespaced.contains_mapped_namespace(HTML));
        assert!(namespaced.contains_mapped_namespace(XSD));
        assert_eq!(namespaced.get_prefix(XSD), prefix_result);
        assert_eq!(namespaced.resolve_prefix(XSD), prefix_result);
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_this_element_none_prefix() {
        let mut document = make_document_node();
        let mut ref_node = make_node(&mut document, "element");
        let namespaced = &mut ref_node as MutRefNamespaced<'_>;

        namespaced.insert_mapping(None, XSD);

        // prefix
        let ns_result = Some(XSD.to_string());

        assert!(namespaced.contains_mapping(None));
        assert!(!namespaced.contains_mapping(Some("xsd")));
        assert_eq!(namespaced.get_namespace(None), ns_result);
        assert_eq!(namespaced.get_namespace(Some("xsd")), None);
        assert_eq!(namespaced.resolve_namespace(None), ns_result);
        assert_eq!(namespaced.resolve_namespace(Some("xsd")), None);

        // namespace
        let prefix_result = NamespacePrefix::Default;

        assert!(!namespaced.contains_mapped_namespace(HTML));
        assert!(namespaced.contains_mapped_namespace(XSD));
        assert_eq!(namespaced.get_prefix(XSD), prefix_result);
        assert_eq!(namespaced.resolve_prefix(XSD), prefix_result);
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_tree_resolve() {
        let mut document = make_document_node();
        //
        // Setup the tree
        //
        let mut ref_node = make_node(&mut document, "element");
        let ref_root = as_element_namespaced_mut(&mut ref_node).unwrap();
        ref_root.insert_mapping(Some("xsd"), XSD);

        let mut teen_1_node = make_node(&mut document, "teen1");
        {
            let ref_teen_ns = as_element_namespaced_mut(&mut teen_1_node).unwrap();
            ref_teen_ns.insert_mapping(None, EX);
        }
        ref_root.append_child(teen_1_node.clone());

        let mut teen_2_node = make_node(&mut document, "teen2");
        {
            let ref_teen_ns = as_element_namespaced_mut(&mut teen_2_node).unwrap();
            ref_teen_ns.insert_mapping(None, HTML);
        }
        ref_root.append_child(teen_2_node.clone());

        let mut child_node = make_node(&mut document, "child");
        {
            let ref_child_ns = as_element_namespaced_mut(&mut child_node).unwrap();
            ref_child_ns.insert_mapping(Some("xslt"), XSLT);
        }
        {
            let ref_teen = as_element_namespaced_mut(&mut teen_2_node).unwrap();
            ref_teen.append_child(child_node.clone());
        }

        let ns_child = &child_node as RefNamespaced<'_>;

        //
        // Get
        //
        assert_eq!(ref_root.get_namespace(Some("xsd")), Some(XSD.to_string()));
        assert_eq!(ref_root.get_namespace(Some("xslt")), None);
        assert_eq!(ns_child.get_namespace(Some("xsd")), None);
        assert_eq!(ns_child.get_namespace(Some("xslt")), Some(XSLT.to_string()));

        //
        // Resolve
        //
        assert_eq!(
            ns_child.resolve_namespace(Some("xsd")),
            Some(XSD.to_string())
        );
        assert_eq!(ns_child.resolve_namespace(None), Some(HTML.to_string()));
        assert_eq!(
            ns_child.resolve_namespace(Some("xslt")),
            Some(XSLT.to_string())
        );

        //
        // Resolve by namespace
        //
        assert_eq!(ns_child.resolve_prefix(XSD), NamespacePrefix::new("xsd"));
        assert_eq!(ns_child.resolve_prefix(HTML), NamespacePrefix::Default);
        assert_eq!(ns_child.resolve_prefix(XSLT), NamespacePrefix::new("xslt"));
    }
}

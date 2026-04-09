//! Shared types and utilities for the CityGML 3 reader.

use std::collections::HashMap;
use std::sync::Arc;

pub(super) const GML_NS: &str = "http://www.opengis.net/gml/3.2";
pub(super) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// A generic, schema-agnostic XML node.
///
/// Retains the full qualified name (prefix + local) so downstream code can
/// distinguish namespaces without a namespace resolver.
#[derive(Debug, Clone)]
pub struct XmlNode {
    /// Qualified element name, e.g. `"bldg:Building"` or `"gml:Polygon"`.
    pub name: String,
    /// XML attributes in document order as `(qualified-name, namespace-uri, value)` triples.
    /// The qualified name is preserved as-is from the source document; the namespace URI
    /// is resolved so matching is prefix-independent.
    pub attrs: Vec<(String, String, String)>,
    /// Child content in document order.
    pub children: Vec<XmlChild>,
}

#[derive(Debug, Clone)]
pub enum XmlChild {
    Element(Arc<XmlNode>),
    Text(String),
}

/// Maps `(canonical-source-url, gml:id)` → owning nodes.
///
/// The URL component prevents collisions when multiple files share the same
/// `gml:id` value, and allows cross-file `xlink:href` resolution to target
/// the correct document.
pub type IdRegistry = HashMap<(String, String), Arc<XmlNode>>;

pub(super) fn local_name(name: &str) -> &str {
    name.rfind(':').map(|i| &name[i + 1..]).unwrap_or(name)
}

pub(super) fn qname(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes).unwrap_or("").to_string()
}

pub(super) fn gml_id_attr(node: &XmlNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|(q, ns, _)| local_name(q) == "id" && ns == GML_NS)
        .map(|(_, _, v)| v.clone())
}

pub(super) fn xlink_href_attr(attrs: &[(String, String, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|(q, ns, _)| local_name(q) == "href" && ns == XLINK_NS)
        .map(|(_, _, v)| v.as_str())
}

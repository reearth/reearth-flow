use std::collections::HashMap;
use std::sync::Arc;

pub(super) const GML_NS: &str = "http://www.opengis.net/gml/3.2";
pub(super) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

#[derive(Debug, Clone)]
pub struct XmlNode {
    pub name: String,
    /// `(qualified-name, namespace-uri, value)` — namespace URI is resolved so matching is prefix-independent.
    pub attrs: Vec<(String, String, String)>,
    pub children: Vec<XmlChild>,
    /// `true` if this node or any descendant carries an `xlink:href`; lets the resolver skip clean subtrees.
    pub has_xlinks: bool,
}

#[derive(Debug, Clone)]
pub enum XmlChild {
    Element(Arc<XmlNode>),
    Text(String),
}

/// `(source-url, gml:id)` → node. URL-keyed so same `gml:id` in different files never collides.
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

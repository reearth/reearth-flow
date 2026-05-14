use std::collections::HashMap;
use std::sync::Arc;

use url::Url;

pub(super) const GML_NS: &str = "http://www.opengis.net/gml/3.2";
pub(super) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

pub type NsId = u32;
pub const EMPTY_NS_ID: NsId = 0;
pub(super) const GML_NS_ID: NsId = 1;
pub(super) const XLINK_NS_ID: NsId = 2;

/// `(qname, ns-id)`
pub type QName = (String, NsId);

#[derive(Debug, Clone)]
pub struct XmlNode {
    pub name: QName,
    pub attrs: Vec<(QName, String)>,
    pub children: Vec<XmlChild>,
    /// Source file URL this node was parsed from. Shared across all nodes from the same file.
    pub source_url: Arc<Url>,
}

impl XmlNode {
    pub fn with_children(&self, children: Vec<XmlChild>) -> Arc<Self> {
        Arc::new(Self {
            name: self.name.clone(),
            attrs: self.attrs.clone(),
            children,
            source_url: Arc::clone(&self.source_url),
        })
    }
}

#[derive(Debug, Clone)]
pub enum XmlChild {
    Element(Arc<XmlNode>),
    Text(String),
}

/// Interns namespace URIs as u32 IDs, avoiding repeated allocation of long URI strings.
/// IDs 0–2 are always pre-assigned: 0="" (no namespace), 1=GML_NS, 2=XLINK_NS.
pub(super) struct NamespaceRegistry {
    uris: Vec<String>,
    index: HashMap<String, NsId>,
}

impl NamespaceRegistry {
    pub(super) fn new() -> Self {
        let mut r = Self {
            uris: Vec::new(),
            index: HashMap::new(),
        };
        for uri in ["", GML_NS, XLINK_NS] {
            let id = r.uris.len() as NsId;
            r.uris.push(uri.to_string());
            r.index.insert(uri.to_string(), id);
        }
        r
    }

    pub(super) fn intern(&mut self, uri: &str) -> NsId {
        if let Some(&id) = self.index.get(uri) {
            return id;
        }
        let id = self.uris.len() as NsId;
        self.uris.push(uri.to_string());
        self.index.insert(uri.to_string(), id);
        id
    }

    /// Returns the ID for a URI only if it was already interned during parsing.
    pub(super) fn get(&self, uri: &str) -> Option<NsId> {
        self.index.get(uri).copied()
    }
}

pub(super) fn local_name(qname: &str) -> &str {
    qname.rfind(':').map(|i| &qname[i + 1..]).unwrap_or(qname)
}

pub(super) fn gml_id_attr(node: &XmlNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "id" && *ns == GML_NS_ID)
        .map(|(_, v)| v.clone())
}

pub(super) fn xlink_href_attr(attrs: &[(QName, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "href" && *ns == XLINK_NS_ID)
        .map(|(_, v)| v.as_str())
}

#[cfg(test)]
pub(crate) fn test_url() -> Arc<Url> {
    Arc::new(Url::parse("file:///test.gml").unwrap())
}

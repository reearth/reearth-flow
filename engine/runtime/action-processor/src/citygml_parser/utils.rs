use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use url::Url;

pub(super) const GML_NS_32: &str = "http://www.opengis.net/gml/3.2";
pub(super) const GML_NS_311: &str = "http://www.opengis.net/gml";
pub(super) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

pub type NsId = u32;
pub const EMPTY_NS_ID: NsId = 0;
pub(super) const GML_NS_ID: NsId = 1;
pub(super) const XLINK_NS_ID: NsId = 2;
pub(super) const GML_NS_311_ID: NsId = 3;

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
/// IDs 0–3 are always pre-assigned: 0="" (no namespace), 1=GML_NS_32, 2=XLINK_NS, 3=GML_NS_311.
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
        for uri in ["", GML_NS_32, XLINK_NS, GML_NS_311] {
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

pub(super) fn gml_id_attr(attrs: &[(QName, String)]) -> Option<String> {
    attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "id" && (*ns == GML_NS_ID || *ns == GML_NS_311_ID))
        .map(|(_, v)| v.clone())
}

pub(super) fn xlink_href_attr(attrs: &[(QName, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "href" && *ns == XLINK_NS_ID)
        .map(|(_, v)| v.as_str())
}

#[cfg(feature = "new-geometry")]
pub(super) fn srs_name_attr(attrs: &[(QName, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|((q, _), _)| local_name(q) == "srsName")
        .map(|(_, v)| v.as_str())
}

/// Parse the trailing EPSG code from a `srsName` URI, e.g.
/// `http://www.opengis.net/def/crs/EPSG/0/6697` -> `6697`. `None` if the URI's
/// last path segment isn't a plain number.
#[cfg(feature = "new-geometry")]
pub(super) fn parse_epsg_from_srs_name(srs_name: &str) -> Option<EpsgCode> {
    srs_name
        .rsplit('/')
        .next()
        .and_then(|code| code.parse::<u16>().ok())
        .map(EpsgCode::new)
}

/// The frame for geometry parsed from `file`: `Crs` if `srs_by_file` has an entry
/// for it, `Euclidean` (no known CRS) otherwise.
#[cfg(feature = "new-geometry")]
pub(super) fn frame_for(file: &str, srs_by_file: &HashMap<String, EpsgCode>) -> CoordinateFrame {
    match srs_by_file.get(file) {
        Some(&epsg) => CoordinateFrame::Crs(epsg),
        None => CoordinateFrame::Euclidean,
    }
}

#[cfg(test)]
pub(crate) fn test_url() -> Arc<Url> {
    Arc::new(Url::parse("file:///test.gml").unwrap())
}

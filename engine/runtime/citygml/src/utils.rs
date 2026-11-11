use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_types::{Attribute, AttributeValue, Attributes};
use url::Url;

/// One value per element, or just the value when there is a single element.
pub(super) fn one_or_array(mut values: Vec<AttributeValue>) -> AttributeValue {
    if values.len() == 1 {
        values.remove(0)
    } else {
        AttributeValue::Array(values)
    }
}

/// Stores the values and hoisted attributes of same-named elements under a node.
#[derive(Default)]
pub(super) struct ElementGroup {
    /// One value per element.
    pub(super) values: Vec<AttributeValue>,
    /// By key suffix (`@uom`, `_code`): one entry per value, `Null` where it has none.
    hoisted: IndexMap<String, Vec<AttributeValue>>,
}

impl ElementGroup {
    /// Adds a value together with the attributes hoisted from it.
    pub(super) fn push(&mut self, value: AttributeValue, hoisted: &[(String, &str)]) {
        let at = self.values.len();
        for (suffix, entry) in hoisted {
            let entries = self.hoisted.entry(suffix.clone()).or_default();
            entries.resize(at, AttributeValue::Null);
            entries.push(AttributeValue::String(entry.to_string()));
        }
        self.values.push(value);
    }

    /// Writes the group into `map`, returning the derived keys left out because the name was taken.
    pub(super) fn insert_into(self, map: &mut Attributes, name: &str) -> Vec<String> {
        let mut left_out = Vec::new();
        let len = self.values.len();
        if map
            .insert(Attribute::new(name), one_or_array(self.values))
            .is_some()
        {
            left_out.push(name.to_string());
        }
        for (suffix, mut entries) in self.hoisted {
            let key = format!("{name}{suffix}");
            if map.contains_key(key.as_str()) {
                left_out.push(key);
                continue;
            }
            entries.resize(len, AttributeValue::Null);
            map.insert(Attribute::new(key), one_or_array(entries));
        }
        left_out
    }
}

pub(super) const GML_NS_32: &str = "http://www.opengis.net/gml/3.2";
pub(super) const GML_NS_311: &str = "http://www.opengis.net/gml";
pub(super) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
pub(super) const CITYGML_NS_20: &str = "http://www.opengis.net/citygml/2.0";
pub(super) const CITYGML_NS_30: &str = "http://www.opengis.net/citygml/3.0";

pub type NsId = u32;
pub const EMPTY_NS_ID: NsId = 0;
pub(super) const GML_NS_ID: NsId = 1;
pub(super) const XLINK_NS_ID: NsId = 2;
pub(super) const GML_NS_311_ID: NsId = 3;
pub(super) const CITYGML_NS_20_ID: NsId = 4;
pub(super) const CITYGML_NS_30_ID: NsId = 5;

/// Which CityGML edition a `Parser` expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CityGmlVersion {
    V2,
    V3,
}

impl CityGmlVersion {
    pub(super) fn core_ns_id(self) -> NsId {
        match self {
            CityGmlVersion::V2 => CITYGML_NS_20_ID,
            CityGmlVersion::V3 => CITYGML_NS_30_ID,
        }
    }
}

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

#[cfg(feature = "new-geometry")]
pub(super) use super::resolver::GeomNode;

/// Legacy never constructs one; this stub only exists so shared code can name the type unconditionally.
#[cfg(not(feature = "new-geometry"))]
#[derive(Debug)]
pub enum GeomNode {}

/// What the source GML said about a carved geometry beyond the geometry itself.
/// Reaches the feature as the member attributes documented on
/// [`MEMBER_LOD_KEY`](super::pipeline::MEMBER_LOD_KEY).
#[derive(Debug, Clone)]
pub struct GeomMeta {
    pub lod: Option<u8>,
    pub property: String,
    pub gml_type: Option<String>,
}

#[derive(Debug, Clone)]
pub enum XmlChild {
    Element(Arc<XmlNode>),
    Text(String),
    Geometry(GeomMeta, Arc<GeomNode>),
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
        for uri in [
            "",
            GML_NS_32,
            XLINK_NS,
            GML_NS_311,
            CITYGML_NS_20,
            CITYGML_NS_30,
        ] {
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

/// The `srsName` attribute of an element, if present.
#[cfg(feature = "new-geometry")]
pub(super) fn srs_name_attr(attrs: &[(QName, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|((q, _), _)| local_name(q) == "srsName")
        .map(|(_, v)| v.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn string(value: &str) -> AttributeValue {
        AttributeValue::String(value.to_string())
    }

    /// A derived `_code` key never replaces an element of that name, and is reported.
    #[test]
    fn a_derived_key_never_replaces_an_element_of_that_name() {
        let mut map = Attributes::new();
        map.insert(Attribute::new("m_code"), string("real"));
        let mut group = ElementGroup::default();
        group.push(string("label"), &[("_code".to_string(), "1")]);

        let left_out = group.insert_into(&mut map, "m");

        assert_eq!(left_out, ["m_code"]);
        assert_eq!(map.get("m_code"), Some(&string("real")));
        assert_eq!(map.get("m"), Some(&string("label")));
    }
}

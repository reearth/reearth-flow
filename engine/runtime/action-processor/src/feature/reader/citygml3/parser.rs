//! Schema-agnostic CityGML 3 parser.

use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;

use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};
use url::Url;

pub(super) const GML_NS: &str = "http://www.opengis.net/gml/3.2";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

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

/// Maps `gml:id` values → owning nodes.
pub type IdRegistry = HashMap<String, Arc<XmlNode>>;

/// A parsed CityGML 3 top-level feature (direct child of `cityObjectMember`
/// or `featureMember`).
#[derive(Debug)]
pub struct TopLevelFeature {
    /// Value of the `gml:id` attribute on the top-level element, if present.
    pub gml_id: Option<String>,
    /// Qualified element name, e.g. `"bldg:Building"`.
    pub feature_type: String,
    /// Full parsed subtree; retained for geometry / subfeature extraction.
    pub node: Arc<XmlNode>,
    /// Source file — retained for diagnostics and future cross-file work.
    #[allow(dead_code)]
    pub source_url: Url,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Encoding error: {0}")]
    Encoding(String),
    #[error("No CityModel root element found")]
    NoCityModel,
    #[error("Unexpected end of file inside CityModel")]
    UnexpectedEof,
}

/// Parse a CityGML 3 document.
///
/// Returns one [`TopLevelFeature`] per `cityObjectMember` / `featureMember`
/// child of the root `CityModel`.  Every `gml:id`-bearing node found in
/// those subtrees is registered into `id_registry`.
pub fn parse(
    source: &[u8],
    source_url: &Url,
    id_registry: &mut IdRegistry,
) -> Result<Vec<TopLevelFeature>, ParseError> {
    let src = std::str::from_utf8(source)
        .map_err(|e| ParseError::Encoding(format!("Non-UTF-8 content: {e}")))?;
    let mut reader = NsReader::from_str(src);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    // Advance past the XML declaration and any preamble to the CityModel root.
    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, .. } if local_name(&name) == "CityModel" => break,
            OwnedEvent::Eof => return Err(ParseError::NoCityModel),
            _ => {}
        }
    }

    let mut features = Vec::new();

    // Stream over CityModel's direct children.
    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, attrs } => {
                let ln = local_name(&name);
                if ln == "cityObjectMember" || ln == "featureMember" {
                    // Parse the complete member element; the top-level feature
                    // is its first element child.
                    let member = parse_element(&mut reader, &mut buf, name, attrs)?;
                    if let Some(XmlChild::Element(feature_node)) = member.children.first() {
                        let feature_node = Arc::clone(feature_node);
                        collect_ids(&feature_node, id_registry);
                        let gml_id = gml_id_attr(&feature_node);
                        let feature_type = feature_node.name.clone();
                        features.push(TopLevelFeature {
                            gml_id,
                            feature_type,
                            node: feature_node,
                            source_url: source_url.clone(),
                        });
                    } else {
                        tracing::warn!("citygml3: empty cityObjectMember/featureMember, skipped");
                    }
                } else {
                    // Other CityModel children (gml:boundedBy, metadata, …) — skip.
                    skip_element(&mut reader, &mut buf)?;
                }
            }
            OwnedEvent::End => break,
            OwnedEvent::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(features)
}

/// Convert a [`TopLevelFeature`] to an engine [`Feature`].
///
/// `resolved` must be the result of calling [`resolve_xlinks`] on `tlf.node`.
pub fn to_feature(tlf: &TopLevelFeature, resolved: &XmlNode) -> Feature {
    let content = node_to_attribute_value(resolved);
    build_feature(&tlf.feature_type, tlf.gml_id.as_deref(), content)
}

/// Recursively walk `node`, replacing elements that carry only an
/// `xlink:href` attribute and no children with the referenced node from
/// `registry`.
///
/// Unresolvable references are left in place. Resolution is single-pass.
pub fn resolve_xlinks(node: &XmlNode, registry: &IdRegistry) -> XmlNode {
    if node.children.is_empty() {
        if let Some(href) = xlink_href_attr(&node.attrs) {
            let id = href
                .strip_prefix('#')
                .or_else(|| href.split_once('#').map(|(_, frag)| frag));
            if let Some(id) = id {
                if let Some(target) = registry.get(id) {
                    let attrs = node
                        .attrs
                        .iter()
                        .filter(|(qname, ns, _)| !(local_name(qname) == "href" && ns == XLINK_NS))
                        .cloned()
                        .collect();
                    return XmlNode {
                        name: node.name.clone(),
                        attrs,
                        children: vec![XmlChild::Element(Arc::clone(target))],
                    };
                }
            }
        }
    }

    let children = node
        .children
        .iter()
        .map(|c| match c {
            XmlChild::Element(e) => XmlChild::Element(Arc::new(resolve_xlinks(e, registry))),
            XmlChild::Text(t) => XmlChild::Text(t.clone()),
        })
        .collect();

    XmlNode {
        name: node.name.clone(),
        attrs: node.attrs.clone(),
        children,
    }
}

/// Convert an [`XmlNode`] to an [`AttributeValue`].
///
/// Rules:
/// - Node with only text content and no XML attributes → `String`
/// - Otherwise → `Map`:
///   - XML attributes as `"@{qualified-name}"` keys (e.g. `"@gml:id"`)
///   - Mixed text content under the `"$"` key
///   - Multiple sibling elements with the same tag name → `Array`
pub fn node_to_attribute_value(node: &XmlNode) -> AttributeValue {
    let mut elem_groups: IndexMap<String, Vec<AttributeValue>> = IndexMap::new();
    let mut text_parts: Vec<String> = Vec::new();

    for child in &node.children {
        match child {
            XmlChild::Element(e) => {
                elem_groups
                    .entry(e.name.clone())
                    .or_default()
                    .push(node_to_attribute_value(e));
            }
            XmlChild::Text(t) => text_parts.push(t.clone()),
        }
    }

    // Pure text with no XML attrs and no element children → bare String.
    if node.attrs.is_empty() && elem_groups.is_empty() {
        return AttributeValue::String(text_parts.join(""));
    }

    let mut map: HashMap<String, AttributeValue> = HashMap::new();

    for (qname, _, v) in &node.attrs {
        map.insert(format!("@{qname}"), AttributeValue::String(v.clone()));
    }
    if !text_parts.is_empty() {
        map.insert("$".into(), AttributeValue::String(text_parts.join("")));
    }
    for (name, mut values) in elem_groups {
        let av = if values.len() == 1 {
            values.pop().unwrap()
        } else {
            AttributeValue::Array(values)
        };
        map.insert(name, av);
    }

    AttributeValue::Map(map)
}

fn build_feature(feature_type: &str, gml_id: Option<&str>, content: AttributeValue) -> Feature {
    let mut attrs = Attributes::new();

    if let AttributeValue::Map(map) = content {
        for (k, v) in map {
            attrs.insert(Attribute::new(k), v);
        }
    }

    let mut feature = Feature::new_with_attributes(attrs);
    feature.update_feature_type(feature_type.to_string());
    if let Some(id) = gml_id {
        feature.update_feature_id(id.to_string());
    }
    feature
}

/// Owned, lifetime-free view of a quick-xml event.
///
/// Extracting owned data immediately inside `next_event` avoids the lifetime
/// conflict that would arise if we tried to hold a borrowed `Event<'_>` across
/// a recursive call to `parse_element`.
enum OwnedEvent {
    Start {
        name: String,
        attrs: Vec<(String, String, String)>,
    },
    End,
    Empty {
        name: String,
        attrs: Vec<(String, String, String)>,
    },
    Text(String),
    Eof,
    Other,
}

fn next_event<R: BufRead>(
    reader: &mut NsReader<R>,
    buf: &mut Vec<u8>,
) -> Result<OwnedEvent, ParseError> {
    let (_, event) = reader
        .read_resolved_event_into(buf)
        .map_err(ParseError::Xml)?;
    match event {
        Event::Start(e) => Ok(OwnedEvent::Start {
            name: qname(e.name().as_ref()),
            attrs: extract_attrs(&e, reader),
        }),
        Event::End(_) => Ok(OwnedEvent::End),
        Event::Empty(e) => Ok(OwnedEvent::Empty {
            name: qname(e.name().as_ref()),
            attrs: extract_attrs(&e, reader),
        }),
        Event::Text(t) => Ok(OwnedEvent::Text(
            t.unescape().map_err(ParseError::Xml)?.trim().to_string(),
        )),
        Event::CData(c) => Ok(OwnedEvent::Text(
            std::str::from_utf8(&c).unwrap_or("").trim().to_string(),
        )),
        Event::Eof => Ok(OwnedEvent::Eof),
        _ => Ok(OwnedEvent::Other),
    }
}

/// Parse one element and its entire subtree.
///
/// `name` and `attrs` are the already-extracted data from the opening tag.
/// The reader is positioned just after that opening tag.
fn parse_element<R: BufRead>(
    reader: &mut NsReader<R>,
    buf: &mut Vec<u8>,
    name: String,
    attrs: Vec<(String, String, String)>,
) -> Result<XmlNode, ParseError> {
    let mut children = Vec::new();

    loop {
        match next_event(reader, buf)? {
            OwnedEvent::Start {
                name: cn,
                attrs: ca,
            } => {
                let child = parse_element(reader, buf, cn, ca)?;
                children.push(XmlChild::Element(Arc::new(child)));
            }
            OwnedEvent::Empty {
                name: cn,
                attrs: ca,
            } => {
                children.push(XmlChild::Element(Arc::new(XmlNode {
                    name: cn,
                    attrs: ca,
                    children: Vec::new(),
                })));
            }
            OwnedEvent::End | OwnedEvent::Eof => break,
            OwnedEvent::Text(t) if !t.is_empty() => {
                children.push(XmlChild::Text(t));
            }
            _ => {}
        }
    }

    Ok(XmlNode {
        name,
        attrs,
        children,
    })
}

/// Skip an already-opened element (its start tag has been consumed).
fn skip_element<R: BufRead>(reader: &mut NsReader<R>, buf: &mut Vec<u8>) -> Result<(), ParseError> {
    let mut depth: usize = 1;
    loop {
        match next_event(reader, buf)? {
            OwnedEvent::Start { .. } => depth += 1,
            OwnedEvent::End => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            OwnedEvent::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

fn collect_ids(node: &Arc<XmlNode>, registry: &mut IdRegistry) {
    if let Some(id) = gml_id_attr(node) {
        registry.insert(id, Arc::clone(node));
    }
    for child in &node.children {
        if let XmlChild::Element(e) = child {
            collect_ids(e, registry);
        }
    }
}

fn extract_attrs<R: BufRead>(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &NsReader<R>,
) -> Vec<(String, String, String)> {
    e.attributes()
        .filter_map(|a| a.ok())
        .map(|a| {
            let qname_str = qname(a.key.as_ref());
            let ns_uri = match reader.resolve_attribute(a.key).0 {
                ResolveResult::Bound(ns) => std::str::from_utf8(ns.into_inner())
                    .unwrap_or("")
                    .to_string(),
                _ => String::new(),
            };
            let v = std::str::from_utf8(a.value.as_ref())
                .unwrap_or("")
                .to_string();
            (qname_str, ns_uri, v)
        })
        .collect()
}

fn qname(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes).unwrap_or("").to_string()
}

fn local_name(name: &str) -> &str {
    name.rfind(':').map(|i| &name[i + 1..]).unwrap_or(name)
}

fn gml_id_attr(node: &XmlNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|(qname, ns, _)| local_name(qname) == "id" && ns == GML_NS)
        .map(|(_, _, v)| v.clone())
}

fn xlink_href_attr(attrs: &[(String, String, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|(qname, ns, _)| local_name(qname) == "href" && ns == XLINK_NS)
        .map(|(_, _, v)| v.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::CitygmlFeatureExt;
    use url::Url;

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn make_node(name: &str, attrs: Vec<(&str, &str, &str)>, children: Vec<XmlChild>) -> XmlNode {
        XmlNode {
            name: name.to_string(),
            attrs: attrs
                .into_iter()
                .map(|(q, ns, v)| (q.to_string(), ns.to_string(), v.to_string()))
                .collect(),
            children,
        }
    }

    fn text(s: &str) -> XmlChild {
        XmlChild::Text(s.to_string())
    }

    fn elem(node: XmlNode) -> XmlChild {
        XmlChild::Element(Arc::new(node))
    }

    // ---- parse ----

    #[test]
    fn parse_errors_for_non_citygml() {
        let xml = b"<Foo/>";
        let mut reg = IdRegistry::new();
        assert!(matches!(
            parse(xml, &dummy_url(), &mut reg),
            Err(ParseError::NoCityModel)
        ));
    }

    #[test]
    fn parse_extracts_top_level_features() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <bldg:lod1Solid/>
    </bldg:Building>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg002"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = IdRegistry::new();
        let features = parse(xml, &dummy_url(), &mut reg).unwrap();

        assert_eq!(features.len(), 2);
        assert_eq!(features[0].gml_id.as_deref(), Some("bldg001"));
        assert_eq!(features[1].gml_id.as_deref(), Some("bldg002"));
        assert_eq!(features[0].feature_type, "bldg:Building");
    }

    #[test]
    fn parse_non_standard_gml_prefix() {
        // Uses 'g:' instead of 'gml:' — id must still be recognized.
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:g="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building g:id="bldg001"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = IdRegistry::new();
        let features = parse(xml, &dummy_url(), &mut reg).unwrap();
        assert_eq!(features[0].gml_id.as_deref(), Some("bldg001"));
        assert!(reg.contains_key("bldg001"));
    }

    #[test]
    fn parse_registers_ids_in_registry() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <bldg:part gml:id="part001"/>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = IdRegistry::new();
        parse(xml, &dummy_url(), &mut reg).unwrap();

        assert!(reg.contains_key("bldg001"));
        assert!(reg.contains_key("part001"));
    }

    #[test]
    fn parse_skips_non_member_children() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <gml:boundedBy><gml:Envelope/></gml:boundedBy>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = IdRegistry::new();
        let features = parse(xml, &dummy_url(), &mut reg).unwrap();
        assert_eq!(features.len(), 1);
    }

    #[test]
    fn parse_errors_on_truncated_citygml() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001"/>
  </core:cityObjectMember>"#;

        let mut reg = IdRegistry::new();
        assert!(matches!(
            parse(xml, &dummy_url(), &mut reg),
            Err(ParseError::UnexpectedEof)
        ));
    }

    #[test]
    fn parse_skips_empty_member_without_panic() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember/>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = IdRegistry::new();
        let features = parse(xml, &dummy_url(), &mut reg).unwrap();
        assert_eq!(features.len(), 1);
    }

    // ---- resolve_xlinks ----

    #[test]
    fn resolve_xlinks_replaces_href_leaf() {
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "poly001")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert("poly001".to_string(), target);

        let node = make_node(
            "bldg:lod1Solid",
            vec![("xlink:href", XLINK_NS, "#poly001")],
            vec![],
        );

        let resolved = resolve_xlinks(&node, &reg);

        assert_eq!(resolved.children.len(), 1);
        if let XmlChild::Element(e) = &resolved.children[0] {
            assert_eq!(e.name, "gml:Polygon");
        } else {
            panic!("expected Element child");
        }
        assert!(!resolved
            .attrs
            .iter()
            .any(|(q, ns, _)| { local_name(q) == "href" && ns == XLINK_NS }));
    }

    #[test]
    fn resolve_xlinks_non_standard_xlink_prefix() {
        // Uses 'xl:href' instead of 'xlink:href' — must still resolve.
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "p1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert("p1".to_string(), target);

        let node = make_node("ref", vec![("xl:href", XLINK_NS, "#p1")], vec![]);
        let resolved = resolve_xlinks(&node, &reg);

        assert_eq!(resolved.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_handles_cross_file_fragment() {
        let target = Arc::new(make_node(
            "gml:Polygon",
            vec![("gml:id", GML_NS, "p1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert("p1".to_string(), target);

        let node = make_node(
            "ref",
            vec![("xlink:href", XLINK_NS, "other.gml#p1")],
            vec![],
        );
        let resolved = resolve_xlinks(&node, &reg);

        assert_eq!(resolved.children.len(), 1);
    }

    #[test]
    fn resolve_xlinks_leaves_unresolvable_in_place() {
        let reg = IdRegistry::new();
        let node = make_node("ref", vec![("xlink:href", XLINK_NS, "#missing")], vec![]);
        let resolved = resolve_xlinks(&node, &reg);

        assert!(resolved.children.is_empty());
        assert!(resolved
            .attrs
            .iter()
            .any(|(q, ns, _)| { local_name(q) == "href" && ns == XLINK_NS }));
    }

    #[test]
    fn resolve_xlinks_recurses_into_children() {
        let target = Arc::new(make_node(
            "gml:Point",
            vec![("gml:id", GML_NS, "pt1")],
            vec![],
        ));
        let mut reg = IdRegistry::new();
        reg.insert("pt1".to_string(), target);

        let inner = make_node("bldg:pos", vec![("xlink:href", XLINK_NS, "#pt1")], vec![]);
        let node = make_node("bldg:Building", vec![], vec![elem(inner)]);

        let resolved = resolve_xlinks(&node, &reg);

        let child = match &resolved.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!("expected element"),
        };
        assert_eq!(child.name, "bldg:pos");
        assert_eq!(child.children.len(), 1);
    }

    // ---- node_to_attribute_value ----

    #[test]
    fn node_to_attribute_value_pure_text() {
        let node = make_node("gml:name", vec![], vec![text("Building A")]);
        let av = node_to_attribute_value(&node);
        assert_eq!(av, AttributeValue::String("Building A".to_string()));
    }

    #[test]
    fn node_to_attribute_value_attrs_become_map() {
        let node = make_node(
            "gml:name",
            vec![("gml:id", GML_NS, "n1")],
            vec![text("foo")],
        );
        let av = node_to_attribute_value(&node);
        let AttributeValue::Map(map) = av else {
            panic!("expected Map");
        };
        // qname is preserved in the key
        assert_eq!(
            map.get("@gml:id"),
            Some(&AttributeValue::String("n1".to_string()))
        );
        assert_eq!(
            map.get("$"),
            Some(&AttributeValue::String("foo".to_string()))
        );
    }

    #[test]
    fn node_to_attribute_value_non_standard_prefix_preserves_qname() {
        // Attribute stored with qname 'g:id' must appear as '@g:id' in output.
        let node = make_node("bldg:Building", vec![("g:id", GML_NS, "bldg001")], vec![]);
        let AttributeValue::Map(map) = node_to_attribute_value(&node) else {
            panic!("expected Map");
        };
        assert_eq!(
            map.get("@g:id"),
            Some(&AttributeValue::String("bldg001".to_string()))
        );
    }

    #[test]
    fn node_to_attribute_value_repeated_children_become_array() {
        let node = make_node(
            "parent",
            vec![],
            vec![
                elem(make_node("item", vec![], vec![text("a")])),
                elem(make_node("item", vec![], vec![text("b")])),
            ],
        );
        let AttributeValue::Map(map) = node_to_attribute_value(&node) else {
            panic!("expected Map");
        };
        let AttributeValue::Array(arr) = map.get("item").unwrap() else {
            panic!("expected Array");
        };
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn node_to_attribute_value_single_child_not_wrapped_in_array() {
        let node = make_node(
            "parent",
            vec![],
            vec![elem(make_node("item", vec![], vec![text("only")]))],
        );
        let AttributeValue::Map(map) = node_to_attribute_value(&node) else {
            panic!("expected Map");
        };
        assert!(matches!(map.get("item"), Some(AttributeValue::String(_))));
    }

    // ---- to_feature ----

    #[test]
    fn to_feature_sets_feature_type_and_id() {
        let node = Arc::new(make_node(
            "bldg:Building",
            vec![("gml:id", GML_NS, "bldg001")],
            vec![],
        ));
        let tlf = TopLevelFeature {
            gml_id: Some("bldg001".to_string()),
            feature_type: "bldg:Building".to_string(),
            node,
            source_url: dummy_url(),
        };
        let reg = IdRegistry::new();
        let resolved = resolve_xlinks(&tlf.node, &reg);
        let feature = to_feature(&tlf, &resolved);

        assert_eq!(feature.feature_type(), Some("bldg:Building".to_string()));
        assert_eq!(feature.feature_id(), Some("bldg001".to_string()));
    }
}

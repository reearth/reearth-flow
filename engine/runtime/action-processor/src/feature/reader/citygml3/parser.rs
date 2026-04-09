use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;

use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};
use url::Url;

use super::utils::{
    gml_id_attr, local_name as utils_local_name, parse_qname, xlink_href_attr, QName, XmlChild,
    XmlNode, GML_NS, XLINK_NS,
};

pub(super) type RawNodeKey = (String, String); // (file_url, gml_id)

pub(crate) struct RawNode {
    pub(crate) name: QName,
    pub(crate) attrs: Vec<(QName, String)>,
    pub(crate) children: Vec<RawChild>,
}

pub(crate) enum RawChild {
    Element(Arc<RawNode>),
    Text(String),
    Ref(RawNodeKey),
}

pub(crate) type RawRegistry = HashMap<RawNodeKey, Arc<RawNode>>;

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

pub fn parse(
    source: &[u8],
    source_url: &Url,
    registry: &mut RawRegistry,
) -> Result<Vec<Arc<RawNode>>, ParseError> {
    let src = std::str::from_utf8(source)
        .map_err(|e| ParseError::Encoding(format!("Non-UTF-8 content: {e}")))?;
    let mut reader = NsReader::from_str(src);
    let mut buf = Vec::new();

    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, .. } if local_name(&name.0) == "CityModel" => break,
            OwnedEvent::Eof => return Err(ParseError::NoCityModel),
            _ => {}
        }
    }

    let mut features = Vec::new();

    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, attrs } => {
                let ln = local_name(&name.0);
                if ln == "cityObjectMember" || ln == "featureMember" {
                    let member = parse_element(&mut reader, &mut buf, name, attrs, source_url)?;
                    if let Some(feature_node) = member.children.iter().find_map(|child| match child
                    {
                        RawChild::Element(node) => Some(Arc::clone(node)),
                        _ => None,
                    }) {
                        collect_ids(&feature_node, source_url.as_str(), registry);
                        features.push(feature_node);
                    } else {
                        tracing::warn!("citygml3: empty cityObjectMember/featureMember, skipped");
                    }
                } else {
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

pub fn to_feature(node: &XmlNode) -> Feature {
    let content = node_to_attribute_value(node);
    build_feature(&node.name.0, gml_id_attr(node).as_deref(), content)
}

pub fn node_to_attribute_value(node: &XmlNode) -> AttributeValue {
    let mut elem_groups: IndexMap<String, Vec<AttributeValue>> = IndexMap::new();
    let mut text_parts: Vec<String> = Vec::new();

    for child in &node.children {
        match child {
            XmlChild::Element(e) => {
                elem_groups
                    .entry(e.name.0.clone())
                    .or_default()
                    .push(node_to_attribute_value(e));
            }
            XmlChild::Text(t) => text_parts.push(t.clone()),
        }
    }

    if node.attrs.is_empty() && elem_groups.is_empty() {
        return AttributeValue::String(text_parts.join(""));
    }

    let mut map: HashMap<String, AttributeValue> = HashMap::new();

    for ((qname, _), v) in &node.attrs {
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

pub(super) fn raw_gml_id(node: &RawNode) -> Option<String> {
    node.attrs
        .iter()
        .find(|((q, ns), _)| local_name(q) == "id" && ns == GML_NS)
        .map(|(_, v)| v.clone())
}

fn collect_ids(node: &Arc<RawNode>, source_url: &str, registry: &mut RawRegistry) {
    if let Some(id) = raw_gml_id(node) {
        registry.insert((source_url.to_string(), id), Arc::clone(node));
    }
    for child in &node.children {
        if let RawChild::Element(e) = child {
            collect_ids(e, source_url, registry);
        }
    }
}

fn href_to_key(href: &str, base: &Url) -> Option<RawNodeKey> {
    if let Some(frag) = href.strip_prefix('#') {
        Some((base.as_str().to_string(), frag.to_string()))
    } else if let Some((file, frag)) = href.split_once('#') {
        base.join(file)
            .ok()
            .map(|u| (u.to_string(), frag.to_string()))
    } else {
        tracing::warn!(href, "citygml3: unsupported xlink:href format, skipped");
        None
    }
}

enum OwnedEvent {
    Start {
        name: QName,
        attrs: Vec<(QName, String)>,
    },
    End,
    Empty {
        name: QName,
        attrs: Vec<(QName, String)>,
    },
    Text(String),
    Eof,
    Other,
}

fn next_event<R: BufRead>(
    reader: &mut NsReader<R>,
    buf: &mut Vec<u8>,
) -> Result<OwnedEvent, ParseError> {
    let (ns_result, event) = reader
        .read_resolved_event_into(buf)
        .map_err(ParseError::Xml)?;
    let elem_ns = match ns_result {
        ResolveResult::Bound(ns) => std::str::from_utf8(ns.into_inner())
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    };
    match event {
        Event::Start(e) => Ok(OwnedEvent::Start {
            name: (parse_qname(e.name().as_ref()), elem_ns),
            attrs: extract_attrs(&e, reader),
        }),
        Event::End(_) => Ok(OwnedEvent::End),
        Event::Empty(e) => Ok(OwnedEvent::Empty {
            name: (parse_qname(e.name().as_ref()), elem_ns),
            attrs: extract_attrs(&e, reader),
        }),
        Event::Text(t) => Ok(OwnedEvent::Text(
            t.unescape().map_err(ParseError::Xml)?.to_string(),
        )),
        Event::CData(c) => Ok(OwnedEvent::Text(
            std::str::from_utf8(&c).unwrap_or("").to_string(),
        )),
        Event::Eof => Ok(OwnedEvent::Eof),
        _ => Ok(OwnedEvent::Other),
    }
}

fn parse_element<R: BufRead>(
    reader: &mut NsReader<R>,
    buf: &mut Vec<u8>,
    name: QName,
    attrs: Vec<(QName, String)>,
    source_url: &Url,
) -> Result<RawNode, ParseError> {
    let href = xlink_href_attr(&attrs)
        .and_then(|href| href_to_key(href, source_url))
        .map(|key| {
            let filtered = attrs
                .iter()
                .filter(|((q, ns), _)| !(local_name(q) == "href" && ns == XLINK_NS))
                .cloned()
                .collect::<Vec<_>>();
            (key, filtered)
        });
    let mut children = Vec::new();

    loop {
        match next_event(reader, buf)? {
            OwnedEvent::Start {
                name: cn,
                attrs: ca,
            } => {
                let child = parse_element(reader, buf, cn, ca, source_url)?;
                children.push(RawChild::Element(Arc::new(child)));
            }
            OwnedEvent::Empty {
                name: cn,
                attrs: ca,
            } => {
                if let Some(href) = xlink_href_attr(&ca) {
                    if let Some(key) = href_to_key(href, source_url) {
                        let filtered: Vec<(QName, String)> = ca
                            .into_iter()
                            .filter(|((q, ns), _)| !(local_name(q) == "href" && ns == XLINK_NS))
                            .collect();
                        children.push(RawChild::Element(Arc::new(RawNode {
                            name: cn,
                            attrs: filtered,
                            children: vec![RawChild::Ref(key)],
                        })));
                    } else {
                        children.push(RawChild::Element(Arc::new(RawNode {
                            name: cn,
                            attrs: ca,
                            children: Vec::new(),
                        })));
                    }
                } else {
                    children.push(RawChild::Element(Arc::new(RawNode {
                        name: cn,
                        attrs: ca,
                        children: Vec::new(),
                    })));
                }
            }
            OwnedEvent::End | OwnedEvent::Eof => break,
            OwnedEvent::Text(t) if !t.trim().is_empty() => {
                children.push(RawChild::Text(t));
            }
            _ => {}
        }
    }

    if let Some((key, filtered_attrs)) = href {
        if !children.is_empty() {
            tracing::warn!(
                element = name.0,
                id = key.1,
                "citygml3: xlink:href element had inline content; inline content ignored"
            );
        }
        return Ok(RawNode {
            name,
            attrs: filtered_attrs,
            children: vec![RawChild::Ref(key)],
        });
    }

    Ok(RawNode {
        name,
        attrs,
        children,
    })
}

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

fn extract_attrs<R: BufRead>(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &NsReader<R>,
) -> Vec<(QName, String)> {
    e.attributes()
        .filter_map(|a| a.ok())
        .map(|a| {
            let qname_str = parse_qname(a.key.as_ref());
            let ns_uri = match reader.resolve_attribute(a.key).0 {
                ResolveResult::Bound(ns) => std::str::from_utf8(ns.into_inner())
                    .unwrap_or("")
                    .to_string(),
                _ => String::new(),
            };
            let v = a.unescape_value().unwrap_or_default().to_string();
            ((qname_str, ns_uri), v)
        })
        .collect()
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

fn local_name(name: &str) -> &str {
    utils_local_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::CitygmlFeatureExt;
    use url::Url;

    use crate::feature::reader::citygml3::utils::{XmlChild, XmlNode, GML_NS, XLINK_NS};

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn make_node(
        name: &str,
        ns: &str,
        attrs: Vec<(&str, &str, &str)>,
        children: Vec<XmlChild>,
    ) -> XmlNode {
        XmlNode {
            name: (name.to_string(), ns.to_string()),
            attrs: attrs
                .into_iter()
                .map(|(q, ns, v)| ((q.to_string(), ns.to_string()), v.to_string()))
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

    #[test]
    fn parse_errors_for_non_citygml() {
        let xml = b"<Foo/>";
        let mut reg = RawRegistry::new();
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();

        assert_eq!(raw.len(), 2);
        assert_eq!(raw_gml_id(&raw[0]), Some("bldg001".to_string()));
        assert_eq!(raw_gml_id(&raw[1]), Some("bldg002".to_string()));
        assert_eq!(raw[0].name.0, "bldg:Building");
    }

    #[test]
    fn parse_non_standard_gml_prefix() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:g="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building g:id="bldg001"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        assert_eq!(raw_gml_id(&raw[0]), Some("bldg001".to_string()));
        assert!(reg.contains_key(&(dummy_url().to_string(), "bldg001".to_string())));
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

        let mut reg = RawRegistry::new();
        parse(xml, &dummy_url(), &mut reg).unwrap();

        let url = dummy_url().to_string();
        assert!(reg.contains_key(&(url.clone(), "bldg001".to_string())));
        assert!(reg.contains_key(&(url, "part001".to_string())));
    }

    #[test]
    fn parse_cross_file_gml_id_collision() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building gml:id="shared001"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let url_a = Url::parse("file:///a.gml").unwrap();
        let url_b = Url::parse("file:///b.gml").unwrap();

        let mut reg = RawRegistry::new();
        parse(xml, &url_a, &mut reg).unwrap();
        parse(xml, &url_b, &mut reg).unwrap();

        assert_eq!(reg.len(), 2);
        assert!(reg.contains_key(&(url_a.to_string(), "shared001".to_string())));
        assert!(reg.contains_key(&(url_b.to_string(), "shared001".to_string())));
        let node_a = reg
            .get(&(url_a.to_string(), "shared001".to_string()))
            .unwrap();
        let node_b = reg
            .get(&(url_b.to_string(), "shared001".to_string()))
            .unwrap();
        assert!(!Arc::ptr_eq(node_a, node_b));
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        assert_eq!(raw.len(), 1);
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

        let mut reg = RawRegistry::new();
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

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        assert_eq!(raw.len(), 1);
    }

    #[test]
    fn node_to_attribute_value_pure_text() {
        let node = make_node("gml:name", GML_NS, vec![], vec![text("Building A")]);
        let av = node_to_attribute_value(&node);
        assert_eq!(av, AttributeValue::String("Building A".to_string()));
    }

    #[test]
    fn node_to_attribute_value_attrs_become_map() {
        let node = make_node(
            "gml:name",
            GML_NS,
            vec![("gml:id", GML_NS, "n1")],
            vec![text("foo")],
        );
        let av = node_to_attribute_value(&node);
        let AttributeValue::Map(map) = av else {
            panic!("expected Map");
        };
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
        let node = make_node(
            "bldg:Building",
            "",
            vec![("g:id", GML_NS, "bldg001")],
            vec![],
        );
        let AttributeValue::Map(map) = node_to_attribute_value(&node) else {
            panic!("expected Map");
        };
        assert_eq!(
            map.get("@g:id"),
            Some(&AttributeValue::String("bldg001".to_string()))
        );
    }

    #[test]
    fn parse_unescapes_attribute_values() {
        let xml = br#"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001" codeSpace="A &amp; B &lt;test&gt;"/>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();

        assert_eq!(
            raw[0]
                .attrs
                .iter()
                .find(|((q, _), _)| q == "codeSpace")
                .map(|(_, v)| v.as_str()),
            Some("A & B <test>")
        );
    }

    #[test]
    fn node_to_attribute_value_repeated_children_become_array() {
        let node = make_node(
            "parent",
            "",
            vec![],
            vec![
                elem(make_node("item", "", vec![], vec![text("a")])),
                elem(make_node("item", "", vec![], vec![text("b")])),
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
            "",
            vec![],
            vec![elem(make_node("item", "", vec![], vec![text("only")]))],
        );
        let AttributeValue::Map(map) = node_to_attribute_value(&node) else {
            panic!("expected Map");
        };
        assert!(matches!(map.get("item"), Some(AttributeValue::String(_))));
    }

    #[test]
    fn to_feature_sets_feature_type_and_id() {
        let node = make_node(
            "bldg:Building",
            "",
            vec![("gml:id", GML_NS, "bldg001")],
            vec![],
        );
        let feature = to_feature(&node);

        assert_eq!(feature.feature_type(), Some("bldg:Building".to_string()));
        assert_eq!(feature.feature_id(), Some("bldg001".to_string()));
    }

    #[test]
    fn resolve_xlink_href_becomes_element_child() {
        use crate::feature::reader::citygml3::xlink;

        let xml = br##"
<core:CityModel
  xmlns:core="http://www.opengis.net/citygml/3.0"
  xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
  xmlns:gml="http://www.opengis.net/gml/3.2"
  xmlns:xlink="http://www.w3.org/1999/xlink">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg001">
      <bldg:lod2Solid>
        <gml:Solid>
          <gml:exterior>
            <gml:Shell>
              <gml:surfaceMember xlink:href="#poly001"/>
            </gml:Shell>
          </gml:exterior>
        </gml:Solid>
      </bldg:lod2Solid>
      <gml:surfaceMember>
        <gml:Polygon gml:id="poly001"/>
      </gml:surfaceMember>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"##;

        let mut reg = RawRegistry::new();
        let raw = parse(xml, &dummy_url(), &mut reg).unwrap();
        let tlf = xlink::resolve(raw.into_iter().next().unwrap(), &reg);

        let building = &tlf;
        let lod2 = match &building.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        let solid = match &lod2.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        let exterior = match &solid.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        let shell = match &exterior.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };
        let surface_member = match &shell.children[0] {
            XmlChild::Element(e) => e,
            _ => panic!(),
        };

        assert_eq!(local_name(&surface_member.name.0), "surfaceMember");
        match &surface_member.children[0] {
            XmlChild::Element(arc) => assert_eq!(local_name(&arc.name.0), "Polygon"),
            _ => panic!("expected Element"),
        }
        assert!(!surface_member
            .attrs
            .iter()
            .any(|((q, ns), _)| local_name(q) == "href" && ns == XLINK_NS));
    }
}

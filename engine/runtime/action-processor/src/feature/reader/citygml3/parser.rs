use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;

use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};
use url::Url;

use super::utils::{gml_id_attr, qname, IdRegistry, XmlChild, XmlNode};

#[derive(Debug)]
pub struct TopLevelFeature {
    pub gml_id: Option<String>,
    pub feature_type: String,
    pub node: Arc<XmlNode>,
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

    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, .. } if local_name(&name) == "CityModel" => break,
            OwnedEvent::Eof => return Err(ParseError::NoCityModel),
            _ => {}
        }
    }

    let mut features = Vec::new();

    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, attrs } => {
                let ln = local_name(&name);
                if ln == "cityObjectMember" || ln == "featureMember" {
                    let member = parse_element(&mut reader, &mut buf, name, attrs)?;
                    if let Some(XmlChild::Element(feature_node)) = member.children.first() {
                        let feature_node = Arc::clone(feature_node);
                        collect_ids(&feature_node, source_url.as_str(), id_registry);
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
                    // gml:boundedBy, metadata, and other non-feature CityModel children
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

pub fn to_feature(tlf: &TopLevelFeature, resolved: &XmlNode) -> Feature {
    let content = node_to_attribute_value(resolved);
    build_feature(&tlf.feature_type, tlf.gml_id.as_deref(), content)
}

/// Mapping rules: pure-text nodes → `String`; otherwise → `Map` where XML
/// attributes become `"@qname"` keys, text content uses `"$"`, and repeated
/// sibling tags collapse into an `Array`.
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

// Owned so `next_event` can return data without lifetime conflicts across recursive `parse_element` calls.
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
                let has_xlinks = super::utils::xlink_href_attr(&ca).is_some();
                children.push(XmlChild::Element(Arc::new(XmlNode {
                    name: cn,
                    attrs: ca,
                    children: Vec::new(),
                    has_xlinks,
                })));
            }
            OwnedEvent::End | OwnedEvent::Eof => break,
            OwnedEvent::Text(t) if !t.is_empty() => {
                children.push(XmlChild::Text(t));
            }
            _ => {}
        }
    }

    let has_xlinks = super::utils::xlink_href_attr(&attrs).is_some()
        || children.iter().any(|c| match c {
            XmlChild::Element(e) => e.has_xlinks,
            XmlChild::Text(_) => false,
        });

    Ok(XmlNode {
        name,
        attrs,
        children,
        has_xlinks,
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

fn collect_ids(node: &Arc<XmlNode>, source_url: &str, registry: &mut IdRegistry) {
    if let Some(id) = gml_id_attr(node) {
        registry.insert((source_url.to_string(), id), Arc::clone(node));
    }
    for child in &node.children {
        if let XmlChild::Element(e) = child {
            collect_ids(e, source_url, registry);
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

fn local_name(name: &str) -> &str {
    super::utils::local_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::CitygmlFeatureExt;
    use url::Url;

    use crate::feature::reader::citygml3::utils::{
        xlink_href_attr, IdRegistry, XmlChild, XmlNode, GML_NS,
    };
    use crate::feature::reader::citygml3::xlink::resolve_xlinks;

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn make_node(name: &str, attrs: Vec<(&str, &str, &str)>, children: Vec<XmlChild>) -> XmlNode {
        let owned_attrs: Vec<(String, String, String)> = attrs
            .into_iter()
            .map(|(q, ns, v)| (q.to_string(), ns.to_string(), v.to_string()))
            .collect();
        let has_xlinks = xlink_href_attr(&owned_attrs).is_some()
            || children.iter().any(|c| match c {
                XmlChild::Element(e) => e.has_xlinks,
                XmlChild::Text(_) => false,
            });
        XmlNode {
            name: name.to_string(),
            attrs: owned_attrs,
            children,
            has_xlinks,
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

        let mut reg = IdRegistry::new();
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

        let mut reg = IdRegistry::new();
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
        let resolved = resolve_xlinks(&tlf.node, &dummy_url(), &reg);
        let feature = to_feature(&tlf, &resolved);

        assert_eq!(feature.feature_type(), Some("bldg:Building".to_string()));
        assert_eq!(feature.feature_id(), Some("bldg001".to_string()));
    }
}

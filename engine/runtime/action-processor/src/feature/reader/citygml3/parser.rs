use std::collections::{hash_map::Entry, HashMap};
use std::io::BufRead;
use std::sync::Arc;

use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};
use url::Url;

use super::utils::{
    gml_id_attr, local_name as utils_local_name, xlink_href_attr, NamespaceRegistry, NsId, QName,
    XmlChild, XmlNode, EMPTY_NS_ID, GML_NS_ID, XLINK_NS_ID,
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
    #[error("Malformed XML: {0}")]
    Malformed(String),
    #[error("No CityModel root element found")]
    NoCityModel,
    #[error("Unexpected end of file inside CityModel")]
    UnexpectedEof,
}

/// First pass parser which builds gml:id and namespace lookups
/// Call `parse()` once per file, then `finish()` to hand off the raw state for xlink resolution.
pub(super) struct Parser {
    raw_registry: RawRegistry,
    pub(super) ns_registry: NamespaceRegistry,
    pending: Vec<Arc<RawNode>>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("pending", &self.pending.len())
            .field("raw_registry", &self.raw_registry.len())
            .finish_non_exhaustive()
    }
}

impl Parser {
    pub(super) fn new() -> Self {
        Self {
            raw_registry: RawRegistry::new(),
            ns_registry: NamespaceRegistry::new(),
            pending: Vec::new(),
        }
    }

    pub(super) fn parse(&mut self, source: &[u8], source_url: &Url) -> Result<(), ParseError> {
        let src = std::str::from_utf8(source)
            .map_err(|e| ParseError::Encoding(format!("Non-UTF-8 content: {e}")))?;
        let mut reader = NsReader::from_str(src);
        let mut buf = Vec::new();

        loop {
            match next_event(&mut reader, &mut buf, &mut self.ns_registry)? {
                OwnedEvent::Start { name, .. } if local_name(&name.0) == "CityModel" => break,
                OwnedEvent::Eof => return Err(ParseError::NoCityModel),
                _ => {}
            }
        }

        loop {
            match next_event(&mut reader, &mut buf, &mut self.ns_registry)? {
                OwnedEvent::Start { name, attrs } => {
                    let ln = local_name(&name.0);
                    if ln == "cityObjectMember" || ln == "featureMember" {
                        let member = parse_element(
                            &mut reader,
                            &mut buf,
                            name,
                            attrs,
                            source_url,
                            &mut self.ns_registry,
                        )?;
                        if let Some(feature_node) =
                            member.children.iter().find_map(|child| match child {
                                RawChild::Element(node) => Some(Arc::clone(node)),
                                _ => None,
                            })
                        {
                            collect_ids(&feature_node, source_url.as_str(), &mut self.raw_registry);
                            self.pending.push(feature_node);
                        } else {
                            tracing::warn!(
                                "citygml3: empty cityObjectMember/featureMember, skipped"
                            );
                        }
                    } else {
                        skip_element(&mut reader, &mut buf, &mut self.ns_registry)?;
                    }
                }
                OwnedEvent::End => break,
                OwnedEvent::Eof => return Err(ParseError::UnexpectedEof),
                _ => {}
            }
        }

        Ok(())
    }

    /// Consume the parser and return raw state for xlink resolution and downstream processing.
    /// Caller is responsible for running `xlink::resolve(pending, &raw_registry)`.
    pub(super) fn finish(self) -> (Vec<Arc<RawNode>>, RawRegistry, NamespaceRegistry) {
        (self.pending, self.raw_registry, self.ns_registry)
    }
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

    // currently unordered because AttributeValue::Map is unordered
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
        .find(|((q, ns), _)| local_name(q) == "id" && *ns == GML_NS_ID)
        .map(|(_, v)| v.clone())
}

fn collect_ids(node: &Arc<RawNode>, source_url: &str, registry: &mut RawRegistry) {
    if let Some(id) = raw_gml_id(node) {
        let key = (source_url.to_string(), id.clone());
        if let Entry::Vacant(entry) = registry.entry(key) {
            entry.insert(Arc::clone(node));
        } else {
            tracing::error!(
                id,
                source_url,
                "citygml3: duplicate gml:id encountered; keeping first definition and skipping duplicate"
            );
        }
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
    ns_reg: &mut NamespaceRegistry,
) -> Result<OwnedEvent, ParseError> {
    let (ns_result, event) = reader
        .read_resolved_event_into(buf)
        .map_err(ParseError::Xml)?;
    let elem_ns_id = match ns_result {
        ResolveResult::Bound(ns) => intern_ns(ns.into_inner(), ns_reg)?,
        _ => EMPTY_NS_ID,
    };
    match event {
        Event::Start(e) => Ok(OwnedEvent::Start {
            name: (decode_utf8(e.name().as_ref(), "element name")?, elem_ns_id),
            attrs: extract_attrs(&e, reader, ns_reg)?,
        }),
        Event::End(_) => Ok(OwnedEvent::End),
        Event::Empty(e) => Ok(OwnedEvent::Empty {
            name: (decode_utf8(e.name().as_ref(), "element name")?, elem_ns_id),
            attrs: extract_attrs(&e, reader, ns_reg)?,
        }),
        Event::Text(t) => Ok(OwnedEvent::Text(
            t.unescape().map_err(ParseError::Xml)?.to_string(),
        )),
        Event::CData(c) => Ok(OwnedEvent::Text(decode_utf8(&c, "CDATA content")?)),
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
    ns_reg: &mut NamespaceRegistry,
) -> Result<RawNode, ParseError> {
    let href = xlink_href_attr(&attrs)
        .and_then(|href| href_to_key(href, source_url))
        .map(|key| {
            let filtered = attrs
                .iter()
                .filter(|((q, ns), _)| !(local_name(q) == "href" && *ns == XLINK_NS_ID))
                .cloned()
                .collect::<Vec<_>>();
            (key, filtered)
        });
    let mut children = Vec::new();

    loop {
        match next_event(reader, buf, ns_reg)? {
            OwnedEvent::Start {
                name: cn,
                attrs: ca,
            } => {
                let child = parse_element(reader, buf, cn, ca, source_url, ns_reg)?;
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
                            .filter(|((q, ns), _)| !(local_name(q) == "href" && *ns == XLINK_NS_ID))
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
            OwnedEvent::End => break,
            OwnedEvent::Eof => return Err(ParseError::UnexpectedEof),
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

fn skip_element<R: BufRead>(
    reader: &mut NsReader<R>,
    buf: &mut Vec<u8>,
    ns_reg: &mut NamespaceRegistry,
) -> Result<(), ParseError> {
    let mut depth: usize = 1;
    loop {
        match next_event(reader, buf, ns_reg)? {
            OwnedEvent::Start { .. } => depth += 1,
            OwnedEvent::End => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            OwnedEvent::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok(())
}

fn extract_attrs<R: BufRead>(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &NsReader<R>,
    ns_reg: &mut NamespaceRegistry,
) -> Result<Vec<(QName, String)>, ParseError> {
    e.attributes()
        .map(|a| {
            let a = a.map_err(|err| ParseError::Malformed(format!("invalid attribute: {err}")))?;
            let qname_str = decode_utf8(a.key.as_ref(), "attribute name")?;
            let ns_id = match reader.resolve_attribute(a.key).0 {
                ResolveResult::Bound(ns) => intern_ns(ns.into_inner(), ns_reg)?,
                _ => EMPTY_NS_ID,
            };
            let v = a
                .unescape_value()
                .map_err(|err| ParseError::Malformed(format!("invalid attribute value: {err}")))?
                .to_string();
            Ok(((qname_str, ns_id), v))
        })
        .collect()
}

fn intern_ns(bytes: &[u8], ns_reg: &mut NamespaceRegistry) -> Result<NsId, ParseError> {
    let s = std::str::from_utf8(bytes)
        .map_err(|e| ParseError::Encoding(format!("invalid UTF-8 in namespace URI: {e}")))?;
    Ok(ns_reg.intern(s))
}

fn decode_utf8(bytes: &[u8], context: &str) -> Result<String, ParseError> {
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|err| ParseError::Encoding(format!("invalid UTF-8 in {context}: {err}")))
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

    use crate::feature::reader::citygml3::utils::{XmlChild, XmlNode, EMPTY_NS_ID, GML_NS_ID};

    fn dummy_url() -> Url {
        Url::parse("file:///test.gml").unwrap()
    }

    fn make_node(
        name: &str,
        ns: NsId,
        attrs: Vec<(&str, NsId, &str)>,
        children: Vec<XmlChild>,
    ) -> XmlNode {
        XmlNode {
            name: (name.to_string(), ns),
            attrs: attrs
                .into_iter()
                .map(|(q, ns, v)| ((q.to_string(), ns), v.to_string()))
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

    fn parse_test(xml: &[u8]) -> Result<(), ParseError> {
        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url())
    }

    #[test]
    fn parse_errors_for_non_citygml() {
        assert!(matches!(
            parse_test(b"<Foo/>"),
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

        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url()).unwrap();
        let (pending, _, _) = parser.finish();

        assert_eq!(pending.len(), 2);
        assert_eq!(raw_gml_id(&pending[0]), Some("bldg001".to_string()));
        assert_eq!(raw_gml_id(&pending[1]), Some("bldg002".to_string()));
        assert_eq!(pending[0].name.0, "bldg:Building");
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

        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url()).unwrap();
        let (pending, raw_reg, _) = parser.finish();
        assert_eq!(raw_gml_id(&pending[0]), Some("bldg001".to_string()));
        assert!(raw_reg.contains_key(&(dummy_url().to_string(), "bldg001".to_string())));
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

        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url()).unwrap();
        let (_, raw_reg, _) = parser.finish();

        let url = dummy_url().to_string();
        assert!(raw_reg.contains_key(&(url.clone(), "bldg001".to_string())));
        assert!(raw_reg.contains_key(&(url, "part001".to_string())));
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

        let mut parser = Parser::new();
        parser.parse(xml, &url_a).unwrap();
        parser.parse(xml, &url_b).unwrap();
        let (_, raw_reg, _) = parser.finish();

        assert_eq!(raw_reg.len(), 2);
        assert!(raw_reg.contains_key(&(url_a.to_string(), "shared001".to_string())));
        assert!(raw_reg.contains_key(&(url_b.to_string(), "shared001".to_string())));
        let node_a = raw_reg
            .get(&(url_a.to_string(), "shared001".to_string()))
            .unwrap();
        let node_b = raw_reg
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

        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url()).unwrap();
        let (pending, _, _) = parser.finish();
        assert_eq!(pending.len(), 1);
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

        assert!(matches!(parse_test(xml), Err(ParseError::UnexpectedEof)));
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

        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url()).unwrap();
        let (pending, _, _) = parser.finish();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn node_to_attribute_value_pure_text() {
        let node = make_node("gml:name", GML_NS_ID, vec![], vec![text("Building A")]);
        let av = node_to_attribute_value(&node);
        assert_eq!(av, AttributeValue::String("Building A".to_string()));
    }

    #[test]
    fn node_to_attribute_value_attrs_become_map() {
        let node = make_node(
            "gml:name",
            GML_NS_ID,
            vec![("gml:id", GML_NS_ID, "n1")],
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
            EMPTY_NS_ID,
            vec![("g:id", GML_NS_ID, "bldg001")],
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

        let mut parser = Parser::new();
        parser.parse(xml, &dummy_url()).unwrap();
        let (pending, _, _) = parser.finish();

        assert_eq!(
            pending[0]
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
            EMPTY_NS_ID,
            vec![],
            vec![
                elem(make_node("item", EMPTY_NS_ID, vec![], vec![text("a")])),
                elem(make_node("item", EMPTY_NS_ID, vec![], vec![text("b")])),
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
            EMPTY_NS_ID,
            vec![],
            vec![elem(make_node(
                "item",
                EMPTY_NS_ID,
                vec![],
                vec![text("only")],
            ))],
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
            EMPTY_NS_ID,
            vec![("gml:id", GML_NS_ID, "bldg001")],
            vec![],
        );
        let feature = to_feature(&node);
        assert_eq!(feature.feature_type(), Some("bldg:Building".to_string()));
        assert_eq!(feature.feature_id(), Some("bldg001".to_string()));
    }
}

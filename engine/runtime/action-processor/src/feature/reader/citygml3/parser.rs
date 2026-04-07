//! Schema-agnostic CityGML 3 parser.
//!
//! This module has no dependency on the processor framework; it can be used
//! and tested in isolation.
//!
//! # Design
//!
//! Parsing is two-phase:
//!
//! 1. **`parse`** — stream the document, build an [`XmlNode`] tree per top-level
//!    feature, register every `gml:id`-bearing node into the caller-supplied
//!    [`IdRegistry`].  The registry is shared across files so cross-file
//!    `xlink:href` targets can be resolved in phase 2.
//!
//! 2. **`to_feature`** — once all files have been parsed, call this for each
//!    buffered [`TopLevelFeature`].  It resolves `xlink:href` references using
//!    the completed registry, converts the tree to [`AttributeValue`], and
//!    returns an engine [`Feature`].
//!
//! # Extensibility notes
//!
//! * **Geometry extraction** — deferred.  The raw [`XmlNode`] tree is kept on
//!   [`TopLevelFeature`] so a future extractor can walk it directly.
//!
//! * **Subfeature extraction** — deferred.  Walk [`XmlNode`] children looking
//!   for elements with `gml:id` that are CityGML feature types; each becomes
//!   its own call to `build_feature`.  The [`TopLevelFeature`] parent id should
//!   be passed down for the relation link.
//!
//! * **Flat vs. nested attribute layout** — isolated in [`build_feature`].
//!   Changing the layout only requires editing that one function.
//!
//! * **Codelist resolution** — hook point: extend [`to_feature`] (or add a
//!   post-processing step) to substitute codelist values before calling
//!   [`build_feature`].
//!
//! * **Cross-file xlink** — pass the same [`IdRegistry`] reference across all
//!   `parse` calls before invoking `to_feature`.

use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;

use indexmap::IndexMap;
use quick_xml::events::Event;
use quick_xml::Reader;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};
use url::Url;

// ------------------------------------------------------------------
// Public types
// ------------------------------------------------------------------

/// A generic, schema-agnostic XML node.
///
/// Retains the full qualified name (prefix + local) so downstream code can
/// distinguish namespaces without a namespace resolver.
#[derive(Debug, Clone)]
pub struct XmlNode {
    /// Qualified element name, e.g. `"bldg:Building"` or `"gml:Polygon"`.
    pub name: String,
    /// XML attributes in document order as `(qualified-name, value)` pairs.
    pub attrs: Vec<(String, String)>,
    /// Child content in document order.
    pub children: Vec<XmlChild>,
}

#[derive(Debug, Clone)]
pub enum XmlChild {
    Element(Arc<XmlNode>),
    Text(String),
}

/// Maps `gml:id` values → owning nodes.
///
/// Kept on the processor and shared across all `parse` calls so that
/// cross-file `xlink:href` resolution is available when [`to_feature`] runs.
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
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

/// Parse a CityGML 3 document.
///
/// Returns one [`TopLevelFeature`] per `cityObjectMember` / `featureMember`
/// child of the root `CityModel`.  Every `gml:id`-bearing node found in
/// those subtrees is registered into `id_registry`.
///
/// Call this once per file, accumulating into the same `id_registry`, before
/// calling [`to_feature`].
pub fn parse(
    source: &[u8],
    source_url: &Url,
    id_registry: &mut IdRegistry,
) -> Result<Vec<TopLevelFeature>, ParseError> {
    let src = std::str::from_utf8(source)
        .map_err(|e| ParseError::Encoding(format!("Non-UTF-8 content: {e}")))?;
    let mut reader = Reader::from_str(src);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    // Advance past the XML declaration and any preamble to the CityModel root.
    loop {
        match next_event(&mut reader, &mut buf)? {
            OwnedEvent::Start { name, .. } if local_name(&name) == "CityModel" => break,
            OwnedEvent::Eof => return Ok(Vec::new()),
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
                    }
                } else {
                    // Other CityModel children (gml:boundedBy, metadata, …) — skip.
                    skip_element(&mut reader, &mut buf)?;
                }
            }
            OwnedEvent::End | OwnedEvent::Eof => break,
            _ => {}
        }
    }

    Ok(features)
}

/// Convert a [`TopLevelFeature`] to an engine [`Feature`].
///
/// Resolves `xlink:href` references using `id_registry`, then converts the
/// node tree to [`AttributeValue`] and builds the output [`Feature`].
///
/// Call after all files have been parsed (so `id_registry` is complete).
pub fn to_feature(tlf: &TopLevelFeature, id_registry: &IdRegistry) -> Feature {
    let resolved = resolve_xlinks(&tlf.node, id_registry);
    let content = node_to_attribute_value(&resolved);
    build_feature(&tlf.feature_type, tlf.gml_id.as_deref(), content)
}

// ------------------------------------------------------------------
// XLink resolution
// ------------------------------------------------------------------

/// Recursively walk `node`, replacing elements that carry only an
/// `xlink:href` attribute and no children with the referenced node from
/// `registry`.
///
/// - Same-file and cross-file references are handled identically.
/// - Unresolvable references (target absent from registry) are left in place.
/// - Resolved targets are not themselves re-resolved (single pass); wrap in a
///   second call to handle cascaded references if needed.
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
                        .filter(|(k, _)| k != "xlink:href")
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

// ------------------------------------------------------------------
// XmlNode → AttributeValue
// ------------------------------------------------------------------

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

    for (k, v) in &node.attrs {
        map.insert(format!("@{k}"), AttributeValue::String(v.clone()));
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

// ------------------------------------------------------------------
// Feature construction — single point for layout decisions
// ------------------------------------------------------------------

/// Build the engine [`Feature`] from a feature type, optional gml:id, and
/// the converted attribute content.
///
/// **Layout is controlled here only.**
/// Current layout: merge content map entries flat into Feature attributes.
/// To switch to a single-key layout, replace the `if let` block with:
/// ```ignore
/// attrs.insert(Attribute::new("content"), content);
/// ```
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

// ------------------------------------------------------------------
// Internal: owned event abstraction
// ------------------------------------------------------------------

/// Owned, lifetime-free view of a quick-xml event.
///
/// Extracting owned data immediately inside `next_event` avoids the lifetime
/// conflict that would arise if we tried to hold a borrowed `Event<'_>` across
/// a recursive call to `parse_element` (which also mutably borrows the reader).
enum OwnedEvent {
    Start {
        name: String,
        attrs: Vec<(String, String)>,
    },
    End,
    Empty {
        name: String,
        attrs: Vec<(String, String)>,
    },
    Text(String),
    Eof,
    Other,
}

fn next_event<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<OwnedEvent, ParseError> {
    match reader.read_event_into(buf).map_err(ParseError::Xml)? {
        Event::Start(e) => Ok(OwnedEvent::Start {
            name: qname(e.name().as_ref()),
            attrs: extract_attrs(&e),
        }),
        Event::End(_) => Ok(OwnedEvent::End),
        Event::Empty(e) => Ok(OwnedEvent::Empty {
            name: qname(e.name().as_ref()),
            attrs: extract_attrs(&e),
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

// ------------------------------------------------------------------
// Internal: recursive element parser
// ------------------------------------------------------------------

/// Parse one element and its entire subtree.
///
/// `name` and `attrs` are the already-extracted data from the opening tag.
/// The reader is positioned just after that opening tag; `parse_element`
/// reads until (and including) the matching closing tag.
fn parse_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    name: String,
    attrs: Vec<(String, String)>,
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
fn skip_element<R: BufRead>(reader: &mut Reader<R>, buf: &mut Vec<u8>) -> Result<(), ParseError> {
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

// ------------------------------------------------------------------
// Internal: id collection
// ------------------------------------------------------------------

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

// ------------------------------------------------------------------
// Internal: attribute helpers
// ------------------------------------------------------------------

fn extract_attrs(e: &quick_xml::events::BytesStart<'_>) -> Vec<(String, String)> {
    e.attributes()
        .filter_map(|a| a.ok())
        .map(|a| {
            let k = qname(a.key.as_ref());
            let v = std::str::from_utf8(a.value.as_ref())
                .unwrap_or("")
                .to_string();
            (k, v)
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
        .find(|(k, _)| k == "gml:id")
        .map(|(_, v)| v.clone())
}

fn xlink_href_attr(attrs: &[(String, String)]) -> Option<&str> {
    attrs
        .iter()
        .find(|(k, _)| k == "xlink:href")
        .map(|(_, v)| v.as_str())
}

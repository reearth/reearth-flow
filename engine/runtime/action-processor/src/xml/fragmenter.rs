use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
    time::Instant,
};

use reearth_flow_common::{str::to_hash, uri::Uri};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{Result, XmlProcessorError};

#[derive(Debug, Clone, Default)]
pub struct XmlFragmenterFactory;

impl ProcessorFactory for XmlFragmenterFactory {
    fn name(&self) -> &str {
        "XMLFragmenter"
    }

    fn description(&self) -> &str {
        "Fragments large XML documents into smaller pieces based on specified element patterns"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(XmlFragmenterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["XML"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: XmlFragmenterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(XmlProcessorError::FragmenterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let XmlFragmenterParam::Url { property } = &params;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let elements_to_match_ast = expr_engine
            .compile(property.elements_to_match.to_string().as_str())
            .map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to comple expr engine with {e:?}"
                ))
            })?;
        let elements_to_exclude_ast = expr_engine
            .compile(property.elements_to_exclude.to_string().as_str())
            .map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to comple expr engine with {e:?}"
                ))
            })?;
        let process = XmlFragmenter {
            global_params: with,
            params,
            elements_to_match_ast,
            elements_to_exclude_ast,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct XmlFragmenter {
    global_params: Option<HashMap<String, serde_json::Value>>,
    params: XmlFragmenterParam,
    elements_to_match_ast: rhai::AST,
    elements_to_exclude_ast: rhai::AST,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropertySchema {
    pub(super) elements_to_match: Expr,
    pub(super) elements_to_exclude: Expr,
    pub(super) attribute: Attribute,
}

/// # XMLFragmenter Parameters
///
/// Configuration for fragmenting XML documents into smaller pieces.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum XmlFragmenterParam {
    #[serde(rename = "url")]
    /// URL-based source configuration for XML fragmenting
    Url {
        #[serde(flatten)]
        property: PropertySchema,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct XmlFragment {
    pub(super) xml_id: String,
    pub(super) fragment: String,
    pub(super) matched_tag: String,
    pub(super) xml_parent_id: Option<String>,
}

impl XmlFragment {
    fn to_hashmap(fragment: XmlFragment) -> HashMap<Attribute, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            Attribute::new("xmlId"),
            AttributeValue::String(fragment.xml_id),
        );
        map.insert(
            Attribute::new("xmlFragment"),
            AttributeValue::String(fragment.fragment),
        );
        map.insert(
            Attribute::new("matchedTag"),
            AttributeValue::String(fragment.matched_tag),
        );
        let attribute = if let Some(xml_parent_id) = fragment.xml_parent_id {
            AttributeValue::String(xml_parent_id)
        } else {
            AttributeValue::Null
        };
        map.insert(Attribute::new("xmlParentId"), attribute);
        map
    }
}

impl Processor for XmlFragmenter {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match &self.params {
            XmlFragmenterParam::Url { property } => {
                send_xml_fragment(
                    &ctx,
                    fw,
                    &self.global_params,
                    &ctx.feature,
                    &property.attribute,
                    &self.elements_to_match_ast,
                    &self.elements_to_exclude_ast,
                )?;
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "XmlFragmenter"
    }
}

fn send_xml_fragment(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    global_params: &Option<HashMap<String, serde_json::Value>>,
    feature: &Feature,
    attribute: &Attribute,
    elements_to_match_ast: &rhai::AST,
    elements_to_exclude_ast: &rhai::AST,
) -> Result<()> {
    let t_total = Instant::now();

    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let expr_engine = Arc::clone(&ctx.expr_engine);

    let scope = feature.new_scope(expr_engine.clone(), global_params);
    let elements_to_match = scope
        .eval_ast::<rhai::Array>(elements_to_match_ast)
        .map_err(|e| {
            XmlProcessorError::Fragmenter(format!("Failed expr engine error with {e:?}"))
        })?;
    let elements_to_match = elements_to_match
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();
    if elements_to_match.is_empty() {
        return Ok(());
    }

    let elements_to_exclude = scope
        .eval_ast::<rhai::Array>(elements_to_exclude_ast)
        .map_err(|e| {
            XmlProcessorError::Fragmenter(format!("Failed expr engine error with {e:?}"))
        })?;
    let elements_to_exclude = elements_to_exclude
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();

    let url = match feature.get(attribute) {
        Some(AttributeValue::String(url)) => {
            Uri::from_str(url).map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?
        }
        _ => return Err(XmlProcessorError::Fragmenter("No url found".to_string())),
    };
    let storage = storage_resolver
        .resolve(&url)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let bytes = storage
        .get_sync(&url.path())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let raw_xml =
        std::str::from_utf8(&bytes).map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    if elements_to_match.is_empty() {
        return Ok(());
    }

    let excluded: HashSet<String> = elements_to_exclude.into_iter().collect();
    let mut match_tags = Vec::new();
    let mut seen = HashSet::new();
    for tag in elements_to_match {
        if excluded.contains(&tag) || !seen.insert(tag.clone()) {
            continue;
        }
        match_tags.push(tag);
    }
    if match_tags.is_empty() {
        return Ok(());
    }

    let t_gen = Instant::now();
    let result = generate_fragment_streaming(ctx, fw, feature, &url, raw_xml, &match_tags);
    tracing::debug!(
        target: "perf",
        total_ms = %t_total.elapsed().as_millis(),
        gen_ms = %t_gen.elapsed().as_millis(),
        "XmlFragmenter::send_xml_fragment END"
    );
    result
}

fn generate_fragment_streaming(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    uri: &Uri,
    raw_xml: &str,
    match_tags: &[String],
) -> Result<()> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let match_tags_set: HashSet<String> = match_tags.iter().cloned().collect();

    // Extract root namespace declarations for injection into fragments
    let root_ns = fastxml::namespace::extract_root_namespaces(raw_xml)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;

    let mut reader = Reader::from_str(raw_xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    // Stack-based parent tracking (replaces separate build_parent_id_map pass)
    let mut parent_stack: Vec<ElementInfo> = Vec::new();

    loop {
        let element_start_pos = reader.buffer_position();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name();
                let name = std::str::from_utf8(name_bytes.as_ref())
                    .unwrap_or("")
                    .to_string();

                if match_tags_set.contains(name.as_str()) {
                    let xml_id = compute_xml_id_from_start_event(uri, e, &name);

                    // Compute parent_id from the immediate parent element on the stack
                    let xml_parent_id = parent_stack
                        .last()
                        .map(|parent| compute_parent_id_from_stack(uri, parent));

                    // Collect existing xmlns prefixes on this element to avoid duplicates
                    let existing_ns: HashSet<String> = e
                        .attributes()
                        .flatten()
                        .filter_map(|attr| {
                            let key = std::str::from_utf8(attr.key.as_ref()).ok()?;
                            key.strip_prefix("xmlns:")
                                .map(|p| p.to_string())
                                .or_else(|| {
                                    if key == "xmlns" {
                                        Some(String::new())
                                    } else {
                                        None
                                    }
                                })
                        })
                        .collect();

                    // Track depth to find matching End tag (consumes entire subtree)
                    let mut depth = 1u32;
                    loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::Start(_)) => depth += 1,
                            Ok(Event::End(_)) => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            Ok(Event::Eof) => break,
                            _ => {}
                        }
                        buf.clear();
                    }
                    let element_end_pos = reader.buffer_position();

                    // Zero-copy: slice the original XML for this element
                    let raw_slice = &raw_xml[element_start_pos as usize..element_end_pos as usize];

                    // Build namespace injection string (only missing ones)
                    let ns_inject = build_ns_inject_string(&root_ns, &existing_ns);

                    // Inject namespaces into the opening tag
                    let fragment = if ns_inject.is_empty() {
                        raw_slice.to_string()
                    } else {
                        inject_ns_into_opening_tag(raw_slice, &ns_inject)
                    };

                    let frag = XmlFragment {
                        xml_id,
                        fragment,
                        matched_tag: name.clone(),
                        xml_parent_id,
                    };

                    let mut value =
                        Feature::new_with_attributes(feature.attributes.as_ref().clone());
                    XmlFragment::to_hashmap(frag)
                        .into_iter()
                        .for_each(|(k, v)| {
                            value.attributes_mut().insert(k, v);
                        });
                    fw.send(ctx.new_with_feature_and_port(value, DEFAULT_PORT.clone()));
                    // Matched element's subtree is fully consumed; do NOT push to parent_stack
                } else {
                    // Non-matched element: push to stack for parent lookups
                    let attrs: Vec<(String, String)> = e
                        .attributes()
                        .flatten()
                        .map(|attr| {
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .unwrap_or("")
                                .to_string();
                            let val = attr.unescape_value().unwrap_or_default().to_string();
                            (key, val)
                        })
                        .collect();
                    parent_stack.push(ElementInfo { qname: name, attrs });
                }
            }
            Ok(Event::End(_)) => {
                parent_stack.pop();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => {
                return Err(XmlProcessorError::Fragmenter(format!(
                    "XML parse error: {e}"
                )));
            }
        }
        buf.clear();
    }

    Ok(())
}

/// Element info stored on the parent tracking stack
struct ElementInfo {
    qname: String,
    attrs: Vec<(String, String)>,
}

/// Compute parent ID from stack entry (same logic as compute_parent_xml_id)
fn compute_parent_id_from_stack(uri: &Uri, parent: &ElementInfo) -> String {
    // Check for 'id' attribute
    for (key, val) in &parent.attrs {
        if key == "id" {
            return val.clone();
        }
    }
    // Check for 'gml:id' attribute
    for (key, val) in &parent.attrs {
        if key == "gml:id" {
            return val.clone();
        }
    }
    // Build hash from non-xmlns attributes (sorted by local name)
    let mut attrs: Vec<(String, String)> = parent
        .attrs
        .iter()
        .filter(|(k, _)| *k != "xmlns" && !k.starts_with("xmlns:"))
        .map(|(k, v)| {
            let local_name = k.split_once(':').map(|(_, local)| local).unwrap_or(k);
            (local_name.to_string(), v.clone())
        })
        .collect();
    attrs.sort_by(|a, b| a.0.cmp(&b.0));
    let key_values: Vec<String> = attrs.iter().map(|(k, v)| format!("{k}={v}")).collect();
    to_hash(format!("{}:{}[{}]", uri, parent.qname, key_values.join(",")).as_str())
}

/// Build a string of xmlns declarations to inject (only those not already present)
fn build_ns_inject_string(
    root_ns: &std::collections::HashMap<String, String>,
    existing_ns: &HashSet<String>,
) -> String {
    let mut ns_inject = String::new();
    for (prefix, uri_val) in root_ns {
        if !existing_ns.contains(prefix) {
            if prefix.is_empty() {
                ns_inject.push_str(&format!(" xmlns=\"{uri_val}\""));
            } else {
                ns_inject.push_str(&format!(" xmlns:{prefix}=\"{uri_val}\""));
            }
        }
    }
    ns_inject
}

/// Inject namespace declarations into the opening tag of an XML slice
fn inject_ns_into_opening_tag(raw_slice: &str, ns_inject: &str) -> String {
    // Find the first '>' which ends the opening tag
    if let Some(first_gt) = raw_slice.find('>') {
        let inject_pos = if first_gt > 0 && raw_slice.as_bytes()[first_gt - 1] == b'/' {
            first_gt - 1 // self-closing: insert before '/'
        } else {
            first_gt // insert before '>'
        };
        let mut fragment = String::with_capacity(raw_slice.len() + ns_inject.len());
        fragment.push_str(&raw_slice[..inject_pos]);
        fragment.push_str(ns_inject);
        fragment.push_str(&raw_slice[inject_pos..]);
        fragment
    } else {
        raw_slice.to_string()
    }
}

/// Compute xml_id from a quick_xml BytesStart event (same logic as compute_xml_id_from_editable_node)
fn compute_xml_id_from_start_event(
    uri: &Uri,
    e: &quick_xml::events::BytesStart,
    name: &str,
) -> String {
    // Check for 'id' attribute (bare or namespaced, e.g. "gml:id").
    // quick_xml exposes the raw attribute name including any prefix, so we treat any *:id
    // as an ID attribute to match the DOM-based get_attributes().get("id") behavior.
    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        if key == "id" || key.ends_with(":id") {
            return attr.unescape_value().unwrap_or_default().to_string();
        }
    }
    // Fall back to hash from attributes (sorted), excluding namespace declarations
    // (xmlns / xmlns:*) to match DOM-based get_attributes() which omits them.
    let mut key_values: Vec<String> = e
        .attributes()
        .flatten()
        .filter_map(|attr| {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            if key == "xmlns" || key.starts_with("xmlns:") {
                return None;
            }
            let value = attr.unescape_value().unwrap_or_default();
            Some(format!("{key}={value}"))
        })
        .collect();
    key_values.sort();
    to_hash(format!("{}:{}[{}]", uri, name, key_values.join(",")).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils;
    use reearth_flow_common::uri::Uri;
    use reearth_flow_runtime::forwarder::{NoopChannelForwarder, ProcessorChannelForwarder};
    use reearth_flow_types::{AttributeValue, Feature};

    /// Helper: call generate_fragment_streaming with in-memory XML and return
    /// the xmlFragment strings produced.
    fn extract_fragment_strings(raw_xml: &str, match_tags: &[&str]) -> Vec<String> {
        let feature = Feature::new_with_attributes(Default::default());
        let ctx = utils::create_default_execute_context(&feature);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let uri = Uri::from_str("file:///test.xml").unwrap();
        let tags: Vec<String> = match_tags.iter().map(|s| s.to_string()).collect();

        generate_fragment_streaming(&ctx, &fw, &feature, &uri, raw_xml, &tags)
            .expect("generate_fragment_streaming should succeed");

        match &fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let features = noop_fw.send_features.lock().unwrap();
                features
                    .iter()
                    .map(|f| match f.get(Attribute::new("xmlFragment")) {
                        Some(AttributeValue::String(s)) => s.clone(),
                        other => panic!("expected xmlFragment string, got: {other:?}"),
                    })
                    .collect()
            }
            _ => unreachable!(),
        }
    }

    /// Baseline: fragments extracted from BOM-free XML must be well-formed.
    #[test]
    fn test_fragment_content_without_bom() {
        let xml = "<root><child id=\"c1\"><data>hello</data></child></root>";
        let fragments = extract_fragment_strings(xml, &["child"]);

        assert_eq!(fragments.len(), 1);
        assert!(
            fragments[0].starts_with("<child"),
            "fragment should start with '<child', got: {}",
            fragments[0]
        );
        assert!(
            fragments[0].ends_with("</child>"),
            "fragment should end with '</child>', got: {}",
            fragments[0]
        );
    }

    /// BOM bug reproduction: when the input XML starts with UTF-8 BOM
    /// (\u{FEFF}), quick_xml's buffer_position() may be offset by 3 bytes,
    /// causing the raw_xml[start..end] slice to produce a corrupted fragment
    /// (shifted by the BOM width).
    #[test]
    fn test_fragment_content_with_bom() {
        let xml = "\u{FEFF}<root><child id=\"c1\"><data>hello</data></child></root>";
        let fragments = extract_fragment_strings(xml, &["child"]);

        assert_eq!(fragments.len(), 1, "should extract exactly one fragment");
        // The fragment must start with the opening tag, not with garbage bytes
        // caused by a 3-byte BOM offset.
        assert!(
            fragments[0].starts_with("<child"),
            "fragment should start with '<child', got: {}",
            fragments[0]
        );
        assert!(
            fragments[0].ends_with("</child>"),
            "fragment should end with '</child>', got: {}",
            fragments[0]
        );
    }
}

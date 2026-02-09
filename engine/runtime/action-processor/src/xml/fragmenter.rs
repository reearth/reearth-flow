use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

/// Get current process memory in MB.
/// macOS: phys_footprint (same as Activity Monitor / top MEM).
/// Other: sysinfo RSS fallback.
fn current_rss_mb() -> f64 {
    #[cfg(target_os = "macos")]
    {
        #[allow(non_camel_case_types)]
        #[repr(C)]
        struct task_vm_info {
            virtual_size: u64,
            region_count: i32,
            page_size: i32,
            resident_size: u64,
            resident_size_peak: u64,
            device: u64,
            device_peak: u64,
            internal: u64,
            internal_peak: u64,
            external: u64,
            external_peak: u64,
            reusable: u64,
            reusable_peak: u64,
            purgeable_volatile_pmap: u64,
            purgeable_volatile_resident: u64,
            purgeable_volatile_virtual: u64,
            compressed: u64,
            compressed_peak: u64,
            compressed_lifetime: u64,
            phys_footprint: u64,
        }
        const TASK_VM_INFO: u32 = 22;
        extern "C" {
            fn mach_task_self() -> u32;
            fn task_info(t: u32, f: u32, o: *mut i32, c: *mut u32) -> i32;
        }
        unsafe {
            let mut info: task_vm_info = std::mem::zeroed();
            let mut count =
                (std::mem::size_of::<task_vm_info>() / std::mem::size_of::<i32>()) as u32;
            let kr = task_info(
                mach_task_self(),
                TASK_VM_INFO,
                &mut info as *mut _ as *mut i32,
                &mut count,
            );
            if kr == 0 {
                return info.phys_footprint as f64 / 1024.0 / 1024.0;
            }
        }
        0.0
    }
    #[cfg(not(target_os = "macos"))]
    {
        use sysinfo::{Pid, ProcessesToUpdate, System};
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        sys.process(pid)
            .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
            .unwrap_or(0.0)
    }
}

static FRAG_PROCESS_COUNT: AtomicUsize = AtomicUsize::new(0);

use fastxml::transform::{EditableNode, StreamTransformer};
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
    let count = FRAG_PROCESS_COUNT.fetch_add(1, Ordering::Relaxed);
    let t_total = Instant::now();
    let rss_start = current_rss_mb();
    tracing::info!(
        "[PERF] XmlFragmenter::send_xml_fragment START | count={} | rss={:.1} MB",
        count,
        rss_start
    );

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
    let raw_xml = String::from_utf8(bytes.to_vec())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    tracing::info!(
        "[PERF] XmlFragmenter::send_xml_fragment file_read | count={} | xml_size={} bytes | rss={:.1} MB",
        count,
        raw_xml.len(),
        current_rss_mb()
    );
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

    let t_parent = Instant::now();
    let parent_map = build_parent_id_map(&raw_xml, &url, &match_tags)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    tracing::info!(
        "[PERF] XmlFragmenter::send_xml_fragment parent_map | count={} | elapsed={}ms | entries={} | rss={:.1} MB",
        count,
        t_parent.elapsed().as_millis(),
        parent_map.len(),
        current_rss_mb()
    );

    let t_gen = Instant::now();
    let result =
        generate_fragment_streaming(ctx, fw, feature, &url, &raw_xml, &parent_map, &match_tags);
    tracing::info!(
        "[PERF] XmlFragmenter::send_xml_fragment END | count={} | total={}ms gen={}ms | rss={:.1} MB (delta={:+.1})",
        count,
        t_total.elapsed().as_millis(),
        t_gen.elapsed().as_millis(),
        current_rss_mb(),
        current_rss_mb() - rss_start
    );
    result
}

fn generate_fragment_streaming(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    uri: &Uri,
    raw_xml: &str,
    parent_map: &HashMap<String, Option<String>>,
    match_tags: &[String],
) -> Result<()> {
    let stream_error: std::rc::Rc<RefCell<Option<XmlProcessorError>>> =
        std::rc::Rc::new(RefCell::new(None));

    let mut transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;

    for tag in match_tags {
        let xpath = format!("//{tag}");
        let ctx = ctx.clone();
        let fw = fw.clone();
        let feature = feature.clone();
        let uri = uri.clone();
        let tag = tag.clone();
        let stream_error = std::rc::Rc::clone(&stream_error);

        transformer = transformer.on(&xpath, move |node| {
            if node.qname() != tag {
                return;
            }
            if stream_error.borrow().is_some() {
                return;
            }

            match build_fragment_from_node(node, &uri, parent_map) {
                Ok(fragment) => {
                    let mut value =
                        Feature::new_with_attributes(feature.attributes.as_ref().clone());
                    XmlFragment::to_hashmap(fragment)
                        .into_iter()
                        .for_each(|(k, v)| {
                            value.attributes_mut().insert(k, v);
                        });
                    fw.send(ctx.new_with_feature_and_port(value, DEFAULT_PORT.clone()));
                }
                Err(err) => {
                    *stream_error.borrow_mut() = Some(err);
                }
            }
        });
    }

    let result = transformer.for_each();
    if let Err(err) = result {
        return Err(XmlProcessorError::Fragmenter(format!("{err:?}")));
    }

    if let Some(err) = stream_error.borrow_mut().take() {
        return Err(err);
    }

    Ok(())
}

fn build_fragment_from_node(
    node: &mut EditableNode,
    uri: &Uri,
    parent_map: &HashMap<String, Option<String>>,
) -> Result<XmlFragment> {
    let xml_id = compute_xml_id_from_editable_node(uri, node);
    let matched_tag = node.qname();
    let fragment = node
        .to_xml_with_namespaces()
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;

    let xml_parent_id = parent_map.get(&xml_id).cloned().flatten();

    Ok(XmlFragment {
        xml_id,
        fragment,
        matched_tag,
        xml_parent_id,
    })
}

fn build_parent_id_map(
    raw_xml: &str,
    uri: &Uri,
    match_tags: &[String],
) -> Result<HashMap<String, Option<String>>, XmlProcessorError> {
    let match_tags: HashSet<String> = match_tags.iter().cloned().collect();
    let map: RefCell<HashMap<String, Option<String>>> = RefCell::new(HashMap::new());
    let uri = uri.clone();

    let transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;

    transformer
        .on_with_context("//*", |node, ctx| {
            let qname = node.qname();
            if !match_tags.contains(&qname) {
                return;
            }

            let xml_id = compute_xml_id_from_editable_node(&uri, node);
            let parent_id = compute_parent_xml_id(&uri, ctx);

            map.borrow_mut().insert(xml_id, parent_id);
        })
        .for_each()
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;

    Ok(map.into_inner())
}

fn compute_parent_xml_id(uri: &Uri, ctx: &fastxml::transform::TransformContext) -> Option<String> {
    let parent = ctx.parent()?;

    // Check for 'id' attribute (handles both 'id' and 'gml:id')
    if let Some(id) = parent.attributes.get("id") {
        return Some(id.clone());
    }
    if let Some(id) = parent.attributes.get("gml:id") {
        return Some(id.clone());
    }

    // Build hash from attributes (excluding xmlns)
    let mut attrs: Vec<(String, String)> = parent
        .attributes
        .iter()
        .filter(|(k, _)| *k != "xmlns" && !k.starts_with("xmlns:"))
        .map(|(k, v)| {
            let local_name = k.split_once(':').map(|(_, local)| local).unwrap_or(k);
            (local_name.to_string(), v.clone())
        })
        .collect();
    attrs.sort_by(|a, b| a.0.cmp(&b.0));

    let key_values: Vec<String> = attrs.iter().map(|(k, v)| format!("{k}={v}")).collect();
    Some(to_hash(
        format!("{}:{}[{}]", uri, parent.qname, key_values.join(",")).as_str(),
    ))
}

fn compute_xml_id_from_editable_node(uri: &Uri, node: &EditableNode) -> String {
    if let Some(id) = node.get_attribute("id") {
        return id;
    }
    let tag = node.qname();
    let mut key_values = node
        .get_attributes()
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>();
    key_values.sort();
    to_hash(format!("{}:{}[{}]", uri, tag, key_values.join(",")).as_str())
}

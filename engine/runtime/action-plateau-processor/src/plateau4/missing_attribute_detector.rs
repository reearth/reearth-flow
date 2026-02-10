use crate::object_list::ObjectListMap;

use super::errors::PlateauProcessorError;
use fastxml::transform::StreamTransformer;
use once_cell::sync::Lazy;
use reearth_flow_common::process::current_rss_mb;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

static MAD_PROCESS_COUNT: AtomicUsize = AtomicUsize::new(0);

static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));
static REQUIRED_PORT: Lazy<Port> = Lazy::new(|| Port::new("required"));
static TARGET_PORT: Lazy<Port> = Lazy::new(|| Port::new("target"));
static DATA_QUALITY_C07_PORT: Lazy<Port> = Lazy::new(|| Port::new("dataQualityC07"));
static DATA_QUALITY_C08_PORT: Lazy<Port> = Lazy::new(|| Port::new("dataQualityC08"));

static FEATURE_TYPE_TO_PART_XPATH: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("bldg:Building", "bldg:consistsOfBuildingPart"),
        ("brid:Bridge", "brid:consistsOfBridgePart"),
        ("tun:Tunnel", "tun:consistsOfTunnelPart"),
        ("uro:UndergroundBuilding", "bldg:consistsOfBuildingPart"),
    ])
});

static LOD_TAG_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"lod([0-4])").expect("Failed to compile LOD tag pattern"));

#[derive(Debug, Clone, Default)]
pub struct MissingAttributeDetectorFactory;

impl ProcessorFactory for MissingAttributeDetectorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.MissingAttributeDetector"
    }

    fn description(&self) -> &str {
        "Detect missing attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MissingAttributeDetectorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            SUMMARY_PORT.clone(),
            REQUIRED_PORT.clone(),
            TARGET_PORT.clone(),
            DATA_QUALITY_C07_PORT.clone(),
            DATA_QUALITY_C08_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: MissingAttributeDetectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::MissingAttributeDetectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::MissingAttributeDetectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::MissingAttributeDetectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = MissingAttributeDetector {
            package_attribute: params.package_attribute,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

/// # MissingAttributeDetector Parameters
///
/// Configuration for detecting missing attributes in PLATEAU4 features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MissingAttributeDetectorParam {
    package_attribute: Attribute,
}

#[derive(Debug, Clone)]
struct MissingAttributeBuffer {
    // Subject Attributes, etc. to be created (except required)
    feature_types_to_target_attributes: HashMap<String, HashSet<String>>,
    feature_types_to_required_attributes: HashMap<String, Vec<Vec<String>>>,
    feature_types_to_conditional_attributes: HashMap<String, Vec<Vec<String>>>,
    required_counter: usize,
    c07_counter: usize,
    c08_counter: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct MissingAttributeDetector {
    package_attribute: Attribute,
    buffer: HashMap<String, MissingAttributeBuffer>,
}

impl Processor for MissingAttributeDetector {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let count = MAD_PROCESS_COUNT.fetch_add(1, Ordering::Relaxed);
        let t_start = Instant::now();
        if count.is_multiple_of(100) {
            tracing::debug!(
                target: "perf",
                count,
                rss_mb = current_rss_mb(),
                "MissingAttributeDetector::process START"
            );
        }

        let feature = &ctx.feature;
        let AttributeValue::String(package) = feature
            .attributes
            .get(&self.package_attribute)
            .ok_or(PlateauProcessorError::MissingAttributeDetector(
                "package attribute empty".to_string(),
            ))?
        else {
            return Err(PlateauProcessorError::MissingAttributeDetector(
                "package attribute empty".to_string(),
            )
            .into());
        };
        let flush = if !self.buffer.contains_key(package) {
            self.buffer.insert(
                package.to_string(),
                MissingAttributeBuffer {
                    feature_types_to_target_attributes: HashMap::new(),
                    feature_types_to_required_attributes: HashMap::new(),
                    feature_types_to_conditional_attributes: HashMap::new(),
                    required_counter: 0,
                    c07_counter: 0,
                    c08_counter: 0,
                },
            );
            true
        } else {
            false
        };

        let (required_features, c07_features, c08_features) =
            self.detect_missing_attributes(package, feature)?;

        for feature in required_features {
            fw.send(ctx.new_with_feature_and_port(feature, REQUIRED_PORT.clone()));
        }

        for feature in c07_features {
            fw.send(ctx.new_with_feature_and_port(feature, DATA_QUALITY_C07_PORT.clone()));
        }

        for feature in c08_features {
            fw.send(ctx.new_with_feature_and_port(feature, DATA_QUALITY_C08_PORT.clone()));
        }

        if flush {
            self.process_group(ctx.as_context(), fw, package.to_string())?;
        }

        if count.is_multiple_of(100) {
            tracing::debug!(
                target: "perf",
                count,
                elapsed_ms = %t_start.elapsed().as_millis(),
                rss_mb = current_rss_mb(),
                "MissingAttributeDetector::process END"
            );
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let features = self.sumary_features(None);
        for (port, features) in features {
            for feature in features {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    port.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "MissingAttributeDetector"
    }
}

impl MissingAttributeDetector {
    fn sumary_features(&self, ignore_package: Option<String>) -> HashMap<Port, Vec<Feature>> {
        let mut summaries = Vec::new();
        let mut targets = Vec::new();
        let ignore_package = ignore_package.unwrap_or_default();
        for (package, buffer) in self
            .buffer
            .iter()
            .filter(|(package, _)| (*package).clone() != ignore_package)
        {
            let mut target_counter = 0;
            for (feature_type, names) in buffer.feature_types_to_target_attributes.iter() {
                target_counter += names.len();
                for name in names {
                    let mut feature = Feature::new_with_attributes(Default::default());
                    feature.insert(
                        "package".to_string(),
                        AttributeValue::String(package.to_string()),
                    );
                    feature.insert(
                        "featureType".to_string(),
                        AttributeValue::String(feature_type.clone()),
                    );
                    feature.insert("missing".to_string(), AttributeValue::String(name.clone()));
                    targets.push(feature);
                }
            }
            let mut feature = Feature::new_with_attributes(Default::default());
            feature.insert(
                "package".to_string(),
                AttributeValue::String(package.to_string()),
            );
            feature.insert(
                "dataFileData".to_string(),
                AttributeValue::Array(vec![
                    AttributeValue::Map(HashMap::from([
                        (
                            "name".to_string(),
                            AttributeValue::String("C05_必須属性等の欠落_Error".to_string()),
                        ),
                        (
                            "count".to_string(),
                            AttributeValue::Number(serde_json::value::Number::from(
                                buffer.required_counter,
                            )),
                        ),
                    ])),
                    AttributeValue::Map(HashMap::from([
                        (
                            "name".to_string(),
                            AttributeValue::String("C06_現われない属性等".to_string()),
                        ),
                        (
                            "count".to_string(),
                            AttributeValue::Number(serde_json::value::Number::from(target_counter)),
                        ),
                    ])),
                    AttributeValue::Map(HashMap::from([
                        (
                            "name".to_string(),
                            AttributeValue::String("C07_品質属性".to_string()),
                        ),
                        (
                            "count".to_string(),
                            AttributeValue::Number(serde_json::value::Number::from(
                                buffer.c07_counter,
                            )),
                        ),
                    ])),
                    AttributeValue::Map(HashMap::from([
                        (
                            "name".to_string(),
                            AttributeValue::String("C08_公共測量品質属性".to_string()),
                        ),
                        (
                            "count".to_string(),
                            AttributeValue::Number(serde_json::value::Number::from(
                                buffer.c08_counter,
                            )),
                        ),
                    ])),
                ]),
            );
            summaries.push(feature);
        }
        HashMap::from([
            (SUMMARY_PORT.clone(), summaries),
            (TARGET_PORT.clone(), targets),
        ])
    }

    fn process_group(
        &mut self,
        ctx: Context,
        fw: &ProcessorChannelForwarder,
        ignore_package: String,
    ) -> super::errors::Result<()> {
        let package = ignore_package.clone();
        let features = self.sumary_features(Some(ignore_package));
        for (port, features) in features {
            for feature in features {
                fw.send(ExecutorContext::new_with_context_feature_and_port(
                    &ctx,
                    feature,
                    port.clone(),
                ));
            }
        }
        let keys = self
            .buffer
            .keys()
            .filter(|key| (*key).clone() != package)
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            self.buffer.remove(&key);
        }
        Ok(())
    }

    fn detect_missing_attributes(
        &mut self,
        package: &String,
        feature: &Feature,
    ) -> super::errors::Result<(Vec<Feature>, Vec<Feature>, Vec<Feature>)> {
        let object_list: ObjectListMap = feature
            .get("objectList")
            .ok_or(PlateauProcessorError::MissingAttributeDetector(
                "object list attribute empty".to_string(),
            ))?
            .clone()
            .into();
        let object_list =
            object_list
                .get(package)
                .ok_or(PlateauProcessorError::MissingAttributeDetector(
                    "object list attribute empty".to_string(),
                ))?;
        let buffer =
            self.buffer
                .get_mut(package)
                .ok_or(PlateauProcessorError::MissingAttributeDetector(
                    "Failed to get buffer".to_string(),
                ))?;
        let AttributeValue::String(xml_content) =
            feature
                .get("xmlFragment")
                .ok_or(PlateauProcessorError::MissingAttributeDetector(
                    "xml fragment attribute empty".to_string(),
                ))?
        else {
            return Err(PlateauProcessorError::MissingAttributeDetector(
                "xml fragment attribute empty".to_string(),
            ));
        };

        let t_detect = Instant::now();
        let collected = collect_all_info(xml_content)?;
        let gml_id = collected.root_gml_id.clone().ok_or(
            PlateauProcessorError::MissingAttributeDetector("Failed to get gml id".to_string()),
        )?;
        let feature_type = collected.root_qname.clone();
        if !buffer
            .feature_types_to_target_attributes
            .contains_key(&feature_type)
        {
            let targets = object_list
                .get(&feature_type)
                .map(|object_list_value| {
                    object_list_value
                        .target
                        .iter()
                        .cloned()
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();
            buffer
                .feature_types_to_target_attributes
                .insert(feature_type.clone(), targets);
        }
        if !buffer
            .feature_types_to_required_attributes
            .contains_key(&feature_type)
        {
            let mut paths: Vec<Vec<String>> = Vec::new();
            if let Some(object_list_value) = object_list.get(&feature_type) {
                for xpath in object_list_value.required.iter() {
                    let mut items: Vec<String> = Vec::new();
                    let mut s: Vec<&str> = xpath.split('/').collect();

                    while !s.is_empty() {
                        let p1 = s.remove(0);
                        if !s.is_empty() {
                            let p2 = s.remove(0);
                            items.push(format!("{p1}/{p2}"));
                        } else {
                            items.push(p1.to_string());
                            break;
                        }
                    }
                    paths.push(items);
                }
            }
            buffer
                .feature_types_to_required_attributes
                .insert(feature_type.clone(), paths);
        }

        if !buffer
            .feature_types_to_conditional_attributes
            .contains_key(&feature_type)
        {
            let mut paths: Vec<Vec<String>> = Vec::new();
            if let Some(object_list_value) = object_list.get(&feature_type) {
                for xpath in object_list_value.conditional.iter() {
                    let mut items: Vec<String> = Vec::new();
                    let mut s: Vec<&str> = xpath.split('/').collect();

                    while !s.is_empty() {
                        let p1 = s.remove(0);
                        if !s.is_empty() {
                            let p2 = s.remove(0);
                            items.push(format!("{p1}/{p2}"));
                        } else {
                            items.push(p1.to_string());
                            break;
                        }
                    }
                    paths.push(items);
                }
            }
            buffer
                .feature_types_to_conditional_attributes
                .insert(feature_type.clone(), paths);
        }
        if let Some(target_attributes) = buffer
            .feature_types_to_target_attributes
            .get_mut(&feature_type)
        {
            for xpath in target_attributes.clone().iter() {
                if path_exists(xpath, &collected.all_qnames, &collected.parent_child_qnames) {
                    target_attributes.remove(xpath);
                }
            }
        }

        let mut missing_required = Vec::new();
        if let Some(required_attributes) = buffer
            .feature_types_to_required_attributes
            .get(&feature_type)
        {
            if !required_attributes.is_empty() {
                for paths in required_attributes.iter() {
                    match paths.len() {
                        0 => {}
                        1 => {
                            if !path_exists(
                                &paths[0],
                                &collected.all_qnames,
                                &collected.parent_child_qnames,
                            ) {
                                missing_required.push(paths[0].clone());
                            }
                        }
                        2.. => {
                            let mut hit = true;
                            for p in &paths[..paths.len() - 1] {
                                if !path_exists(
                                    p,
                                    &collected.all_qnames,
                                    &collected.parent_child_qnames,
                                ) {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit
                                && !path_exists(
                                    &paths[paths.len() - 1],
                                    &collected.all_qnames,
                                    &collected.parent_child_qnames,
                                )
                            {
                                let joined = paths.join("/");
                                missing_required.push(joined);
                            }
                        }
                    }
                }
                buffer.required_counter += missing_required.len();
            }
        }

        let mut missing_conditional = Vec::new();
        if let Some(conditional_attributes) = buffer
            .feature_types_to_conditional_attributes
            .get(&feature_type)
        {
            if !conditional_attributes.is_empty() {
                for paths in conditional_attributes.iter() {
                    match paths.len() {
                        0 => {}
                        1 => {
                            if !path_exists(
                                &paths[0],
                                &collected.all_qnames,
                                &collected.parent_child_qnames,
                            ) {
                                missing_conditional.push(paths[0].clone());
                            }
                        }
                        2.. => {
                            let mut hit = true;
                            for p in &paths[..paths.len() - 1] {
                                if !path_exists(
                                    p,
                                    &collected.all_qnames,
                                    &collected.parent_child_qnames,
                                ) {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit
                                && !path_exists(
                                    &paths[paths.len() - 1],
                                    &collected.all_qnames,
                                    &collected.parent_child_qnames,
                                )
                            {
                                let joined = paths.join("/");
                                missing_conditional.push(joined);
                            }
                        }
                    }
                }
            }
        }
        let part_xpath = FEATURE_TYPE_TO_PART_XPATH.get(feature_type.as_str());

        let severity = if let Some(part_qname) = part_xpath {
            if collected.root_child_qnames.contains(*part_qname) {
                "Warn"
            } else {
                "Error"
            }
        } else {
            "Warn"
        };

        let count = MAD_PROCESS_COUNT.load(Ordering::Relaxed);
        if count.is_multiple_of(100) {
            tracing::debug!(
                target: "perf",
                elapsed_ms = %t_detect.elapsed().as_millis(),
                xml_size_bytes = xml_content.len(),
                rss_mb = current_rss_mb(),
                "MissingAttributeDetector::detect single-pass"
            );
        }

        // C07/C08 Data Quality Attribute validation
        let lod_count = count_lod_from_collected(&collected, package);
        let (c07_errors, c08_errors) = validate_data_quality_from_collected(&collected, &lod_count);

        // Update counters
        buffer.c07_counter += c07_errors.len();
        buffer.c08_counter += c08_errors.len();

        let mut result: Vec<(String, &str)> = Vec::new();

        for rquired in &missing_required {
            result.push((rquired.clone(), "Error"));
        }
        for condition in &missing_conditional {
            result.push((condition.clone(), severity));
        }

        let required_features = result
            .into_iter()
            .map(|(xpath, severity)| {
                let mut f = create_error_feature(feature, &gml_id, &feature_type);
                f.insert(
                    "severity".to_string(),
                    AttributeValue::String(severity.to_string()),
                );
                f.insert(
                    "missing".to_string(),
                    AttributeValue::String(xpath.to_string()),
                );
                f
            })
            .collect::<Vec<_>>();

        // Generate C07 features
        let c07_features = c07_errors
            .into_iter()
            .map(|(lod, xpath)| {
                let mut f = create_error_feature(feature, &gml_id, &feature_type);
                f.insert(
                    "lod".to_string(),
                    AttributeValue::String(format!("LOD{lod}")),
                );
                f.insert("missing".to_string(), AttributeValue::String(xpath));
                f
            })
            .collect::<Vec<_>>();

        // Generate C08 features
        let c08_features = c08_errors
            .into_iter()
            .map(|(lod, xpath)| {
                let mut f = create_error_feature(feature, &gml_id, &feature_type);
                f.insert(
                    "lod".to_string(),
                    AttributeValue::String(format!("LOD{lod}")),
                );
                f.insert("missing".to_string(), AttributeValue::String(xpath));
                f
            })
            .collect::<Vec<_>>();

        Ok((required_features, c07_features, c08_features))
    }
}

/// Create a lightweight error Feature by copying only the attributes needed by
/// downstream processors (`package`, `udxDirs`, `path`) from the parent Feature
/// instead of cloning the entire (potentially large) parent.
fn create_error_feature(parent: &Feature, gml_id: &str, feature_type: &str) -> Feature {
    let mut attrs = Attributes::new();
    for key in ["package", "udxDirs", "path"] {
        if let Some(val) = parent.get(key) {
            attrs.insert(Attribute::new(key), val.clone());
        }
    }
    attrs.insert(
        Attribute::new("gmlId"),
        AttributeValue::String(gml_id.to_string()),
    );
    attrs.insert(
        Attribute::new("featureType"),
        AttributeValue::String(feature_type.to_string()),
    );
    let mut f = Feature::new_with_attributes(attrs);
    f.update_feature_type(feature_type.to_string());
    f.update_feature_id(gml_id.to_string());
    f
}

/// All information collected in a single StreamTransformer pass over the XML.
#[derive(Debug, Default)]
struct CollectedInfo {
    /// Root element qualified name (e.g. "dem:ReliefFeature")
    root_qname: String,
    /// Root element gml:id
    root_gml_id: Option<String>,
    /// Qualified names of root's direct children (for `/*/*` LOD detection)
    root_child_names: Vec<String>,
    /// Qualified names of root's direct children (for `/*/child` existence)
    root_child_qnames: HashSet<String>,
    /// Text content of dem:lod root child (for DEM package LOD)
    dem_lod_text: Option<String>,
    /// All element qualified names found in the document (for `//qname` checks)
    all_qnames: HashSet<String>,
    /// All (parent_qname, child_qname) pairs (for `//parent/child` checks)
    parent_child_qnames: HashSet<(String, String)>,
    /// All element local names found (for `local-name()='X'` checks)
    all_local_names: HashSet<String>,
    /// Text contents indexed by local name (only for data quality attributes)
    local_name_texts: HashMap<String, Vec<String>>,
}

/// Collect all needed information in a single streaming pass.
fn collect_all_info(raw_xml: &str) -> Result<CollectedInfo, PlateauProcessorError> {
    use std::cell::RefCell;

    let collected: RefCell<CollectedInfo> = RefCell::new(CollectedInfo::default());

    let transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;

    // Process root element to collect basic info and all descendants
    transformer
        .on("/*", |node| {
            let mut info = collected.borrow_mut();
            let attrs = node.get_attributes();
            info.root_gml_id = attrs.get("id").or_else(|| attrs.get("gml:id")).cloned();
            let root_qname = node.qname();
            info.all_qnames.insert(root_qname.clone());
            info.all_local_names.insert(node.name());
            info.root_qname = root_qname.clone();

            // Collect direct children of root and recurse into all descendants
            for child in node.children() {
                let child_qname = child.qname();
                let child_name = child.name();
                info.root_child_names.push(child_name.clone());
                info.root_child_qnames.insert(child_qname.clone());
                info.parent_child_qnames
                    .insert((root_qname.clone(), child_qname.clone()));

                // DEM lod text
                if child_qname == "dem:lod" {
                    info.dem_lod_text = child.get_content();
                }

                // Collect this child and all its descendants
                info.all_qnames.insert(child_qname.clone());
                info.all_local_names.insert(child_name);
                collect_descendants(&child, &mut info);
            }
        })
        .for_each()
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to collect element info: {e:?}"
            ))
        })?;

    Ok(collected.into_inner())
}

/// Recursively collect information from all descendant elements
fn collect_descendants(node: &fastxml::transform::EditableNodeRef, info: &mut CollectedInfo) {
    for child in node.children() {
        let qname = child.qname();
        let local_name = child.name();
        let parent_qname = node.qname();

        // Collect basic info
        info.all_qnames.insert(qname.clone());
        info.all_local_names.insert(local_name.clone());
        info.parent_child_qnames
            .insert((parent_qname, qname.clone()));

        // Collect text content for data quality local-name based checks
        if local_name.starts_with("geometrySrcDesc")
            || local_name.starts_with("srcScale")
            || local_name.starts_with("publicSurveySrcDesc")
        {
            let content = child.get_content().unwrap_or_default();
            info.local_name_texts
                .entry(local_name)
                .or_default()
                .push(content);
        }

        // Recurse into children
        collect_descendants(&child, info);
    }
}

/// Check if a path expression exists in collected data.
/// Handles single qnames, two-level parent/child, and multi-level paths (e.g. "a/b/c/d").
fn path_exists(
    path: &str,
    all_qnames: &HashSet<String>,
    parent_child_qnames: &HashSet<(String, String)>,
) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    match parts.len() {
        0 => false,
        1 => all_qnames.contains(parts[0]),
        _ => {
            // Check that all consecutive parent-child pairs exist in the document
            parts
                .windows(2)
                .all(|w| parent_child_qnames.contains(&(w[0].to_string(), w[1].to_string())))
        }
    }
}

fn count_lod_from_collected(collected: &CollectedInfo, package: &str) -> [usize; 5] {
    let mut lod_count = [0; 5];

    if package == "dem" {
        if let Some(text) = &collected.dem_lod_text {
            if let Ok(lod) = text.trim().parse::<usize>() {
                if lod <= 4 {
                    lod_count[lod] += 1;
                }
            }
        }
    } else {
        for tag in &collected.root_child_names {
            if let Some(captures) = LOD_TAG_PATTERN.captures(tag) {
                if let Some(lod_match) = captures.get(1) {
                    if let Ok(lod) = lod_match.as_str().parse::<usize>() {
                        if lod <= 4 {
                            lod_count[lod] += 1;
                        }
                    }
                }
            }
        }
    }

    lod_count
}

#[allow(clippy::type_complexity)]
fn validate_data_quality_from_collected(
    collected: &CollectedInfo,
    lod_count: &[usize; 5],
) -> (Vec<(usize, String)>, Vec<(usize, String)>) {
    let mut c07_errors = Vec::new();
    let mut c08_errors = Vec::new();

    if !collected.all_local_names.contains("DataQualityAttribute") {
        return (c07_errors, c08_errors);
    }

    for (lod, &count) in lod_count.iter().enumerate() {
        if count == 0 {
            continue;
        }

        let geom_key = format!("geometrySrcDescLod{lod}");
        let geom_texts = collected.local_name_texts.get(&geom_key);

        match geom_texts {
            None => {
                // Missing geometrySrcDescLod{N} - C07 error
                c07_errors.push((
                    lod,
                    format!("uro:DataQualityAttribute/uro:geometrySrcDescLod{lod}"),
                ));
            }
            Some(texts) if texts.len() == 1 => {
                let text = &texts[0];
                if !text.is_empty() && text.trim() == "000" {
                    // C08: Check srcScaleLod{N}
                    let src_scale_key = format!("srcScaleLod{lod}");
                    if !collected.all_local_names.contains(&src_scale_key) {
                        c08_errors.push((
                            lod,
                            format!("uro:PublicSurveyDataQualityAttribute/uro:srcScaleLod{lod}"),
                        ));
                    }

                    // C08: Check publicSurveySrcDescLod{N}
                    let public_survey_key = format!("publicSurveySrcDescLod{lod}");
                    if !collected.all_local_names.contains(&public_survey_key) {
                        c08_errors.push((
                            lod,
                            format!(
                                "uro:PublicSurveyDataQualityAttribute/uro:publicSurveySrcDescLod{lod}"
                            ),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    (c07_errors, c08_errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_list::{ObjectList, ObjectListValue};
    use crate::tests::utils::{create_default_execute_context, create_default_node_context};
    use indexmap::IndexMap;
    use reearth_flow_runtime::{
        event::EventHub,
        forwarder::{NoopChannelForwarder, ProcessorChannelForwarder},
        node::ProcessorFactory,
    };
    use reearth_flow_types::Feature;
    use std::collections::HashMap;

    #[test]
    fn test_with_valid_input() -> Result<(), BoxedError> {
        // Arrange
        let feature = create_feature_with_valid_xml();

        // Act
        let fw = run_processor_with_feature(feature)?;
        let required_outputs = extract_outputs_by_port(&fw, "required")?;

        // Assert
        assert_eq!(required_outputs.len(), 0);
        Ok(())
    }

    #[test]
    fn test_c05_missing_required_attribute() -> Result<(), BoxedError> {
        // Arrange
        let feature = create_feature_with_missing_required_attribute();

        // Act
        let fw = run_processor_with_feature(feature)?;
        let required_outputs = extract_outputs_by_port(&fw, "required")?;

        // Assert
        // Verify that C05 error was detected for missing required attribute
        assert_eq!(
            required_outputs.len(),
            1,
            "Expected exactly one required attribute error"
        );

        let error_output = &required_outputs[0];
        assert_eq!(
            error_output.get("missing"),
            Some(&AttributeValue::String("core:creationDate".to_string())),
            "Expected missing attribute to be core:creationDate"
        );

        assert_eq!(
            error_output.get("featureType"),
            Some(&AttributeValue::String("bldg:Building".to_string())),
            "Expected feature type to be bldg:Building"
        );

        assert_eq!(
            error_output.get("severity"),
            Some(&AttributeValue::String("Error".to_string())),
            "Expected severity to be Error"
        );

        Ok(())
    }

    #[test]
    fn test_c06_missing_target_attribute() -> Result<(), BoxedError> {
        // Arrange
        let feature = create_feature_with_missing_target_attribute();

        // Act
        let fw = run_processor_with_feature(feature)?;
        let summary_outputs = extract_outputs_by_port(&fw, "summary")?;

        // Assert
        assert_eq!(
            summary_outputs.len(),
            1,
            "Expected exactly one summary output"
        );

        let summary = &summary_outputs[0];
        let data_file_data = match summary.get("dataFileData").unwrap() {
            AttributeValue::Array(arr) => arr,
            _ => panic!("Expected dataFileData to be an array"),
        };

        let expected = AttributeValue::Map(HashMap::from([
            (
                "name".to_string(),
                AttributeValue::String("C06_現われない属性等".to_string()),
            ),
            (
                "count".to_string(),
                AttributeValue::Number(serde_json::Number::from(1)),
            ),
        ]));
        data_file_data
            .iter()
            .find(|item| **item == expected)
            .expect("Expected C06 entry in summary");

        Ok(())
    }

    #[test]
    fn test_c07_missing_geometry_src_desc() -> Result<(), BoxedError> {
        // Arrange
        let feature = create_feature_with_missing_geometry_src_desc();

        // Act
        let fw = run_processor_with_feature(feature)?;
        let c07_outputs = extract_outputs_by_port(&fw, "dataQualityC07")?;

        // Assert
        assert_eq!(c07_outputs.len(), 1, "Expected exactly one C07 output");

        let c07_output = &c07_outputs[0];
        assert_eq!(
            c07_output.get("lod"),
            Some(&AttributeValue::String("LOD1".to_string())),
            "Expected LOD to be LOD1"
        );

        assert_eq!(
            c07_output.get("missing"),
            Some(&AttributeValue::String(
                "uro:DataQualityAttribute/uro:geometrySrcDescLod1".to_string()
            )),
            "Expected missing attribute to be geometrySrcDescLod1"
        );

        assert_eq!(
            c07_output.get("featureType"),
            Some(&AttributeValue::String("bldg:Building".to_string())),
            "Expected feature type to be bldg:Building"
        );

        Ok(())
    }

    #[test]
    fn test_c08_missing_public_survey_attribute() -> Result<(), BoxedError> {
        // Arrange
        let feature = create_feature_with_missing_public_survey_attribute();

        // Act
        let fw = run_processor_with_feature(feature)?;
        let c08_outputs = extract_outputs_by_port(&fw, "dataQualityC08")?;

        // Assert
        assert_eq!(c08_outputs.len(), 2, "Expected exactly two C08 outputs");

        c08_outputs
            .iter()
            .find(|output| {
                matches!(
                    output.get("missing"),
                    Some(AttributeValue::String(missing))
                    if missing == "uro:PublicSurveyDataQualityAttribute/uro:srcScaleLod1"
                )
            })
            .expect("Expected srcScaleLod1 error");

        c08_outputs
            .iter()
            .find(|output| {
                matches!(
                    output.get("missing"),
                    Some(AttributeValue::String(missing))
                    if missing == "uro:PublicSurveyDataQualityAttribute/uro:publicSurveySrcDescLod1"
                )
            })
            .expect("Expected publicSurveySrcDescLod1 error");

        Ok(())
    }

    //
    // Create test feature
    //

    // Feature creation functions for different test scenarios
    fn create_feature_with_valid_xml() -> Feature {
        let gml_id = format!("bldg_{}", uuid::Uuid::new_v4());
        let xml_fragment = format!(
            r#"
<bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
    xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.0"
    gml:id="{gml_id}">
  <core:creationDate>2025-03-21</core:creationDate>
  <bldg:class>3003</bldg:class>
  <bldg:usage>411</bldg:usage>
</bldg:Building>"#
        );

        create_feature_with_custom_object_list(
            "test.gml",
            "bldg",
            &xml_fragment,
            &gml_id,
            vec!["core:creationDate".to_string()],
            vec![],
            vec![],
        )
    }

    fn create_feature_with_missing_required_attribute() -> Feature {
        let gml_id = format!("bldg_{}", uuid::Uuid::new_v4());
        let xml_fragment = format!(
            r#"
<bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
    xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.0"
    gml:id="{gml_id}">
  <!-- Missing required attribute: core:creationDate -->
  <bldg:class>3003</bldg:class>
  <bldg:usage>411</bldg:usage>
</bldg:Building>"#
        );

        create_feature_with_custom_object_list(
            "test.gml",
            "bldg",
            &xml_fragment,
            &gml_id,
            vec!["core:creationDate".to_string()],
            vec![],
            vec![],
        )
    }

    fn create_feature_with_missing_target_attribute() -> Feature {
        let gml_id = format!("bldg_{}", uuid::Uuid::new_v4());
        let xml_fragment = format!(
            r#"
<bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
    xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.0"
    gml:id="{gml_id}">
  <core:creationDate>2025-03-21</core:creationDate>
  <bldg:class>3003</bldg:class>
  <bldg:usage>411</bldg:usage>
</bldg:Building>"#
        );

        create_feature_with_custom_object_list(
            "test.gml",
            "bldg",
            &xml_fragment,
            &gml_id,
            vec!["core:creationDate".to_string()],
            vec!["bldg:measuredHeight".to_string()], // This attribute is missing from XML
            vec![],
        )
    }

    // C07
    fn create_feature_with_missing_geometry_src_desc() -> Feature {
        let gml_id = format!("bldg_{}", uuid::Uuid::new_v4());
        let xml_fragment = format!(
            r#"
<bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
    xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.0"
    gml:id="{gml_id}">
  <core:creationDate>2025-03-21</core:creationDate>
  <bldg:class>3003</bldg:class>
  <bldg:usage>411</bldg:usage>
  <bldg:lod1Solid>
    <gml:Solid>
      <gml:exterior>
        <gml:CompositeSurface>
          <gml:surfaceMember>
            <gml:Polygon>
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>0 0 0 1 0 0 1 1 0 0 1 0 0 0 0</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:CompositeSurface>
      </gml:exterior>
    </gml:Solid>
  </bldg:lod1Solid>
  <uro:bldgDataQualityAttribute>
    <uro:DataQualityAttribute>
      <!-- Missing geometrySrcDescLod1 -->
    </uro:DataQualityAttribute>
  </uro:bldgDataQualityAttribute>
</bldg:Building>"#
        );

        create_feature_with_custom_object_list(
            "test.gml",
            "bldg",
            &xml_fragment,
            &gml_id,
            vec!["core:creationDate".to_string()],
            vec![],
            vec![],
        )
    }

    // C08
    fn create_feature_with_missing_public_survey_attribute() -> Feature {
        let gml_id = format!("bldg_{}", uuid::Uuid::new_v4());
        let xml_fragment = format!(
            r#"
<bldg:Building xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
    xmlns:core="http://www.opengis.net/citygml/2.0"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:uro="https://www.geospatial.jp/iur/uro/3.0"
    gml:id="{gml_id}">
  <core:creationDate>2025-03-21</core:creationDate>
  <bldg:class>3003</bldg:class>
  <bldg:usage>411</bldg:usage>
  <bldg:lod1Solid>
    <gml:Solid>
      <gml:exterior>
        <gml:CompositeSurface>
          <gml:surfaceMember>
            <gml:Polygon>
              <gml:exterior>
                <gml:LinearRing>
                  <gml:posList>0 0 0 1 0 0 1 1 0 0 1 0 0 0 0</gml:posList>
                </gml:LinearRing>
              </gml:exterior>
            </gml:Polygon>
          </gml:surfaceMember>
        </gml:CompositeSurface>
      </gml:exterior>
    </gml:Solid>
  </bldg:lod1Solid>
  <uro:bldgDataQualityAttribute>
    <uro:DataQualityAttribute>
      <uro:geometrySrcDescLod1>000</uro:geometrySrcDescLod1>
      <!-- Missing publicSurveyDataQualityAttribute with PublicSurveyDataQualityAttribute -->
    </uro:DataQualityAttribute>
  </uro:bldgDataQualityAttribute>
</bldg:Building>"#
        );

        create_feature_with_custom_object_list(
            "test.gml",
            "bldg",
            &xml_fragment,
            &gml_id,
            vec!["core:creationDate".to_string()],
            vec![],
            vec![],
        )
    }

    fn create_feature_with_custom_object_list(
        name: &str,
        package: &str,
        xml_content: &str,
        gml_id: &str,
        required: Vec<String>,
        target: Vec<String>,
        conditional: Vec<String>,
    ) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String(name.to_string()),
        );
        attributes.insert(
            Attribute::new("package"),
            AttributeValue::String(package.to_string()),
        );
        attributes.insert(
            Attribute::new("path"),
            AttributeValue::String("file:///test.gml".to_string()),
        );
        attributes.insert(
            Attribute::new("xmlFragment"),
            AttributeValue::String(xml_content.to_string()),
        );
        attributes.insert(
            Attribute::new("gmlId"),
            AttributeValue::String(gml_id.to_string()),
        );

        let object_list_value = ObjectListValue {
            required,
            target,
            conditional,
        };

        let mut object_list_types = HashMap::new();
        object_list_types.insert("bldg:Building".to_string(), object_list_value);
        let object_list = ObjectList::new(object_list_types);

        let object_list_map =
            HashMap::from([(package.to_string(), AttributeValue::from(object_list))]);

        attributes.insert(
            Attribute::new("objectList"),
            AttributeValue::Map(object_list_map),
        );

        Feature::new_with_attributes(attributes)
    }

    //
    // Run processor
    //

    fn run_processor_with_feature(
        feature: Feature,
    ) -> Result<ProcessorChannelForwarder, BoxedError> {
        let factory = MissingAttributeDetectorFactory {};
        let params_map = HashMap::from([(
            "packageAttribute".to_string(),
            serde_json::Value::String("package".to_string()),
        )]);

        let ctx = create_default_node_context();
        let mut processor: Box<dyn Processor> = factory.build(
            ctx,
            EventHub::new(1024),
            "test".to_string(),
            Some(params_map),
        )?;

        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let ctx = create_default_execute_context(feature);
        processor.process(ctx, &fw)?;

        let ctx = create_default_node_context();
        processor.finish(ctx, &fw)?;

        Ok(fw)
    }

    fn extract_outputs_by_port(
        fw: &ProcessorChannelForwarder,
        port_name: &str,
    ) -> Result<Vec<Feature>, BoxedError> {
        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();

                let outputs: Vec<Feature> = send_ports
                    .iter()
                    .enumerate()
                    .filter(|(_, port)| port.as_ref() == port_name)
                    .map(|(i, _)| send_features[i].clone())
                    .collect();

                Ok(outputs)
            }
            ProcessorChannelForwarder::ChannelManager(_) => {
                Err("Expected Noop forwarder for testing".into())
            }
        }
    }
}

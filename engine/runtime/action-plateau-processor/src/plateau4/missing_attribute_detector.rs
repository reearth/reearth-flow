use crate::object_list::ObjectListMap;

use super::errors::PlateauProcessorError;
use nusamai_citygml::GML31_NS;
use once_cell::sync::Lazy;
use reearth_flow_common::xml;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

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
    Lazy::new(|| Regex::new(r".+:lod([0-4]).+").expect("Failed to compile LOD tag pattern"));

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
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
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
                    let mut feature = Feature::new();
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
            let mut feature = Feature::new();
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
            .get(&"objectList".to_string())
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
        let AttributeValue::String(xml_content) = feature.get(&"xmlFragment".to_string()).ok_or(
            PlateauProcessorError::MissingAttributeDetector(
                "xml fragment attribute empty".to_string(),
            ),
        )?
        else {
            return Err(PlateauProcessorError::MissingAttributeDetector(
                "xml fragment attribute empty".to_string(),
            ));
        };

        let Ok(document) = xml::parse(xml_content) else {
            return Err(PlateauProcessorError::MissingAttributeDetector(
                "Failed to parse XML".to_string(),
            ));
        };
        let xml_ctx = xml::create_context(&document).map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to create xml context: {e}"
            ))
        })?;
        let root_node = match xml::get_root_readonly_node(&document) {
            Ok(node) => node,
            Err(e) => {
                return Err(PlateauProcessorError::MissingAttributeDetector(format!(
                    "Failed to get root node: {e}"
                )));
            }
        };
        let gml_id = root_node
            .get_attribute_ns(
                "id",
                String::from_utf8(GML31_NS.into_inner().to_vec())
                    .map_err(|e| PlateauProcessorError::MissingAttributeDetector(format!("{e:?}")))?
                    .as_str(),
            )
            .ok_or(PlateauProcessorError::MissingAttributeDetector(
                "Failed to get gml id".to_string(),
            ))?;
        let feature_type = xml::get_readonly_node_tag(&root_node);
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
                let xpath_with_prefix = format!(".//{xpath}");
                let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath_with_prefix);
                let node =
                    xml::find_readonly_nodes_by_xpath(&xml_ctx, &converted_xpath, &root_node)
                        .map_err(|e| {
                            PlateauProcessorError::MissingAttributeDetector(format!(
                                "Failed to find node by xpath '{}' at {}:{}: {e}",
                                xpath,
                                file!(),
                                line!()
                            ))
                        })?;
                if !node.is_empty() {
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
                            let xpath = format!(".//{}", paths[0]);
                            let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath);
                            let node = xml::find_readonly_nodes_by_xpath(
                                &xml_ctx,
                                &converted_xpath,
                                &root_node,
                            )
                            .map_err(|e| {
                                PlateauProcessorError::MissingAttributeDetector(format!(
                                    "Failed to find node by xpath '{}' at {}:{}: {e}",
                                    xpath,
                                    file!(),
                                    line!()
                                ))
                            })?;

                            if node.is_empty() {
                                missing_required.push(paths[0].clone());
                            }
                        }
                        2.. => {
                            let mut hit = true;
                            for p in &paths[..paths.len() - 1] {
                                let xpath = format!(".//{p}");
                                let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath);
                                let node = xml::find_readonly_nodes_by_xpath(
                                    &xml_ctx,
                                    &converted_xpath,
                                    &root_node,
                                )
                                .map_err(|e| {
                                    PlateauProcessorError::MissingAttributeDetector(format!(
                                        "Failed to find node by xpath '{}' at {}:{}: {e}",
                                        xpath,
                                        file!(),
                                        line!()
                                    ))
                                })?;
                                if node.is_empty() {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit {
                                let xpath = format!(".//{}", paths[paths.len() - 1]);
                                let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath);
                                let node = xml::find_readonly_nodes_by_xpath(
                                    &xml_ctx,
                                    &converted_xpath,
                                    &root_node,
                                )
                                .map_err(|e| {
                                    PlateauProcessorError::MissingAttributeDetector(format!(
                                        "Failed to find node by xpath '{}' at {}:{}: {e}",
                                        xpath,
                                        file!(),
                                        line!()
                                    ))
                                })?;
                                if node.is_empty() {
                                    let joined = paths.join("/");
                                    missing_required.push(joined);
                                }
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
                            let xpath = format!(".//{}", paths[0]);
                            let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath);
                            let node = xml::find_readonly_nodes_by_xpath(
                                &xml_ctx,
                                &converted_xpath,
                                &root_node,
                            )
                            .map_err(|e| {
                                PlateauProcessorError::MissingAttributeDetector(format!(
                                    "Failed to find node by xpath '{}' at {}:{}: {e}",
                                    xpath,
                                    file!(),
                                    line!()
                                ))
                            })?;

                            if node.is_empty() {
                                missing_conditional.push(paths[0].clone());
                            }
                        }
                        2.. => {
                            let mut hit = true;
                            for p in &paths[..paths.len() - 1] {
                                let xpath = format!(".//{p}");
                                let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath);
                                let node = xml::find_readonly_nodes_by_xpath(
                                    &xml_ctx,
                                    &converted_xpath,
                                    &root_node,
                                )
                                .map_err(|e| {
                                    PlateauProcessorError::MissingAttributeDetector(format!(
                                        "Failed to find node by xpath '{}' at {}:{}: {e}",
                                        xpath,
                                        file!(),
                                        line!()
                                    ))
                                })?;
                                if node.is_empty() {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit {
                                let xpath = format!(".//{}", paths[paths.len() - 1]);
                                let converted_xpath = convert_xpath_prefixes_to_local_name(&xpath);
                                let node = xml::find_readonly_nodes_by_xpath(
                                    &xml_ctx,
                                    &converted_xpath,
                                    &root_node,
                                )
                                .map_err(|e| {
                                    PlateauProcessorError::MissingAttributeDetector(format!(
                                        "Failed to find node by xpath '{}' at {}:{}: {e}",
                                        xpath,
                                        file!(),
                                        line!()
                                    ))
                                })?;
                                if node.is_empty() {
                                    let joined = paths.join("/");
                                    missing_conditional.push(joined);
                                }
                            }
                        }
                    }
                }
            }
        }
        let xpath = FEATURE_TYPE_TO_PART_XPATH.get(feature_type.as_str());

        let severity = if let Some(xpath) = xpath {
            let converted_xpath = convert_xpath_prefixes_to_local_name(xpath);
            let node = xml::find_readonly_nodes_by_xpath(&xml_ctx, &converted_xpath, &root_node)
                .map_err(|e| {
                    PlateauProcessorError::MissingAttributeDetector(format!(
                        "Failed to find node by xpath '{}' at {}:{}: {e}",
                        xpath,
                        file!(),
                        line!()
                    ))
                })?;
            if node.is_empty() {
                "Error"
            } else {
                "Warn"
            }
        } else {
            "Warn"
        };

        // C07/C08 Data Quality Attribute validation
        let lod_count = count_lod_geometries(&xml_ctx, &root_node, package)?;
        let (c07_errors, c08_errors) =
            validate_data_quality_attributes(&xml_ctx, &root_node, &lod_count, package)?;

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
                let mut feature = feature.clone();
                feature.insert(
                    "gmlId".to_string(),
                    AttributeValue::String(gml_id.to_string()),
                );
                feature.update_feature_type(feature_type.clone());
                feature.update_feature_id(gml_id.to_string());
                feature.insert(
                    "featureType".to_string(),
                    AttributeValue::String(feature_type.clone()),
                );
                feature.insert(
                    "severity".to_string(),
                    AttributeValue::String(severity.to_string()),
                );
                feature.insert(
                    "missing".to_string(),
                    AttributeValue::String(xpath.to_string()),
                );
                feature
            })
            .collect::<Vec<_>>();

        // Generate C07 features
        let c07_features = c07_errors
            .into_iter()
            .map(|(lod, xpath)| {
                let mut feature = feature.clone();
                feature.insert(
                    "gmlId".to_string(),
                    AttributeValue::String(gml_id.to_string()),
                );
                feature.update_feature_type(feature_type.clone());
                feature.update_feature_id(gml_id.to_string());
                feature.insert(
                    "featureType".to_string(),
                    AttributeValue::String(feature_type.clone()),
                );
                feature.insert(
                    "lod".to_string(),
                    AttributeValue::String(format!("LOD{lod}")),
                );
                feature.insert("missing".to_string(), AttributeValue::String(xpath));
                feature
            })
            .collect::<Vec<_>>();

        // Generate C08 features
        let c08_features = c08_errors
            .into_iter()
            .map(|(lod, xpath)| {
                let mut feature = feature.clone();
                feature.insert(
                    "gmlId".to_string(),
                    AttributeValue::String(gml_id.to_string()),
                );
                feature.update_feature_type(feature_type.clone());
                feature.update_feature_id(gml_id.to_string());
                feature.insert(
                    "featureType".to_string(),
                    AttributeValue::String(feature_type.clone()),
                );
                feature.insert(
                    "lod".to_string(),
                    AttributeValue::String(format!("LOD{lod}")),
                );
                feature.insert("missing".to_string(), AttributeValue::String(xpath));
                feature
            })
            .collect::<Vec<_>>();

        Ok((required_features, c07_features, c08_features))
    }
}

fn convert_xpath_prefixes_to_local_name(xpath: &str) -> String {
    // Handle special prefixes like .// and preserve them
    if let Some(remaining) = xpath.strip_prefix(".//") {
        let converted_remaining = convert_xpath_prefixes_to_local_name(remaining);
        return format!(".//{converted_remaining}");
    }

    // Split the path by '/' and convert each prefixed element separately
    let parts: Vec<&str> = xpath.split('/').collect();
    let converted_parts: Vec<String> = parts
        .iter()
        .map(|part| {
            if part.is_empty() {
                // Preserve empty parts (like double slashes)
                part.to_string()
            } else if let Some((_prefix, element)) = part.split_once(':') {
                // Use only local-name() to avoid version dependency issues
                // This works for any namespace version (e.g., uro/3.0, uro/3.1, etc.)
                format!("*[local-name()='{element}']")
            } else {
                part.to_string()
            }
        })
        .collect();

    converted_parts.join("/")
}

fn replace_namespace_with_prefix(tag: &str) -> String {
    // Simple implementation to convert namespace to prefix
    // e.g., "{http://www.geospatial.jp/iur/uro/3.1}geometrySrcDescLod1" -> "uro:geometrySrcDescLod1"
    if let Some(start) = tag.find('}') {
        let namespace = &tag[1..start];
        let local_name = &tag[start + 1..];

        let prefix = match namespace {
            "https://www.geospatial.jp/iur/uro/2.0"
            | "https://www.geospatial.jp/iur/uro/3.0"
            | "https://www.geospatial.jp/iur/uro/3.1" => "uro",
            "http://www.opengis.net/citygml/relief/2.0" => "dem",
            "http://www.opengis.net/citygml/building/2.0" => "bldg",
            "http://www.opengis.net/citygml/bridge/2.0" => "brid",
            "http://www.opengis.net/citygml/tunnel/2.0" => "tun",
            _ => return tag.to_string(),
        };

        format!("{prefix}:{local_name}")
    } else {
        tag.to_string()
    }
}

fn count_lod_geometries(
    xml_ctx: &xml::XmlContext,
    root_node: &xml::XmlRoNode,
    package: &str,
) -> Result<[usize; 5], PlateauProcessorError> {
    let mut lod_count = [0; 5];

    if package == "dem" {
        // Special handling for DEM package
        let xpath = "./*[local-name()='lod']";
        let nodes = xml::find_readonly_nodes_by_xpath(xml_ctx, xpath, root_node).map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to find DEM LOD node: {e}"
            ))
        })?;

        if let Some(node) = nodes.first() {
            let text = node.get_content();
            if let Ok(lod) = text.trim().parse::<usize>() {
                if lod <= 4 {
                    lod_count[lod] += 1;
                }
            }
        }
    } else {
        // General LOD pattern matching for other packages
        let xpath = "./*";
        let nodes = xml::find_readonly_nodes_by_xpath(xml_ctx, xpath, root_node).map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to find child nodes: {e}"
            ))
        })?;

        for node in nodes {
            let tag = xml::get_readonly_node_tag(&node);
            let prefixed_tag = replace_namespace_with_prefix(&tag);

            if let Some(captures) = LOD_TAG_PATTERN.captures(&prefixed_tag) {
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

    Ok(lod_count)
}

#[allow(clippy::type_complexity)]
fn validate_data_quality_attributes(
    xml_ctx: &xml::XmlContext,
    root_node: &xml::XmlRoNode,
    lod_count: &[usize; 5],
    _package: &str,
) -> Result<(Vec<(usize, String)>, Vec<(usize, String)>), PlateauProcessorError> {
    let mut c07_errors = Vec::new();
    let mut c08_errors = Vec::new();

    // Find DataQualityAttribute section (nested under bldgDataQualityAttribute)
    let data_quality_xpath = ".//*[local-name()='DataQualityAttribute']";
    let data_quality_nodes =
        xml::find_readonly_nodes_by_xpath(xml_ctx, data_quality_xpath, root_node).map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to find DataQualityAttribute: {e}"
            ))
        })?;

    if let Some(data_quality_attr) = data_quality_nodes.first() {
        // Check each LOD that has geometry elements
        for (lod, &count) in lod_count.iter().enumerate() {
            if count > 0 {
                // C07: Check for geometrySrcDescLod{N}
                let geom_src_desc_xpath = format!("./*[local-name()='geometrySrcDescLod{lod}']");
                let geom_nodes = xml::find_readonly_nodes_by_xpath(
                    xml_ctx,
                    &geom_src_desc_xpath,
                    data_quality_attr,
                )
                .map_err(|e| {
                    PlateauProcessorError::MissingAttributeDetector(format!(
                        "Failed to find geometrySrcDescLod{lod}: {e}"
                    ))
                })?;

                if geom_nodes.is_empty() {
                    // Missing geometrySrcDescLod{N} - C07 error
                    c07_errors.push((
                        lod,
                        format!("uro:DataQualityAttribute/uro:geometrySrcDescLod{lod}"),
                    ));
                } else if geom_nodes.len() == 1 {
                    // Check if value is "000" for C08 validation
                    let text = geom_nodes[0].get_content();
                    if !text.is_empty() && text.trim() == "000" {
                        // C08: Check PublicSurveyDataQualityAttribute sub-elements
                        let public_survey_base_xpath = "./*[local-name()='publicSurveyDataQualityAttribute']/*[local-name()='PublicSurveyDataQualityAttribute']";

                        // Check srcScaleLod{N}
                        let src_scale_xpath = format!(
                            "{public_survey_base_xpath}/*[local-name()='srcScaleLod{lod}']"
                        );
                        let src_scale_nodes = xml::find_readonly_nodes_by_xpath(
                            xml_ctx,
                            &src_scale_xpath,
                            data_quality_attr,
                        )
                        .map_err(|e| {
                            PlateauProcessorError::MissingAttributeDetector(format!(
                                "Failed to find srcScaleLod{lod}: {e}"
                            ))
                        })?;

                        if src_scale_nodes.is_empty() {
                            c08_errors.push((
                                lod,
                                format!(
                                    "uro:PublicSurveyDataQualityAttribute/uro:srcScaleLod{lod}"
                                ),
                            ));
                        }

                        // Check publicSurveySrcDescLod{N}
                        let public_survey_src_desc_xpath = format!("{public_survey_base_xpath}/*[local-name()='publicSurveySrcDescLod{lod}']");
                        let public_survey_nodes = xml::find_readonly_nodes_by_xpath(
                            xml_ctx,
                            &public_survey_src_desc_xpath,
                            data_quality_attr,
                        )
                        .map_err(|e| {
                            PlateauProcessorError::MissingAttributeDetector(format!(
                                "Failed to find publicSurveySrcDescLod{lod}: {e}"
                            ))
                        })?;

                        if public_survey_nodes.is_empty() {
                            c08_errors.push((lod, format!("uro:PublicSurveyDataQualityAttribute/uro:publicSurveySrcDescLod{lod}")));
                        }
                    }
                }
            }
        }
    }

    Ok((c07_errors, c08_errors))
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
            error_output.get(&Attribute::new("missing")),
            Some(&AttributeValue::String("core:creationDate".to_string())),
            "Expected missing attribute to be core:creationDate"
        );

        assert_eq!(
            error_output.get(&Attribute::new("featureType")),
            Some(&AttributeValue::String("bldg:Building".to_string())),
            "Expected feature type to be bldg:Building"
        );

        assert_eq!(
            error_output.get(&Attribute::new("severity")),
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
        let data_file_data = match summary.get(&Attribute::new("dataFileData")).unwrap() {
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
            c07_output.get(&Attribute::new("lod")),
            Some(&AttributeValue::String("LOD1".to_string())),
            "Expected LOD to be LOD1"
        );

        assert_eq!(
            c07_output.get(&Attribute::new("missing")),
            Some(&AttributeValue::String(
                "uro:DataQualityAttribute/uro:geometrySrcDescLod1".to_string()
            )),
            "Expected missing attribute to be geometrySrcDescLod1"
        );

        assert_eq!(
            c07_output.get(&Attribute::new("featureType")),
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
                    output.get(&Attribute::new("missing")),
                    Some(AttributeValue::String(missing))
                    if missing == "uro:PublicSurveyDataQualityAttribute/uro:srcScaleLod1"
                )
            })
            .expect("Expected srcScaleLod1 error");

        c08_outputs
            .iter()
            .find(|output| {
                matches!(
                    output.get(&Attribute::new("missing")),
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

        let mut feature = Feature::new();
        feature.attributes = attributes;
        feature
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

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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));
static REQUIRED_PORT: Lazy<Port> = Lazy::new(|| Port::new("required"));
static TARGET_PORT: Lazy<Port> = Lazy::new(|| Port::new("target"));

static FEATURE_TYPE_TO_PART_XPATH: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("bldg:Building", "bldg:consistsOfBuildingPart"),
        ("brid:Bridge", "brid:consistsOfBridgePart"),
        ("tun:Tunnel", "tun:consistsOfTunnelPart"),
        ("uro:UndergroundBuilding", "bldg:consistsOfBuildingPart"),
    ])
});

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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MissingAttributeDetectorParam {
    package_attribute: Attribute,
}

#[derive(Debug, Clone)]
struct MissingAttributeBuffer {
    feature_types_to_target_attributes: HashMap<String, HashSet<String>>, // Subject Attributes, etc. to be created (except required)
    feature_types_to_required_attributes: HashMap<String, Vec<Vec<String>>>,
    feature_types_to_conditional_attributes: HashMap<String, Vec<Vec<String>>>,
    required_counter: usize,
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
                },
            );
            true
        } else {
            false
        };
        let features = self.detect_missing_attributes(package, feature)?;
        for feature in features {
            fw.send(ctx.new_with_feature_and_port(feature, REQUIRED_PORT.clone()));
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
                AttributeValue::Map(HashMap::from([
                    (
                        "必須属性等の欠落_Error".to_string(),
                        AttributeValue::Number(serde_json::value::Number::from(
                            buffer.required_counter,
                        )),
                    ),
                    (
                        "現われなかった属性等".to_string(),
                        AttributeValue::Number(serde_json::value::Number::from(target_counter)),
                    ),
                ])),
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
    ) -> super::errors::Result<Vec<Feature>> {
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
                    .map_err(|e| {
                        PlateauProcessorError::MissingAttributeDetector(format!("{e:?}"))
                    })?
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
                let node = xml::find_readonly_nodes_by_xpath(&xml_ctx, xpath, &root_node).map_err(
                    |e| {
                        PlateauProcessorError::MissingAttributeDetector(format!(
                            "Failed to find node by xpath: {e}"
                        ))
                    },
                )?;
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
                            let node =
                                xml::find_readonly_nodes_by_xpath(&xml_ctx, &xpath, &root_node)
                                    .map_err(|e| {
                                        PlateauProcessorError::MissingAttributeDetector(format!(
                                            "Failed to find node by xpath: {e}"
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
                                let node =
                                    xml::find_readonly_nodes_by_xpath(&xml_ctx, &xpath, &root_node)
                                        .map_err(|e| {
                                            PlateauProcessorError::MissingAttributeDetector(
                                                format!("Failed to find node by xpath: {e}"),
                                            )
                                        })?;
                                if node.is_empty() {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit {
                                let xpath = format!(".//{}", paths[paths.len() - 1]);
                                let node =
                                    xml::find_readonly_nodes_by_xpath(&xml_ctx, &xpath, &root_node)
                                        .map_err(|e| {
                                            PlateauProcessorError::MissingAttributeDetector(
                                                format!("Failed to find node by xpath: {e}"),
                                            )
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
                            let node =
                                xml::find_readonly_nodes_by_xpath(&xml_ctx, &xpath, &root_node)
                                    .map_err(|e| {
                                        PlateauProcessorError::MissingAttributeDetector(format!(
                                            "Failed to find node by xpath: {e}"
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
                                let node =
                                    xml::find_readonly_nodes_by_xpath(&xml_ctx, &xpath, &root_node)
                                        .map_err(|e| {
                                            PlateauProcessorError::MissingAttributeDetector(
                                                format!("Failed to find node by xpath: {e}"),
                                            )
                                        })?;
                                if node.is_empty() {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit {
                                let xpath = format!(".//{}", paths[paths.len() - 1]);
                                let node =
                                    xml::find_readonly_nodes_by_xpath(&xml_ctx, &xpath, &root_node)
                                        .map_err(|e| {
                                            PlateauProcessorError::MissingAttributeDetector(
                                                format!("Failed to find node by xpath: {e}"),
                                            )
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
            let node =
                xml::find_readonly_nodes_by_xpath(&xml_ctx, xpath, &root_node).map_err(|e| {
                    PlateauProcessorError::MissingAttributeDetector(format!(
                        "Failed to find node by xpath: {e}"
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

        let mut result: Vec<(String, &str)> = Vec::new();

        for rquired in &missing_required {
            result.push((rquired.clone(), "Error"));
        }
        for condition in &missing_conditional {
            result.push((condition.clone(), severity));
        }

        let features = result
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
        Ok(features)
    }
}

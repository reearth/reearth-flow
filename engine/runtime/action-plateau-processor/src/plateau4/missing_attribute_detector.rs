use crate::object_list::ObjectListMap;

use super::errors::PlateauProcessorError;
use fastxml::transform::StreamTransformer;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use quick_xml::{events::Event, Reader};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::Cell;
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

        let root_info = parse_root_info(xml_content)?;
        let gml_id = root_info.gml_id.ok_or(PlateauProcessorError::MissingAttributeDetector(
            "Failed to get gml id".to_string(),
        ))?;
        let feature_type = root_info.qname;
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
                let xpath_with_prefix = format!("//{xpath}");
                            let exists = stream_exists(xml_content, &xpath_with_prefix)?;
                if exists {
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
                            let xpath = format!("//{}", paths[0]);
                            let exists = stream_exists(xml_content, &xpath)?;
                            if !exists {
                                missing_required.push(paths[0].clone());
                            }
                        }
                        2.. => {
                            let mut hit = true;
                            for p in &paths[..paths.len() - 1] {
                                let xpath = format!("//{p}");
                                let exists = stream_exists(xml_content, &xpath)?;
                                if !exists {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit {
                                let xpath = format!("//{}", paths[paths.len() - 1]);
                                let exists = stream_exists(xml_content, &xpath)?;
                                if !exists {
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
                            let xpath = format!("//{}", paths[0]);
                            let exists = stream_exists(xml_content, &xpath)?;
                            if !exists {
                                missing_conditional.push(paths[0].clone());
                            }
                        }
                        2.. => {
                            let mut hit = true;
                            for p in &paths[..paths.len() - 1] {
                                let xpath = format!("//{p}");
                                let exists = stream_exists(xml_content, &xpath)?;
                                if !exists {
                                    hit = false;
                                    break;
                                }
                            }
                            if hit {
                                let xpath = format!("//{}", paths[paths.len() - 1]);
                                let exists = stream_exists(xml_content, &xpath)?;
                                if !exists {
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
            let xpath = format!("/*/{xpath}");
            let exists = stream_exists(xml_content, &xpath)?;
            if !exists {
                "Error"
            } else {
                "Warn"
            }
        } else {
            "Warn"
        };

        // C07/C08 Data Quality Attribute validation
        let lod_count = count_lod_geometries(xml_content, package)?;
        let (c07_errors, c08_errors) =
            validate_data_quality_attributes(xml_content, &lod_count, package)?;

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

fn count_lod_geometries(
    raw_xml: &str,
    package: &str,
) -> Result<[usize; 5], PlateauProcessorError> {
    let mut lod_count = [0; 5];

    if package == "dem" {
        // Special handling for DEM package
        let xpath = "/*/dem:lod";
        let texts = stream_collect_texts(raw_xml, xpath)?;
        if let Some(text) = texts.first() {
            if let Ok(lod) = text.trim().parse::<usize>() {
                if lod <= 4 {
                    lod_count[lod] += 1;
                }
            }
        }
    } else {
        // General LOD pattern matching for other packages
        let xpath = "/*/*";
        let tags = stream_collect_names(raw_xml, xpath)?;
        for tag in tags {
            if let Some(captures) = LOD_TAG_PATTERN.captures(&tag) {
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

fn uro_xpath(local_name: &str) -> String {
    format!("//*[local-name()='{local_name}']")
}

#[allow(clippy::type_complexity)]
fn validate_data_quality_attributes(
    raw_xml: &str,
    lod_count: &[usize; 5],
    _package: &str,
) -> Result<(Vec<(usize, String)>, Vec<(usize, String)>), PlateauProcessorError> {
    let mut c07_errors = Vec::new();
    let mut c08_errors = Vec::new();

    // Find DataQualityAttribute section (nested under bldgDataQualityAttribute)
    let data_quality_xpath = uro_xpath("DataQualityAttribute");
    if stream_exists(raw_xml, &data_quality_xpath)? {
        // Check each LOD that has geometry elements
        for (lod, &count) in lod_count.iter().enumerate() {
            if count > 0 {
                // C07: Check for geometrySrcDescLod{N}
                let geom_src_desc_xpath = uro_xpath(&format!("geometrySrcDescLod{lod}"));
                let geom_texts = stream_collect_texts(raw_xml, &geom_src_desc_xpath)?;

                if geom_texts.is_empty() {
                    // Missing geometrySrcDescLod{N} - C07 error
                    c07_errors.push((
                        lod,
                        format!("uro:DataQualityAttribute/uro:geometrySrcDescLod{lod}"),
                    ));
                } else {
                    // Check if value is "000" for C08 validation
                    let text = geom_texts.first().cloned().unwrap_or_default();
                    if !text.is_empty() && text.trim() == "000" {
                        // C08: Check PublicSurveyDataQualityAttribute sub-elements
                        // Check srcScaleLod{N}
                        let src_scale_xpath = uro_xpath(&format!("srcScaleLod{lod}"));
                        let src_scale_exists = stream_exists(raw_xml, &src_scale_xpath)?;

                        if !src_scale_exists {
                            c08_errors.push((
                                lod,
                                format!(
                                    "uro:PublicSurveyDataQualityAttribute/uro:srcScaleLod{lod}"
                                ),
                            ));
                        }

                        // Check publicSurveySrcDescLod{N}
                        let public_survey_src_desc_xpath =
                            uro_xpath(&format!("publicSurveySrcDescLod{lod}"));
                        let public_survey_exists =
                            stream_exists(raw_xml, &public_survey_src_desc_xpath)?;

                        if !public_survey_exists {
                            c08_errors.push((
                                lod,
                                format!(
                                    "uro:PublicSurveyDataQualityAttribute/uro:publicSurveySrcDescLod{lod}"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok((c07_errors, c08_errors))
}

#[derive(Clone, Debug)]
struct RootInfo {
    qname: String,
    gml_id: Option<String>,
}

fn parse_root_info(raw_xml: &str) -> Result<RootInfo, PlateauProcessorError> {
    let mut reader = Reader::from_str(raw_xml);
    reader.config_mut().trim_text(false);
    reader.config_mut().expand_empty_elements = true;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = e.name();
                let qname = std::str::from_utf8(name.as_ref()).map_err(|e| {
                    PlateauProcessorError::MissingAttributeDetector(format!("{e:?}"))
                })?;
                let mut gml_id = None;

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = std::str::from_utf8(attr.key.as_ref()).map_err(|e| {
                        PlateauProcessorError::MissingAttributeDetector(format!("{e:?}"))
                    })?;
                    let value = attr.unescape_value().map_err(|e| {
                        PlateauProcessorError::MissingAttributeDetector(format!("{e:?}"))
                    })?;

                    if key == "id" || key == "gml:id" {
                        gml_id = Some(value.to_string());
                    }
                }

                return Ok(RootInfo {
                    qname: qname.to_string(),
                    gml_id,
                });
            }
            Ok(Event::Eof) => {
                return Err(PlateauProcessorError::MissingAttributeDetector(
                    "Failed to parse XML root element".to_string(),
                ))
            }
            Err(err) => {
                return Err(PlateauProcessorError::MissingAttributeDetector(format!(
                    "Failed to parse XML root element: {err:?}"
                )))
            }
            _ => {}
        }
        buf.clear();
    }
}

fn stream_exists(raw_xml: &str, xpath: &str) -> Result<bool, PlateauProcessorError> {
    let found = Cell::new(false);
    let transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;
    transformer
        .on(xpath, |_| {
            found.set(true);
        })
        .for_each()
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to evaluate xpath '{xpath}': {e:?}"
            ))
        })?;
    Ok(found.get())
}

fn stream_collect_texts(raw_xml: &str, xpath: &str) -> Result<Vec<String>, PlateauProcessorError> {
    let transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;
    transformer
        .collect(xpath, |node| node.get_content().unwrap_or_default())
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to evaluate xpath '{xpath}': {e:?}"
            ))
        })
}

fn stream_collect_names(
    raw_xml: &str,
    xpath: &str,
) -> Result<Vec<String>, PlateauProcessorError> {
    let transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;
    transformer
        .collect(xpath, |node| node.name())
        .map_err(|e| {
            PlateauProcessorError::MissingAttributeDetector(format!(
                "Failed to evaluate xpath '{xpath}': {e:?}"
            ))
        })
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

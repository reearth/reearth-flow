use std::{collections::HashMap, str::FromStr, sync::Arc};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reearth_flow_common::{
    uri::Uri,
    xml::{self, XmlDocument, XmlRoNode},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::errors::PlateauProcessorError;

use super::{
    types::SchemaFeature,
    utils::{create_codelist_map, generate_xpath_to_properties},
};

pub(crate) static FILE_PATH_PORT: Lazy<Port> = Lazy::new(|| Port::new("filePath"));
pub(crate) static ATTRIBUTE_FEATURE_PORT: Lazy<Port> = Lazy::new(|| Port::new("attributeFeature"));
pub(crate) static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));

#[derive(Debug, Default, Clone)]
struct Attributes(HashMap<String, serde_json::Value>);

impl Attributes {
    fn new() -> Self {
        Default::default()
    }

    fn to_hash_map(&self) -> HashMap<String, serde_json::Value> {
        self.0.clone()
    }

    fn set(&mut self, key: String, value: serde_json::Value) {
        self.0.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    fn append(&mut self, key: String, value: serde_json::Value) {
        match self.0.get(&key) {
            Some(serde_json::Value::Array(arr)) => {
                let mut arr = arr.clone();
                arr.push(value);
                self.0.insert(key, serde_json::Value::Array(arr));
            }
            _ => {
                self.0.insert(key, serde_json::Value::Array(vec![value]));
            }
        }
    }

    fn iter(&self) -> std::collections::hash_map::Iter<'_, String, serde_json::Value> {
        self.0.iter()
    }
}

impl TryFrom<Attributes> for serde_json::Value {
    type Error = PlateauProcessorError;
    fn try_from(value: Attributes) -> Result<Self, PlateauProcessorError> {
        serde_json::to_value(value.0).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot convert to json with error = {e:?}"
            ))
        })
    }
}

impl TryFrom<Attributes> for AttributeValue {
    type Error = PlateauProcessorError;
    fn try_from(value: Attributes) -> Result<Self, PlateauProcessorError> {
        Ok(AttributeValue::from(
            serde_json::to_value(value.0).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot convert to json with error = {e:?}"
                ))
            })?,
        ))
    }
}

struct SchemaDef(HashMap<String, SchemaFeature>);

impl SchemaDef {
    fn new(value: HashMap<String, SchemaFeature>) -> Self {
        Self(value)
    }

    fn get(&self, key: &str) -> Option<&SchemaFeature> {
        self.0.get(key)
    }
}

static GEN_ATTR_TYPES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("gen:stringAttribute", "string"),
        ("gen:intAttribute", "int"),
        ("gen:doubleAttribute", "double"),
        ("gen:dateAttribute", "date"),
        ("gen:uriAttribute", "uri"),
        ("gen:measureAttribute", "measure"),
        ("gen:genericAttributeSet", "attributeSet"),
    ])
});

static LOD_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r".+:lod(\d).+").unwrap());

static GENERIC_TAG_SET: &str = "gen:genericAttributeSet";

static NON_NUMERIC_TAGS: Lazy<Vec<&'static str>> = Lazy::new(|| vec!["uro:lodType"]);
static NON_NUMERIC_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\+-]?(\d+\.?\d*|\.\d+)[eE][\+-]?\d+").unwrap());
static IGNORE_TYPES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "xs:string",
        "gml:CodeType",
        "xs:date",
        "xs:gYear",
        "xs:anyURI",
    ]
});
static INFINITY_STRS: Lazy<Vec<&'static str>> = Lazy::new(|| vec!["inf", "-inf", "nan"]);

static PART_FEATURE_TYPES: Lazy<Vec<&'static str>> =
    Lazy::new(|| vec!["bldg:BuildingPart", "brid:BridgePart", "tun:TunnelPart"]);

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct GenericAttribute {
    r#type: String,
    name: String,
    value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    uom: Option<String>,
}

impl TryFrom<GenericAttribute> for serde_json::Value {
    type Error = PlateauProcessorError;
    fn try_from(value: GenericAttribute) -> Result<Self, PlateauProcessorError> {
        serde_json::to_value(value).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot convert to json with error = {e:?}"
            ))
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct LodCount {
    lod0: i32,
    lod1: i32,
    lod2: i32,
    lod3: i32,
    lod4: i32,
}

impl LodCount {
    fn new() -> Self {
        Default::default()
    }

    fn plus_str(&mut self, lod: String) {
        match lod.as_str() {
            "0" => self.lod0 += 1,
            "1" => self.lod1 += 1,
            "2" => self.lod2 += 1,
            "3" => self.lod3 += 1,
            "4" => self.lod4 += 1,
            _ => (),
        }
    }
    fn plus_i32(&mut self, lod: i32) {
        match lod {
            0 => self.lod0 += 1,
            1 => self.lod1 += 1,
            2 => self.lod2 += 1,
            3 => self.lod3 += 1,
            4 => self.lod4 += 1,
            _ => (),
        }
    }

    fn plus(&mut self, other: &Self) {
        self.lod1 += other.lod1;
        self.lod2 += other.lod2;
        self.lod3 += other.lod3;
        self.lod4 += other.lod4;
    }

    fn max_lod(&self) -> i32 {
        if self.lod4 > 0 {
            4
        } else if self.lod3 > 0 {
            3
        } else if self.lod2 > 0 {
            2
        } else if self.lod1 > 0 {
            1
        } else {
            0
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FilePathResponse {
    root: String,
    package: String,
    admin: String,
    area: String,
    udx_dirs: String,
    city_gml_path: String,
    mesh_code: String,
    city_code: String,
    city_name: String,
    feature_types: Vec<String>,
    max_lod: i32,
}

impl TryFrom<FilePathResponse> for AttributeValue {
    type Error = PlateauProcessorError;

    fn try_from(value: FilePathResponse) -> Result<Self, PlateauProcessorError> {
        Ok(AttributeValue::from(serde_json::to_value(value).map_err(
            |e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot convert to json with error = {e:?}"
                ))
            },
        )?))
    }
}

impl TryFrom<FilePathResponse> for Feature {
    type Error = PlateauProcessorError;

    fn try_from(value: FilePathResponse) -> Result<Self, PlateauProcessorError> {
        let attributes = serde_json::to_value(value).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot convert to json with error = {e:?}"
            ))
        })?;
        let attributes = match attributes {
            serde_json::Value::Object(v) => v
                .into_iter()
                .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v)))
                .collect(),
            _ => IndexMap::new(),
        };
        Ok(Feature::new_with_attributes(attributes))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct FeatureResponse {
    gml_id: String,
    root: String,
    package: String,
    udx_dirs: String,
    city_gml_path: String,
    mesh_code: String,
    city_code: String,
    city_name: String,
    feature_type: String,
    attributes: HashMap<String, serde_json::Value>,
    num_lod0: i32,
    num_lod1: i32,
    num_lod2: i32,
    num_lod3: i32,
    num_lod4: i32,
    xml_id: String,
    xml_parent_id: Option<String>,
}

impl TryFrom<FeatureResponse> for AttributeValue {
    type Error = PlateauProcessorError;

    fn try_from(value: FeatureResponse) -> Result<Self, PlateauProcessorError> {
        Ok(AttributeValue::from(serde_json::to_value(value).map_err(
            |e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot convert to json with error = {e:?}"
                ))
            },
        )?))
    }
}

impl TryFrom<FeatureResponse> for Feature {
    type Error = PlateauProcessorError;

    fn try_from(value: FeatureResponse) -> Result<Self, PlateauProcessorError> {
        let attributes = serde_json::to_value(value).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot convert to json with error = {e:?}"
            ))
        })?;
        let attributes = match attributes {
            serde_json::Value::Object(v) => v
                .into_iter()
                .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v)))
                .collect(),
            _ => IndexMap::new(),
        };
        Ok(Feature::new_with_attributes(attributes))
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct SummaryResponse {
    num_features: i32,
    max_lod: i32,
    codelists: HashMap<String, HashMap<String, String>>,
    xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>>,
}

impl TryFrom<SummaryResponse> for AttributeValue {
    type Error = PlateauProcessorError;

    fn try_from(value: SummaryResponse) -> Result<Self, PlateauProcessorError> {
        Ok(AttributeValue::from(serde_json::to_value(value).map_err(
            |e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot convert to json with error = {e:?}"
                ))
            },
        )?))
    }
}

impl TryFrom<SummaryResponse> for Feature {
    type Error = PlateauProcessorError;

    fn try_from(value: SummaryResponse) -> Result<Self, PlateauProcessorError> {
        let attributes = serde_json::to_value(value).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot convert to json with error = {e:?}"
            ))
        })?;
        let attributes = match attributes {
            serde_json::Value::Object(v) => v
                .into_iter()
                .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v)))
                .collect(),
            _ => IndexMap::new(),
        };
        Ok(Feature::new_with_attributes(attributes))
    }
}

impl SummaryResponse {
    fn new(
        codelists_setting: &HashMap<String, HashMap<String, String>>,
        xpath_to_properties_setting: &HashMap<String, HashMap<String, SchemaFeature>>,
    ) -> Self {
        let mut codelists = HashMap::new();
        for (key, value) in codelists_setting.iter() {
            codelists.insert(key.clone(), value.clone());
        }
        let mut xpath_to_properties = HashMap::new();
        for (key, value) in xpath_to_properties_setting.iter() {
            xpath_to_properties.insert(key.clone(), value.clone());
        }
        Self {
            codelists,
            xpath_to_properties,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct XmlAttributeExtractorFactory;

impl ProcessorFactory for XmlAttributeExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU3.XMLAttributeExtractor"
    }

    fn description(&self) -> &str {
        "Extracts attributes from XML fragments based on a schema definition"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(XmlAttributeExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            ATTRIBUTE_FEATURE_PORT.clone(),
            SUMMARY_PORT.clone(),
            FILE_PATH_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: XmlAttributeExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::XmlAttributeExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let xpath_to_properties = {
            let schema_json = params.schema_json.clone().ok_or(
                PlateauProcessorError::XmlAttributeExtractorFactory(
                    "Missing required parameter `with`".to_string(),
                ),
            )?;
            let dm_geom_to_xml = params
                .extract_dm_geometry_as_xml_fragment
                .unwrap_or_default();
            generate_xpath_to_properties(schema_json, dm_geom_to_xml)?
        };
        let except_feature_types = params.except_feature_types.clone().unwrap_or_default();
        let process = XmlAttributeExtractor {
            xpath_to_properties,
            except_feature_types,
            codelists_map: HashMap::new(),
            features_group: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct XmlAttributeExtractor {
    xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>>,
    except_feature_types: Vec<String>,
    codelists_map: HashMap<String, HashMap<String, HashMap<String, String>>>,
    features_group: HashMap<String, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct XmlAttributeExtractorParam {
    city_code: Option<String>,
    target_packages: Option<Vec<String>>,
    add_nsprefix_to_feature_types: Option<bool>,
    except_feature_types: Option<Vec<String>>,
    extract_dm_geometry_as_xml_fragment: Option<bool>,
    schema_json: Option<String>,
}

impl Processor for XmlAttributeExtractor {
    fn num_threads(&self) -> usize {
        5
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        // Codelist dictionary creation
        let dir_codelists = match feature.get(&Attribute::new("dirCodelists")) {
            Some(AttributeValue::String(dir)) => dir,
            v => {
                return Err(PlateauProcessorError::XmlAttributeExtractor(format!(
                    "No dirCodelists value with {v:?}"
                ))
                .into())
            }
        };
        if !self.codelists_map.contains_key(dir_codelists) {
            let dir = Uri::from_str(dir_codelists).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot parse uri with error = {e:?}"
                ))
            })?;
            if dir.is_dir() {
                let codelists = create_codelist_map(Arc::clone(&ctx.storage_resolver), &dir)?;
                if !codelists.is_empty() {
                    self.codelists_map
                        .insert(dir_codelists.to_string(), codelists);
                }
            }
        }
        if let Some(AttributeValue::String(city_gml_path)) =
            feature.attributes.get(&Attribute::new("cityGmlPath"))
        {
            self.features_group
                .entry(city_gml_path.to_string())
                .or_default()
                .push(feature.clone());
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let codelists = self
            .codelists_map
            .iter()
            .fold(HashMap::new(), |mut acc, (_k, v)| {
                acc.extend(v.clone());
                acc
            });
        let mut summary = SummaryResponse::new(&codelists, &self.xpath_to_properties);
        let mut size = 0;

        for (city_gml_path, value) in self.features_group.iter() {
            let city_gml_path = Uri::from_str(city_gml_path).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot create uri with error = {e:?}"
                ))
            })?;
            let mut xml_id_to_feature_and_attribute = HashMap::<String, (Uuid, Attributes)>::new();
            let mut part_features = Vec::<(Uuid, String)>::new();
            let mut total_lod = LodCount::new();
            let mut row_map =
                HashMap::<Uuid, (IndexMap<Attribute, AttributeValue>, Attributes)>::new();
            let first = value
                .first()
                .ok_or(PlateauProcessorError::XmlAttributeExtractor(
                    "No Value".to_string(),
                ))?;
            for row in value.iter() {
                let feature = &row.attributes;
                let row_id = Uuid::new_v4();
                let xml_fragment = match feature.get(&Attribute::new("xmlFragment")).ok_or(
                    PlateauProcessorError::XmlAttributeExtractor("No xmlFragment".to_string()),
                )? {
                    AttributeValue::String(document) => document,
                    _ => {
                        return Err(PlateauProcessorError::XmlAttributeExtractor(
                            "Invalid Input. supported only String".to_string(),
                        )
                        .into())
                    }
                };
                let document = xml::parse(xml_fragment).map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot parse xml with error = {e:?}"
                    ))
                })?;
                let context = xml::create_context(&document).map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot create context with error = {e:?}"
                    ))
                })?;
                let gid = context.evaluate("/*/@gml:id");
                if gid.is_err() {
                    continue;
                }
                let gid = xml::collect_text_value(&gid.unwrap());
                let mcode = match feature.get(&Attribute::new("meshCode")) {
                    Some(AttributeValue::String(mcode)) => mcode,
                    _ => {
                        return Err(PlateauProcessorError::XmlAttributeExtractor(
                            "Invalid Input. supported only String".to_string(),
                        )
                        .into())
                    }
                };
                let all_node = xml::evaluate(&document, "/*").map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot evaluate xml with error = {e:?}"
                    ))
                })?;
                let all_node = all_node.get_readonly_nodes_as_vec();
                let root = all_node
                    .first()
                    .ok_or(PlateauProcessorError::XmlAttributeExtractor(
                        "No Root Node".to_string(),
                    ))?;
                let tag = xml::get_readonly_node_tag(root);
                let schema_def = self.xpath_to_properties.get(tag.as_str());
                if schema_def.is_none() || self.except_feature_types.contains(&tag) {
                    continue;
                }
                let schema_def = SchemaDef::new(schema_def.unwrap().clone());
                let (mut attr, lod) = walk_node(
                    &city_gml_path,
                    &codelists,
                    &schema_def,
                    &document,
                    root,
                    tag.clone(),
                    true,
                )
                .map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot walk node with error = {e:?}"
                    ))
                })?;
                let Some(lod) = lod else {
                    continue;
                };
                total_lod.plus(&lod);
                attr.set("gml:Id".to_string(), serde_json::Value::String(gid.clone()));
                attr.set(
                    "meshCode".to_string(),
                    serde_json::Value::String(mcode.clone()),
                );
                let xml_id = feature
                    .get(&Attribute::new("xmlId"))
                    .map(|v| match v {
                        AttributeValue::String(xml_id) => xml_id,
                        _ => "",
                    })
                    .ok_or(PlateauProcessorError::XmlAttributeExtractor(
                        "No xml Id".to_string(),
                    ))?;
                attr.set(
                    "featureType".to_string(),
                    serde_json::Value::String(tag.clone()),
                );
                xml_id_to_feature_and_attribute.insert(xml_id.to_string(), (row_id, attr.clone()));
                let mut result_feature = feature.clone();
                result_feature.insert(
                    Attribute::new("featureType"),
                    AttributeValue::String(tag.clone()),
                );
                result_feature.insert(
                    Attribute::new("meshCode"),
                    AttributeValue::String(mcode.clone()),
                );
                result_feature.insert(Attribute::new("gmlId"), AttributeValue::String(gid.clone()));
                result_feature.insert(
                    Attribute::new("lod0"),
                    AttributeValue::Number(serde_json::Number::from(lod.lod0)),
                );
                result_feature.insert(
                    Attribute::new("lod1"),
                    AttributeValue::Number(serde_json::Number::from(lod.lod1)),
                );
                result_feature.insert(
                    Attribute::new("lod2"),
                    AttributeValue::Number(serde_json::Number::from(lod.lod2)),
                );
                result_feature.insert(
                    Attribute::new("lod3"),
                    AttributeValue::Number(serde_json::Number::from(lod.lod3)),
                );
                result_feature.insert(
                    Attribute::new("lod4"),
                    AttributeValue::Number(serde_json::Number::from(lod.lod4)),
                );
                row_map.insert(row_id, (result_feature, attr.clone()));
                if PART_FEATURE_TYPES.contains(&tag.as_str()) {
                    let xml_parent_id = feature
                        .get(&Attribute::new("xmlParentId"))
                        .map(|v| match v {
                            AttributeValue::String(xml_parent_id) => xml_parent_id,
                            _ => "",
                        })
                        .ok_or(PlateauProcessorError::XmlAttributeExtractor(
                            "No xml Parent Id".to_string(),
                        ))?;
                    part_features.push((row_id, xml_parent_id.to_string()));
                }
            }
            for (_row_id, xml_parent_id) in part_features {
                let target = xml_id_to_feature_and_attribute.get(&xml_parent_id);
                if let Some((row_id, parent_attr)) = target {
                    let (_feature, attr) = row_map.get_mut(row_id).unwrap();
                    for (key, value) in parent_attr.iter() {
                        if !attr.contains_key(key.as_str()) {
                            attr.set(key.clone(), value.clone());
                        }
                    }
                }
            }
            for (_xml_id, (row_id, attr)) in xml_id_to_feature_and_attribute.iter() {
                let (feature, _attr) = row_map.get(row_id).unwrap();
                let Some(xml_parent_id) = feature
                    .get(&Attribute::new("xmlParentId"))
                    .filter(|v| match v {
                        AttributeValue::String(v) => !v.is_empty(),
                        _ => false,
                    })
                    .map(|v| match v {
                        AttributeValue::String(v) => v,
                        _ => "",
                    })
                else {
                    continue;
                };
                let ancestors = ancestor_attributes(
                    xml_parent_id.to_string(),
                    &xml_id_to_feature_and_attribute,
                    &row_map,
                )
                .map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot get ancestor attributes with error = {e:?}"
                    ))
                })?;
                let attr = if !ancestors.is_empty() {
                    let mut attr = attr.clone();
                    let ancestor = ancestors.first().unwrap().clone();
                    attr.set(
                        "ancestors".to_string(),
                        serde_json::Value::Array(
                            ancestors
                                .into_iter()
                                .map(|x| x.try_into().unwrap())
                                .collect(),
                        ),
                    );
                    let feature_type = ancestor
                        .get("featureType")
                        .map(|v| match v {
                            serde_json::Value::String(v) => v,
                            _ => "",
                        })
                        .ok_or(PlateauProcessorError::XmlAttributeExtractor(
                            "Ancestor has no feature type".to_string(),
                        ))?;
                    if ["bldg:Building", "bldg:BuildingPart"].contains(&feature_type)
                        && attr
                            .get("featureType")
                            .map(|v| match v {
                                serde_json::Value::String(v) => v,
                                _ => "",
                            })
                            .unwrap_or_default()
                            != "bldg:BuildingPart"
                    {
                        attr.set(
                            "uro:BuildingIDAttribute".to_string(),
                            ancestor
                                .get("uro:BuildingIDAttribute")
                                .unwrap_or(&serde_json::Value::Null)
                                .clone(),
                        );
                    }
                    attr
                } else {
                    attr.clone()
                };
                let (feature, _attr) = row_map.get_mut(row_id).unwrap();
                feature.insert(Attribute::new("attributes"), attr.try_into().unwrap());
            }
            for (_, (feature, _attr)) in row_map.iter() {
                size += 1;
                let feature_response = create_feature_response(&city_gml_path, feature);
                let feature: Feature = feature_response.try_into()?;
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    ATTRIBUTE_FEATURE_PORT.clone(),
                ));
            }
            // create file path response
            let max_lod = total_lod.max_lod();
            let file_path = create_file_path_response(&city_gml_path, first, max_lod);
            let file_path: Feature = file_path.try_into()?;
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                file_path,
                FILE_PATH_PORT.clone(),
            ));
            if summary.max_lod < max_lod {
                summary.max_lod = max_lod;
            }
        }
        summary.num_features = size;
        let summary: Feature = summary.try_into()?;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            &ctx,
            summary,
            SUMMARY_PORT.clone(),
        ));
        Ok(())
    }

    fn name(&self) -> &str {
        "XmlAttributeExtractor"
    }
}

fn create_feature_response(
    city_gml_path: &Uri,
    feature: &IndexMap<Attribute, AttributeValue>,
) -> FeatureResponse {
    FeatureResponse {
        gml_id: feature
            .get(&Attribute::new("gmlId"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        root: feature
            .get(&Attribute::new("root"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        package: feature
            .get(&Attribute::new("package"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        udx_dirs: feature
            .get(&Attribute::new("udxDirs"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        city_gml_path: city_gml_path.to_string(),
        mesh_code: feature
            .get(&Attribute::new("meshCode"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        city_code: feature
            .get(&Attribute::new("cityCode"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        city_name: feature
            .get(&Attribute::new("cityName"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        feature_type: feature
            .get(&Attribute::new("featureType"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        attributes: feature
            .get(&Attribute::new("attributes"))
            .map(|v| match v {
                AttributeValue::Map(v) => {
                    v.clone().into_iter().map(|(k, v)| (k, v.into())).collect()
                }
                _ => HashMap::new(),
            })
            .unwrap_or_default(),
        num_lod0: feature
            .get(&Attribute::new("lod1"))
            .map(|v| match v {
                AttributeValue::Number(v) => v.as_i64().unwrap_or(0) as i32,
                _ => 0,
            })
            .unwrap_or_default(),

        num_lod1: feature
            .get(&Attribute::new("lod1"))
            .map(|v| match v {
                AttributeValue::Number(v) => v.as_i64().unwrap_or(0) as i32,
                _ => 0,
            })
            .unwrap_or_default(),
        num_lod2: feature
            .get(&Attribute::new("lod2"))
            .map(|v| match v {
                AttributeValue::Number(v) => v.as_i64().unwrap_or(0) as i32,
                _ => 0,
            })
            .unwrap_or_default(),
        num_lod3: feature
            .get(&Attribute::new("lod3"))
            .map(|v| match v {
                AttributeValue::Number(v) => v.as_i64().unwrap_or(0) as i32,
                _ => 0,
            })
            .unwrap_or_default(),
        num_lod4: feature
            .get(&Attribute::new("lod4"))
            .map(|v| match v {
                AttributeValue::Number(v) => v.as_i64().unwrap_or(0) as i32,
                _ => 0,
            })
            .unwrap_or_default(),
        xml_id: feature
            .get(&Attribute::new("xmlId"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        xml_parent_id: feature
            .get(&Attribute::new("xmlParentId"))
            .map(|v| match v {
                AttributeValue::String(v) => Some(v.to_string()),
                _ => None,
            })
            .unwrap_or_default(),
    }
}

fn create_file_path_response(
    city_gml_path: &Uri,
    feature: &Feature,
    max_lod: i32,
) -> FilePathResponse {
    let feature = &feature.attributes;
    FilePathResponse {
        root: feature
            .get(&Attribute::new("root"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        package: feature
            .get(&Attribute::new("package"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        admin: feature
            .get(&Attribute::new("admin"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        area: feature
            .get(&Attribute::new("area"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        udx_dirs: feature
            .get(&Attribute::new("udxDirs"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        city_gml_path: city_gml_path.to_string(),
        mesh_code: feature
            .get(&Attribute::new("meshCode"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        city_code: feature
            .get(&Attribute::new("cityCode"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        city_name: feature
            .get(&Attribute::new("cityName"))
            .map(|v| match v {
                AttributeValue::String(v) => v.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default(),
        feature_types: feature
            .get(&Attribute::new("featureTypes"))
            .map(|v| match v {
                AttributeValue::Array(v) => v
                    .iter()
                    .map(|v| match v {
                        AttributeValue::String(v) => v.to_string(),
                        _ => "".to_string(),
                    })
                    .collect(),
                _ => Vec::new(),
            })
            .unwrap_or_default(),
        max_lod,
    }
}

fn ancestor_attributes(
    xml_parent_id: String,
    xml_id_to_feature_and_attribute: &HashMap<String, (Uuid, Attributes)>,
    rows_map: &HashMap<Uuid, (IndexMap<Attribute, AttributeValue>, Attributes)>,
) -> super::errors::Result<Vec<Attributes>> {
    let mut result = Vec::new();
    let (row_id, attr) = match xml_id_to_feature_and_attribute.get(xml_parent_id.as_str()) {
        Some((row_id, attr)) => (row_id, attr),
        None => return Ok(result),
    };
    result.insert(0, attr.clone());
    let feature_type = attr
        .get("featureType")
        .map(|v| match v {
            serde_json::Value::String(v) => v,
            _ => "",
        })
        .ok_or(PlateauProcessorError::XmlAttributeExtractor(
            "No Feature Type".to_string(),
        ))?;
    if PART_FEATURE_TYPES.contains(&feature_type) {
        return Ok(result);
    }
    let (row, _attr) = rows_map
        .get(row_id)
        .ok_or(PlateauProcessorError::XmlAttributeExtractor(
            "No Parent".to_string(),
        ))?;
    let xml_parent_id = row
        .get(&Attribute::new("xmlParentId"))
        .map(|v| match v {
            AttributeValue::String(v) => v,
            _ => "",
        })
        .ok_or(PlateauProcessorError::XmlAttributeExtractor(
            "No Parent Id".to_string(),
        ))?;
    let mut parent = ancestor_attributes(
        xml_parent_id.to_string(),
        xml_id_to_feature_and_attribute,
        rows_map,
    )
    .map_err(|e| {
        PlateauProcessorError::XmlAttributeExtractor(format!(
            "Cannot get ancestor attributes with error = {e:?}"
        ))
    })?;
    parent.append(&mut result);
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn walk_node(
    city_gml_path: &Uri,
    codelists: &HashMap<String, HashMap<String, String>>,
    schema_def: &SchemaDef,
    document: &XmlDocument,
    parent: &XmlRoNode,
    xpath: String,
    _lod_count: bool,
) -> super::errors::Result<(Attributes, Option<LodCount>)> {
    let ctx = xml::create_context(document).map_err(|e| {
        PlateauProcessorError::XmlAttributeExtractor(format!(
            "Cannot create context with error = {e:?}"
        ))
    })?;
    let nodes = xml::find_readonly_nodes_by_xpath(&ctx, "./*", parent).map_err(|e| {
        PlateauProcessorError::XmlAttributeExtractor(format!(
            "Cannot evaluate xml with error = {e:?}"
        ))
    })?;
    let mut lod_count = LodCount::new();
    let mut result = Attributes::new();
    let attr_types = GEN_ATTR_TYPES.keys().copied().collect::<Vec<_>>();
    for node in nodes {
        let tag = xml::get_readonly_node_tag(&node);
        if let Some(cap) = LOD_PATTERN.captures(tag.as_str()) {
            let lod = cap.get(1).map(|lod| lod.as_str());
            if let Some(v) = lod {
                lod_count.plus_str(v.to_string());
            }
        } else if tag == "dem:load" {
            let value = node.get_content();
            let lod = value.parse::<i32>();
            if let Ok(lod) = lod {
                lod_count.plus_i32(lod);
            }
        }
        if attr_types.contains(&tag.as_str()) {
            let generic_attribute = walk_generic_node(document, &node).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot walk generic node with error = {e:?}"
                ))
            })?;
            if result.contains_key("gen:genericAttribute") {
                let mut target = result
                    .get("gen:genericAttribute")
                    .map(|v| match v {
                        serde_json::Value::Array(v) => v.clone(),
                        _ => vec![],
                    })
                    .unwrap_or_default();
                target.extend(generic_attribute.into_iter().flat_map(|x| x.try_into()));
                result.set(
                    "gen:genericAttribute".to_string(),
                    serde_json::Value::Array(target),
                );
            } else {
                result.set(
                    "gen:genericAttribute".to_string(),
                    serde_json::Value::Array(
                        generic_attribute
                            .into_iter()
                            .map(|x| serde_json::to_value(x).unwrap())
                            .collect(),
                    ),
                );
            }
            continue;
        }
        let props = schema_def.get(format!("{xpath}/{tag}").as_str());
        if props.is_none() {
            continue;
        }
        let props = props.unwrap();
        let flag = props.flag.clone().unwrap_or("<unknown>".to_string());
        if ["feature", "geometry"].contains(&flag.as_str()) {
            continue;
        }
        let tpe = props.r#type.clone();
        let multi = match props.max_occurs.as_str() {
            "unbounded" => true,
            max_occurs => max_occurs.to_string().parse::<i32>().unwrap_or(0) > 1,
        };
        if tpe == "core:AddressPropertyType" {
            let address = get_address(document, &node).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot get address with error = {e:?}"
                ))
            })?;
            result.set(tag, serde_json::Value::String(address));
            continue;
        }
        match flag.as_str() {
            "fragment" => {
                let fragment = xml::readonly_node_to_xml_string(document, &node).map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot convert node to xml with error = {e:?}"
                    ))
                })?;
                result.set(tag, serde_json::Value::String(fragment));
                continue;
            }
            "role" => {
                let (attr, _lod) = walk_node(
                    city_gml_path,
                    codelists,
                    schema_def,
                    document,
                    &node,
                    format!("{xpath}/{tag}"),
                    false,
                )
                .map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot walk node with error = {e:?}"
                    ))
                })?;
                result.extend(attr);
                continue;
            }
            "parent" => {
                let (attr, _lod) = walk_node(
                    city_gml_path,
                    codelists,
                    schema_def,
                    document,
                    &node,
                    format!("{xpath}/{tag}"),
                    false,
                )
                .map_err(|e| {
                    PlateauProcessorError::XmlAttributeExtractor(format!(
                        "Cannot walk node with error = {e:?}"
                    ))
                })?;
                if multi {
                    result.append(
                        tag,
                        serde_json::to_value(attr.to_hash_map()).map_err(|e| {
                            PlateauProcessorError::XmlAttributeExtractor(format!(
                                "Cannot convert to json with error = {e:?}"
                            ))
                        })?,
                    );
                } else {
                    result.set(
                        tag,
                        serde_json::to_value(attr.to_hash_map()).map_err(|e| {
                            PlateauProcessorError::XmlAttributeExtractor(format!(
                                "Cannot convert to json with error = {e:?}"
                            ))
                        })?,
                    );
                }
                continue;
            }
            _ => (),
        }
        // XML要素の値
        let text = node.get_content();
        if text.is_empty() {
            continue;
        }
        let code_space = match node.get_attribute_node("codeSpace") {
            Some(attr) => attr.get_content(),
            None => "".to_string(),
        };
        if code_space.is_empty() {
            let codelist = city_gml_path.join(code_space.as_str()).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot join uri with error = {e:?}"
                ))
            })?;
            let codelist = codelists.get(codelist.to_string().as_str());
            let value = match codelist {
                Some(codelist) => codelist.get(text.as_str()).cloned().unwrap_or(text.clone()),
                None => text.clone(),
            };
            if multi {
                result.append(tag.clone(), serde_json::Value::String(value));
                result.append(
                    format!("{}_code", tag.clone()),
                    serde_json::Value::String(text.clone()),
                );
            } else {
                result.set(tag.clone(), serde_json::Value::String(value));
                result.set(
                    format!("{}_code", tag.clone()),
                    serde_json::Value::String(text.clone()),
                );
            }
        } else {
            let value = if IGNORE_TYPES.contains(&tpe.as_str())
                || NON_NUMERIC_PATTERN.is_match(&text)
                || INFINITY_STRS.contains(&text.to_lowercase().as_str())
                || NON_NUMERIC_TAGS.contains(&tag.as_str())
            {
                serde_json::Value::String(text)
            } else {
                text.parse::<i32>()
                    .map(|x| serde_json::Value::Number(serde_json::Number::from(x)))
                    .unwrap_or_else(|_| serde_json::Value::String(text))
            };
            if multi {
                result.append(tag.clone(), value);
            } else {
                result.set(tag.clone(), value);
            }
        }
        if let Some(uom) = node.get_attribute_node("uom") {
            if multi {
                result.append(
                    format!("{}_uom", tag.clone()),
                    serde_json::Value::String(uom.get_content()),
                );
            } else {
                result.set(
                    format!("{}_uom", tag.clone()),
                    serde_json::Value::String(uom.get_content()),
                );
            }
        }
    }
    Ok((result, Some(lod_count)))
}

fn walk_generic_node(
    document: &XmlDocument,
    node: &XmlRoNode,
) -> super::errors::Result<Vec<GenericAttribute>> {
    let tag = xml::get_readonly_node_tag(node);
    let typ = match GEN_ATTR_TYPES.get(tag.as_str()) {
        Some(typ) => typ,
        None => "unknown",
    };
    let mut result = Vec::new();
    if tag == *GENERIC_TAG_SET {
        let ctx = xml::create_context(document).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot create context with error = {e:?}"
            ))
        })?;
        let children = xml::find_readonly_nodes_by_xpath(&ctx, "./*", node).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot evaluate xml with error = {e:?}"
            ))
        })?;
        let mut attribute_set = Vec::new();
        for child in children {
            let attributes = walk_generic_node(document, &child).map_err(|e| {
                PlateauProcessorError::XmlAttributeExtractor(format!(
                    "Cannot walk generic node with error = {e:?}"
                ))
            })?;
            attribute_set.extend(attributes);
        }
        if !attribute_set.is_empty() {
            result.push(GenericAttribute {
                r#type: typ.to_string(),
                name: node.get_name(),
                value: serde_json::Value::Array(
                    attribute_set
                        .into_iter()
                        .map(|x| serde_json::to_value(x).unwrap())
                        .collect(),
                ),
                uom: None,
            });
        }
    } else {
        let ctx = xml::create_context(document).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot create context with error = {e:?}"
            ))
        })?;
        let nodes = xml::find_readonly_nodes_by_xpath(&ctx, "./gen:value", node).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot evaluate xml with error = {e:?}"
            ))
        })?;
        let value = nodes
            .first()
            .ok_or(PlateauProcessorError::XmlAttributeExtractor(
                "No Value".to_string(),
            ))?;
        result.push(GenericAttribute {
            r#type: typ.to_string(),
            name: node
                .get_attribute_node("name")
                .map(|attr| attr.get_content())
                .unwrap_or_default(),
            value: serde_json::Value::String(value.get_content()),
            uom: if typ == "measure" {
                value
                    .get_attribute_node("uom")
                    .map(|attr| attr.get_content())
            } else {
                None
            },
        });
    }
    Ok(result)
}

fn get_address(document: &XmlDocument, node: &XmlRoNode) -> super::errors::Result<String> {
    let ctx = xml::create_context(document).map_err(|e| {
        PlateauProcessorError::XmlAttributeExtractor(format!(
            "Cannot create context with error = {e:?}"
        ))
    })?;
    let nodes =
        xml::find_readonly_nodes_by_xpath(&ctx, ".//xAL:LocalityName", node).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot evaluate xml with error = {e:?}"
            ))
        })?;
    let mut result = Vec::<String>::new();
    nodes.iter().for_each(|node| {
        result.push(node.get_content());
    });
    let nodes =
        xml::find_readonly_nodes_by_xpath(&ctx, ".//xAL:DependentLocality", node).map_err(|e| {
            PlateauProcessorError::XmlAttributeExtractor(format!(
                "Cannot evaluate xml with error = {e:?}"
            ))
        })?;
    for node in nodes {
        let attribute_node = node.get_attribute_node("Type");
        match attribute_node {
            Some(attribute_node) if attribute_node.get_content() == "district" => {
                let nodes = xml::find_readonly_nodes_by_xpath(&ctx, "./*", &attribute_node)
                    .map_err(|e| {
                        PlateauProcessorError::XmlAttributeExtractor(format!(
                            "Cannot evaluate xml with error = {e:?}"
                        ))
                    })?;
                nodes.iter().for_each(|node| {
                    result.push(node.get_content());
                });
            }
            _ => (),
        }
    }
    Ok(result.join(""))
}

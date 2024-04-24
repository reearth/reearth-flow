use std::collections::HashMap;
use std::str::FromStr;

use itertools::{self, Itertools};
use once_cell::sync::Lazy;
use reearth_flow_action::{
    error, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue, Port, Result,
    DEFAULT_PORT,
};
use reearth_flow_action::{Attribute, Dataframe, Feature};
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;
use reearth_flow_common::xml::XmlDocument;
use reearth_flow_common::xml::XmlNode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::SchemaFeature;
use super::types::Settings;
use super::types::DICTIONARIES_INITIATOR_SETTINGS_PORT;

pub(crate) static FILE_PATH_PORT: Lazy<Port> = Lazy::new(|| Port::new("file_path"));
pub(crate) static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));

#[derive(Default, Clone)]
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
        !self.0.contains_key(key)
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

    fn iter(&self) -> std::collections::hash_map::Iter<String, serde_json::Value> {
        self.0.iter()
    }
}

impl TryFrom<Attributes> for serde_json::Value {
    type Error = error::Error;
    fn try_from(value: Attributes) -> Result<Self, error::Error> {
        serde_json::to_value(value.0).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })
    }
}

impl TryFrom<Attributes> for AttributeValue {
    type Error = error::Error;
    fn try_from(value: Attributes) -> Result<Self, error::Error> {
        Ok(AttributeValue::from(
            serde_json::to_value(value.0).map_err(|e| {
                error::Error::output(format!("Cannot convert to json with error = {:?}", e))
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

static GEN_ATTR_TYPES: Lazy<HashMap<&'static str, String>> = Lazy::new(|| {
    HashMap::from([
        ("gen:stringAttribute", "string".to_string()),
        ("gen:intAttribute", "int".to_string()),
        ("gen:doubleAttribute", "double".to_string()),
        ("gen:dateAttribute", "date".to_string()),
        ("gen:uriAttribute", "uri".to_string()),
        ("gen:measureAttribute", "measure".to_string()),
        ("gen:genericAttributeSet", "attributeSet".to_string()),
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
    uom: Option<String>,
}

impl TryFrom<GenericAttribute> for serde_json::Value {
    type Error = error::Error;
    fn try_from(value: GenericAttribute) -> Result<Self, error::Error> {
        serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct LodCount {
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
            "1" => self.lod1 += 1,
            "2" => self.lod2 += 1,
            "3" => self.lod3 += 1,
            "4" => self.lod4 += 1,
            _ => (),
        }
    }
    fn plus_i32(&mut self, lod: i32) {
        match lod {
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
    type Error = error::Error;

    fn try_from(value: FilePathResponse) -> Result<Self, error::Error> {
        Ok(AttributeValue::from(serde_json::to_value(value).map_err(
            |e| error::Error::output(format!("Cannot convert to json with error = {:?}", e)),
        )?))
    }
}

impl TryFrom<FilePathResponse> for Feature {
    type Error = error::Error;

    fn try_from(value: FilePathResponse) -> Result<Self, error::Error> {
        let attributes = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        Ok(attributes.into())
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
    num_lod1: i32,
    num_lod2: i32,
    num_lod3: i32,
    num_lod4: i32,
    xml_id: String,
    xml_parent_id: Option<String>,
}

impl TryFrom<FeatureResponse> for AttributeValue {
    type Error = error::Error;

    fn try_from(value: FeatureResponse) -> Result<Self, error::Error> {
        Ok(AttributeValue::from(serde_json::to_value(value).map_err(
            |e| error::Error::output(format!("Cannot convert to json with error = {:?}", e)),
        )?))
    }
}

impl TryFrom<FeatureResponse> for Feature {
    type Error = error::Error;

    fn try_from(value: FeatureResponse) -> Result<Self, error::Error> {
        let attributes = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        Ok(attributes.into())
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
    type Error = error::Error;

    fn try_from(value: SummaryResponse) -> Result<Self, error::Error> {
        Ok(AttributeValue::from(serde_json::to_value(value).map_err(
            |e| error::Error::output(format!("Cannot convert to json with error = {:?}", e)),
        )?))
    }
}

impl TryFrom<SummaryResponse> for Dataframe {
    type Error = error::Error;

    fn try_from(value: SummaryResponse) -> Result<Self, error::Error> {
        let attributes = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        let feature: Feature = attributes.into();
        Ok(Dataframe {
            features: vec![feature],
        })
    }
}

impl SummaryResponse {
    fn new(settings: &Settings) -> Self {
        let mut codelists = HashMap::new();
        for (key, value) in settings.codelists.iter() {
            codelists.insert(key.clone(), value.clone());
        }
        let mut xpath_to_properties = HashMap::new();
        for (key, value) in settings.xpath_to_properties.iter() {
            xpath_to_properties.insert(key.clone(), value.clone());
        }
        Self {
            codelists,
            xpath_to_properties,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XmlAttributeExtractor;

#[async_trait::async_trait]
#[typetag::serde(name = "PLATEAU.XMLAttributeExtractor")]
impl AsyncAction for XmlAttributeExtractor {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(error::Error::input("No Default Port"))?;
        let settings = inputs
            .get(&DICTIONARIES_INITIATOR_SETTINGS_PORT)
            .ok_or(error::Error::input("No Settings"))?;
        let settings: Settings = settings
            .clone()
            .try_into()
            .map_err(|e| error::Error::input(format!("Invalid Settings. {}", e)))?;
        let mut result = Vec::<FeatureResponse>::new();
        let mut file_path_responses = Vec::<FilePathResponse>::new();
        let mut summary = SummaryResponse::new(&settings);
        let part = input
            .features
            .iter()
            .group_by(
                |&row| match row.attributes.get(&Attribute::new("cityGmlPath")) {
                    Some(AttributeValue::String(city_gml_path)) => city_gml_path,
                    _ => "",
                },
            )
            .into_iter()
            .map(|(key, group)| (key, group.collect::<Vec<_>>()))
            .collect::<Vec<_>>();
        for (city_gml_path, value) in part {
            let city_gml_path = Uri::from_str(city_gml_path).map_err(|e| {
                error::Error::internal_runtime(format!("Cannot create uri with error = {:?}", e))
            })?;
            let mut xml_id_to_feature_and_attribute = HashMap::<String, (Uuid, Attributes)>::new();
            let mut part_features = Vec::<(Uuid, String)>::new();
            let mut total_lod = LodCount::new();
            let mut row_map =
                HashMap::<Uuid, (HashMap<Attribute, AttributeValue>, Attributes)>::new();
            let first = value.first().ok_or(error::Error::input("No Value"))?;
            for row in &value {
                let feature = &row.attributes;
                let row_id = Uuid::new_v4();
                let xml_fragment = match feature
                    .get(&Attribute::new("xmlFragment"))
                    .ok_or(error::Error::input("No xmlFragment"))?
                {
                    AttributeValue::String(document) => document,
                    _ => return Err(error::Error::input("Invalid Input. supported only String")),
                };
                let document = xml::parse(xml_fragment).map_err(|e| {
                    error::Error::internal_runtime(format!("Cannot parse xml with error = {:?}", e))
                })?;
                let context = xml::create_context(&document).map_err(|e| {
                    error::Error::internal_runtime(format!(
                        "Cannot create context with error = {:?}",
                        e
                    ))
                })?;
                let gid = context.evaluate("/*/@gml:id");
                if gid.is_err() {
                    continue;
                }
                let gid = xml::collect_text_value(&gid.unwrap());
                let mcode = match feature.get(&Attribute::new("meshCode")) {
                    Some(AttributeValue::String(mcode)) => mcode,
                    _ => return Err(error::Error::input("Invalid Input. supported only String")),
                };
                let all_node = xml::evaluate(&document, "/*").map_err(|e| {
                    error::Error::internal_runtime(format!(
                        "Cannot evaluate xml with error = {:?}",
                        e
                    ))
                })?;
                let all_node = all_node.get_nodes_as_vec();
                let root = all_node
                    .first()
                    .ok_or(error::Error::input("No Root Node"))?;
                let tag = root.get_name();
                let schema_def = settings.xpath_to_properties.get(tag.as_str());
                if schema_def.is_none() || settings.except_feature_types.contains(&tag) {
                    continue;
                }
                let schema_def = SchemaDef::new(schema_def.unwrap().clone());
                let (mut attr, lod) = walk_node(
                    &ctx,
                    &city_gml_path,
                    &settings,
                    &schema_def,
                    &document,
                    root,
                    tag.clone(),
                )
                .map_err(|e| {
                    error::Error::internal_runtime(format!("Cannot walk node with error = {:?}", e))
                })?;
                total_lod.plus(&lod);
                let xml_id = feature
                    .get(&Attribute::new("xmlId"))
                    .map(|v| match v {
                        AttributeValue::String(xml_id) => xml_id,
                        _ => "",
                    })
                    .ok_or(error::Error::input("No xml Id"))?;
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
                        .ok_or(error::Error::input("No xml Parent Id"))?;
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
                let xml_parent_id = feature
                    .get(&Attribute::new("xmlParentId"))
                    .map(|v| match v {
                        AttributeValue::String(v) => v,
                        _ => "",
                    })
                    .ok_or(error::Error::input("No Parent Id"))?;
                let ancestors = ancestor_attributes(
                    xml_parent_id.to_string(),
                    &xml_id_to_feature_and_attribute,
                    &row_map,
                )
                .map_err(|e| {
                    error::Error::internal_runtime(format!(
                        "Cannot get ancestor attributes with error = {:?}",
                        e
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
                        .ok_or(error::Error::validate("Ancestor has no feature type"))?;
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
            result.append(
                &mut row_map
                    .into_iter()
                    .map(|(_, (feature, _attr))| create_feature_response(&city_gml_path, &feature))
                    .collect(),
            );
            // create file path response
            let max_lod = total_lod.max_lod();
            file_path_responses.push(create_file_path_response(&city_gml_path, first, max_lod));
            if summary.max_lod < max_lod {
                summary.max_lod = max_lod;
            }
        }

        let mut ports = ActionDataframe::new();
        summary.num_features = result.len() as i32;
        ports.insert(SUMMARY_PORT.clone(), summary.try_into()?);
        let features = file_path_responses
            .into_iter()
            .map(|x| x.try_into().unwrap())
            .collect::<Vec<Feature>>();
        ports.insert(FILE_PATH_PORT.clone(), Dataframe::new(features));
        ports.insert(
            DEFAULT_PORT.clone(),
            Dataframe::new(result.into_iter().map(|x| x.try_into().unwrap()).collect()),
        );
        Ok(ports)
    }
}

fn create_feature_response(
    city_gml_path: &Uri,
    feature: &HashMap<Attribute, AttributeValue>,
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
    rows_map: &HashMap<Uuid, (HashMap<Attribute, AttributeValue>, Attributes)>,
) -> Result<Vec<Attributes>> {
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
        .ok_or(error::Error::input("No Feature Type"))?;
    if PART_FEATURE_TYPES.contains(&feature_type) {
        return Ok(result);
    }
    let (row, _attr) = rows_map
        .get(row_id)
        .ok_or(error::Error::input("No Parent"))?;
    let xml_parent_id = row
        .get(&Attribute::new("xmlParentId"))
        .map(|v| match v {
            AttributeValue::String(v) => v,
            _ => "",
        })
        .ok_or(error::Error::input("No Parent Id"))?;
    let mut parent = ancestor_attributes(
        xml_parent_id.to_string(),
        xml_id_to_feature_and_attribute,
        rows_map,
    )
    .map_err(|e| {
        error::Error::internal_runtime(format!(
            "Cannot get ancestor attributes with error = {:?}",
            e
        ))
    })?;
    parent.append(&mut result);
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn walk_node(
    action_ctx: &ActionContext,
    city_gml_path: &Uri,
    settings: &Settings,
    schema_def: &SchemaDef,
    document: &XmlDocument,
    parent: &XmlNode,
    xpath: String,
) -> Result<(Attributes, LodCount)> {
    let ctx = xml::create_context(document).map_err(|e| {
        error::Error::internal_runtime(format!("Cannot create context with error = {:?}", e))
    })?;
    let nodes = ctx.node_evaluate("./*", parent).map_err(|e| {
        error::Error::internal_runtime(format!("Cannot evaluate xml with error = {:?}", e))
    })?;
    let nodes = nodes.get_nodes_as_vec();
    let mut lod_count = LodCount::new();
    let mut result = Attributes::new();

    for node in nodes {
        let tag = xml::get_node_tag(&node);
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
        let attr_types = GEN_ATTR_TYPES.keys().copied().collect::<Vec<_>>();
        if attr_types.contains(&tag.as_str()) {
            let generic_attribute = walk_generic_node(document, &node).map_err(|e| {
                error::Error::internal_runtime(format!(
                    "Cannot walk generic node with error = {:?}",
                    e
                ))
            })?;
            result.set(
                "gen:genericAttribute".to_string(),
                serde_json::Value::Array(
                    generic_attribute
                        .into_iter()
                        .map(|x| serde_json::to_value(x).unwrap())
                        .collect(),
                ),
            );
            continue;
        }
        let props = schema_def.get(format!("{}/{}", xpath, tag).as_str());
        if props.is_none() {
            action_ctx.action_log(format!(
                "Not found properties of xpath = {} tag = {}",
                xpath, tag
            ));
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
                error::Error::internal_runtime(format!("Cannot get address with error = {:?}", e))
            })?;
            result.set(tag, serde_json::Value::String(address));
            continue;
        }
        match flag.as_str() {
            "fragment" => {
                let mut node = node.clone();
                let fragment = xml::node_to_xml_string(document, &mut node).map_err(|e| {
                    error::Error::internal_runtime(format!(
                        "Cannot convert node to xml with error = {:?}",
                        e
                    ))
                })?;
                result.set(tag, serde_json::Value::String(fragment));
                continue;
            }
            "role" => {
                let (attr, lod) = walk_node(
                    action_ctx,
                    city_gml_path,
                    settings,
                    schema_def,
                    document,
                    &node,
                    xpath.clone(),
                )
                .map_err(|e| {
                    error::Error::internal_runtime(format!("Cannot walk node with error = {:?}", e))
                })?;
                result.extend(attr);
                lod_count.plus(&lod);
                continue;
            }
            "parent" => {
                let (attr, lod) = walk_node(
                    action_ctx,
                    city_gml_path,
                    settings,
                    schema_def,
                    document,
                    &node,
                    xpath.clone(),
                )
                .map_err(|e| {
                    error::Error::internal_runtime(format!("Cannot walk node with error = {:?}", e))
                })?;
                lod_count.plus(&lod);
                if multi {
                    result.append(
                        tag,
                        serde_json::to_value(attr.to_hash_map()).map_err(|e| {
                            error::Error::internal_runtime(format!(
                                "Cannot convert to json with error = {:?}",
                                e
                            ))
                        })?,
                    );
                } else {
                    result.set(
                        tag,
                        serde_json::to_value(attr.to_hash_map()).map_err(|e| {
                            error::Error::internal_runtime(format!(
                                "Cannot convert to json with error = {:?}",
                                e
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
                error::Error::internal_runtime(format!("Cannot join uri with error = {:?}", e))
            })?;
            let codelist = settings.codelists.get(codelist.to_string().as_str());
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
    Ok((result, lod_count))
}

fn walk_generic_node(document: &XmlDocument, node: &XmlNode) -> Result<Vec<GenericAttribute>> {
    let tag = xml::get_node_tag(node);
    let typ = match GEN_ATTR_TYPES.get(tag.as_str()) {
        Some(typ) => typ.clone(),
        None => "unknown".to_string(),
    };
    let mut result = Vec::new();
    if tag == *GENERIC_TAG_SET {
        let ctx = xml::create_context(document).map_err(|e| {
            error::Error::internal_runtime(format!("Cannot create context with error = {:?}", e))
        })?;
        let nodes = ctx.node_evaluate("./*", node).map_err(|e| {
            error::Error::internal_runtime(format!("Cannot evaluate xml with error = {:?}", e))
        })?;
        let children = nodes.get_nodes_as_vec();
        let mut attribute_set = Vec::new();
        for child in children {
            let attributes = walk_generic_node(document, &child).map_err(|e| {
                error::Error::internal_runtime(format!(
                    "Cannot walk generic node with error = {:?}",
                    e
                ))
            })?;
            attribute_set.extend(attributes);
        }
        if !attribute_set.is_empty() {
            result.push(GenericAttribute {
                r#type: typ.to_string(),
                name: tag.clone(),
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
            error::Error::internal_runtime(format!("Cannot create context with error = {:?}", e))
        })?;
        let nodes = ctx.node_evaluate("./gen:value", node).map_err(|e| {
            error::Error::internal_runtime(format!("Cannot evaluate xml with error = {:?}", e))
        })?;
        let nodes = nodes.get_nodes_as_vec();
        let value = nodes.first().ok_or(error::Error::input("No Value"))?;
        result.push(GenericAttribute {
            r#type: typ.to_string(),
            name: tag.clone(),
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

fn get_address(document: &XmlDocument, node: &XmlNode) -> Result<String> {
    let ctx = xml::create_context(document).map_err(|e| {
        error::Error::internal_runtime(format!("Cannot create context with error = {:?}", e))
    })?;
    let nodes = ctx
        .node_evaluate(".//xAL:LocalityName", node)
        .map_err(|e| {
            error::Error::internal_runtime(format!("Cannot evaluate xml with error = {:?}", e))
        })?;
    let mut result = Vec::<String>::new();
    let nodes = nodes.get_nodes_as_vec();
    nodes.iter().for_each(|node| {
        result.push(node.get_content());
    });
    let nodes = ctx
        .node_evaluate(".//xAL:DependentLocality", node)
        .map_err(|e| {
            error::Error::internal_runtime(format!("Cannot evaluate xml with error = {:?}", e))
        })?;
    let nodes = nodes.get_nodes_as_vec();
    for node in nodes {
        let attribute_node = node.get_attribute_node("Type");
        match attribute_node {
            Some(attribute_node) if attribute_node.get_content() == "district" => {
                let nodes = ctx.node_evaluate("./*", &attribute_node).map_err(|e| {
                    error::Error::internal_runtime(format!(
                        "Cannot evaluate xml with error = {:?}",
                        e
                    ))
                })?;
                let nodes = nodes.get_nodes_as_vec();
                nodes.iter().for_each(|node| {
                    result.push(node.get_content());
                });
            }
            _ => (),
        }
    }
    Ok(result.join(""))
}

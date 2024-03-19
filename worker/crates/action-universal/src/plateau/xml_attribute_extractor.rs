use std::{collections::HashMap, sync::Arc};

use itertools::{self, Itertools};
use once_cell::sync::Lazy;
use reearth_flow_common::xml;
use reearth_flow_common::xml::XmlDocument;
use reearth_flow_common::xml::XmlNode;
use regex::Regex;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error, utils, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Result,
    DEFAULT_PORT, REJECTED_PORT,
};

use super::types::SchemaFeature;
use super::types::Settings;
use super::types::DICTIONARIES_INITIATOR_SETTINGS_PORT;

#[derive(Default)]
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

    fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_not_empty(&self) -> bool {
        !self.0.is_empty()
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

static GENERIC_TAG: &str = "gen:genericAttribute";
static GENERIC_TAG_SET: &str = "gen:genericAttributeSet";

static NON_NUMERIC_TAGS: Lazy<Vec<&'static str>> = Lazy::new(|| vec!["uro:lodType"]);

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

struct GmlAttribute {
    xml_fragment: String,
    gml_id: String,
    mesh_code: String,
    feature_type: String,
    generic_attribute: Vec<GenericAttribute>,
    attributes: HashMap<String, String>,
}

#[derive(Default)]
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
    fn plus(&mut self, lod: i32) {
        match lod {
            1 => self.lod1 += 1,
            2 => self.lod2 += 1,
            3 => self.lod3 += 1,
            4 => self.lod4 += 1,
            _ => (),
        }
    }
}

struct GroupFeature {
    xml_id_to_fearture_and_attribute: HashMap<String, (SchemaFeature, GmlAttribute)>,
    part_features: Vec<SchemaFeature>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XmlAttributeExtractor;

#[async_trait::async_trait]
#[typetag::serde(name = "XMLAttributeExtractor")]
impl Action for XmlAttributeExtractor {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(error::Error::input("No Input"))?;
        let input = inputs
            .get(DEFAULT_PORT)
            .ok_or(error::Error::input("No Default Port"))?;
        let input = input.as_ref().ok_or(error::Error::input("No Value"))?;
        let settings = inputs
            .get(DICTIONARIES_INITIATOR_SETTINGS_PORT)
            .ok_or(error::Error::input("No Settings"))?;
        let settings = settings
            .as_ref()
            .ok_or(error::Error::input("No Settings Value"))?;
        let settings: Settings = settings
            .clone()
            .try_into()
            .map_err(|e| error::Error::input(format!("Invalid Settings. {}", e)))?;

        let data = match input {
            ActionValue::Array(data) => {
                let part = data
                    .iter()
                    .group_by(|&row| match row {
                        ActionValue::Map(row) => match row.get("cityGmlPath") {
                            Some(ActionValue::String(city_gml_path)) => city_gml_path,
                            _ => "",
                        },
                        _ => "",
                    })
                    .into_iter()
                    .map(|(key, group)| (key, group.collect::<Vec<_>>()))
                    .collect::<Vec<_>>();

                for (city_gml_path, value) in part {
                    for row in value {
                        let feature = match row {
                            ActionValue::Map(row) => row,
                            _ => {
                                return Err(error::Error::input(
                                    "Invalid Input. supported only Map",
                                ))
                            }
                        };
                        let xml_fragment = match feature
                            .get("xmlFragment")
                            .ok_or(error::Error::input("No xmlFragment"))?
                        {
                            ActionValue::String(document) => document,
                            _ => {
                                return Err(error::Error::input(
                                    "Invalid Input. supported only String",
                                ))
                            }
                        };
                        let document = xml::parse(xml_fragment).map_err(|e| {
                            error::Error::internal_runtime(format!(
                                "Cannot parse xml with error = {:?}",
                                e
                            ))
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
                        let mcode = match feature.get("meshCode") {
                            Some(ActionValue::String(mcode)) => mcode,
                            _ => {
                                return Err(error::Error::input(
                                    "Invalid Input. supported only String",
                                ))
                            }
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
                        let schema_def = settings.xpath_to_properties.get(root.get_name().as_str());
                        if schema_def.is_none()
                            || settings.except_feature_types.contains(&root.get_name())
                        {
                            continue;
                        }
                        let gml_attribute = GmlAttribute {
                            xml_fragment: xml_fragment.to_string(),
                            gml_id: gid,
                            mesh_code: mcode.to_string(),
                            feature_type: root.get_name(),
                            generic_attribute: Default::default(),
                            attributes: Default::default(),
                        };
                    }
                }
            }
            _ => return Err(error::Error::validate("Input is not an array")),
        };

        let result = ActionDataframe::new();
        Ok(result)
    }
}

fn walk_node(
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
        let tag = xml::get_node_tag(&node).map_err(|e| {
            error::Error::internal_runtime(format!("Cannot get node tag with error = {:?}", e))
        })?;
        if let Some(cap) = LOD_PATTERN.captures(tag.as_str()) {
            let lod = cap.get(1).map(|lod| lod.as_str());
            if let Some(v) = lod {
                lod_count.plus_str(v.to_string());
            }
        } else if tag == "dem:load" {
            let value = node.get_content();
            let lod = value.parse::<i32>();
            if let Ok(lod) = lod {
                lod_count.plus(lod);
            }
        }
        let attr_types = GEN_ATTR_TYPES.keys().copied().collect::<Vec<_>>();
        if attr_types.contains(&tag.as_str()) {
            let generic_attribute = walk_generic_node(document, &node).map_err(|e| {
                error::Error::internal_runtime(format!("Cannot walk generic node with error = {:?}", e))
            })?;
            result.set("gen:genericAttribute".to_string(), serde_json::Value::Array(
                generic_attribute
                    .into_iter()
                    .map(|x| serde_json::to_value(x).unwrap())
                    .collect(),
            ));
            continue;
        }
        // TODO # 属性のプロパティ取得
    }
    Ok((result, lod_count))
}

fn walk_generic_node(document: &XmlDocument, node: &XmlNode) -> Result<Vec<GenericAttribute>> {
    let tag = xml::get_node_tag(&node).map_err(|e| {
        error::Error::internal_runtime(format!("Cannot get node tag with error = {:?}", e))
    })?;
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

use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_types::{Attribute, AttributeValue};

#[derive(Debug, Clone, Default)]
pub(super) struct Flattener {
    pub(super) risk_to_attribute_definitions: HashMap<String, IndexMap<String, AttributeValue>>,
}

impl Flattener {
    pub(super) fn extract_fld_risk_attribute(
        &mut self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let Some(disaster_risks) = attributes.get("uro:RiverFloodingRiskAttribute") else {
            return HashMap::new();
        };
        let disaster_risks = match disaster_risks {
            AttributeValue::Array(disaster_risks) => disaster_risks,
            AttributeValue::Map(disaster_risks) => {
                &vec![AttributeValue::Map(disaster_risks.clone())]
            }
            _ => return HashMap::new(),
        };
        let mut result = HashMap::new();
        for risk in disaster_risks {
            let risk_obj = match risk.as_map() {
                Some(obj) => obj,
                None => continue,
            };
            let desc = risk_obj.get("uro:description").map(|v| v.to_string());
            let admin = risk_obj.get("uro:adminType").map(|v| v.to_string());
            let scale = risk_obj.get("uro:scale").map(|v| v.to_string());

            if desc.is_none() || admin.is_none() || scale.is_none() {
                continue;
            }
            let desc = desc.unwrap();
            let admin = admin.unwrap();
            let scale = scale.unwrap();

            let basename = format!("{desc}（{admin}管理区間）_{scale}");

            let mut rank = risk_obj.get("uro:rank").map(|v| v.to_string());
            let mut rank_code = risk_obj.get("uro:rank_code").map(|v| v.to_string());
            if rank.is_none() {
                rank = risk_obj.get("uro:rankOrg").map(|v| v.to_string());
                rank_code = risk_obj.get("uro:rankOrg_code").map(|v| v.to_string());
            }

            let depth = risk_obj.get("uro:depth").map(|v| v.to_string());
            let duration = risk_obj.get("uro:duration").map(|v| v.to_string());

            let attribs = vec![
                ("浸水ランク", AttributeValue::default_string(), rank),
                (
                    "浸水ランクコード",
                    AttributeValue::default_number(),
                    rank_code,
                ),
                ("浸水深", AttributeValue::default_number(), depth),
                ("浸水継続時間", AttributeValue::default_number(), duration),
            ];

            for (label, value, value_opt) in attribs {
                if let Some(value_str) = value_opt {
                    let name = format!("{basename}_{label}");
                    result.insert(
                        Attribute::new(name.clone()),
                        AttributeValue::String(value_str),
                    );
                    self.risk_to_attribute_definitions
                        .entry("fld".to_string())
                        .or_default()
                        .insert(name.clone(), value);
                }
            }
        }
        result
    }

    pub(super) fn extract_tnm_htd_ifld_risk_attribute(
        &mut self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let src = [
            ("uro:TsunamiRiskAttribute", "津波浸水想定", "tnm"),
            ("uro:HighTideRiskAttribute", "高潮浸水想定", "htd"),
            ("uro:InlandFloodingRiskAttribute", "内水浸水想定", "ifld"),
            (
                "uro:ReservoirFloodingRiskAttribute",
                "ため池浸水想定",
                "rfld",
            ),
        ];

        let mut result = HashMap::new();
        for (tag, title, package) in src.iter() {
            let Some(disaster_risks) = attributes.get(*tag) else {
                continue;
            };
            let disaster_risks = match disaster_risks {
                AttributeValue::Array(disaster_risks) => disaster_risks,
                AttributeValue::Map(disaster_risks) => {
                    &vec![AttributeValue::Map(disaster_risks.clone())]
                }
                _ => return HashMap::new(),
            };

            for risk_value in disaster_risks {
                let risk_obj = match risk_value.as_map() {
                    Some(obj) => obj,
                    None => continue,
                };

                let desc_opt = risk_obj.get("uro:description").map(|v| v.to_string());
                if desc_opt.is_none() {
                    continue;
                }
                let desc = desc_opt.unwrap();
                let basename = format!("{title}_{desc}");

                let mut rank_opt = risk_obj.get("uro:rank").map(|v| v.to_string());
                let mut rank_code_opt = risk_obj.get("uro:rank_code").map(|v| v.to_string());
                if rank_opt.is_none() {
                    rank_opt = risk_obj.get("uro:rankOrg").map(|v| v.to_string());
                    rank_code_opt = risk_obj.get("uro:rankOrg_code").map(|v| v.to_string());
                }

                let depth_opt = risk_obj.get("uro:depth").map(|v| v.to_string());

                let attribs = vec![
                    ("浸水ランク", rank_opt, AttributeValue::default_string()),
                    (
                        "浸水ランクコード",
                        rank_code_opt,
                        AttributeValue::default_number(),
                    ),
                    ("浸水深", depth_opt, AttributeValue::default_number()),
                ];

                for (label, value_str_opt, value_type) in attribs {
                    if let Some(value_str) = value_str_opt {
                        let name = format!("{basename}_{label}");
                        result.insert(
                            Attribute::new(name.clone()),
                            AttributeValue::String(value_str),
                        );
                        self.risk_to_attribute_definitions
                            .entry(package.to_string())
                            .or_default()
                            .insert(name, value_type);
                    }
                }
            }
        }
        result
    }

    pub(super) fn extract_lsld_risk_attribute(
        &mut self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let Some(disaster_risks) = attributes.get("uro:LandSlideRiskAttribute") else {
            return HashMap::new();
        };
        let disaster_risks = match disaster_risks {
            AttributeValue::Array(disaster_risks) => disaster_risks,
            AttributeValue::Map(disaster_risks) => {
                &vec![AttributeValue::Map(disaster_risks.clone())]
            }
            _ => return HashMap::new(),
        };
        let mut result = HashMap::new();
        for risk_value in disaster_risks {
            let risk_obj = match risk_value.as_map() {
                Some(obj) => obj,
                None => continue,
            };

            let desc_opt = risk_obj.get("uro:description").map(|v| v.to_string());
            let area_type_opt = risk_obj.get("uro:areaType").map(|v| v.to_string());
            let type_code_opt = risk_obj.get("uro:areaType_code").map(|v| v.to_string());

            if desc_opt.is_none() || type_code_opt.is_none() {
                continue;
            }
            let desc = desc_opt.unwrap();
            let type_code_str = type_code_opt.unwrap();

            let type_code_int: i32 = match type_code_str.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if type_code_int > 2 {
                continue;
            }

            let entries = vec![
                (
                    format!("土砂災害リスク_{desc}_区域区分"),
                    area_type_opt.unwrap_or("".to_string()),
                    AttributeValue::default_string(),
                ),
                (
                    format!("土砂災害リスク_{desc}_区域区分コード"),
                    type_code_str,
                    AttributeValue::default_number(),
                ),
            ];

            for (attr_key, attr_value, value_type) in entries {
                if attr_value.is_empty() {
                    continue;
                }
                result.insert(
                    Attribute::new(attr_key.clone()),
                    AttributeValue::String(attr_value.clone()),
                );
                self.risk_to_attribute_definitions
                    .entry("lsld".to_string())
                    .or_default()
                    .insert(attr_key, value_type);
            }
        }
        result
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(super) struct CommonAttributeProcessor {
    max_lod: i64,
    gml_path_to_max_lod: HashMap<String, i64>,
    attribute_to_attribute_type: HashMap<String, String>,
}

impl CommonAttributeProcessor {
    pub(super) fn get_generic_schema(&self) -> HashMap<Attribute, AttributeValue> {
        let mut result = HashMap::new();
        for (key, value) in self.attribute_to_attribute_type.iter() {
            match value.as_str() {
                "string" | "date" => {
                    result.insert(
                        Attribute::new(key.clone()),
                        AttributeValue::default_string(),
                    );
                }
                "int" | "double" | "measure" => {
                    result.insert(
                        Attribute::new(key.clone()),
                        AttributeValue::default_number(),
                    );
                }
                _ => {}
            }
        }
        result
    }
    fn flatten_generic_attribute(
        &mut self,
        attribute: &HashMap<String, AttributeValue>,
        prefix: &str,
    ) -> HashMap<Attribute, AttributeValue> {
        let mut result = HashMap::new();
        if let (Some(AttributeValue::String(name)), Some(AttributeValue::String(typ))) =
            (attribute.get("name"), attribute.get("type"))
        {
            let name = format!("{prefix}{name}");
            let value = attribute.get("value").unwrap_or(&AttributeValue::Null);
            if typ == "attributeSet" {
                if let AttributeValue::Array(attribute_set) = value {
                    for attribute in attribute_set {
                        if let AttributeValue::Map(attribute) = attribute {
                            let prefix = format!("{name}_");
                            self.flatten_generic_attribute(attribute, prefix.as_str());
                        }
                    }
                    return result;
                }
            } else if matches!(value, AttributeValue::Null) {
                return result;
            }
            result.insert(Attribute::new(name.clone()), value.clone());
            self.attribute_to_attribute_type
                .insert(name.clone(), typ.clone());
            if typ == "measure" {
                if let Some(uom) = attribute.get("uom") {
                    let name = format!("{name}_uom");
                    result.insert(Attribute::new(name.clone()), uom.clone());
                    self.attribute_to_attribute_type
                        .insert(name, "string".to_string());
                }
            }
        }
        result
    }

    pub(super) fn flatten_generic_attributes(
        &mut self,
        attribute: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        self.flatten_generic_attribute(attribute, "")
    }

    #[allow(dead_code)]
    fn extract_lod_types(
        &self,
        attrib: &HashMap<String, AttributeValue>,
        parent_tag: &str,
    ) -> HashMap<Attribute, AttributeValue> {
        let mut result = HashMap::new();
        let parent_attr = match attrib.get(parent_tag) {
            Some(AttributeValue::Array(parent_attr)) => parent_attr,
            _ => return result,
        };
        if parent_attr.is_empty() {
            return result;
        }
        let first_element = match &parent_attr[0] {
            AttributeValue::Map(first_element) => first_element,
            _ => return result,
        };
        let lod_types = match first_element.get("uro:lodType") {
            Some(AttributeValue::Array(lod_types)) => lod_types,
            _ => return result,
        };
        for lod_type in lod_types {
            let s = match lod_type.to_string().chars().next() {
                Some(s) => s,
                None => continue,
            };
            if s.to_string() == "2" || s.to_string() == "3" || s.to_string() == "4" {
                let key = format!("lod_type_{s}");
                let attribute_name = Attribute::new(key.clone());
                result.insert(attribute_name, lod_type.clone());
            }
        }
        result
    }
}

pub(super) fn get_value_from_json_path(
    paths: &[&str],
    attrib: &HashMap<String, AttributeValue>,
) -> Option<AttributeValue> {
    let key = paths.first()?;
    let value = attrib.get(*key)?;
    let value = if let AttributeValue::Array(array) = value {
        if let Some(AttributeValue::Map(map)) = &array.first() {
            get_value_from_json_path(&paths[1..], map)
        } else if *key == "uro:lodType" {
            // FIXME: this should be list joining with comma
            Some(AttributeValue::String(value.to_string()))
        } else {
            // take first element
            Some(array.first()?.clone())
        }
    } else if let AttributeValue::Number(num) = value {
        match num.as_i64() {
            Some(num) if num == 9999 || num == -9999 => None,
            _ => Some(value.clone()),
        }
    } else if let AttributeValue::Map(value) = value {
        get_value_from_json_path(&paths[1..], value)
    } else {
        Some(value.clone())
    };
    value
}

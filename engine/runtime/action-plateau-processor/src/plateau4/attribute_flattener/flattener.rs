use std::collections::HashMap;

use reearth_flow_types::{Attribute, AttributeValue};

#[derive(Debug, Clone, Default)]
pub(super) struct Flattener;

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

            let basename = format!("{}（{}管理区間）_{}", desc, admin, scale);

            let mut rank = risk_obj.get("uro:rank").map(|v| v.to_string());
            let mut rank_code = risk_obj.get("uro:rank_code").map(|v| v.to_string());
            if rank.is_none() {
                rank = risk_obj.get("uro:rankOrg").map(|v| v.to_string());
                rank_code = risk_obj.get("uro:rankOrg_code").map(|v| v.to_string());
            }

            let depth = risk_obj.get("uro:depth").map(|v| v.to_string());
            let duration = risk_obj.get("uro:duration").map(|v| v.to_string());

            let attribs = vec![
                ("浸水ランク", rank),
                ("浸水ランクコード", rank_code),
                ("浸水深", depth),
                ("浸水継続時間", duration),
            ];

            for (label, value_opt) in attribs {
                if let Some(value_str) = value_opt {
                    let name = format!("{}_{}", basename, label);
                    result.insert(Attribute::new(name), AttributeValue::String(value_str));
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
            ("uro:TsunamiRiskAttribute", "津波浸水想定"),
            ("uro:HighTideRiskAttribute", "高潮浸水想定"),
            ("uro:InlandFloodingRiskAttribute", "内水浸水想定"),
            ("uro:ReservoirFloodingRiskAttribute", "ため池浸水想定"),
        ];

        let mut result = HashMap::new();
        for (tag, title) in src.iter() {
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
                let basename = format!("{}_{}", title, desc);

                let mut rank_opt = risk_obj.get("uro:rank").map(|v| v.to_string());
                let mut rank_code_opt = risk_obj.get("uro:rank_code").map(|v| v.to_string());
                if rank_opt.is_none() {
                    rank_opt = risk_obj.get("uro:rankOrg").map(|v| v.to_string());
                    rank_code_opt = risk_obj.get("uro:rankOrg_code").map(|v| v.to_string());
                }

                let depth_opt = risk_obj.get("uro:depth").map(|v| v.to_string());

                let attribs = vec![
                    ("浸水ランク", rank_opt),
                    ("浸水ランクコード", rank_code_opt),
                    ("浸水深", depth_opt),
                ];

                for (label, value_str_opt) in attribs {
                    if let Some(value_str) = value_str_opt {
                        let name = format!("{}_{}", basename, label);
                        result.insert(Attribute::new(name), AttributeValue::String(value_str));
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
                    format!("土砂災害リスク_{}_区域区分", desc),
                    area_type_opt.unwrap_or("".to_string()),
                ),
                (
                    format!("土砂災害リスク_{}_区域区分コード", desc),
                    type_code_str,
                ),
            ];

            for (attr_key, attr_value) in entries {
                if attr_value.is_empty() {
                    continue;
                }
                result.insert(
                    Attribute::new(attr_key.clone()),
                    AttributeValue::String(attr_value.clone()),
                );
            }
        }
        result
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(super) struct CommonAttributeProcessor {
    max_lod: i64,
    gml_path_to_max_lod: HashMap<String, i64>,
}

impl CommonAttributeProcessor {
    #[allow(dead_code)]
    fn flatten_generic_attributes(
        &mut self,
        attrib: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        if let Some(AttributeValue::Map(obj_map)) = attrib.get("gen:genericAttribute") {
            obj_map
                .iter()
                .filter(|(k, _)| k.as_str() != "type")
                .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                .collect()
        } else {
            HashMap::new()
        }
    }

    #[allow(dead_code)]
    fn update_max_lod(&mut self, attributes: &HashMap<Attribute, AttributeValue>) {
        let gml_path = match attributes.get(&Attribute::new("cityGmlPath")) {
            Some(AttributeValue::String(gml_path)) => gml_path.clone(),
            _ => return,
        };
        let mut gml_max_lod = *self.gml_path_to_max_lod.get(&gml_path).unwrap_or(&0);
        for lod in 0..5 {
            let key = format!("numLod{}", lod);
            let attribute_name = Attribute::new(key.clone());
            let num_lod = match attributes.get(&attribute_name) {
                Some(AttributeValue::Number(num_lod)) => num_lod,
                _ => continue,
            };
            if num_lod.as_i64().unwrap() > 0 {
                if self.max_lod < lod {
                    self.max_lod = lod;
                }
                if gml_max_lod < lod {
                    gml_max_lod = lod;
                    self.gml_path_to_max_lod
                        .insert(gml_path.clone(), gml_max_lod);
                }
            }
        }
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
                let key = format!("lod_type_{}", s);
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
    if let AttributeValue::Map(map) = &value {
        get_value_from_json_path(&paths[1..], map)
    } else if *key == "uro:lodType" {
        Some(AttributeValue::String(value.to_string()))
    } else {
        Some(value.clone())
    }
}

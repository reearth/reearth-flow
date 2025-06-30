use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use itertools::Itertools;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use serde_json::Value;

use super::errors::PlateauProcessorError;

const DELIM: &str = "_";

static FLATTEN_PREFIXES: Lazy<HashSet<String>> = Lazy::new(|| {
    let flatten_attributes = vec![
        ("gml:name", ""),
        ("bldg:class", ""),
        ("bldg:usage", ""),
        ("bldg:yearOfConstruction", ""),
        ("bldg:measuredHeight", ""),
        ("bldg:storeysAboveGround", ""),
        ("bldg:storeysBelowGround", ""),
        ("bldg:address", ""),
        ("uro:buildingIDAttribute", "uro:buildingID"),
        ("uro:buildingIDAttribute", "uro:branchID"),
        ("uro:buildingIDAttribute", "uro:partID"),
        ("uro:buildingIDAttribute", "uro:prefecture"),
        ("uro:buildingIDAttribute", "uro:city"),
        (
            "uro:buildingDetailAttribute",
            "uro:serialNumberOfBuildingCertification",
        ),
        ("uro:buildingDetailAttribute", "uro:siteArea"),
        ("uro:buildingDetailAttribute", "uro:totalFloorArea"),
        ("uro:buildingDetailAttribute", "uro:buildingFootprintArea"),
        ("uro:buildingDetailAttribute", "uro:buildingRoofEdgeArea"),
        ("uro:buildingDetailAttribute", "uro:buildingStructureType"),
        ("uro:buildingDetailAttribute", "uro:fireproofStructureType"),
        ("uro:buildingDetailAttribute", "uro:urbanPlanType"),
        ("uro:buildingDetailAttribute", "uro:areaClassificationType"),
        ("uro:buildingDetailAttribute", "uro:districtsAndZonesType"),
        ("uro:buildingDetailAttribute", "uro:landUseType"),
        ("uro:buildingDetailAttribute", "uro:vacancy"),
        ("uro:buildingDetailAttribute", "uro:buildingCoverageRate"),
        ("uro:buildingDetailAttribute", "uro:floorAreaRate"),
        (
            "uro:buildingDetailAttribute",
            "uro:specifiedBuildingCoverageRate",
        ),
        ("uro:buildingDetailAttribute", "uro:specifiedFloorAreaRate"),
        ("uro:buildingDetailAttribute", "uro:standardFloorAreaRate"),
        ("uro:buildingDetailAttribute", "uro:buildingHeight"),
        ("uro:buildingDetailAttribute", "uro:eaveHeight"),
        ("uro:buildingDetailAttribute", "uro:surveyYear"),
        ("uro:largeCustomerFacilityAttribute", "uro:class"),
        ("uro:largeCustomerFacilityAttribute", "uro:name"),
        ("uro:largeCustomerFacilityAttribute", "uro:capacity"),
        ("uro:largeCustomerFacilityAttribute", "uro:totalFloorArea"),
        ("uro:largeCustomerFacilityAttribute", "uro:inauguralDate"),
        ("uro:largeCustomerFacilityAttribute", "uro:yearOpened"),
        ("uro:largeCustomerFacilityAttribute", "uro:yearClosed"),
        ("uro:largeCustomerFacilityAttribute", "uro:urbanPlanType"),
        (
            "uro:largeCustomerFacilityAttribute",
            "uro:areaClassificationType",
        ),
        (
            "uro:largeCustomerFacilityAttribute",
            "uro:districtsAndZonesType",
        ),
        ("uro:largeCustomerFacilityAttribute", "uro:landUseType"),
        ("uro:largeCustomerFacilityAttribute", "uro:surveyYear"),
        ("uro:buildingDataQualityAttribute", "uro:lod1HeightType"),
        ("uro:realEstateIDAttribute", "uro:realEstateIDOfBuilding"),
        ("uro:realEstateIDAttribute", "uro:matchingScore"),
    ];
    let mut flatten_prefixes = HashSet::new();
    flatten_attributes.iter().for_each(|(k1, k2)| {
        flatten_prefixes.insert(k1.to_string());
        if !k2.is_empty() {
            flatten_prefixes.insert(format!("{k1}{DELIM}{k2}"));
        }
    });
    flatten_prefixes
});

#[derive(Debug, Clone, Default)]
struct Flattener {
    risk_to_attr_defs: HashMap<String, HashMap<String, String>>,
}

impl Flattener {
    fn new() -> Self {
        Flattener {
            risk_to_attr_defs: HashMap::new(),
        }
    }

    fn extract_fld_risk_attribute(
        &mut self,
        city_gml_attribute: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let disaster_risks = match city_gml_attribute.get("uro:buildingDisasterRiskAttribute") {
            Some(AttributeValue::Array(disaster_risks)) => disaster_risks,
            _ => return HashMap::new(),
        };
        let mut result = HashMap::new();
        for risk in disaster_risks {
            let risk_map = match risk {
                AttributeValue::Map(risk_map) => risk_map,
                _ => continue,
            };
            let desc = risk_map.get("uro:description").map(|v| v.to_string());
            let admin = risk_map.get("uro:adminType").map(|v| v.to_string());
            let scale = risk_map.get("uro:scale").map(|v| v.to_string());

            if desc.is_none() || admin.is_none() || scale.is_none() {
                continue;
            }

            let basename = format!(
                "{}（{}管理区間）_{}",
                desc.unwrap(),
                admin.unwrap(),
                scale.unwrap()
            );

            let rank = risk_map
                .get("uro:rank")
                .or_else(|| risk_map.get("uro:rank"));
            let depth = risk_map.get("uro:depth");
            let duration = risk_map.get("uro:duration");

            let attrib = vec![
                ("浸水ランク", rank),
                ("浸水深", depth),
                ("浸水継続時間", duration),
            ];

            for (k, v) in attrib {
                let value = match v {
                    Some(value) => value,
                    None => continue,
                };
                let name = format!("{basename}{DELIM}{k}");
                let attribute_name = Attribute::new(name.clone());
                result.insert(attribute_name, value.clone());
                self.risk_to_attr_defs.entry("fld".to_string()).or_default();
            }
        }
        result
    }

    pub fn extract_tnm_htd_ifld_risk_attribute(
        &mut self,
        city_gml_attribute: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let mut result = HashMap::new();
        let src = vec![
            ("uro:buildingTsunamiRiskAttribute", "津波浸水想定", "tnm"),
            ("uro:buildingHighTideRiskAttribute", "高潮浸水想定", "htd"),
            (
                "uro:buildingInlandFloodingRiskAttribute",
                "内水浸水想定",
                "ifld",
            ),
        ];

        for (tag, title, package) in src {
            let disaster_risks = match city_gml_attribute.get(tag) {
                Some(AttributeValue::Array(disaster_risks)) => disaster_risks,
                _ => continue,
            };

            for risk in disaster_risks {
                let risk_map = match risk {
                    AttributeValue::Map(risk_map) => risk_map,
                    _ => continue,
                };

                let desc = risk_map.get("uro:description").map(|v| v.to_string());
                if desc.is_none() {
                    continue;
                }

                let basename = format!("{}_{}", title, desc.unwrap());

                let rank = risk_map
                    .get("uro:rank")
                    .or_else(|| risk_map.get("uro:rankOrg"));
                let rank_code = risk_map
                    .get("uro:rank_code")
                    .or_else(|| risk_map.get("uro:rankOrg_code"));
                let depth = risk_map.get("uro:depth");

                let attrib = vec![
                    ("浸水ランク", rank),
                    ("浸水ランクコード", rank_code),
                    ("浸水深", depth),
                ];

                for (k, v) in attrib {
                    let value = match v {
                        Some(value) => value,
                        None => continue,
                    };
                    let name = format!("{basename}_{k}");
                    let attribute_name = Attribute::new(name.clone());
                    result.insert(attribute_name, value.clone());
                    self.risk_to_attr_defs
                        .entry(package.to_string())
                        .or_default();
                }
            }
        }
        result
    }

    pub fn extract_lsld_risk_attribute(
        &mut self,
        city_gml_attribute: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let mut result = HashMap::new();
        let disaster_risks = match city_gml_attribute.get("uro:buildingLandSlideRiskAttribute") {
            Some(AttributeValue::Array(disaster_risks)) => disaster_risks,
            _ => return HashMap::new(),
        };
        for risk in disaster_risks {
            let risk_map = match risk {
                AttributeValue::Map(risk_map) => risk_map,
                _ => continue,
            };
            let desc = risk_map.get("uro:description");
            let area_type = risk_map.get("uro:areaType");
            let area_type_code = risk_map.get("uro:areaType_code");
            let area_type_code_count = match area_type_code {
                Some(AttributeValue::Number(n)) => n.as_i64(),
                _ => None,
            };
            if desc.is_none() || area_type_code.is_none() || area_type_code_count.unwrap() > 2 {
                continue;
            }

            let attrib = vec![
                (
                    format!("土砂災害リスク_{}_区域区分", desc.unwrap()),
                    area_type,
                ),
                (
                    format!("土砂災害リスク_{}_区域区分コード", desc.unwrap()),
                    area_type_code,
                ),
            ];

            for (k, v) in attrib {
                let value = match v {
                    Some(value) => value,
                    None => continue,
                };
                let attribute_name = Attribute::new(k.clone());
                result.insert(attribute_name, value.clone());
                self.risk_to_attr_defs
                    .entry("lsld".to_string())
                    .or_default();
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
struct CommonAttributeProcessor {
    max_lod: i64,
    gml_path_to_max_lod: HashMap<String, i64>,
}

impl CommonAttributeProcessor {
    fn flatten_attribute(key: &str, attribute: &AttributeValue) -> HashMap<String, AttributeValue> {
        if !FLATTEN_PREFIXES.contains(key) {
            return HashMap::from([(key.to_string(), attribute.clone())]);
        }
        match attribute {
            AttributeValue::Array(value) => {
                let mut result = HashMap::new();
                for (i, v) in value.iter().enumerate() {
                    let new_key = if value.len() == 1 {
                        key.to_string()
                    } else {
                        format!("{key}{DELIM}{i}")
                    };
                    let new_value = Self::flatten_attribute(new_key.as_str(), v);
                    result.extend(new_value);
                }
                result
            }
            AttributeValue::Map(value) => {
                let mut result = HashMap::new();
                for (k, v) in value {
                    let new_key = format!("{key}{DELIM}{k}");
                    let new_value = Self::flatten_attribute(new_key.as_str(), v);
                    result.extend(new_value);
                }
                result
            }
            v => HashMap::from([(key.to_string(), v.clone())]),
        }
    }

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

    fn update_max_lod(&mut self, attributes: &IndexMap<Attribute, AttributeValue>) {
        let gml_path = match attributes.get(&Attribute::new("cityGmlPath")) {
            Some(AttributeValue::String(gml_path)) => gml_path.clone(),
            _ => return,
        };
        let mut gml_max_lod = *self.gml_path_to_max_lod.get(&gml_path).unwrap_or(&0);
        for lod in 0..5 {
            let key = format!("numLod{lod}");
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

#[derive(Debug, Clone, Default)]
pub struct AttributeFlattenerFactory;

impl ProcessorFactory for AttributeFlattenerFactory {
    fn name(&self) -> &str {
        "PLATEAU3.AttributeFlattener"
    }

    fn description(&self) -> &str {
        "Flatten attributes for building feature"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let flattener = Flattener::new();
        let common_processor = CommonAttributeProcessor {
            max_lod: 0,
            gml_path_to_max_lod: HashMap::new(),
        };

        let process = AttributeFlattener {
            flattener,
            common_processor,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeFlattener {
    flattener: Flattener,
    common_processor: CommonAttributeProcessor,
}

impl Processor for AttributeFlattener {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(AttributeValue::Map(city_gml_attribute)) = feature.get(&"cityGmlAttributes")
        else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No cityGmlAttributes found with feature id = {:?}",
                feature.id
            ))
            .into());
        };
        let Some(AttributeValue::String(ftype)) = city_gml_attribute.get("type") else {
            return Err(PlateauProcessorError::AttributeFlattener(format!(
                "No type found with feature id = {:?}",
                feature.id
            ))
            .into());
        };
        let mut flattened = HashMap::new();
        if ftype.as_str() == "bldg:Building" {
            flattened.extend(
                self.flattener
                    .extract_fld_risk_attribute(city_gml_attribute),
            );
            flattened.extend(
                self.flattener
                    .extract_tnm_htd_ifld_risk_attribute(city_gml_attribute),
            );
            flattened.extend(
                self.flattener
                    .extract_lsld_risk_attribute(city_gml_attribute),
            );
            flattened.extend(
                self.common_processor
                    .flatten_generic_attributes(city_gml_attribute),
            );
            flattened.extend(
                self.common_processor
                    .extract_lod_types(city_gml_attribute, "uro:buildingDataQualityAttribute"),
            );
        } else {
            // 子要素の場合はルート要素（Building）の属性を抽出してマージする。
            let root_city_gml_attribute = match city_gml_attribute.get("ancestors") {
                Some(AttributeValue::Map(attributes)) => attributes.clone(),
                _ =>
                // default to empty
                {
                    HashMap::new()
                }
            };
            for (name, vroot) in &root_city_gml_attribute {
                let v = flattened.get(&Attribute::new(name));

                let value = if v.is_none() || v == Some(vroot) {
                    vroot.clone()
                } else {
                    AttributeValue::String(format!(
                        "{} {}",
                        vroot.as_string().unwrap_or_default(),
                        v.and_then(|v| v.as_string()).unwrap_or_default()
                    ))
                };
                flattened.insert(Attribute::new(name), value.clone());
            }

            if ftype == "bldg:BuildingPart" {
                flattened.extend(
                    self.flattener
                        .extract_fld_risk_attribute(&root_city_gml_attribute),
                );
                flattened.extend(
                    self.flattener
                        .extract_tnm_htd_ifld_risk_attribute(&root_city_gml_attribute),
                );
                flattened.extend(
                    self.flattener
                        .extract_lsld_risk_attribute(&root_city_gml_attribute),
                );

                flattened.extend(
                    self.flattener
                        .extract_fld_risk_attribute(city_gml_attribute),
                );
                flattened.extend(
                    self.flattener
                        .extract_tnm_htd_ifld_risk_attribute(city_gml_attribute),
                );
                flattened.extend(
                    self.flattener
                        .extract_lsld_risk_attribute(city_gml_attribute),
                );
            }
            self.common_processor.update_max_lod(&feature.attributes);
        }
        // フラットにする属性の設定
        let mut new_city_gml_attribute = IndexMap::new();
        for (k, v) in city_gml_attribute.iter() {
            let new_value = CommonAttributeProcessor::flatten_attribute(k, v);
            new_city_gml_attribute.extend(new_value);
        }
        let mut feature = feature.clone();
        feature.attributes.extend(
            new_city_gml_attribute
                .iter()
                .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                .collect::<IndexMap<Attribute, AttributeValue>>(),
        );
        feature.remove(&Attribute::new("cityGmlAttributes"));
        feature.attributes.extend(flattened);
        let keys = feature.attributes.keys().cloned().collect_vec();
        let attributes = &mut feature.attributes;
        for key in keys.iter() {
            if (key.to_string().starts_with("uro:") || key.to_string().starts_with("bldg:"))
                && key.to_string().ends_with("_type")
            {
                attributes.swap_remove(key);
            }
            if ["gen:genericAttribute", "uro:buildingDisasterRiskAttribute"]
                .contains(&key.to_string().as_str())
            {
                attributes.swap_remove(key);
            }
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFlattener"
    }
}

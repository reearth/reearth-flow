use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde_json::Value;

use super::errors::PlateauProcessorError;

const DELIM: &str = "_";

#[derive(Debug, Clone, Default)]
struct Flattener {
    flatten_attrs: HashMap<&'static str, Vec<&'static str>>,
    existing_flatten_attrs: HashSet<Attribute>,
    risk_to_attr_defs: HashMap<String, HashMap<String, String>>,
    fld_attrs_sorter: HashMap<(i64, i64, i64, i64), String>,
}

impl Flattener {
    fn new() -> Self {
        let flatten_attrs_data = vec![
            ("gml:name", ""),
            ("bldg:class", ""),
            ("bldg:usage", ""),
            ("bldg:yearOfConstruction", ""),
            ("bldg:measuredHeight", ""),
            ("bldg:storeysAboveGround", ""),
            ("bldg:storeysBelowGround", ""),
            ("bldg:address", ""),
            ("uro:BuildingIDAttribute", "uro:buildingID"),
            ("uro:BuildingIDAttribute", "uro:branchID"),
            ("uro:BuildingIDAttribute", "uro:partID"),
            ("uro:BuildingIDAttribute", "uro:prefecture"),
            ("uro:BuildingIDAttribute", "uro:city"),
            (
                "uro:BuildingDetailAttribute",
                "uro:serialNumberOfBuildingCertification",
            ),
            ("uro:BuildingDetailAttribute", "uro:siteArea"),
            ("uro:BuildingDetailAttribute", "uro:totalFloorArea"),
            ("uro:BuildingDetailAttribute", "uro:buildingFootprintArea"),
            ("uro:BuildingDetailAttribute", "uro:buildingRoofEdgeArea"),
            ("uro:BuildingDetailAttribute", "uro:buildingStructureType"),
            ("uro:BuildingDetailAttribute", "uro:fireproofStructureType"),
            ("uro:BuildingDetailAttribute", "uro:urbanPlanType"),
            ("uro:BuildingDetailAttribute", "uro:areaClassificationType"),
            ("uro:BuildingDetailAttribute", "uro:districtsAndZonesType"),
            ("uro:BuildingDetailAttribute", "uro:landUseType"),
            ("uro:BuildingDetailAttribute", "uro:vacancy"),
            ("uro:BuildingDetailAttribute", "uro:buildingCoverageRate"),
            ("uro:BuildingDetailAttribute", "uro:floorAreaRate"),
            (
                "uro:BuildingDetailAttribute",
                "uro:specifiedBuildingCoverageRate",
            ),
            ("uro:BuildingDetailAttribute", "uro:specifiedFloorAreaRate"),
            ("uro:BuildingDetailAttribute", "uro:standardFloorAreaRate"),
            ("uro:BuildingDetailAttribute", "uro:buildingHeight"),
            ("uro:BuildingDetailAttribute", "uro:eaveHeight"),
            ("uro:BuildingDetailAttribute", "uro:surveyYear"),
            ("uro:LargeCustomerFacilityAttribute", "uro:class"),
            ("uro:LargeCustomerFacilityAttribute", "uro:name"),
            ("uro:LargeCustomerFacilityAttribute", "uro:capacity"),
            ("uro:LargeCustomerFacilityAttribute", "uro:totalFloorArea"),
            ("uro:LargeCustomerFacilityAttribute", "uro:inauguralDate"),
            ("uro:LargeCustomerFacilityAttribute", "uro:yearOpened"),
            ("uro:LargeCustomerFacilityAttribute", "uro:yearClosed"),
            ("uro:LargeCustomerFacilityAttribute", "uro:urbanPlanType"),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:areaClassificationType",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:districtsAndZonesType",
            ),
            ("uro:LargeCustomerFacilityAttribute", "uro:landUseType"),
            ("uro:LargeCustomerFacilityAttribute", "uro:surveyYear"),
            ("uro:BuildingDataQualityAttribute", "uro:lod1HeightType"),
            ("uro:RealEstateIDAttribute", "uro:realEstateIDOfBuilding"),
            ("uro:RealEstateIDAttribute", "uro:matchingScore"),
        ];

        let mut flatten_attrs = HashMap::new();
        for (t1, t2) in flatten_attrs_data.clone() {
            flatten_attrs.entry(t1).or_insert_with(Vec::new).push(t2);
        }

        Flattener {
            flatten_attrs,
            existing_flatten_attrs: HashSet::new(),
            risk_to_attr_defs: HashMap::new(),
            fld_attrs_sorter: HashMap::new(),
        }
    }

    fn flatten_attributes(
        &mut self,
        city_gml_attributeute: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let mut attrib = HashMap::new();

        for (t1, t2s) in &self.flatten_attrs {
            if let Some(value) = city_gml_attributeute.get(*t1) {
                if let AttributeValue::Array(arr) = value {
                    if let Some(value) = arr.first() {
                        self.process_value(t1, value, t2s, &mut attrib);
                    }
                } else {
                    self.process_value(t1, value, t2s, &mut attrib);
                }
            }
        }
        self.existing_flatten_attrs.extend(attrib.keys().cloned());
        attrib
    }

    fn process_value(
        &self,
        t1: &str,
        value: &AttributeValue,
        t2s: &[&str],
        attrib: &mut HashMap<Attribute, AttributeValue>,
    ) {
        if let AttributeValue::Map(map) = value {
            for t2 in t2s {
                if let Some(v) = map.get(*t2) {
                    let key = format!("{}{}{}", t1, DELIM, t2);
                    attrib.insert(Attribute::new(key), v.clone());
                }
            }
        } else {
            attrib.insert(Attribute::new(t1.to_string()), value.clone());
        }
    }

    fn extract_fld_risk_attribute(
        &mut self,
        city_gml_attribute: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        let disaster_risks = match city_gml_attribute.get("uro:BuildingRiverFloodingRiskAttribute")
        {
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
                .or_else(|| risk_map.get("uro:rankOrg"));
            let rank_code = risk_map
                .get("uro:rank_code")
                .or_else(|| risk_map.get("uro:rankOrg_code"));
            let depth = risk_map.get("uro:depth");
            let duration = risk_map.get("uro:duration");

            let attrib = vec![
                ("浸水ランク", rank, 1),
                ("浸水ランクコード", rank_code, 2),
                ("浸水深", depth, 3),
                ("浸水継続時間", duration, 4),
            ];

            let desc_code = risk_map.get("uro:description_code");
            if desc_code.is_none() {
                continue;
            }
            let desc_code = match desc_code {
                Some(AttributeValue::Number(n)) => n.as_i64().unwrap(),
                _ => continue,
            };

            let admin_code = risk_map.get("uro:adminType_code");
            let admin_code = match admin_code {
                Some(AttributeValue::Number(n)) => n.as_i64().unwrap(),
                _ => continue,
            };

            let scale_code = risk_map.get("uro:scale_code");
            let scale_code = match scale_code {
                Some(AttributeValue::Number(n)) => n.as_i64().unwrap(),
                _ => continue,
            };

            for (k, v, order) in attrib {
                let value = match v {
                    Some(value) => value,
                    None => continue,
                };
                let name = format!("{}_{}", basename, k);
                let attribute_name = Attribute::new(name.clone());
                result.insert(attribute_name, value.clone());
                self.risk_to_attr_defs.entry("fld".to_string()).or_default();

                self.fld_attrs_sorter
                    .insert((desc_code, admin_code, scale_code, order), name);
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
            ("uro:BuildingTsunamiRiskAttribute", "津波浸水想定", "tnm"),
            ("uro:BuildingHighTideRiskAttribute", "高潮浸水想定", "htd"),
            (
                "uro:BuildingInlandFloodingRiskAttribute",
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
                    let name = format!("{}_{}", basename, k);
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
        let disaster_risks = match city_gml_attribute.get("uro:BuildingLandSlideRiskAttribute") {
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
    gen_attr_to_type: HashMap<String, String>,
    max_lod: i64,
    gml_path_to_max_lod: HashMap<String, i64>,
}

impl CommonAttributeProcessor {
    fn flatten_generic_attributes(
        &mut self,
        attrib: &HashMap<String, AttributeValue>,
    ) -> HashMap<Attribute, AttributeValue> {
        fn flatten(
            feat: &mut HashMap<Attribute, AttributeValue>,
            obj: &HashMap<String, AttributeValue>,
            gen_attr_to_type: &mut HashMap<String, String>,
            prefix: String,
        ) {
            let name = match obj.get("name") {
                Some(AttributeValue::String(name)) => name.clone(),
                _ => return,
            };
            let type_ = match obj.get("type") {
                Some(AttributeValue::String(type_)) => type_.clone(),
                _ => return,
            };
            let name = format!("{}{}", prefix, name);
            let value = obj.get("value");

            if type_ == "attributeSet" {
                let subs = match value {
                    Some(AttributeValue::Array(subs)) => subs,
                    _ => return,
                };
                let new_prefix = format!("{}_{}", name, "");
                for sub in subs {
                    if let AttributeValue::Map(sub_map) = sub {
                        flatten(feat, sub_map, gen_attr_to_type, new_prefix.clone());
                    }
                }
            } else if let Some(value) = value {
                let attribute_name = Attribute::new(name.clone());
                feat.insert(attribute_name, value.clone());
                gen_attr_to_type.insert(name.clone(), type_.clone());

                if type_ == "measure" {
                    let uom = match obj.get("uom") {
                        Some(AttributeValue::String(uom)) => uom.clone(),
                        _ => return,
                    };
                    let name_uom = format!("{}_uom", name);
                    let attribute_name = Attribute::new(name_uom.clone());
                    feat.insert(attribute_name, AttributeValue::String(uom.clone()));
                    gen_attr_to_type.insert(name_uom, "string".to_string());
                }
            }
        }
        let mut result = HashMap::new();
        let generic_attributes = match attrib.get("gen:genericAttribute") {
            Some(AttributeValue::Array(generic_attributes)) => generic_attributes,
            _ => return result,
        };
        for obj in generic_attributes {
            if let AttributeValue::Map(obj_map) = obj {
                flatten(
                    &mut result,
                    obj_map,
                    &mut self.gen_attr_to_type,
                    "".to_string(),
                );
            }
        }
        result
    }

    fn update_max_lod(&mut self, feature: &Feature) {
        let gml_path = match feature.attributes.get(&Attribute::new("cityGmlPath")) {
            Some(AttributeValue::String(gml_path)) => gml_path.clone(),
            _ => return,
        };
        let mut gml_max_lod = *self.gml_path_to_max_lod.get(&gml_path).unwrap_or(&0);
        for lod in 0..5 {
            let key = format!("numLod{}", lod);
            let attribute_name = Attribute::new(key.clone());
            let num_lod = match feature.attributes.get(&attribute_name) {
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
                let key = format!("lod_type_{}", s);
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
        "PLATEAU.AttributeFlattener"
    }

    fn description(&self) -> &str {
        "AttributeFlattener"
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
            gen_attr_to_type: HashMap::new(),
            max_lod: 0,
            gml_path_to_max_lod: HashMap::new(),
        };

        let process = AttributeFlattener {
            flattener,
            common_processor,
            features: Vec::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeFlattener {
    flattener: Flattener,
    common_processor: CommonAttributeProcessor,
    features: Vec<Feature>,
}

impl Processor for AttributeFlattener {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
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
        let mut flattened = self.flattener.flatten_attributes(city_gml_attribute);
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
                    .extract_lod_types(city_gml_attribute, "uro:BuildingDataQualityAttribute"),
            );
        } else {
            // // 子要素の場合はルート要素（Building）の属性を抽出してマージする。
            // let root_ city_gml_attribute = match attributes.get(&Attribute::new("ancestors")) {
            //     Some(AttributeValue::Map(attributes)) => attributes.clone(),
            //     _ =>
            //     // default to empty
            //     {
            //         HashMap::new()
            //     }
            // };
            // let froot = self.flattener.flatten_attributes(root_ city_gml_attribute.clone());
            // for (name, vroot) in &froot {
            //     let v = flattened.get(name);

            //     let value = if v.is_none() || v == Some(vroot) {
            //         vroot.clone()
            //     } else {
            //         AttributeValue::String(format!(
            //             "{} {}",
            //             match &vroot {
            //                 AttributeValue::String(s) => s,
            //                 _ => panic!("Expected AttributeValue::String"),
            //             },
            //             match v {
            //                 Some(AttributeValue::String(ref s)) => s,
            //                 _ => panic!("Expected Some(AttributeValue::String)"),
            //             }
            //         ))
            //     };
            //     flattened.insert(name.clone(), value.clone());
            // }

            // if ftype == &AttributeValue::String("bldg::BuildingPart".to_string()) {
            //     self.flattener
            //         .extract_fld_risk_attribute(feature, root_ city_gml_attribute.clone());
            //     self.flattener
            //         .extract_tnm_htd_ifld_risk_attribute(feature, root_ city_gml_attribute.clone());
            //     self.flattener
            //         .extract_lsld_risk_attribute(feature, root_ city_gml_attribute.clone());

            //     // BuildingPart に記述されている災害リスク属性も抽出する。
            //     // 規格外ではあるが BuildingPart 下位に災害リスク属性が記述されているデータもあるため。
            //     self.flattener
            //         .extract_fld_risk_attribute(feature,  city_gml_attribute.clone());
            //     self.flattener
            //         .extract_tnm_htd_ifld_risk_attribute(feature,  city_gml_attribute.clone());
            //     self.flattener
            //         .extract_lsld_risk_attribute(feature,  city_gml_attribute.clone());
            // }

            // // フラットにする属性の設定
            // for (name, value) in &flattened {
            //     feature
            //         .attributes
            //         .insert(Attribute::new(name.clone()), value.clone());
            // }

            self.common_processor.update_max_lod(feature);
            self.features.push(feature.clone());
        }
        let mut feature = feature.clone();
        feature.attributes.extend(flattened);
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.features.is_empty() {
            return Ok(());
        }

        // スキーマ定義名
        let first_feature = &self.features[0];
        let city_code = match first_feature.attributes.get(&Attribute::new("cityCode")) {
            Some(AttributeValue::String(s)) => s.clone(),
            _ => {
                return Err(PlateauProcessorError::AttributeFlattener(
                    "No city code found".to_string(),
                )
                .into())
            }
        };
        let schema_definition = format!("building_{}", city_code);

        // スキーマ属性を作成してデータフィーチャーに設定、出力
        let mut attrib = HashMap::new();
        attrib.insert(
            "schemaDefinition".to_string(),
            AttributeValue::String(schema_definition),
        );
        attrib.insert(
            "maxLod".to_string(),
            AttributeValue::Number(self.common_processor.max_lod.into()),
        );

        attrib.insert("isFilepathFeature".to_string(), AttributeValue::Bool(false));

        for mut feature in self.features.clone() {
            for (name, value) in &attrib {
                let attribute_name = Attribute::new(name.clone());
                feature.attributes.insert(attribute_name, value.clone());
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                DEFAULT_PORT.clone(),
            ));
        }

        for path in self.common_processor.gml_path_to_max_lod.keys().sorted() {
            let mut feature = first_feature.clone();

            attrib.insert(
                "cityCode".to_string(),
                AttributeValue::String(city_code.clone()),
            );
            attrib.insert(
                "cityGmlPath".to_string(),
                AttributeValue::String(path.clone()),
            );
            attrib.insert(
                "maxLod".to_string(),
                AttributeValue::Number(self.common_processor.gml_path_to_max_lod[path].into()),
            );
            attrib.insert("isFilepathFeature".to_string(), AttributeValue::Bool(true));
            for (name, value) in &attrib {
                let attribute_name = Attribute::new(name.clone());
                feature.attributes.insert(attribute_name, value.clone());
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                DEFAULT_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFlattener"
    }
}

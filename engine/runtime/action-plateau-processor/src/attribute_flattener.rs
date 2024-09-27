use super::errors::PlateauProcessorError;
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
use std::collections::{HashMap, HashSet};

const DELIM: &str = "_";

#[derive(Debug, Clone, Default)]
struct Flattener {
    flatten_attrs: HashMap<&'static str, Vec<&'static str>>,
    existing_flatten_attrs: HashSet<String>,
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
        all_attrib: HashMap<String, AttributeValue>,
    ) -> HashMap<String, AttributeValue> {
        let mut attrib = HashMap::new();

        for (t1, t2s) in &self.flatten_attrs {
            if let Some(value) = all_attrib.get(*t1) {
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
        t1: &&str,
        value: &AttributeValue,
        t2s: &Vec<&str>,
        attrib: &mut HashMap<String, AttributeValue>,
    ) {
        if let AttributeValue::Map(map) = value {
            for t2 in t2s {
                if let Some(v) = map.get(*t2) {
                    let key = format!("{}{}{}", t1, DELIM, t2);
                    attrib.insert(key, v.clone());
                }
            }
        } else {
            attrib.insert(t1.to_string(), value.clone());
        }
    }

    fn extract_fld_risk_attribute(
        &mut self,
        feature: &mut Feature,
        all_attrib: HashMap<String, AttributeValue>,
    ) {
        let disaster_risks = match all_attrib.get("uro:BuildingRiverFloodingRiskAttribute") {
            Some(AttributeValue::Array(disaster_risks)) => disaster_risks,
            _ => return,
        };
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
                feature.attributes.insert(attribute_name, value.clone());
                self.risk_to_attr_defs.entry("fld".to_string()).or_default();

                self.fld_attrs_sorter
                    .insert((desc_code, admin_code, scale_code, order), name);
            }
        }
    }

    pub fn extract_tnm_htd_ifld_risk_attribute(
        &mut self,
        feature: &mut Feature,
        all_attrib: HashMap<String, AttributeValue>,
    ) {
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
            let disaster_risks = match all_attrib.get(tag) {
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
                    feature.attributes.insert(attribute_name, value.clone());
                    self.risk_to_attr_defs
                        .entry(package.to_string())
                        .or_default();
                }
            }
        }
    }

    pub fn extract_lsld_risk_attribute(
        &mut self,
        feature: &mut Feature,
        all_attrib: HashMap<String, AttributeValue>,
    ) {
        let disaster_risks = match all_attrib.get("uro:BuildingLandSlideRiskAttribute") {
            Some(AttributeValue::Array(disaster_risks)) => disaster_risks,
            _ => return,
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
                feature.attributes.insert(attribute_name, value.clone());
                self.risk_to_attr_defs
                    .entry("lsld".to_string())
                    .or_default();
            }
        }
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
        feature: &mut Feature,
        attrib: HashMap<String, AttributeValue>,
    ) {
        fn flatten(
            feat: &mut Feature,
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
                feat.attributes.insert(attribute_name, value.clone());
                gen_attr_to_type.insert(name.clone(), type_.clone());

                if type_ == "measure" {
                    let uom = match obj.get("uom") {
                        Some(AttributeValue::String(uom)) => uom.clone(),
                        _ => return,
                    };
                    let name_uom = format!("{}_uom", name);
                    let attribute_name = Attribute::new(name_uom.clone());
                    feat.attributes
                        .insert(attribute_name, AttributeValue::String(uom.clone()));
                    gen_attr_to_type.insert(name_uom, "string".to_string());
                }
            }
        }

        let generic_attributes = match attrib.get("gen:genericAttribute") {
            Some(AttributeValue::Array(generic_attributes)) => generic_attributes,
            _ => return,
        };
        for obj in generic_attributes {
            if let AttributeValue::Map(obj_map) = obj {
                flatten(feature, obj_map, &mut self.gen_attr_to_type, "".to_string());
            }
        }
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
        feature: &mut Feature,
        attrib: HashMap<String, AttributeValue>,
        parent_tag: &str,
    ) {
        let parent_attr = match attrib.get(parent_tag) {
            Some(AttributeValue::Array(parent_attr)) => parent_attr,
            _ => return,
        };
        if parent_attr.is_empty() {
            return;
        }
        let first_element = match &parent_attr[0] {
            AttributeValue::Map(first_element) => first_element,
            _ => return,
        };
        let lod_types = match first_element.get("uro:lodType") {
            Some(AttributeValue::Array(lod_types)) => lod_types,
            _ => return,
        };
        for lod_type in lod_types {
            let s = match lod_type.to_string().chars().next() {
                Some(s) => s,
                None => continue,
            };
            if s.to_string() == "2" || s.to_string() == "3" || s.to_string() == "4" {
                let key = format!("lod_type_{}", s);
                let attribute_name = Attribute::new(key.clone());
                feature.attributes.insert(attribute_name, lod_type.clone());
            }
        }
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
        mut ctx: ExecutorContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &mut ctx.feature;

        let attributes = feature.attributes.clone();

        let ftype = attributes.get(&Attribute::new("featureType")).unwrap();

        let all_attrib = match feature.get(&Attribute::new("attributes")) {
            Some(AttributeValue::Map(attributes)) => attributes.clone(),
            v => {
                return Err(PlateauProcessorError::AttributeFlattener(format!(
                    "No attributes found: {:?}",
                    v
                ))
                .into())
            }
        };

        let mut flattened = self.flattener.flatten_attributes(all_attrib.clone());

        if ftype == &AttributeValue::String("bldg::Building".to_string()) {
            // 災害リスク属性の抽出
            self.flattener
                .extract_fld_risk_attribute(feature, all_attrib.clone());
            self.flattener
                .extract_tnm_htd_ifld_risk_attribute(feature, all_attrib.clone());
            self.flattener
                .extract_lsld_risk_attribute(feature, all_attrib.clone());

            self.common_processor
                .flatten_generic_attributes(feature, all_attrib.clone());
            self.common_processor.extract_lod_types(
                feature,
                all_attrib.clone(),
                "uro:BuildingDataQualityAttribute",
            )
        } else {
            // 子要素の場合はルート要素（Building）の属性を抽出してマージする。
            let root_all_attrib = match attributes.get(&Attribute::new("ancestors")) {
                Some(AttributeValue::Map(attributes)) => attributes.clone(),
                _ =>
                // default to empty
                {
                    HashMap::new()
                }
            };
            let froot = self.flattener.flatten_attributes(root_all_attrib.clone());
            for (name, vroot) in &froot {
                let v = flattened.get(name);

                let value = if v.is_none() || v == Some(vroot) {
                    vroot.clone()
                } else {
                    AttributeValue::String(format!(
                        "{} {}",
                        match &vroot {
                            AttributeValue::String(s) => s,
                            _ => panic!("Expected AttributeValue::String"),
                        },
                        match v {
                            Some(AttributeValue::String(ref s)) => s,
                            _ => panic!("Expected Some(AttributeValue::String)"),
                        }
                    ))
                };
                flattened.insert(name.clone(), value.clone());
            }

            if ftype == &AttributeValue::String("bldg::BuildingPart".to_string()) {
                self.flattener
                    .extract_fld_risk_attribute(feature, root_all_attrib.clone());
                self.flattener
                    .extract_tnm_htd_ifld_risk_attribute(feature, root_all_attrib.clone());
                self.flattener
                    .extract_lsld_risk_attribute(feature, root_all_attrib.clone());

                // BuildingPart に記述されている災害リスク属性も抽出する。
                // 規格外ではあるが BuildingPart 下位に災害リスク属性が記述されているデータもあるため。
                self.flattener
                    .extract_fld_risk_attribute(feature, all_attrib.clone());
                self.flattener
                    .extract_tnm_htd_ifld_risk_attribute(feature, all_attrib.clone());
                self.flattener
                    .extract_lsld_risk_attribute(feature, all_attrib.clone());
            }

            // フラットにする属性の設定
            for (name, value) in &flattened {
                feature
                    .attributes
                    .insert(Attribute::new(name.clone()), value.clone());
            }

            self.common_processor.update_max_lod(feature);
            self.features.push(feature.clone());
        }
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

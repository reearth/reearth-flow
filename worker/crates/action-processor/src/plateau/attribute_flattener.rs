use super::errors::PlateauProcessorError;
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

#[derive(Debug)]
struct Flattener {
    flatten_attrs: HashMap<&'static str, Vec<&'static str>>,
    existing_flatten_attrs: HashSet<String>,
    risk_to_attr_defs: HashMap<String, HashMap<String, String>>,
    fld_attrs_sorter: HashMap<(i64, i64, i64, i64), String>,
}

impl Flattener {
    fn new() -> Self {
        let flatten_attrs_data = vec![
            ("gml:name", "", ""),
            ("bldg:class", "", ""),
            ("bldg:usage", "", ""),
            ("bldg:yearOfConstruction", "", "fme_int16"),
            ("bldg:measuredHeight", "", "fme_real64"),
            ("bldg:storeysAboveGround", "", "fme_int16"),
            ("bldg:storeysBelowGround", "", "fme_int16"),
            ("bldg:address", "", ""),
            ("uro:BuildingIDAttribute", "uro:buildingID", ""),
            ("uro:BuildingIDAttribute", "uro:branchID", "fme_int16"),
            ("uro:BuildingIDAttribute", "uro:partID", "fme_int16"),
            ("uro:BuildingIDAttribute", "uro:prefecture", ""),
            ("uro:BuildingIDAttribute", "uro:city", ""),
            (
                "uro:BuildingDetailAttribute",
                "uro:serialNumberOfBuildingCertification",
                "",
            ),
            ("uro:BuildingDetailAttribute", "uro:siteArea", "fme_real64"),
            (
                "uro:BuildingDetailAttribute",
                "uro:totalFloorArea",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:buildingFootprintArea",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:buildingRoofEdgeArea",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:buildingStructureType",
                "",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:fireproofStructureType",
                "",
            ),
            ("uro:BuildingDetailAttribute", "uro:urbanPlanType", ""),
            (
                "uro:BuildingDetailAttribute",
                "uro:areaClassificationType",
                "",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:districtsAndZonesType",
                "",
            ),
            ("uro:BuildingDetailAttribute", "uro:landUseType", ""),
            ("uro:BuildingDetailAttribute", "uro:vacancy", ""),
            (
                "uro:BuildingDetailAttribute",
                "uro:buildingCoverageRate",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:floorAreaRate",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:specifiedBuildingCoverageRate",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:specifiedFloorAreaRate",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:standardFloorAreaRate",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:buildingHeight",
                "fme_real64",
            ),
            (
                "uro:BuildingDetailAttribute",
                "uro:eaveHeight",
                "fme_real64",
            ),
            ("uro:BuildingDetailAttribute", "uro:surveyYear", "fme_int16"),
            ("uro:LargeCustomerFacilityAttribute", "uro:class", ""),
            ("uro:LargeCustomerFacilityAttribute", "uro:name", ""),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:capacity",
                "fme_int32",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:totalFloorArea",
                "fme_real64",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:inauguralDate",
                "fme_date",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:yearOpened",
                "fme_int16",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:yearClosed",
                "fme_int16",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:urbanPlanType",
                "",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:areaClassificationType",
                "",
            ),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:districtsAndZonesType",
                "",
            ),
            ("uro:LargeCustomerFacilityAttribute", "uro:landUseType", ""),
            (
                "uro:LargeCustomerFacilityAttribute",
                "uro:surveyYear",
                "fme_int16",
            ),
            ("uro:BuildingDataQualityAttribute", "uro:lod1HeightType", ""),
            (
                "uro:RealEstateIDAttribute",
                "uro:realEstateIDOfBuilding",
                "",
            ),
            (
                "uro:RealEstateIDAttribute",
                "uro:matchingScore",
                "fme_int16",
            ),
        ];

        let mut flatten_attrs = HashMap::new();
        for (t1, t2, _) in flatten_attrs_data {
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
        if let Some(AttributeValue::Array(disaster_risks)) =
            all_attrib.get("uro:BuildingRiverFloodingRiskAttribute")
        {
            for risk in disaster_risks {
                if let AttributeValue::Map(risk_map) = risk {
                    let desc = risk_map.get("uro:description").and_then(|v| {
                        if let AttributeValue::String(s) = v {
                            Some(s)
                        } else {
                            None
                        }
                    });
                    let admin = risk_map.get("uro:adminType").and_then(|v| {
                        if let AttributeValue::String(s) = v {
                            Some(s)
                        } else {
                            None
                        }
                    });
                    let scale = risk_map.get("uro:scale").and_then(|v| {
                        if let AttributeValue::String(s) = v {
                            Some(s)
                        } else {
                            None
                        }
                    });

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
                        ("浸水ランク", rank, "fme_buffer", 1),
                        ("浸水ランクコード", rank_code, "fme_uint16", 2),
                        ("浸水深", depth, "fme_real64", 3),
                        ("浸水継続時間", duration, "fme_real64", 4),
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

                    for (k, v, t, order) in attrib {
                        if let Some(value) = v {
                            let name = format!("{}_{}", basename, k);
                            let attribute_name = Attribute::new(name.clone());
                            feature.attributes.insert(attribute_name, value.clone());
                            self.risk_to_attr_defs
                                .entry("fld".to_string())
                                .or_default()
                                .insert(name.clone(), t.to_string());

                            self.fld_attrs_sorter
                                .insert((desc_code, admin_code, scale_code, order), name);
                        }
                    }
                }
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
            if let Some(AttributeValue::Array(disaster_risks)) = all_attrib.get(tag) {
                for risk in disaster_risks {
                    if let AttributeValue::Map(risk_map) = risk {
                        let desc = risk_map.get("uro:description").and_then(|v| {
                            if let AttributeValue::String(s) = v {
                                Some(s)
                            } else {
                                None
                            }
                        });
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
                            ("浸水ランク", rank, "fme_buffer"),
                            ("浸水ランクコード", rank_code, "fme_uint16"),
                            ("浸水深", depth, "fme_real64"),
                        ];

                        for (k, v, t) in attrib {
                            if let Some(value) = v {
                                let name = format!("{}_{}", basename, k);
                                let attribute_name = Attribute::new(name.clone());
                                feature.attributes.insert(attribute_name, value.clone());
                                self.risk_to_attr_defs
                                    .entry(package.to_string())
                                    .or_default()
                                    .insert(name, t.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn extract_lsld_risk_attribute(
        &mut self,
        feature: &mut Feature,
        all_attrib: HashMap<String, AttributeValue>,
    ) {
        if let Some(AttributeValue::Array(disaster_risks)) =
            all_attrib.get("uro:BuildingLandSlideRiskAttribute")
        {
            for risk in disaster_risks {
                if let AttributeValue::Map(risk_map) = risk {
                    let desc = risk_map.get("uro:description");
                    let area_type = risk_map.get("uro:areaType");
                    let area_type_code = risk_map.get("uro:areaType_code");
                    let area_type_code_count = match area_type_code {
                        Some(AttributeValue::Number(n)) => n.as_i64(),
                        _ => None,
                    };
                    if desc.is_none()
                        || area_type_code.is_none()
                        || area_type_code_count.unwrap() > 2
                    {
                        continue;
                    }

                    let attrib = vec![
                        (
                            format!("土砂災害リスク_{}_区域区分", desc.unwrap()),
                            area_type,
                            "fme_buffer",
                        ),
                        (
                            format!("土砂災害リスク_{}_区域区分コード", desc.unwrap()),
                            area_type_code,
                            "fme_uint16",
                        ),
                    ];

                    for (k, v, t) in attrib {
                        if let Some(value) = v {
                            let attribute_name = Attribute::new(k.clone());
                            feature.attributes.insert(attribute_name, value.clone());
                            self.risk_to_attr_defs
                                .entry("lsld".to_string())
                                .or_default()
                                .insert(k, t.to_string());
                        }
                    }
                }
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
        let process = AttributeFlattener {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct AttributeFlattener {}

impl Processor for AttributeFlattener {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        5
    }

    fn process(
        &mut self,
        mut ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &mut ctx.feature;

        let mut flattener = Flattener::new();

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

        let flattened = flattener.flatten_attributes(all_attrib.clone());

        println!("{:?}", flattened);

        if ftype == &AttributeValue::String("bldg::Building".to_string()) {
            // 災害リスク属性の抽出
            flattener.extract_fld_risk_attribute(feature, all_attrib.clone());
            flattener.extract_tnm_htd_ifld_risk_attribute(feature, all_attrib.clone());
            flattener.extract_lsld_risk_attribute(feature, all_attrib.clone());
        } else {
            // 子要素の場合はルート要素（Building）の属性を抽出してマージする。
        }

        let feature = feature.clone();

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AttributeFlattener"
    }
}

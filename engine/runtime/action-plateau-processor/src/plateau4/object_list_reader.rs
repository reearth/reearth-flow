//! PLATEAU 4 (CityGML 2.0) objectlist reader: the single `A.3.1_取得項目一覧`
//! sheet.

use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use calamine::{RangeDeserializerBuilder, Reader};

use crate::common::object_list_extractor::ObjectListReader;
use crate::object_list::{
    is_cell_marked, open_workbook, Error, FeatureTypes, ObjectList, ObjectListMap, ObjectListValue,
    Result,
};

const SHEET_NAME: &str = "A.3.1_取得項目一覧";

const MUTABLE_PREFIXES: [(&str, &str, &str); 5] = [
    (
        "fld",
        "uro:floodingRiskAttribute",
        "uro:floodingRiskAttribute/uro:RiverFloodingRiskAttribute",
    ),
    (
        "tnm",
        "uro:floodingRiskAttribute",
        "uro:floodingRiskAttribute/uro:TsunamiRiskAttribute",
    ),
    (
        "htd",
        "uro:floodingRiskAttribute",
        "uro:floodingRiskAttribute/uro:HighTideRiskAttribute",
    ),
    (
        "ifld",
        "uro:floodingRiskAttribute",
        "uro:floodingRiskAttribute/uro:InlandFloodingRiskAttribute",
    ),
    (
        "rfld",
        "uro:floodingRiskAttribute",
        "uro:floodingRiskAttribute/uro:ReservoirFloodingRiskAttribute",
    ),
];

#[derive(Debug)]
pub(crate) struct Plateau4ObjectListReader;

impl ObjectListReader for Plateau4ObjectListReader {
    fn read(&self, bytes: Bytes) -> Result<(FeatureTypes, ObjectListMap)> {
        parse(bytes)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Record {
    feature_prefix: String,
    feature_type: String,
    category: String,
    xpath: String,
    required: bool,
}

impl Record {
    fn from_row_with_state(mut row: Vec<String>, state: &AttributeState) -> Self {
        if let Some(feature_type) = &state.feature_type {
            row[1] = feature_type.clone();
        }
        let attributes = state.get_attributes();
        for (i, attr) in attributes.iter().enumerate() {
            row[i + 2] = attr.clone();
        }
        Self::from(row)
    }
}

impl From<Vec<String>> for Record {
    fn from(columns: Vec<String>) -> Self {
        let feature_prefix = columns.first().unwrap_or(&"".to_string()).clone();
        let feature_type = columns.get(1).unwrap_or(&"".to_string()).clone();
        let category = columns.get(6).unwrap_or(&"".to_string()).clone();

        let xpath1 = columns.get(2).cloned().unwrap_or("".to_string());
        let xpath2 = columns.get(3).cloned().unwrap_or("".to_string());
        let xpath3 = columns.get(4).cloned().unwrap_or("".to_string());
        let xpath4 = columns.get(5).cloned().unwrap_or("".to_string());
        let xpath = vec![
            xpath1.clone(),
            xpath2.clone(),
            xpath3.clone(),
            xpath4.clone(),
        ];
        let xpath = xpath
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join("/");
        let xpath = xpath.replace("(", "").replace(")", "").replace(".", "/");

        let is_unknown_value = columns.get(13).cloned().unwrap_or("".to_string());
        let required = is_cell_marked(&is_unknown_value);
        Self {
            feature_prefix,
            feature_type,
            category,
            xpath,
            required,
        }
    }
}

fn group_by_feature_type(records: Vec<Record>) -> HashMap<String, ObjectListValue> {
    let mut types = HashMap::<String, ObjectListValue>::new();
    for record in records {
        let value = types
            .entry(record.feature_type.clone())
            .or_insert(ObjectListValue {
                required: vec![],
                target: vec![],
                conditional: vec![],
            });
        if record.required {
            value.required.push(record.xpath.clone());
        } else {
            value.target.push(record.xpath.clone());
        }
    }
    types
}

#[derive(Clone, Debug, Default)]
struct AttributeState {
    feature_type: Option<String>,
    attribute1: Option<String>,
    attribute2: Option<String>,
    attribute3: Option<String>,
    attribute4: Option<String>,
}

impl AttributeState {
    fn update_feature_type(&mut self, value: &str) {
        if !value.is_empty() {
            self.feature_type = Some(value.to_string());
            self.clear_attributes();
        }
    }

    fn update_attribute(&mut self, level: usize, value: &str) {
        if value.is_empty() {
            return;
        }
        match level {
            1 => {
                self.attribute1 = Some(value.to_string());
                self.clear_attributes_after(1);
            }
            2 => {
                self.attribute2 = Some(value.to_string());
                self.clear_attributes_after(2);
            }
            3 => {
                self.attribute3 = Some(value.to_string());
                self.clear_attributes_after(3);
            }
            4 => {
                self.attribute4 = Some(value.to_string());
            }
            _ => {}
        }
    }

    fn clear_attributes(&mut self) {
        self.attribute1 = None;
        self.attribute2 = None;
        self.attribute3 = None;
        self.attribute4 = None;
    }

    fn clear_attributes_after(&mut self, level: usize) {
        if level <= 1 {
            self.attribute2 = None;
        }
        if level <= 2 {
            self.attribute3 = None;
        }
        if level <= 3 {
            self.attribute4 = None;
        }
    }

    fn get_attributes(&self) -> Vec<String> {
        vec![
            self.attribute1.clone().unwrap_or_default(),
            self.attribute2.clone().unwrap_or_default(),
            self.attribute3.clone().unwrap_or_default(),
            self.attribute4.clone().unwrap_or_default(),
        ]
    }
}

fn should_process_row(columns: &[String]) -> bool {
    let has_create = columns
        .get(8)
        .map(|is_create| is_cell_marked(is_create))
        .unwrap_or(false);

    let has_valid_category = columns
        .get(6)
        .map(|category| !category.is_empty() && ["主題", "関連役割"].contains(&category.as_str()))
        .unwrap_or(false);

    has_create && has_valid_category
}

fn expand_row_for_special_prefix(row: Vec<String>) -> Vec<Vec<String>> {
    if let Some(prefix) = row.first() {
        if prefix.starts_with("fld/") {
            return ["fld", "tnm", "htd", "ifld", "rfld"]
                .iter()
                .map(|prefix| {
                    let mut new_row = row.clone();
                    new_row[0] = prefix.to_string();
                    new_row
                })
                .collect();
        }
    }
    vec![row]
}

fn parse(bytes: Bytes) -> Result<(FeatureTypes, ObjectListMap)> {
    let mut workbook = open_workbook(bytes)?;
    let range = workbook
        .worksheet_range(SHEET_NAME)
        .map_err(|source| Error::Sheet {
            name: SHEET_NAME.to_string(),
            source,
        })?;
    let iter = RangeDeserializerBuilder::new().from_range(&range)?;

    let mut prefixes = HashMap::<String, Vec<Record>>::new();
    let mut state = AttributeState::default();

    for row in iter {
        let columns: Vec<String> = row?;
        if let Some(feature_type) = columns.get(1) {
            state.update_feature_type(feature_type);
        }
        for i in 0..4 {
            if let Some(attribute) = columns.get(i + 2) {
                state.update_attribute(i + 1, attribute);
            }
        }
        if !should_process_row(&columns) {
            continue;
        }

        let expanded_rows = expand_row_for_special_prefix(columns);
        for row in expanded_rows {
            if let Some(prefix) = row.first() {
                let records = prefixes.entry(prefix.clone()).or_default();
                records.push(Record::from_row_with_state(row, &state));
            }
        }
    }
    let mut object_list = prefixes
        .into_iter()
        .map(|(prefix, records)| (prefix, group_by_feature_type(records)))
        .collect::<HashMap<String, HashMap<String, ObjectListValue>>>();

    for (prefix, not_attribute, require_attribute) in MUTABLE_PREFIXES {
        if let Some(value) = object_list.get_mut(prefix) {
            for v in value.values_mut() {
                v.target
                    .retain(|x| !x.starts_with(not_attribute) || x.starts_with(require_attribute));
            }
        }
    }
    let feature_types = object_list
        .iter()
        .map(|(prefix, object_list)| (prefix.clone(), object_list.keys().cloned().collect()))
        .collect::<HashMap<String, Vec<String>>>();
    process_object_list(&mut object_list);

    let object_list = object_list
        .into_iter()
        .map(|(prefix, object_list)| (prefix, ObjectList::new(object_list)))
        .collect();
    Ok((
        FeatureTypes::new(feature_types),
        ObjectListMap::new(object_list),
    ))
}

fn process_object_list(objectlist: &mut HashMap<String, HashMap<String, ObjectListValue>>) {
    let targets = vec![
        ("bldg", ("bldg:Building", "bldg:BuildingPart")),
        ("brid", ("brid:Bridge", "brid:BridgePart")),
        ("tun", ("tun:Tunnel", "tun:TunnelPart")),
        ("ubld", ("uro:UndergroundBuilding", "bldg:BuildingPart")),
    ];

    // targets をループ
    for (prefix, (root, part)) in targets {
        let Some(value) = objectlist.get_mut(prefix) else {
            continue;
        };

        let root_required_set: HashSet<String> = value
            .get(root)
            .map(|v| v.required.clone())
            .map(|arr| {
                arr.iter()
                    .filter(|x| !x.starts_with("uro:builgindIDAttribute"))
                    .cloned()
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        let part_required_set: HashSet<String> = value
            .get(part)
            .map(|v| v.required.clone())
            .map(|arr| {
                arr.iter()
                    .filter(|x| !x.starts_with("uro:buildingIDAttribute"))
                    .cloned()
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();

        if !root_required_set.is_empty() && !part_required_set.is_empty() {
            let u: HashSet<_> = root_required_set
                .intersection(&part_required_set)
                .cloned()
                .collect();

            if !u.is_empty() {
                if let Some(root_obj_value) = value.get_mut(root) {
                    let mut required = root_required_set
                        .difference(&u)
                        .cloned()
                        .collect::<Vec<_>>();
                    required.sort();
                    root_obj_value.required = required;
                    let mut u = u.iter().cloned().collect::<Vec<String>>();
                    u.sort();
                    root_obj_value.conditional = u;
                }

                if let Some(root_obj_value) = value.get_mut(part) {
                    let mut required = root_required_set
                        .difference(&u)
                        .cloned()
                        .collect::<Vec<_>>();
                    required.sort();
                    root_obj_value.required = required;
                    let mut u = u.iter().cloned().collect::<Vec<String>>();
                    u.sort();
                    root_obj_value.conditional = u;
                }
            }
        }
    }
}

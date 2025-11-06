use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
};

use bytes::Bytes;
use calamine::{RangeDeserializerBuilder, Reader, Xlsx};
use once_cell::sync::Lazy;
use reearth_flow_types::AttributeValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

static MUTABLE_PREFIXES: Lazy<HashMap<&str, (&str, &str)>> = Lazy::new(|| {
    HashMap::from([
        (
            "fld",
            (
                "uro:floodingRiskAttribute",
                "uro:floodingRiskAttribute/uro:RiverFloodingRiskAttribute",
            ),
        ),
        (
            "tnm",
            (
                "uro:floodingRiskAttribute",
                "uro:floodingRiskAttribute/uro:TsunamiRiskAttribute",
            ),
        ),
        (
            "htd",
            (
                "uro:floodingRiskAttribute",
                "uro:floodingRiskAttribute/uro:HighTideRiskAttribute",
            ),
        ),
        (
            "ifld",
            (
                "uro:floodingRiskAttribute",
                "uro:floodingRiskAttribute/uro:InlandFloodingRiskAttribute",
            ),
        ),
        (
            "rfld",
            (
                "uro:floodingRiskAttribute",
                "uro:floodingRiskAttribute/uro:ReservoirFloodingRiskAttribute",
            ),
        ),
    ])
});

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Parse error: {0}")]
    Parse(String),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub(crate) feature_prefix: String,
    pub(crate) feature_type: String,
    pub(crate) category: String,
    pub(crate) xpath: String,
    pub(crate) required: bool,
}

impl Record {
    pub(crate) fn from_row_with_state(mut row: Vec<String>, state: &AttributeState) -> Self {
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
        let required = !is_unknown_value.is_empty();
        Self {
            feature_prefix,
            feature_type,
            category,
            xpath,
            required,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FeatureTypes(HashMap<String, Vec<String>>);

impl FeatureTypes {
    pub(crate) fn new(types: HashMap<String, Vec<String>>) -> Self {
        Self(types)
    }

    pub(crate) fn into_inner(self) -> HashMap<String, Vec<String>> {
        self.0
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (String, Vec<String>)> {
        self.0.into_iter()
    }
}

impl From<FeatureTypes> for AttributeValue {
    fn from(value: FeatureTypes) -> Self {
        let map = value
            .into_inner()
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    AttributeValue::Array(
                        v.iter()
                            .map(|s| AttributeValue::String(s.clone()))
                            .collect(),
                    ),
                )
            })
            .collect::<HashMap<String, AttributeValue>>();
        AttributeValue::Map(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectListMap(HashMap<String, ObjectList>);

impl ObjectListMap {
    pub(crate) fn new(types: HashMap<String, ObjectList>) -> Self {
        Self(types)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (String, ObjectList)> {
        self.0.into_iter()
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut ObjectList> {
        self.0.get_mut(key)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&ObjectList> {
        self.0.get(key)
    }
}

impl From<AttributeValue> for ObjectListMap {
    fn from(value: AttributeValue) -> Self {
        let map = value
            .as_map()
            .map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), ObjectList::from(v.clone())))
                    .collect::<HashMap<String, ObjectList>>()
            })
            .unwrap_or_default();
        Self::new(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectList(HashMap<String, ObjectListValue>);

impl ObjectList {
    pub(crate) fn new(types: HashMap<String, ObjectListValue>) -> Self {
        Self(types)
    }

    pub(crate) fn get_feature_types(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    pub(crate) fn get(&self, key: &str) -> Option<&ObjectListValue> {
        self.0.get(key)
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut ObjectListValue> {
        self.0.get_mut(key)
    }

    pub(crate) fn keys(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
}

impl From<AttributeValue> for ObjectList {
    fn from(value: AttributeValue) -> Self {
        let map = value
            .as_map()
            .map(|map| {
                map.iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect::<HashMap<String, ObjectListValue>>()
            })
            .unwrap_or_default();
        Self::new(map)
    }
}

impl From<ObjectList> for AttributeValue {
    fn from(value: ObjectList) -> Self {
        let map = value
            .0
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().into()))
            .collect::<HashMap<String, AttributeValue>>();
        AttributeValue::Map(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct ObjectListValue {
    pub(crate) required: Vec<String>,
    pub(crate) target: Vec<String>,
    pub(crate) conditional: Vec<String>,
}

impl From<AttributeValue> for ObjectListValue {
    fn from(value: AttributeValue) -> Self {
        let map = value
            .as_map()
            .map(|map| {
                let required = map
                    .get("required")
                    .map(|v| v.as_vec().unwrap_or_default())
                    .unwrap_or_default()
                    .iter()
                    .map(|v| v.as_string().unwrap_or_default())
                    .collect::<Vec<String>>();
                let target = map
                    .get("target")
                    .map(|v| v.as_vec().unwrap_or_default())
                    .unwrap_or_default()
                    .iter()
                    .map(|v| v.as_string().unwrap_or_default())
                    .collect::<Vec<String>>();
                let conditional = map
                    .get("conditional")
                    .map(|v| v.as_vec().unwrap_or_default())
                    .unwrap_or_default()
                    .iter()
                    .map(|v| v.as_string().unwrap_or_default())
                    .collect::<Vec<String>>();
                Self {
                    required,
                    target,
                    conditional,
                }
            })
            .unwrap_or_default();
        map
    }
}

impl From<ObjectListValue> for AttributeValue {
    fn from(value: ObjectListValue) -> Self {
        let mut map = HashMap::<String, AttributeValue>::new();
        map.insert(
            "required".to_string(),
            AttributeValue::Array(
                value
                    .required
                    .into_iter()
                    .map(AttributeValue::String)
                    .collect(),
            ),
        );
        map.insert(
            "target".to_string(),
            AttributeValue::Array(
                value
                    .target
                    .into_iter()
                    .map(AttributeValue::String)
                    .collect(),
            ),
        );
        map.insert(
            "conditional".to_string(),
            AttributeValue::Array(
                value
                    .conditional
                    .into_iter()
                    .map(AttributeValue::String)
                    .collect(),
            ),
        );
        AttributeValue::Map(map)
    }
}

impl From<Vec<Record>> for ObjectList {
    fn from(records: Vec<Record>) -> Self {
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
        Self::new(types)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AttributeState {
    pub(crate) feature_type: Option<String>,
    pub(crate) attribute1: Option<String>,
    pub(crate) attribute2: Option<String>,
    pub(crate) attribute3: Option<String>,
    pub(crate) attribute4: Option<String>,
}

impl AttributeState {
    pub(crate) fn update_feature_type(&mut self, value: &str) {
        if !value.is_empty() {
            self.feature_type = Some(value.to_string());
            self.clear_attributes();
        }
    }

    pub(crate) fn update_attribute(&mut self, level: usize, value: &str) {
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

    pub(crate) fn get_attributes(&self) -> Vec<String> {
        vec![
            self.attribute1.clone().unwrap_or_default(),
            self.attribute2.clone().unwrap_or_default(),
            self.attribute3.clone().unwrap_or_default(),
            self.attribute4.clone().unwrap_or_default(),
        ]
    }
}

fn open_workbook(bytes: Bytes) -> Result<Xlsx<Cursor<Bytes>>, Error> {
    let reader = Cursor::new(bytes);
    calamine::open_workbook_from_rs(reader).map_err(|e| Error::Parse(format!("{e:?}")))
}

pub(crate) fn should_process_row(columns: &[String]) -> bool {
    let has_create = columns
        .get(8)
        .map(|is_create| !is_create.is_empty())
        .unwrap_or(false);

    let has_valid_category = columns
        .get(6)
        .map(|category| !category.is_empty() && ["主題", "関連役割"].contains(&category.as_str()))
        .unwrap_or(false);

    has_create && has_valid_category
}

pub(crate) fn expand_row_for_special_prefix(row: Vec<String>) -> Vec<Vec<String>> {
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

pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn parse(bytes: Bytes) -> Result<(FeatureTypes, ObjectListMap)> {
    let mut workbook = open_workbook(bytes)?;
    let range = workbook
        .worksheet_range("A.3.1_取得項目一覧")
        .map_err(|e| Error::Parse(format!("{e:?}")))?;
    let iter = RangeDeserializerBuilder::new()
        .from_range(&range)
        .map_err(|e| Error::Parse(format!("{e:?}")))?;

    let mut prefixes = HashMap::<String, Vec<Record>>::new();
    let mut state = AttributeState::default();

    for row in iter {
        let columns: Vec<String> = row.map_err(|e| Error::Parse(format!("{e:?}")))?;
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
        .map(|(prefix, records)| (prefix, ObjectList::from(records)))
        .collect::<HashMap<String, ObjectList>>();

    for (prefix, (not_attribute, require_attribute)) in MUTABLE_PREFIXES.clone() {
        if let Some(value) = object_list.get_mut(prefix) {
            let keys = value.keys();
            for key in keys.iter() {
                if let Some(v) = value.get_mut(key) {
                    v.target.retain(|x| {
                        !x.starts_with(not_attribute) || x.starts_with(require_attribute)
                    });
                }
            }
        }
    }
    let feature_types = object_list
        .iter()
        .map(|(prefix, object_list)| (prefix.clone(), object_list.get_feature_types()))
        .collect::<HashMap<String, Vec<String>>>();
    let mut object_list = ObjectListMap::new(object_list);
    process_object_list(&mut object_list);

    Ok((FeatureTypes::new(feature_types), object_list))
}

fn process_object_list(objectlist: &mut ObjectListMap) {
    if objectlist.is_empty() {
        return;
    }
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

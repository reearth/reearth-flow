use std::{collections::HashMap, io::Cursor};

use bytes::Bytes;
use calamine::{RangeDeserializerBuilder, Reader, Xlsx};
use reearth_flow_types::AttributeValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(dead_code)]
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
pub(crate) struct FeatureTypes {
    pub(crate) prefix: String,
    pub(crate) types: Vec<String>,
}

impl From<FeatureTypes> for AttributeValue {
    fn from(value: FeatureTypes) -> Self {
        let mut map = HashMap::<String, AttributeValue>::new();
        map.insert("prefix".to_string(), AttributeValue::String(value.prefix));
        map.insert(
            "types".to_string(),
            AttributeValue::Array(
                value
                    .types
                    .into_iter()
                    .map(AttributeValue::String)
                    .collect(),
            ),
        );
        AttributeValue::Map(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectList {
    pub(crate) prefix: String,
    pub(crate) types: HashMap<String, ObjectListValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectListValue {
    pub(crate) required: Vec<String>,
    pub(crate) target: Vec<String>,
}

impl From<ObjectList> for AttributeValue {
    fn from(value: ObjectList) -> Self {
        let mut map = HashMap::<String, AttributeValue>::new();
        map.insert("prefix".to_string(), AttributeValue::String(value.prefix));
        map.insert(
            "types".to_string(),
            AttributeValue::Map(
                value
                    .types
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect(),
            ),
        );
        AttributeValue::Map(map)
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
        AttributeValue::Map(map)
    }
}

impl From<(String, Vec<Record>)> for ObjectList {
    fn from((prefix, records): (String, Vec<Record>)) -> Self {
        let mut types = HashMap::<String, ObjectListValue>::new();
        for record in records {
            let value = types
                .entry(record.feature_type.clone())
                .or_insert(ObjectListValue {
                    required: vec![],
                    target: vec![],
                });
            if record.required {
                value.required.push(record.xpath.clone());
            } else {
                value.target.push(record.xpath.clone());
            }
        }
        Self { prefix, types }
    }
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

fn open_workbook(bytes: Bytes) -> Result<Xlsx<Cursor<Bytes>>, Error> {
    let reader = Cursor::new(bytes);
    calamine::open_workbook_from_rs(reader).map_err(|e| Error::Parse(format!("{:?}", e)))
}

fn should_process_row(columns: &[String]) -> bool {
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

#[allow(dead_code)]
pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(dead_code)]
pub(crate) fn parse(bytes: Bytes) -> Result<(Vec<FeatureTypes>, Vec<ObjectList>)> {
    let mut workbook = open_workbook(bytes)?;
    let range = workbook
        .worksheet_range("A.3.1_取得項目一覧")
        .map_err(|e| Error::Parse(format!("{:?}", e)))?;
    let iter = RangeDeserializerBuilder::new()
        .from_range(&range)
        .map_err(|e| Error::Parse(format!("{:?}", e)))?;

    let mut prefixes = HashMap::<String, Vec<Record>>::new();
    let mut state = AttributeState::default();

    for row in iter {
        let columns: Vec<String> = row.map_err(|e| Error::Parse(format!("{:?}", e)))?;
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
    let object_list = prefixes
        .into_iter()
        .map(|(prefix, records)| ObjectList::from((prefix, records)))
        .collect::<Vec<ObjectList>>();
    let feature_types = object_list
        .iter()
        .map(|object_list| FeatureTypes {
            prefix: object_list.prefix.clone(),
            types: object_list.types.keys().cloned().collect(),
        })
        .collect::<Vec<FeatureTypes>>();
    Ok((feature_types, object_list))
}

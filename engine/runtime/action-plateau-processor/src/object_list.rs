use std::{collections::HashMap, io::Cursor};

use bytes::Bytes;
use calamine::{RangeDeserializerBuilder, Reader, Xlsx};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectList {
    pub(crate) prefix: String,
    pub(crate) types: HashMap<String, ObjectListValue>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectListValue {
    pub(crate) required: Vec<String>,
    pub(crate) target: Vec<String>,
}

#[allow(dead_code)]
pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(dead_code)]
pub(crate) fn parse(bytes: Bytes) -> Result<(Vec<FeatureTypes>, Vec<ObjectList>)> {
    let reader = Cursor::new(bytes);
    let mut workbook: Xlsx<_> =
        calamine::open_workbook_from_rs(reader).map_err(|e| Error::Parse(format!("{:?}", e)))?;
    let range = workbook
        .worksheet_range("A.3.1_取得項目一覧")
        .map_err(|e| Error::Parse(format!("{:?}", e)))?;
    let iter = RangeDeserializerBuilder::new()
        .from_range(&range)
        .map_err(|e| Error::Parse(format!("{:?}", e)))?;

    let mut prefixes = HashMap::<String, Vec<Record>>::new();

    let mut current_feature_type: Option<String> = None;
    let mut attribute1: Option<String> = None;
    let mut attribute2: Option<String> = None;
    let mut attribute3: Option<String> = None;
    let mut attribute4: Option<String> = None;
    for row in iter {
        let columns: Vec<String> = row.map_err(|e| Error::Parse(format!("{:?}", e)))?;
        if let Some(feature_type) = columns.get(1) {
            if !feature_type.is_empty() {
                current_feature_type = Some(feature_type.clone());
                attribute1 = None;
                attribute2 = None;
                attribute3 = None;
                attribute4 = None;
            }
        }
        if let Some(attribute) = columns.get(2) {
            if !attribute.is_empty() {
                attribute1 = Some(attribute.clone());
                attribute2 = None;
                attribute3 = None;
                attribute4 = None;
            }
        }
        if let Some(attribute) = columns.get(3) {
            if !attribute.is_empty() {
                attribute2 = Some(attribute.clone());
                attribute3 = None;
                attribute4 = None;
            }
        }
        if let Some(attribute) = columns.get(4) {
            if !attribute.is_empty() {
                attribute3 = Some(attribute.clone());
                attribute4 = None;
            }
        }
        if let Some(attribute) = columns.get(5) {
            if !attribute.is_empty() {
                attribute4 = Some(attribute.clone());
            }
        }
        if let Some(is_create) = columns.get(8) {
            if is_create.is_empty() {
                continue;
            }
        } else {
            continue;
        }
        if let Some(category) = columns.get(6) {
            if category.is_empty() || !["主題", "関連役割"].contains(&category.as_str()) {
                continue;
            }
        } else {
            continue;
        }
        let mut rows = vec![columns.clone()];
        if let Some(prefix) = columns.first() {
            if prefix.starts_with("fld/") {
                rows.clear();
                for prefix in ["fld", "tnm", "htd", "ifld", "rfld"] {
                    let mut row = columns.clone();
                    row[0] = prefix.to_string();
                    rows.push(row);
                }
            }
        }
        for row in rows.iter() {
            let mut row = row.clone();
            if let Some(current_feature_type) = &current_feature_type {
                row[1] = current_feature_type.clone();
            }
            if let Some(prefix) = row.first() {
                let value = prefixes.entry(prefix.clone()).or_default();
                row[2] = attribute1.clone().unwrap_or("".to_string());
                row[3] = attribute2.clone().unwrap_or("".to_string());
                row[4] = attribute3.clone().unwrap_or("".to_string());
                row[5] = attribute4.clone().unwrap_or("".to_string());
                value.push(row.clone().into());
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

//! Objectlist output types and xlsx helpers shared across PLATEAU generations.

use std::{collections::HashMap, io::Cursor};

use bytes::Bytes;
use calamine::{DeError, Xlsx, XlsxError};
use reearth_flow_types::{Attribute, AttributeValue, Attributes};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Failed to open workbook: {0}")]
    Workbook(XlsxError),
    #[error("Failed to read sheet {name}: {source}")]
    Sheet { name: String, source: XlsxError },
    #[error("Failed to deserialize row: {0}")]
    Row(#[from] DeError),
    #[error("No model sheet found")]
    NoModelSheet,
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FeatureTypes(HashMap<String, Vec<String>>);

impl FeatureTypes {
    pub(crate) fn new(types: HashMap<String, Vec<String>>) -> Self {
        Self(types)
    }

    pub(crate) fn into_inner(self) -> HashMap<String, Vec<String>> {
        self.0
    }
}

impl From<FeatureTypes> for AttributeValue {
    fn from(value: FeatureTypes) -> Self {
        let map = value
            .into_inner()
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.as_str()),
                    AttributeValue::Array(
                        v.iter()
                            .map(|s| AttributeValue::String(s.clone()))
                            .collect(),
                    ),
                )
            })
            .collect::<Attributes>();
        AttributeValue::Map(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectListMap(HashMap<String, ObjectList>);

impl ObjectListMap {
    pub(crate) fn new(types: HashMap<String, ObjectList>) -> Self {
        Self(types)
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (String, ObjectList)> {
        self.0.into_iter()
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
                    .map(|(k, v)| (k.to_string(), ObjectList::from(v.clone())))
                    .collect::<HashMap<String, ObjectList>>()
            })
            .unwrap_or_default();
        Self::new(map)
    }
}

impl From<ObjectListMap> for AttributeValue {
    fn from(value: ObjectListMap) -> Self {
        let map = value
            .into_iter()
            .map(|(k, v)| (Attribute::new(k.as_str()), v.into()))
            .collect::<Attributes>();
        AttributeValue::Map(map)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ObjectList(HashMap<String, ObjectListValue>);

impl ObjectList {
    pub(crate) fn new(types: HashMap<String, ObjectListValue>) -> Self {
        Self(types)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&ObjectListValue> {
        self.0.get(key)
    }
}

impl From<AttributeValue> for ObjectList {
    fn from(value: AttributeValue) -> Self {
        let map = value
            .as_map()
            .map(|map| {
                map.iter()
                    .map(|(k, v)| (k.to_string(), v.clone().into()))
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
            .map(|(k, v)| (Attribute::new(k.as_str()), v.clone().into()))
            .collect::<Attributes>();
        AttributeValue::Map(map)
    }
}

/// Attribute paths of one feature type that the missing-attribute detector checks.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct ObjectListValue {
    /// C05 (必須属性等の欠落): reported as an error on each feature that lacks it.
    pub(crate) required: Vec<String>,
    /// C06 (現われない属性等): reported once if no feature of the type has it.
    pub(crate) target: Vec<String>,
    /// C05 attributes required on both a root (e.g. `bldg:Building`) and its
    /// part, so either may carry them. A missing one is a warning on a part or
    /// on a root that has parts, and an error on a root without parts.
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
        let mut map = Attributes::new();
        map.insert(
            Attribute::new("required"),
            AttributeValue::Array(
                value
                    .required
                    .into_iter()
                    .map(AttributeValue::String)
                    .collect(),
            ),
        );
        map.insert(
            Attribute::new("target"),
            AttributeValue::Array(
                value
                    .target
                    .into_iter()
                    .map(AttributeValue::String)
                    .collect(),
            ),
        );
        map.insert(
            Attribute::new("conditional"),
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

pub(crate) fn open_workbook(bytes: Bytes) -> Result<Xlsx<Cursor<Bytes>>, Error> {
    let reader = Cursor::new(bytes);
    calamine::open_workbook_from_rs(reader).map_err(Error::Workbook)
}

/// Checks if a cell value has a mark (e.g., circle mark "○").
/// Returns true if the cell contains any non-whitespace characters.
/// Ignores full-width spaces (U+3000), half-width spaces (U+0020), and other whitespace.
pub(crate) fn is_cell_marked(value: &str) -> bool {
    !value.chars().all(|c| c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cell_marked() {
        // Marked cells (should return true)
        assert!(is_cell_marked("○"));
        assert!(is_cell_marked("●"));
        assert!(is_cell_marked("あ"));
        assert!(is_cell_marked("abc"));
        assert!(is_cell_marked(" ○ ")); // mark with surrounding spaces
        assert!(is_cell_marked("　○　")); // mark with surrounding full-width spaces

        // Empty or whitespace-only cells (should return false)
        assert!(!is_cell_marked(""));
        assert!(!is_cell_marked(" ")); // single half-width space
        assert!(!is_cell_marked("  ")); // multiple half-width spaces
        assert!(!is_cell_marked("　")); // single full-width space (U+3000)
        assert!(!is_cell_marked("　　")); // multiple full-width spaces
        assert!(!is_cell_marked(" 　 ")); // mixed half-width and full-width spaces
        assert!(!is_cell_marked("\t")); // tab
        assert!(!is_cell_marked("\n")); // newline
        assert!(!is_cell_marked(" \t\n　")); // mixed whitespace
    }
}

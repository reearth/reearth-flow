//! Conversion of Flow's attribute keys to the legacy keys of truths marked `legacy_keys`.
//!
//! The legacy keys differ from the current ones as follows:
//! - `:` is written as `:`, not `__`, at any depth of the nested `attributes` and
//!   `dm_attributes` JSON as well.
//! - A risk column is named by its label, e.g. `荒川水系荒川（国管理区間）_L2（想定最大規模）_浸水ランク`
//!   for `fld_1_1_2_rank`, and there is no `{group}_name` column.
//! - A generic attribute column is named by the attribute name, e.g. `延べ面積換算係数` for
//!   `gen_1_value`, and there is no `gen_{i}_name` column.

use serde_json::{Map, Value};
use std::collections::HashMap;

/// Keys holding nested attributes serialized as JSON.
const NESTED_JSON_KEYS: &[&str] = &["attributes", "dm_attributes"];

/// Value column suffixes of the risk groups and their legacy labels, longest suffix first.
const FLD_COLUMNS: &[(&str, &str)] = &[
    ("rank_code", "浸水ランクコード"),
    ("rank", "浸水ランク"),
    ("depth", "浸水深"),
    ("duration", "浸水継続時間"),
];
const FLOOD_COLUMNS: &[(&str, &str)] = &[
    ("rank_code", "浸水ランクコード"),
    ("rank", "浸水ランク"),
    ("depth", "浸水深"),
];
const LSLD_COLUMNS: &[(&str, &str)] = &[
    ("area_type_code", "区域区分コード"),
    ("area_type", "区域区分"),
];

/// Value column suffix and its legacy label.
type Columns = &'static [(&'static str, &'static str)];

/// Risk package, number of codes in its group key, and its value columns.
const RISK_GROUPS: &[(&str, usize, Columns)] = &[
    ("fld", 3, FLD_COLUMNS),
    ("tnm", 1, FLOOD_COLUMNS),
    ("htd", 1, FLOOD_COLUMNS),
    ("ifld", 1, FLOOD_COLUMNS),
    ("rfld", 1, FLOOD_COLUMNS),
    ("lsld", 1, LSLD_COLUMNS),
];

/// A column of a risk or generic attribute group.
#[derive(Debug, PartialEq)]
enum GroupColumn<'a> {
    /// `{group}_name` or `gen_{i}_name`, holding the label of the group
    Name { group: &'a str },
    /// A value column, named `{label}{suffix}` in the legacy convention
    Value { group: &'a str, suffix: String },
}

fn is_code(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Whether `group` is a risk group key with `codes` codes after the package, e.g. `fld_1_1_2`.
fn is_risk_group(group: &str, package: &str, codes: usize) -> bool {
    group
        .strip_prefix(package)
        .and_then(|rest| rest.strip_prefix('_'))
        .is_some_and(|rest| {
            let parts: Vec<&str> = rest.split('_').collect();
            parts.len() == codes && parts.iter().all(|p| is_code(p))
        })
}

fn parse_group_column(key: &str) -> Option<GroupColumn<'_>> {
    if let Some(rest) = key.strip_prefix("gen_") {
        let (index, column) = rest.split_once('_')?;
        if index.is_empty() || !index.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let group = &key[..4 + index.len()];
        return match column {
            "name" => Some(GroupColumn::Name { group }),
            "value" => Some(GroupColumn::Value {
                group,
                suffix: String::new(),
            }),
            "uom" => Some(GroupColumn::Value {
                group,
                suffix: "_uom".to_string(),
            }),
            _ => None,
        };
    }

    for (package, codes, columns) in RISK_GROUPS {
        if let Some(group) = key.strip_suffix("_name") {
            if is_risk_group(group, package, *codes) {
                return Some(GroupColumn::Name { group });
            }
        }
        for (suffix, label) in columns.iter() {
            if let Some(group) = key.strip_suffix(&format!("_{suffix}")) {
                if is_risk_group(group, package, *codes) {
                    return Some(GroupColumn::Value {
                        group,
                        suffix: format!("_{label}"),
                    });
                }
            }
        }
    }
    None
}

/// Converts `__` between a name character and a letter to `:`, e.g. `uro__city` to `uro:city`.
/// Leading underscores are kept.
fn colon_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    let mut out = String::with_capacity(key.len());
    let mut i = 0;
    while i < chars.len() {
        let is_separator = chars[i] == '_'
            && chars.get(i + 1) == Some(&'_')
            && i > 0
            && chars[i - 1].is_ascii_alphanumeric()
            && chars.get(i + 2).is_some_and(|c| c.is_ascii_alphabetic());
        if is_separator {
            out.push(':');
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn reject_colon(key: &str) -> Result<(), String> {
    match key.contains(':') {
        true => Err(format!(
            "key {key:?} has a colon, which is not a valid property ID"
        )),
        false => Ok(()),
    }
}

/// Labels of the risk and generic attribute groups, keyed by group key.
#[derive(Debug, Default)]
pub struct GroupLabels(HashMap<String, String>);

impl GroupLabels {
    /// Collects the labels from the `{group}_name` columns of `props`.
    pub fn add_feature(&mut self, props: &Map<String, Value>) {
        for (key, value) in props {
            if let (Some(GroupColumn::Name { group }), Some(label)) =
                (parse_group_column(key), value.as_str())
            {
                self.0.insert(group.to_string(), label.to_string());
            }
        }
    }

    /// Returns the legacy key of `key`, or `None` for a `{group}_name` column, which has none.
    pub fn legacy_key(&self, key: &str) -> Result<Option<String>, String> {
        reject_colon(key)?;
        match parse_group_column(key) {
            Some(GroupColumn::Name { .. }) => Ok(None),
            Some(GroupColumn::Value { group, suffix }) => {
                let label = self
                    .0
                    .get(group)
                    .ok_or_else(|| format!("no `{group}_name` label for column {key:?}"))?;
                Ok(Some(format!("{label}{suffix}")))
            }
            None => Ok(Some(colon_key(key))),
        }
    }

    /// Returns `object` with legacy keys, dropping the `{group}_name` columns.
    pub fn legacy_object(&self, object: Map<String, Value>) -> Result<Map<String, Value>, String> {
        let mut out = Map::new();
        for (key, value) in object {
            let Some(legacy) = self.legacy_key(&key)? else {
                continue;
            };
            let value = if NESTED_JSON_KEYS.contains(&key.as_str()) {
                legacy_nested_json(value)?
            } else {
                value
            };
            if out.insert(legacy.clone(), value).is_some() {
                return Err(format!("legacy key {legacy:?} occurs twice"));
            }
        }
        Ok(out)
    }

    /// Returns `value` with legacy keys in every object at any depth.
    pub fn legacy_value(&self, value: Value) -> Result<Value, String> {
        match value {
            Value::Object(object) => {
                let object = self.legacy_object(object)?;
                Ok(Value::Object(
                    object
                        .into_iter()
                        .map(|(k, v)| Ok((k, self.legacy_value(v)?)))
                        .collect::<Result<_, String>>()?,
                ))
            }
            Value::Array(arr) => Ok(Value::Array(
                arr.into_iter()
                    .map(|v| self.legacy_value(v))
                    .collect::<Result<_, _>>()?,
            )),
            other => Ok(other),
        }
    }
}

/// Converts the keys of the nested JSON `value` at any depth by `colon_key`.
fn colon_keys(value: Value) -> Result<Value, String> {
    match value {
        Value::Object(object) => Ok(Value::Object(
            object
                .into_iter()
                .map(|(k, v)| {
                    reject_colon(&k)?;
                    Ok((colon_key(&k), colon_keys(v)?))
                })
                .collect::<Result<_, String>>()?,
        )),
        Value::Array(arr) => Ok(Value::Array(
            arr.into_iter().map(colon_keys).collect::<Result<_, _>>()?,
        )),
        other => Ok(other),
    }
}

/// Converts the keys in a nested attributes JSON string by `colon_key`.
fn legacy_nested_json(value: Value) -> Result<Value, String> {
    let Value::String(json) = &value else {
        return Ok(value);
    };
    let Ok(parsed) = serde_json::from_str::<Value>(json) else {
        return Ok(value);
    };
    let converted = colon_keys(parsed)?;
    serde_json::to_string(&converted)
        .map(Value::String)
        .map_err(|e| format!("Failed to serialize nested attributes: {e}"))
}

/// Returns the properties of one feature with legacy keys, labelled by its own `{group}_name` columns.
pub fn legacy_feature(props: Map<String, Value>) -> Result<Map<String, Value>, String> {
    let mut labels = GroupLabels::default();
    labels.add_feature(&props);
    labels.legacy_object(props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn object(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn test_legacy_feature() {
        let props = object(json!({
            "gml_id": "b1",
            "_lod": 1,
            "bldg__class": "普通建物",
            "uro__BuildingDetailAttribute_uro__landUseType": "住宅用地",
            "fld_31_2_2_name": "荒川水系神田川流域（都道府県管理区間）_L2（想定最大規模）",
            "fld_31_2_2_rank": "0.5m未満",
            "fld_31_2_2_rank_code": "1",
            "htd_2_name": "高潮浸水想定_東京都高潮浸水想定区域図",
            "htd_2_depth": "0.9",
            "lsld_1_name": "土砂災害リスク_土石流",
            "lsld_1_area_type_code": 1,
            "gen_1_name": "延べ面積換算係数",
            "gen_1_value": "0.65",
            "gen_2_name": "高さ",
            "gen_2_value": 1.5,
            "gen_2_uom": "m",
            "attributes": r#"{"gml__id":"b1","feature_type":"bldg:Building","ancestors":[{"uro__lodType":"2"}]}"#,
        }));
        let legacy = legacy_feature(props).unwrap();
        assert_eq!(
            Value::Object(legacy),
            json!({
                "gml_id": "b1",
                "_lod": 1,
                "bldg:class": "普通建物",
                "uro:BuildingDetailAttribute_uro:landUseType": "住宅用地",
                "荒川水系神田川流域（都道府県管理区間）_L2（想定最大規模）_浸水ランク": "0.5m未満",
                "荒川水系神田川流域（都道府県管理区間）_L2（想定最大規模）_浸水ランクコード": "1",
                "高潮浸水想定_東京都高潮浸水想定区域図_浸水深": "0.9",
                "土砂災害リスク_土石流_区域区分コード": 1,
                "延べ面積換算係数": "0.65",
                "高さ": 1.5,
                "高さ_uom": "m",
                "attributes": r#"{"gml:id":"b1","feature_type":"bldg:Building","ancestors":[{"uro:lodType":"2"}]}"#,
            })
        );
    }

    #[test]
    fn test_colon_key_keeps_leading_underscores() {
        assert_eq!(colon_key("__feature_type"), "__feature_type");
        assert_eq!(colon_key("_lod"), "_lod");
        assert_eq!(colon_key("gml_id"), "gml_id");
        assert_eq!(colon_key("uro__rank_code"), "uro:rank_code");
    }

    #[test]
    fn test_colon_in_flow_key_is_rejected() {
        let props = object(json!({"bldg:class": "普通建物"}));
        assert!(legacy_feature(props).is_err());
        let props = object(json!({"attributes": r#"{"bldg:class":"普通建物"}"#}));
        assert!(legacy_feature(props).is_err());
    }

    #[test]
    fn test_value_column_without_label_is_rejected() {
        let props = object(json!({"fld_1_1_1_rank": "0.5m未満"}));
        assert!(legacy_feature(props).is_err());
    }

    #[test]
    fn test_schema_keys_labelled_by_features() {
        let mut labels = GroupLabels::default();
        labels.add_feature(&object(
            json!({"fld_1_1_1_name": "利根川水系利根川（国管理区間）_L1（計画規模）"}),
        ));
        let schema =
            json!({"gml_id": {}, "fld_1_1_1_name": {}, "fld_1_1_1_depth": {}, "bldg__class": {}});
        assert_eq!(
            labels.legacy_value(schema).unwrap(),
            json!({
                "gml_id": {},
                "利根川水系利根川（国管理区間）_L1（計画規模）_浸水深": {},
                "bldg:class": {},
            })
        );
    }
}

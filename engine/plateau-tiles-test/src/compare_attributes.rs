use serde_json::Value;
use std::collections::HashMap;

/// Detects if the given path is a risk type based on known risk type identifiers.
/// Risk types: fld, tnm, htd, ifld, rfld
fn is_risk_type(path: &str) -> bool {
    const RISK_TYPES: &[&str] = &["fld", "tnm", "htd", "ifld", "rfld"];
    RISK_TYPES.iter().any(|risk_type| {
        path.contains(&format!("_op_{}_", risk_type))
            || path.contains(&format!("_op_{}", risk_type))
    })
}

/// Creates a composite key for feature comparison.
/// For risk types (fld, tnm, htd, ifld, rfld) which don't have gml_id,
/// use {path}/{uro:rank_code} to create a unique key.
/// For DmGeometricAttribute features (which can have multiple children per parent),
/// use gml_id + dm_geometryType_code to create a unique key.
/// For other features, use gml_id alone (extracted from props).
pub fn make_feature_key(props: &Value, path: Option<&str>) -> String {
    let getter = |key| {
        // FME has unreliable number vs string types so we convert everything to string here
        if let Some(value) = props.get(key) {
            match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            }
        } else {
            String::new()
        }
    };

    // For risk types, use path/rank_code as the key
    if let Some(p) = path {
        if is_risk_type(p) {
            let rank_code = getter("uro:rank_code");
            if !rank_code.is_empty() {
                return format!("{}/{}", p, rank_code);
            }
            let rankorg_code = getter("uro:rankOrg_code");
            if !rankorg_code.is_empty() {
                return format!("{}/{}", p, rankorg_code);
            }
            panic!(
                "both uro:rank_code and uro:rankOrg_code are missing in {}",
                p
            );
        }
    }

    // Extract gml_id from props if present
    let gml_id = getter("gml_id");

    // Check if this is a DmGeometricAttribute feature
    if let Some(dm_feature_type) = props.get("dm_feature_type").and_then(|v| v.as_str()) {
        if dm_feature_type == "DmGeometricAttribute" {
            // Use dm_geometryType_code as discriminator for DM features
            if let Some(dm_geometry_type_code) =
                props.get("dm_geometryType_code").and_then(|v| v.as_str())
            {
                return format!("{}__dm_{}", gml_id, dm_geometry_type_code);
            }
        }
    }
    gml_id.to_string()
}

#[derive(Debug)]
pub struct AttributeComparer {
    identifier: String,
    casts: HashMap<String, CastConfig>,
    values: HashMap<String, Value>,
    mismatches: Vec<(String, String, Value, Value)>,
}

#[derive(Debug, Clone)]
pub enum CastConfig {
    String,
    Float { epsilon: Option<f64> },
    Int,
    Json,
    ListToDict { key: String },
    IgnoreBoth,
}

/// Returns only the structural casts (Json, ListToDict, IgnoreBoth) that need
/// to be applied to both sides for data-equivalence checks within the same dataset.
/// Primitive casts (String, Float, Int) are dropped since they only normalize
/// FME's unreliable types toward ground truth.
pub fn structural_casts(casts: &HashMap<String, CastConfig>) -> HashMap<String, CastConfig> {
    casts
        .iter()
        .filter(|(_, v)| {
            matches!(
                v,
                CastConfig::Json | CastConfig::ListToDict { .. } | CastConfig::IgnoreBoth
            )
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

impl AttributeComparer {
    pub fn new(
        identifier: String,
        casts: HashMap<String, CastConfig>,
        values: HashMap<String, Value>,
    ) -> Self {
        Self {
            identifier,
            casts,
            values,
            mismatches: Vec::new(),
        }
    }

    fn get_nested<'a>(obj: &'a Value, path: &str) -> Option<&'a Value> {
        let tokens = Self::tokenize_path(path);
        Self::get_nested_impl(obj, &tokens)
    }

    fn tokenize_path(path: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_bracket = false;

        for ch in path.chars() {
            match ch {
                '.' if !in_bracket => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                '[' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    in_bracket = true;
                    current.push(ch);
                }
                ']' => {
                    current.push(ch);
                    tokens.push(current.clone());
                    current.clear();
                    in_bracket = false;
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    fn get_nested_impl<'a>(obj: &'a Value, tokens: &[String]) -> Option<&'a Value> {
        if tokens.is_empty() {
            return Some(obj);
        }

        let token = &tokens[0];
        if token.starts_with('[') && token.ends_with(']') {
            // Array index
            let idx_str = &token[1..token.len() - 1];
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(arr) = obj.as_array() {
                    if let Some(value) = arr.get(idx) {
                        return Self::get_nested_impl(value, &tokens[1..]);
                    }
                }
            }
        } else if let Some(obj_map) = obj.as_object() {
            // Object key
            if let Some(value) = obj_map.get(token) {
                return Self::get_nested_impl(value, &tokens[1..]);
            }
        }

        None
    }

    fn cast_attr(&self, key: &str, value: Value) -> Value {
        if let Some(cast) = self.casts.get(key) {
            match cast {
                CastConfig::String => {
                    if let Some(s) = value.as_str() {
                        Value::String(s.to_string())
                    } else {
                        Value::String(value.to_string())
                    }
                }
                CastConfig::Float { .. } => {
                    // Convert to number if possible
                    match &value {
                        Value::Number(_) => value,
                        Value::String(s) => {
                            if let Ok(f) = s.parse::<f64>() {
                                serde_json::json!(f)
                            } else {
                                value
                            }
                        }
                        _ => value,
                    }
                }
                CastConfig::Int => {
                    // Convert to integer if possible
                    match &value {
                        Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                serde_json::json!(i)
                            } else if let Some(f) = n.as_f64() {
                                serde_json::json!(f.round() as i64)
                            } else {
                                value
                            }
                        }
                        Value::String(s) => {
                            if let Ok(i) = s.parse::<i64>() {
                                serde_json::json!(i)
                            } else if let Ok(f) = s.parse::<f64>() {
                                serde_json::json!(f.round() as i64)
                            } else {
                                value
                            }
                        }
                        _ => value,
                    }
                }
                CastConfig::Json => {
                    if let Some(s) = value.as_str() {
                        serde_json::from_str(s).unwrap_or(value)
                    } else {
                        value
                    }
                }
                CastConfig::ListToDict { key: dict_key } => {
                    if let Some(arr) = value.as_array() {
                        let mut map = serde_json::Map::new();
                        for item in arr {
                            if let Some(k) = Self::get_nested(item, dict_key) {
                                if let Some(k_str) = k.as_str() {
                                    map.insert(k_str.to_string(), item.clone());
                                }
                            }
                        }
                        Value::Object(map)
                    } else {
                        value
                    }
                }
                CastConfig::IgnoreBoth => Value::Null,
            }
        } else {
            value
        }
    }

    fn compare_recurse(&mut self, key: &str, mut v1: Value, mut v2: Value) {
        if let Some(cast) = self.casts.get(key) {
            match cast {
                CastConfig::IgnoreBoth => {
                    return;
                }
                CastConfig::ListToDict { .. } | CastConfig::Json => {
                    v1 = self.cast_attr(key, v1);
                    v2 = self.cast_attr(key, v2);
                }
                _ => {
                    // apply cast only to v1
                    v1 = self.cast_attr(key, v1);
                }
            }
        };
        // If key matches in values, replace v1 completely
        if let Some(replacement) = self.values.get(key) {
            v1 = replacement.clone();
        }

        // Type checking with tolerance
        if !self.types_match(&v1, &v2) {
            if let Some(v2_bool) = v2.as_bool() {
                if self.value_as_bool(&v1) == Some(v2_bool) {
                    return;
                }
            }
            self.mismatches
                .push((self.identifier.clone(), key.to_string(), v1, v2));
            return;
        }

        match (&v1, &v2) {
            (Value::Object(obj1), Value::Object(obj2)) => {
                let all_keys: std::collections::HashSet<_> =
                    obj1.keys().chain(obj2.keys()).collect();
                for k in all_keys {
                    let new_key = if key.is_empty() {
                        format!(".{}", k)
                    } else {
                        format!("{}.{}", key, k)
                    };
                    let val1 = obj1.get(k).cloned().unwrap_or(Value::Null);
                    let val2 = obj2.get(k).cloned().unwrap_or(Value::Null);
                    self.compare_recurse(&new_key, val1, val2);
                }
            }
            (Value::Array(arr1), Value::Array(arr2)) => {
                if arr1.len() != arr2.len() {
                    self.mismatches
                        .push((self.identifier.clone(), key.to_string(), v1, v2));
                    return;
                }
                for (idx, (val1, val2)) in arr1.iter().zip(arr2.iter()).enumerate() {
                    let new_key = format!("{}[{}]", key, idx);
                    self.compare_recurse(&new_key, val1.clone(), val2.clone());
                }
            }
            (Value::String(s1), Value::String(s2)) => {
                if s1.trim() != s2.trim() {
                    self.mismatches
                        .push((self.identifier.clone(), key.to_string(), v1, v2));
                }
            }
            _ => {
                // Check if we should compare as floats with tolerance
                if let Some(CastConfig::Float { epsilon }) = self.casts.get(key) {
                    if let (Some(f1), Some(f2)) = (v1.as_f64(), v2.as_f64()) {
                        let matches = if let Some(eps) = epsilon {
                            (f1 - f2).abs() <= *eps
                        } else {
                            f1 == f2 // Exact match when no epsilon provided
                        };

                        if !matches {
                            self.mismatches.push((
                                self.identifier.clone(),
                                key.to_string(),
                                v1,
                                v2,
                            ));
                        }
                        return;
                    }
                }

                if v1 != v2 {
                    self.mismatches
                        .push((self.identifier.clone(), key.to_string(), v1, v2));
                }
            }
        }
    }

    fn types_match(&self, v1: &Value, v2: &Value) -> bool {
        matches!(
            (v1, v2),
            (Value::Null, Value::Null)
                | (Value::Bool(_), Value::Bool(_))
                | (Value::Number(_), Value::Number(_))
                | (Value::String(_), Value::String(_))
                | (Value::Array(_), Value::Array(_))
                | (Value::Object(_), Value::Object(_))
        )
    }

    fn value_as_bool(&self, value: &Value) -> Option<bool> {
        match value {
            Value::Bool(b) => Some(*b),
            Value::Number(n) => Some(n.as_f64().map(|f| f != 0.0).unwrap_or(false)),
            Value::String(s) => {
                let lower = s.to_lowercase();
                if lower == "true" || lower == "1" {
                    Some(true)
                } else if lower == "false" || lower == "0" || lower.is_empty() {
                    Some(false)
                } else {
                    None
                }
            }
            Value::Null => Some(false),
            _ => None,
        }
    }

    pub fn compare(&mut self, attr1: &Value, attr2: &Value) -> Result<(), String> {
        if attr1.is_null() || attr2.is_null() {
            return Err(format!(
                "Missing attributes for identifier: {} for {}",
                self.identifier,
                if attr1.is_null() { "FME" } else { "flow" }
            ));
        }

        self.compare_recurse("", attr1.clone(), attr2.clone());

        if !self.mismatches.is_empty() {
            for (gid, k, v1, v2) in &self.mismatches {
                eprintln!("MISMATCH gml_id={} key={} v1={:?} v2={:?}", gid, k, v1, v2);
            }
            return Err(format!(
                "Attribute mismatches found for identifier: {}",
                self.identifier
            ));
        }

        Ok(())
    }
}

pub fn analyze_attributes(
    ident: &str,
    attr1: &Value,
    attr2: &Value,
    casts: HashMap<String, CastConfig>,
    values: HashMap<String, Value>,
) -> Result<(), String> {
    let mut comparer = AttributeComparer::new(ident.to_string(), casts, values);
    comparer.compare(attr1, attr2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cast_string_match() {
        let mut casts = HashMap::new();
        casts.insert(".value".to_string(), CastConfig::String);

        let v1 = json!({"value": 123});
        let v2 = json!({"value": "123"});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cast_string_mismatch() {
        let mut casts = HashMap::new();
        casts.insert(".value".to_string(), CastConfig::String);

        let v1 = json!({"value": 123});
        let v2 = json!({"value": "456"});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_cast_json_match() {
        let mut casts = HashMap::new();
        casts.insert(".data".to_string(), CastConfig::Json);

        let v1 = json!({"data": "{\"key\": \"value\"}"});
        let v2 = json!({"data": {"key": "value"}});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cast_json_mismatch() {
        let mut casts = HashMap::new();
        casts.insert(".data".to_string(), CastConfig::Json);

        let v1 = json!({"data": "{\"key\": \"value1\"}"});
        let v2 = json!({"data": {"key": "value2"}});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_cast_list_to_dict_match() {
        let mut casts = HashMap::new();
        casts.insert(
            ".items".to_string(),
            CastConfig::ListToDict {
                key: ".id".to_string(),
            },
        );

        let v1 = json!({"items": [
            {"id": "a", "value": 1},
            {"id": "b", "value": 2}
        ]});
        let v2 = json!({"items": {
            "a": {"id": "a", "value": 1},
            "b": {"id": "b", "value": 2}
        }});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cast_list_to_dict_mismatch() {
        let mut casts = HashMap::new();
        casts.insert(
            ".items".to_string(),
            CastConfig::ListToDict {
                key: ".id".to_string(),
            },
        );

        let v1 = json!({"items": [
            {"id": "a", "value": 1},
            {"id": "b", "value": 2}
        ]});
        let v2 = json!({"items": {
            "a": {"id": "a", "value": 99},
            "b": {"id": "b", "value": 2}
        }});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_no_cast_match() {
        let casts = HashMap::new();

        let v1 = json!({"x": 1, "y": "test"});
        let v2 = json!({"x": 1, "y": "test"});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_cast_mismatch() {
        let casts = HashMap::new();

        let v1 = json!({"x": 1, "y": "test"});
        let v2 = json!({"x": 2, "y": "test"});

        let result = analyze_attributes("test", &v1, &v2, casts, HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_cast_float() {
        let mut casts = HashMap::new();
        casts.insert(".value".to_string(), CastConfig::Float { epsilon: None });

        // Exact match - should pass
        let v1 = json!({"value": 1.5});
        let v2 = json!({"value": 1.5});
        assert!(analyze_attributes("test", &v1, &v2, casts.clone(), HashMap::new()).is_ok());

        // String to float conversion - should pass
        let v1 = json!({"value": "1.5"});
        let v2 = json!({"value": 1.5});
        assert!(analyze_attributes("test", &v1, &v2, casts.clone(), HashMap::new()).is_ok());

        // Different values with no epsilon - should fail
        let v1 = json!({"value": 1.5});
        let v2 = json!({"value": 1.500001});
        assert!(analyze_attributes("test", &v1, &v2, casts, HashMap::new()).is_err());

        // With epsilon tolerance - should pass
        let mut casts_eps = HashMap::new();
        casts_eps.insert(
            ".value".to_string(),
            CastConfig::Float {
                epsilon: Some(0.001),
            },
        );
        let v1 = json!({"value": 1.5});
        let v2 = json!({"value": 1.500001});
        assert!(analyze_attributes("test", &v1, &v2, casts_eps, HashMap::new()).is_ok());
    }
}

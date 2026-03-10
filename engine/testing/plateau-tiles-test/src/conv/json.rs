use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::apply_casts_to_value;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn write_json(
    source: &Path,
    output: &Path,
    json_path: Option<&str>,
    casts: &HashMap<String, CastConfigValue>,
) -> Result<(), String> {
    let content = fs::read(source).map_err(|e| format!("Failed to read {:?}: {}", source, e))?;
    let mut data: Value = serde_json::from_slice(&content)
        .map_err(|e| format!("Failed to parse JSON {:?}: {}", source, e))?;

    if let Some(json_path) = json_path {
        data = json_path
            .split('.')
            .filter(|t| !t.is_empty())
            .try_fold(data, |cur, token| {
                cur.get(token)
                    .cloned()
                    .ok_or_else(|| format!("json_path '{}' not found in {:?}", json_path, source))
            })?;
    }

    if !casts.is_empty() {
        let cast_map = convert_casts(casts)?;
        data = apply_casts_to_value(data, "", &cast_map);
    }

    let out =
        serde_json::to_vec_pretty(&data).map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::write(output, out).map_err(|e| format!("Failed to write {:?}: {}", output, e))?;
    Ok(())
}

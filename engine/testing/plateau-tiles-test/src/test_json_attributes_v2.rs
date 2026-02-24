use crate::cast_config::CastConfigValue;
use crate::compare_attributes::analyze_attributes;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct JsonFileV2Config {
    pub flow: FlowSource,
    pub truth: TruthSource,
    #[serde(default)]
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

#[derive(Debug, Deserialize)]
pub struct FlowSource {
    pub path: String,
    pub json_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TruthSource {
    pub path: String,
}

fn extract_json_path(value: &Value, path: &str) -> Option<Value> {
    let mut current = value;
    for token in path.split('.').filter(|t| !t.is_empty()) {
        current = current.get(token)?;
    }
    Some(current.clone())
}

pub fn test_json_attributes_v2(
    output_dir: &Path,
    testcase_dir: &Path,
    config: &HashMap<String, JsonFileV2Config>,
) -> Result<(), String> {
    for (name, cfg) in config {
        let flow_file = output_dir.join(&cfg.flow.path);
        assert!(flow_file.exists(), "missing flow file {:?}", flow_file);

        let truth_file = testcase_dir.join(&cfg.truth.path);
        assert!(truth_file.exists(), "missing truth file {:?}", truth_file);

        let flow_content = fs::read(&flow_file).map_err(|e| e.to_string())?;
        let flow_data: Value = serde_json::from_slice(&flow_content)
            .map_err(|e| format!("Failed to parse flow JSON {}: {}", cfg.flow.path, e))?;

        let flow_data = if let Some(ref json_path) = cfg.flow.json_path {
            extract_json_path(&flow_data, json_path).ok_or_else(|| {
                format!(
                    "json_path '{}' not found in flow file {}",
                    json_path, cfg.flow.path
                )
            })?
        } else {
            flow_data
        };

        let truth_content = fs::read(&truth_file).map_err(|e| e.to_string())?;
        let truth_data: Value = serde_json::from_slice(&truth_content)
            .map_err(|e| format!("Failed to parse truth JSON {}: {}", cfg.truth.path, e))?;

        tracing::debug!("Comparing JSON (v2): {} flow={}", name, cfg.flow.path);

        let casts = if let Some(ref casts_cfg) = cfg.casts {
            crate::cast_config::convert_casts(casts_cfg)?
        } else {
            HashMap::new()
        };

        // Use analyze_attributes: flow as attr1 (casts applied to it), truth as attr2
        analyze_attributes(name, &flow_data, &truth_data, casts, HashMap::new())?;

        tracing::debug!("OK: json_attributes_v2 '{}'", name);
    }

    Ok(())
}

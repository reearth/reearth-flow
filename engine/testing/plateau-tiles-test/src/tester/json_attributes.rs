use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct JsonFileConfig {
    pub path: String,
    #[serde(default)]
    pub extracted: bool,
    #[serde(default)]
    pub must_not_exist: bool,
    pub casts: Option<HashMap<String, CastConfigValue>>,
    pub values: Option<HashMap<String, Value>>,
}

pub fn test_json_attributes(
    truth_source_path: &Path,
    flow_source_path: &Path,
    truth_extracted_path: &Path,
    flow_extracted_path: &Path,
    config: &HashMap<String, JsonFileConfig>,
) -> Result<(), String> {
    for (name, file_cfg) in config {
        let file_path = &file_cfg.path;
        let (truth_base, flow_base) = if file_cfg.extracted {
            (truth_extracted_path, flow_extracted_path)
        } else {
            (truth_source_path, flow_source_path)
        };
        let truth_file = truth_base.join(file_path);
        let flow_file = flow_base.join(file_path);

        if file_cfg.must_not_exist {
            assert!(
                !flow_file.exists(),
                "flow file should not exist: {:?}",
                flow_file
            );
            tracing::debug!("Verified files do not exist: {} ({})", name, file_path);
            continue;
        }

        assert!(truth_file.exists(), "missing truth file {:?}", truth_file);
        assert!(flow_file.exists(), "missing flow file {:?}", flow_file);

        let truth_content = fs::read(&truth_file).map_err(|e| e.to_string())?;
        let truth_content = if truth_content.starts_with(&[0xEF, 0xBB, 0xBF]) {
            &truth_content[3..]
        } else {
            &truth_content
        };
        let truth_data: serde_json::Value = serde_json::from_slice(truth_content)
            .map_err(|e| format!("Failed to parse truth JSON: {}", e))?;
        let flow_content = fs::read(&flow_file).map_err(|e| e.to_string())?;
        let flow_data: serde_json::Value = serde_json::from_slice(&flow_content)
            .map_err(|e| format!("Failed to parse Flow JSON: {}", e))?;

        let casts = if let Some(casts_cfg) = &file_cfg.casts {
            convert_casts(casts_cfg)?
        } else {
            HashMap::new()
        };

        let values = file_cfg.values.clone().unwrap_or_default();

        tracing::debug!("Comparing JSON file: {} ({})", name, file_path);
        analyze_attributes(name, &truth_data, &flow_data, casts, values)?;
    }

    Ok(())
}

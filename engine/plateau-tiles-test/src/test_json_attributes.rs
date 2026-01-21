use crate::cast_config::{convert_casts, CastConfigValue};
use crate::compare_attributes::analyze_attributes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct JsonFileConfig {
    pub path: String,
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

pub fn test_json_attributes(
    fme_path: &Path,
    flow_path: &Path,
    config: &HashMap<String, JsonFileConfig>,
) -> Result<(), String> {
    for (name, file_cfg) in config {
        let file_path = &file_cfg.path;
        let fme_file = fme_path.join(file_path);
        let flow_file = flow_path.join(file_path);

        // Read FME file or treat as empty if doesn't exist
        let fme_data: serde_json::Value = if fme_file.exists() {
            let fme_content = fs::read(&fme_file).map_err(|e| e.to_string())?;
            let fme_content = if fme_content.starts_with(&[0xEF, 0xBB, 0xBF]) {
                &fme_content[3..]
            } else {
                &fme_content
            };
            serde_json::from_slice(fme_content)
                .map_err(|e| format!("Failed to parse FME JSON: {}", e))?
        } else {
            serde_json::json!({})
        };

        // Read Flow file or treat as empty if doesn't exist
        let flow_data: serde_json::Value = if flow_file.exists() {
            let flow_content = fs::read(&flow_file).map_err(|e| e.to_string())?;
            serde_json::from_slice(&flow_content)
                .map_err(|e| format!("Failed to parse Flow JSON: {}", e))?
        } else {
            serde_json::json!({})
        };

        let casts = if let Some(casts_cfg) = &file_cfg.casts {
            convert_casts(casts_cfg)?
        } else {
            HashMap::new()
        };

        tracing::debug!("Comparing JSON file: {} ({})", name, file_path);
        analyze_attributes(name, &fme_data, &flow_data, casts)?;
    }

    Ok(())
}

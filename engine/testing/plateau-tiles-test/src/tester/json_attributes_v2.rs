use crate::compare_attributes::analyze_attributes;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct JsonFileV2Config {
    pub path: String,
}

pub fn test_json_attributes_v2(
    output_dir: &Path,
    testcase_dir: &Path,
    config: &HashMap<String, JsonFileV2Config>,
) -> Result<(), String> {
    for (name, cfg) in config {
        let flow_file = output_dir.join("flow_extracted").join(&cfg.path);
        assert!(flow_file.exists(), "missing flow file {:?}", flow_file);

        let truth_file = testcase_dir.join(&cfg.path);
        assert!(truth_file.exists(), "missing truth file {:?}", truth_file);

        let flow_content = fs::read(&flow_file).map_err(|e| e.to_string())?;
        let flow_data: Value = serde_json::from_slice(&flow_content)
            .map_err(|e| format!("Failed to parse flow JSON {}: {}", cfg.path, e))?;

        let truth_content = fs::read(&truth_file).map_err(|e| e.to_string())?;
        let truth_data: Value = serde_json::from_slice(&truth_content)
            .map_err(|e| format!("Failed to parse truth JSON {}: {}", cfg.path, e))?;

        tracing::debug!("Comparing JSON (v2): {} path={}", name, cfg.path);

        analyze_attributes(
            name,
            &truth_data,
            &flow_data,
            HashMap::new(),
            HashMap::new(),
        )?;

        tracing::debug!("OK: json_attributes_v2 '{}'", name);
    }

    Ok(())
}

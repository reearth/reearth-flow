use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct KeyOrderConfig {
    pub path: String,
    #[serde(default)]
    pub extracted: bool,
    pub json_path: String,
    pub expected_order: Vec<String>,
}

/// Extract a nested value from a JSON value using a dot-separated path (e.g. ".properties").
fn get_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.').filter(|s| !s.is_empty()) {
        current = current.get(segment)?;
    }
    Some(current)
}

pub fn test_json_object_key_order(
    flow_source_path: &Path,
    flow_extracted_path: &Path,
    cfg: &KeyOrderConfig,
) -> Result<(), String> {
    let flow_base = if cfg.extracted {
        flow_extracted_path
    } else {
        flow_source_path
    };

    let flow_file = flow_base.join(&cfg.path);
    assert!(flow_file.exists(), "missing flow file {:?}", flow_file);

    let flow_content = fs::read(&flow_file).map_err(|e| e.to_string())?;
    let flow_data: Value = serde_json::from_slice(&flow_content)
        .map_err(|e| format!("Failed to parse Flow JSON: {}", e))?;

    let obj = get_at_path(&flow_data, &cfg.json_path)
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            format!(
                "json_path '{}' does not point to an object in {}",
                cfg.json_path, cfg.path
            )
        })?;

    let actual_keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

    // Filter actual keys to only those in expected_order
    let actual_ordered: Vec<&str> = actual_keys
        .iter()
        .copied()
        .filter(|k| cfg.expected_order.iter().any(|e| e == k))
        .collect();

    let expected: Vec<&str> = cfg.expected_order.iter().map(|s| s.as_str()).collect();

    if actual_ordered != expected {
        // Find first mismatch index
        let mismatch_idx = actual_ordered
            .iter()
            .zip(expected.iter())
            .position(|(a, e)| a != e)
            .unwrap_or(actual_ordered.len().min(expected.len()));

        let actual_at = actual_ordered.get(mismatch_idx).unwrap_or(&"<missing>");
        let expected_at = expected.get(mismatch_idx).unwrap_or(&"<missing>");

        return Err(format!(
            "Key order mismatch at '{}' in {} at index {}:\n  expected: {}\n  actual:   {}",
            cfg.json_path, cfg.path, mismatch_idx, expected_at, actual_at
        ));
    }

    tracing::debug!(
        "Key order OK at '{}' in {} ({} keys checked)",
        cfg.json_path,
        cfg.path,
        expected.len()
    );

    Ok(())
}

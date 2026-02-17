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

fn check_key_order(
    actual_keys: &[&str],
    expected: &[&str],
    json_path: &str,
    file_path: &str,
) -> Result<(), String> {
    // Filter actual keys to only those in expected
    let actual_ordered: Vec<&str> = actual_keys
        .iter()
        .copied()
        .filter(|k| expected.contains(k))
        .collect();

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
            json_path, file_path, mismatch_idx, expected_at, actual_at
        ));
    }

    Ok(())
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
    let expected: Vec<&str> = cfg.expected_order.iter().map(|s| s.as_str()).collect();

    check_key_order(&actual_keys, &expected, &cfg.json_path, &cfg.path)?;

    tracing::debug!(
        "Key order OK at '{}' in {} ({} keys checked)",
        cfg.json_path,
        cfg.path,
        expected.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_expected_key_is_detected() {
        // A key listed in expected but absent from actual must not pass
        // silently — the filtered actual list will be shorter, causing a mismatch.
        let actual = vec!["a", "c"];
        let expected = vec!["a", "b", "c"];
        let err = check_key_order(&actual, &expected, "", "").unwrap_err();
        assert!(
            err.contains("Key order mismatch"),
            "missing key should cause mismatch, got: {err}"
        );
    }

    #[test]
    fn wrong_order_is_detected() {
        let actual = vec!["a", "b", "c"];
        let expected = vec!["c", "b", "a"];
        let err = check_key_order(&actual, &expected, "", "").unwrap_err();
        assert!(
            err.contains("Key order mismatch"),
            "wrong order should cause mismatch, got: {err}"
        );
    }
}

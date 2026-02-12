use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct JsonFileV2Config {
    pub flow: FlowSource,
    pub truth: TruthSource,
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

        if flow_data != truth_data {
            let diff = json_diff("", &truth_data, &flow_data);
            if !diff.is_empty() {
                for d in &diff {
                    eprintln!("MISMATCH [{}] {}", name, d);
                }
                return Err(format!(
                    "json_attributes_v2 '{}': {} mismatches found",
                    name,
                    diff.len()
                ));
            }
        }

        tracing::debug!("OK: json_attributes_v2 '{}'", name);
    }

    Ok(())
}

/// Produces human-readable diff lines between two JSON values.
fn json_diff(path: &str, expected: &Value, actual: &Value) -> Vec<String> {
    let mut diffs = Vec::new();

    match (expected, actual) {
        (Value::Object(exp), Value::Object(act)) => {
            let all_keys: std::collections::HashSet<_> =
                exp.keys().chain(act.keys()).collect();
            for k in all_keys {
                let child_path = if path.is_empty() {
                    format!(".{}", k)
                } else {
                    format!("{}.{}", path, k)
                };
                match (exp.get(k), act.get(k)) {
                    (Some(e), Some(a)) => {
                        diffs.extend(json_diff(&child_path, e, a));
                    }
                    (Some(e), None) => {
                        diffs.push(format!("key {} missing in flow (expected {:?})", child_path, e));
                    }
                    (None, Some(a)) => {
                        diffs.push(format!("key {} unexpected in flow (got {:?})", child_path, a));
                    }
                    (None, None) => unreachable!(),
                }
            }
        }
        (Value::Array(exp), Value::Array(act)) => {
            if exp.len() != act.len() {
                diffs.push(format!(
                    "key {} array length mismatch: expected {} got {}",
                    path,
                    exp.len(),
                    act.len()
                ));
            }
            for (i, (e, a)) in exp.iter().zip(act.iter()).enumerate() {
                let child_path = format!("{}[{}]", path, i);
                diffs.extend(json_diff(&child_path, e, a));
            }
        }
        _ => {
            if expected != actual {
                diffs.push(format!(
                    "key {} expected {:?} got {:?}",
                    path, expected, actual
                ));
            }
        }
    }

    diffs
}

use serde;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::path::{Path, PathBuf};

pub enum SerdeFormat {
    Json,
    Yaml,
    Unknown,
}

/// Expands !include directives in YAML content
///
/// This function processes YAML strings containing !include directives and replaces them
/// with the content of the referenced files. The file paths are resolved relative to the
/// workflow file's directory if a base_path is provided.
pub fn expand_yaml_includes(yaml_content: &str, base_path: Option<&Path>) -> crate::Result<String> {
    use regex::Regex;

    // Match patterns like: !include path/to/file.txt
    let include_pattern = Regex::new(r"!include\s+([^\s\r\n]+)")
        .map_err(|e| crate::Error::Serde(format!("Failed to create regex: {}", e)))?;

    let mut expanded = yaml_content.to_string();
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 10; // Prevent infinite loops

    // Keep expanding until no more !include directives are found
    while include_pattern.is_match(&expanded) && iterations < MAX_ITERATIONS {
        iterations += 1;
        let mut new_content = String::new();
        let mut last_end = 0;

        for cap in include_pattern.captures_iter(&expanded) {
            let full_match = cap.get(0).unwrap();
            let match_start = full_match.start();
            let match_end = full_match.end();
            let file_path_str = cap.get(1).unwrap().as_str();

            // Add content before this match
            new_content.push_str(&expanded[last_end..match_start]);

            // Resolve the file path
            let resolved_path = if let Some(base) = base_path {
                let mut path = PathBuf::from(base);
                path.push(file_path_str);
                path
            } else {
                PathBuf::from(file_path_str)
            };

            // Read the included file
            let included_content = std::fs::read_to_string(&resolved_path)
                .map_err(|e| crate::Error::Serde(format!(
                    "Failed to read included file {:?}: {}", resolved_path, e
                )))?;

            // Find the indentation of the line containing !include
            let line_start = expanded[..match_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_before_include = &expanded[line_start..match_start];
            let base_indent = line_before_include.chars().take_while(|c| c.is_whitespace()).count();

            // Determine if this is a scalar value context (key: !include) or object/array context (- !include)
            let is_scalar_context = line_before_include.trim_start().contains(':');
            let is_array_item = line_before_include.trim_start().starts_with('-');

            let formatted_content = if is_scalar_context && !is_array_item {
                // Scalar context: format as YAML literal block scalar
                // The '|-' chomps the final newline, and content is indented relative to the key
                format!("|-\n{}",
                    included_content.trim().lines()
                        .map(|line| format!("{}{}", " ".repeat(base_indent + 2), line))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else if is_array_item {
                // Array item context (- !include): first line continues after dash, rest indented
                let lines: Vec<&str> = included_content.trim().lines().collect();
                if lines.is_empty() {
                    String::new()
                } else {
                    let mut result = lines[0].to_string(); // First line: no extra indent
                    for line in &lines[1..] {
                        if line.trim().is_empty() {
                            result.push('\n');
                        } else {
                            // Subsequent lines: indent to align with first line content (base + 2 for "- ")
                            result.push_str(&format!("\n{}{}", " ".repeat(base_indent + 2), line));
                        }
                    }
                    result
                }
            } else {
                // Object context: insert raw YAML with proper indentation
                included_content.trim().lines()
                    .map(|line| {
                        if line.trim().is_empty() {
                            String::new()
                        } else {
                            format!("{}{}", " ".repeat(base_indent), line)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            new_content.push_str(&formatted_content);
            last_end = match_end;
        }

        // Add remaining content after last match
        new_content.push_str(&expanded[last_end..]);
        expanded = new_content;
    }

    if iterations >= MAX_ITERATIONS {
        return Err(crate::Error::Serde(
            "Maximum !include expansion depth exceeded (possible circular include)".to_string()
        ));
    }

    Ok(expanded)
}

pub fn from_str<'a, T>(s: &'a str) -> crate::Result<T>
where
    T: serde::Deserialize<'a>,
{
    let format = determine_format(s);
    match format {
        SerdeFormat::Json => {
            serde_json::from_str(s).map_err(|e| crate::Error::Serde(format!("{e}")))
        }
        SerdeFormat::Yaml => {
            serde_yaml::from_str(s).map_err(|e| crate::Error::Serde(format!("{e}")))
        }
        SerdeFormat::Unknown => Err(crate::Error::Serde("Unknown format".to_string())),
    }
}

pub fn determine_format(input: &str) -> SerdeFormat {
    if serde_json::from_str::<JsonValue>(input).is_ok() {
        SerdeFormat::Json
    } else if serde_yaml::from_str::<YamlValue>(input).is_ok() {
        SerdeFormat::Yaml
    } else {
        SerdeFormat::Unknown
    }
}

pub fn merge_value(a: &mut JsonValue, b: JsonValue) {
    match (a, b) {
        (JsonValue::Object(ref mut a_map), JsonValue::Object(b_map)) => {
            for (k, v) in b_map {
                merge_value(a_map.entry(k).or_insert(JsonValue::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

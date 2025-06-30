use serde;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

pub enum SerdeFormat {
    Json,
    Yaml,
    Unknown,
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

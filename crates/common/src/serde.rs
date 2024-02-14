use serde;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

enum SerdeFormat {
    Json,
    Yaml,
    Unknown,
}

pub fn from_str<'a, T>(s: &'a str) -> anyhow::Result<T>
where
    T: serde::Deserialize<'a>,
{
    let format = determine_format(s);
    match format {
        SerdeFormat::Json => serde_json::from_str(s).map_err(|e| anyhow::anyhow!(e)),
        SerdeFormat::Yaml => serde_yaml::from_str(s).map_err(|e| anyhow::anyhow!(e)),
        SerdeFormat::Unknown => Err(anyhow::anyhow!("Unknown format")),
    }
}

fn determine_format(input: &str) -> SerdeFormat {
    if serde_json::from_str::<JsonValue>(input).is_ok() {
        SerdeFormat::Json
    } else if serde_yaml::from_str::<YamlValue>(input).is_ok() {
        SerdeFormat::Yaml
    } else {
        SerdeFormat::Unknown
    }
}

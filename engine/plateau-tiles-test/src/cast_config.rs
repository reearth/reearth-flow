use crate::compare_attributes::CastConfig;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CastConfigValue {
    Simple(String),
    ComplexListToDict {
        comparator: String,
        key: String,
    },
    ComplexFloat {
        comparator: String,
        epsilon: Option<f64>,
    },
}

pub fn convert_casts(
    casts_cfg: &HashMap<String, CastConfigValue>,
) -> Result<HashMap<String, CastConfig>, String> {
    let mut casts = HashMap::new();
    for (key, value) in casts_cfg {
        let cast = match value {
            CastConfigValue::Simple(s) => match s.as_str() {
                "string" => CastConfig::String,
                "float" => CastConfig::Float { epsilon: None },
                "int" => CastConfig::Int,
                "json" => CastConfig::Json,
                "ignore_both" => CastConfig::IgnoreBoth,
                _ => return Err(format!("Unknown cast type: {}", s)),
            },
            CastConfigValue::ComplexListToDict {
                comparator,
                key: dict_key,
            } => {
                if comparator == "list_to_dict" {
                    CastConfig::ListToDict {
                        key: dict_key.clone(),
                    }
                } else {
                    return Err(format!("Unknown comparator: {}", comparator));
                }
            }
            CastConfigValue::ComplexFloat {
                comparator,
                epsilon,
            } => {
                if comparator == "float" {
                    CastConfig::Float { epsilon: *epsilon }
                } else {
                    return Err(format!("Unknown comparator: {}", comparator));
                }
            }
        };
        casts.insert(key.clone(), cast);
    }
    Ok(casts)
}

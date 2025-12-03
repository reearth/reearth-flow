use crate::compare_attributes::CastConfig;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CastConfigValue {
    Simple(String),
    Complex {
        comparator: String,
        key: String,
    },
}

pub fn convert_casts(casts_cfg: &HashMap<String, CastConfigValue>) -> Result<HashMap<String, CastConfig>, String> {
    let mut casts = HashMap::new();
    for (key, value) in casts_cfg {
        let cast = match value {
            CastConfigValue::Simple(s) => match s.as_str() {
                "string" => CastConfig::String,
                "json" => CastConfig::Json,
                _ => return Err(format!("Unknown cast type: {}", s)),
            },
            CastConfigValue::Complex {
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
        };
        casts.insert(key.clone(), cast);
    }
    Ok(casts)
}

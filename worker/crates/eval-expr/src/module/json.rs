use rhai::export_module;

#[export_module]
pub(crate) mod json_module {
    use reearth_flow_common::json;
    use rhai::plugin::*;

    use crate::utils::{dynamic_to_value, value_to_dynamic};

    pub fn find_value_by_json_path(content: rhai::Map, json_path: &str) -> Vec<Dynamic> {
        let mut target = serde_json::Map::new();
        for (key, value) in content {
            target.insert(key.to_string(), dynamic_to_value(&value));
        }
        let Ok(result) = json::find_by_json_path(serde_json::Value::Object(target), json_path)
        else {
            return vec![];
        };
        result.into_iter().map(|v| value_to_dynamic(&v)).collect()
    }

    pub fn exists_value_by_json_path(content: rhai::Map, json_path: &str) -> bool {
        !find_value_by_json_path(content, json_path).is_empty()
    }
}

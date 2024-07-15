use rhai::export_module;

#[export_module]
pub(crate) mod json_module {
    use std::collections::HashMap;

    use reearth_flow_common::json;
    use rhai::plugin::*;

    pub fn find_value_by_json_path(
        content: HashMap<String, serde_json::Value>,
        json_path: &str,
    ) -> Vec<serde_json::Value> {
        let content: serde_json::Map<String, serde_json::Value> = content.into_iter().collect();
        let Ok(result) = json::find_by_json_path(serde_json::Value::Object(content), json_path)
        else {
            return vec![];
        };
        result
    }

    pub fn exists_value_by_json_path(
        content: HashMap<String, serde_json::Value>,
        json_path: &str,
    ) -> bool {
        !find_value_by_json_path(content, json_path).is_empty()
    }
}

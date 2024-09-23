use rhai::export_module;

#[export_module]
pub(crate) mod collection_module {
    use rhai::plugin::*;

    use crate::utils::dynamic_to_value;

    pub fn join_array(content: rhai::Array, sep: String) -> String {
        content
            .iter()
            .map(|x| {
                let value = dynamic_to_value(x);
                match value {
                    serde_json::Value::String(s) => s,
                    _ => "".to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join(&sep)
    }
}

#[cfg(test)]
mod tests {
    use super::collection_module::*;

    #[test]
    fn test_join_array() {
        // Test case 1: Joining an array of strings
        let content = vec![
            rhai::Dynamic::from("a"),
            rhai::Dynamic::from("b"),
            rhai::Dynamic::from("c"),
        ];
        let result = join_array(content, ",".to_string());
        assert_eq!(result, "a,b,c");

        // Test case 2: Joining an array of mixed types
        let content = vec![
            rhai::Dynamic::from("a"),
            rhai::Dynamic::from("2"),
            rhai::Dynamic::from("c"),
        ];
        let result = join_array(content, ",".to_string());
        assert_eq!(result, "a,2,c");
    }
}

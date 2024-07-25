use jsonpath_lib::Selector;

pub fn find_by_json_path(
    content: serde_json::Value,
    json_path: &str,
) -> crate::Result<Vec<serde_json::Value>> {
    let mut selector = Selector::new();
    let selector = selector.str_path(json_path).map_err(crate::Error::json)?;
    selector
        .value(&content)
        .select()
        .map_err(crate::Error::json)
        .map(|values| values.into_iter().cloned().collect())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_by_json_path() {
        // Test case 1: Valid JSON path with multiple matches
        let content = serde_json::json!({
            "data": {
                "users": [
                    { "name": "Alice", "age": 25 },
                    { "name": "Bob", "age": 30 },
                    { "name": "Charlie", "age": 35 }
                ]
            }
        });
        let json_path = "$.data.users[*].name";
        let expected_result = vec![
            serde_json::json!("Alice"),
            serde_json::json!("Bob"),
            serde_json::json!("Charlie"),
        ];
        assert_eq!(
            find_by_json_path(content, json_path).unwrap(),
            expected_result
        );

        // Test case 2: Valid JSON path with no matches
        let content = serde_json::json!({
            "data": {
                "users": []
            }
        });
        let json_path = "$.data.users[*].name";
        let expected_result: Vec<serde_json::Value> = vec![];
        assert_eq!(
            find_by_json_path(content, json_path).unwrap(),
            expected_result
        );

        // Test case 3: Invalid JSON path
        let content = serde_json::json!({
            "data": {
                "users": [
                    { "name": "Alice", "age": 25 },
                    { "name": "Bob", "age": 30 },
                    { "name": "Charlie", "age": 35 }
                ]
            }
        });
        let json_path = "$.data.users[*].email"; // Invalid path, 'email' does not exist
        assert!(find_by_json_path(content, json_path).unwrap().is_empty());
    }
}

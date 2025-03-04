use jsonpath_lib::Selector;
use serde_json::Value;

pub fn find_by_json_path(content: serde_json::Value, json_path: &str) -> crate::Result<Vec<Value>> {
    let mut selector = Selector::new();
    let selector = selector.str_path(json_path).map_err(crate::Error::json)?;
    selector
        .value(&content)
        .select()
        .map_err(crate::Error::json)
        .map(|values| values.into_iter().cloned().collect())
}

pub fn json_merge_patch(target: &mut Value, patch: &Value) {
    match patch {
        Value::Object(patch_obj) => {
            if !(target.is_object()
                || target.is_array() && patch_obj.keys().all(|key| key.parse::<usize>().is_ok()))
            {
                *target = Value::Object(serde_json::Map::new());
            }

            if let Value::Object(target_obj) = target {
                for (key, value) in patch_obj {
                    if value.is_null() {
                        target_obj.remove(key);
                    } else {
                        let target_value =
                            target_obj.entry(key.clone()).or_insert_with(|| Value::Null);
                        json_merge_patch(target_value, value);
                    }
                }
            } else if let Value::Array(target_arr) = target {
                for (key, value) in patch_obj {
                    if let Ok(index) = key.parse::<usize>() {
                        if value.is_null() && index < target_arr.len() {
                            target_arr.remove(index);
                        } else if index < target_arr.len() {
                            json_merge_patch(&mut target_arr[index], value);
                        } else {
                            // Handling the case where the index is greater than the current length of the target array
                            while target_arr.len() < index {
                                target_arr.push(Value::Null);
                            }
                            target_arr.push(value.clone());
                        }
                    }
                }
            }
        }
        Value::Array(patch_arr) => {
            *target = serde_json::Value::Array(
                patch_arr
                    .clone()
                    .into_iter()
                    .filter(|value| !value.is_null())
                    .collect(),
            );
        }
        _ => *target = patch.clone(),
    }
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
            "tran:auxiliaryTrafficArea": [
                {
                  "tran:function": [
                    "島"
                  ],
                  "type": "tran:AuxiliaryTrafficArea",
                  "id": "traf_897791ad-869a-4f35-b231-58c94c7a3379"
                },
                {
                  "id": "traf_17f9f8d9-5a76-4393-80da-8e81b30a1aa7",
                  "type": "tran:AuxiliaryTrafficArea",
                  "tran:function": [
                    "島"
                  ]
                }
              ],
              "gml:name": [
                "国道20号"
              ],
              "tran:trafficArea": [
                {
                  "id": "traf_f4203996-deea-4b89-a3b6-9c0756e32de9",
                  "tran:function": [
                    "歩道部"
                  ],
                  "type": "tran:TrafficArea"
                },
                {
                  "id": "traf_e5076339-0092-4fc1-8620-4d9210ab7c43",
                  "type": "tran:TrafficArea",
                  "tran:function": [
                    "車道交差部"
                  ]
                },
                {
                  "type": "tran:TrafficArea",
                  "id": "traf_1c1bb84f-6e9c-4815-9b7a-f68af72bc647",
                  "tran:function": [
                    "歩道部"
                  ]
                }
              ],
              "uro:roadStructureAttribute": [
                {
                  "type": "uro:RoadStructureAttribute",
                  "uro:sectionType": "交差部　"
                }
              ],
              "id": "tran_e46ee147-e87b-493e-acd5-815f5ee9b8e4",
              "type": "tran:Road",
              "core:creationDate": "2024-03-15",
              "tran:class": "道路",
              "tran:usage": [
                "緊急輸送道路（第一次緊急輸送道路）"
              ],
              "tran:function": [
                "一般国道"
              ],
              "uro:tranDataQualityAttribute": {
                "uro:geometrySrcDesc": [
                  "空中写真測量"
                ],
                "type": "uro:DataQualityAttribute",
                "uro:srcScale": [
                  "地図情報レベル2500"
                ]
              }
        });
        let json_path = "$..[?(@.id && @.type)]"; // Invalid path, 'email' does not exist
        let expected_result: Vec<serde_json::Value> = vec![
            serde_json::json!({
              "tran:function": [
                "島"
              ],
              "type": "tran:AuxiliaryTrafficArea",
              "id": "traf_897791ad-869a-4f35-b231-58c94c7a3379"
            }),
            serde_json::json!({
              "id": "traf_17f9f8d9-5a76-4393-80da-8e81b30a1aa7",
              "type": "tran:AuxiliaryTrafficArea",
              "tran:function": [
                "島"
              ]
            }),
            serde_json::json!({
              "id": "traf_f4203996-deea-4b89-a3b6-9c0756e32de9",
              "tran:function": [
                "歩道部"
              ],
              "type": "tran:TrafficArea"
            }),
            serde_json::json!({
              "id": "traf_e5076339-0092-4fc1-8620-4d9210ab7c43",
              "type": "tran:TrafficArea",
              "tran:function": [
                "車道交差部"
              ]
            }),
            serde_json::json!({
              "type": "tran:TrafficArea",
              "id": "traf_1c1bb84f-6e9c-4815-9b7a-f68af72bc647",
              "tran:function": [
                "歩道部"
              ]
            }),
        ];
        assert_eq!(
            find_by_json_path(content, json_path).unwrap(),
            expected_result
        );
    }

    #[test]
    fn test_json_merge_patch_should_merge_objects_and_override_field() {
        let mut target = serde_json::from_str(r#"{"a": "b", "c": {"d": "e", "f": "g"}}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": "z", "c": {"f": null}}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        let expected: serde_json::Value =
            serde_json::from_str(r#"{"a": "z", "c": {"d": "e"}}"#).unwrap();
        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_override_field_in_object() {
        let mut target = serde_json::from_str(r#"{"a": "b"}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": "c"}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": "c"}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_add_field_to_object() {
        let mut target = serde_json::from_str(r#"{"a": "b"}"#).unwrap();
        let patch = serde_json::from_str(r#"{"b": "c"}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": "b", "b": "c"}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_remove_field_from_object() {
        let mut target = serde_json::from_str(r#"{"a": "b", "b": "c"}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": null}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"b": "c"}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_override_field_in_array() {
        let mut target = serde_json::from_str(r#"{"a": ["b"]}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": "c"}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": "c"}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_replace_array_with_scalar() {
        let mut target = serde_json::from_str(r#"{"a": "c"}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": ["b"]}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": ["b"]}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_merge_objects_in_object() {
        let mut target = serde_json::from_str(r#"{"a": {"b": "c"}}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": {"b": "d", "c": null}}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": {"b": "d"}}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_replace_array_with_value() {
        let mut target = serde_json::from_str(r#"{"a": [{"b": "c"}]}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": [1]}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": [1]}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_merge_nested_objects_and_remove_leaf_nodes() {
        let mut target = serde_json::from_str(r#"{}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": {"bb": {"ccc": null}}}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": {"bb": {}}}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_replace_scalar_with_scalar() {
        let mut target = serde_json::from_str(r#"{"a": "b"}"#).unwrap();
        let patch = serde_json::from_str(r#"["c"]"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"["c"]"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_replace_scalar_with_null() {
        let mut target = serde_json::from_str(r#"{"a": "foo"}"#).unwrap();
        let patch = serde_json::Value::Null;

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, serde_json::Value::Null);
    }

    #[test]
    fn test_json_merge_patch_should_replace_scalar_with_string() {
        let mut target = serde_json::from_str(r#"{"a": "foo"}"#).unwrap();
        let patch = serde_json::from_str(r#""bar""#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#""bar""#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_merge_null_with_scalar() {
        let mut target = serde_json::from_str(r#"{"e": null}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"e": null, "a": 1}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_replace_array_with_object() {
        let mut target = serde_json::from_str(r#"{"a": []}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": {"b": "c"}}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": {"b": "c"}}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_merge_objects_in_array() {
        let mut target = serde_json::from_str(r#"[{"a": "b"}, {"c": "d"}]"#).unwrap();
        let patch = serde_json::from_str(r#"{"1": {"e": "f"}}"#).unwrap();
        let expected: serde_json::Value =
            serde_json::from_str(r#"[{"a": "b"}, {"c": "d", "e": "f"}]"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_replace_object_with_array() {
        let mut target = serde_json::from_str(r#"{"a": {"b": "c"}}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": []}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{"a": []}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_merge_arrays() {
        let mut target = serde_json::from_str(r#"["a", "b"]"#).unwrap();
        let patch = serde_json::from_str(r#"["c", "d"]"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"["c", "d"]"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_remove_key_from_object() {
        let mut target = serde_json::from_str(r#"{"a": "b"}"#).unwrap();
        let patch = serde_json::from_str(r#"{"a": null}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"{}"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_remove_index_from_array() {
        let mut target = serde_json::from_str(r#"["a", "b"]"#).unwrap();
        let patch = serde_json::from_str(r#"{"1": null}"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"["a"]"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }

    #[test]
    fn test_json_merge_patch_should_remove_array_element() {
        let mut target = serde_json::from_str(r#"[1, 2, 3]"#).unwrap();
        let patch = serde_json::from_str(r#"[null, 2]"#).unwrap();
        let expected: serde_json::Value = serde_json::from_str(r#"[2]"#).unwrap();

        json_merge_patch(&mut target, &patch);

        assert_eq!(target, expected);
    }
}

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
}

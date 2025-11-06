#[cfg(test)]
mod tests {
    use crate::object_list::*;
    use reearth_flow_types::AttributeValue;
    use std::collections::HashMap;

    #[test]
    fn test_record_from_vec_basic() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "attr2".to_string(),
            "attr3".to_string(),
            "attr4".to_string(),
            "主題".to_string(),
            "".to_string(),
            "create".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "unknown".to_string(),
        ];

        let record = Record::from(columns);
        
        assert_eq!(record.feature_prefix, "bldg");
        assert_eq!(record.feature_type, "bldg:Building");
        assert_eq!(record.category, "主題");
        assert_eq!(record.xpath, "attr1/attr2/attr3/attr4");
        assert_eq!(record.required, true);
    }

    #[test]
    fn test_record_from_vec_no_required() {
        let columns = vec![
            "tun".to_string(),
            "tun:Tunnel".to_string(),
            "gml:name".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "主題".to_string(),
            "".to_string(),
            "create".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(), // Empty means not required
        ];

        let record = Record::from(columns);
        
        assert_eq!(record.feature_prefix, "tun");
        assert_eq!(record.feature_type, "tun:Tunnel");
        assert_eq!(record.xpath, "gml:name");
        assert_eq!(record.required, false);
    }

    #[test]
    fn test_record_xpath_parentheses_removal() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "(attr1)".to_string(),
            "attr2.test".to_string(),
            "".to_string(),
            "".to_string(),
            "主題".to_string(),
        ];

        let record = Record::from(columns);
        
        // Parentheses and dots should be replaced
        assert_eq!(record.xpath, "attr1/attr2/test");
    }

    #[test]
    fn test_attribute_state_update_feature_type() {
        let mut state = AttributeState::default();
        
        state.update_feature_type("bldg:Building");
        assert_eq!(state.feature_type, Some("bldg:Building".to_string()));
        
        // Empty string should not update
        state.update_feature_type("");
        assert_eq!(state.feature_type, Some("bldg:Building".to_string()));
    }

    #[test]
    fn test_attribute_state_update_attributes() {
        let mut state = AttributeState::default();
        
        state.update_attribute(1, "attr1");
        state.update_attribute(2, "attr2");
        state.update_attribute(3, "attr3");
        state.update_attribute(4, "attr4");
        
        let attrs = state.get_attributes();
        assert_eq!(attrs, vec!["attr1", "attr2", "attr3", "attr4"]);
    }

    #[test]
    fn test_attribute_state_clear_after_update() {
        let mut state = AttributeState::default();
        
        state.update_attribute(1, "attr1");
        state.update_attribute(2, "attr2");
        state.update_attribute(3, "attr3");
        state.update_attribute(4, "attr4");
        
        // Updating level 2 should clear 3 and 4
        state.update_attribute(2, "new_attr2");
        
        let attrs = state.get_attributes();
        assert_eq!(attrs, vec!["attr1", "new_attr2", "", ""]);
    }

    #[test]
    fn test_should_process_row_valid() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "attr2".to_string(),
            "attr3".to_string(),
            "attr4".to_string(),
            "主題".to_string(), // index 6 - valid category
            "".to_string(),
            "create".to_string(), // index 8 - has create
        ];

        assert_eq!(should_process_row(&columns), true);
    }

    #[test]
    fn test_should_process_row_invalid_category() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "attr2".to_string(),
            "attr3".to_string(),
            "attr4".to_string(),
            "invalid".to_string(), // index 6 - invalid category
            "".to_string(),
            "create".to_string(), // index 8 - has create
        ];

        assert_eq!(should_process_row(&columns), false);
    }

    #[test]
    fn test_should_process_row_no_create() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "attr2".to_string(),
            "attr3".to_string(),
            "attr4".to_string(),
            "主題".to_string(), // index 6 - valid category
            "".to_string(),
            "".to_string(), // index 8 - no create
        ];

        assert_eq!(should_process_row(&columns), false);
    }

    #[test]
    fn test_expand_row_for_special_prefix_fld() {
        let row = vec![
            "fld/test".to_string(),
            "feature".to_string(),
        ];

        let expanded = expand_row_for_special_prefix(row);
        
        assert_eq!(expanded.len(), 5);
        assert_eq!(expanded[0][0], "fld");
        assert_eq!(expanded[1][0], "tnm");
        assert_eq!(expanded[2][0], "htd");
        assert_eq!(expanded[3][0], "ifld");
        assert_eq!(expanded[4][0], "rfld");
        
        // Other columns should remain the same
        for row in &expanded {
            assert_eq!(row[1], "feature");
        }
    }

    #[test]
    fn test_expand_row_for_special_prefix_non_fld() {
        let row = vec![
            "bldg".to_string(),
            "feature".to_string(),
        ];

        let expanded = expand_row_for_special_prefix(row.clone());
        
        // Should return the original row unchanged
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0], row);
    }

    #[test]
    fn test_feature_types_new() {
        let mut types = HashMap::new();
        types.insert(
            "bldg".to_string(),
            vec!["bldg:Building".to_string(), "bldg:BuildingPart".to_string()],
        );

        let feature_types = FeatureTypes::new(types.clone());
        assert_eq!(feature_types.into_inner(), types);
    }

    #[test]
    fn test_feature_types_to_attribute_value() {
        let mut types = HashMap::new();
        types.insert(
            "bldg".to_string(),
            vec!["bldg:Building".to_string()],
        );

        let feature_types = FeatureTypes::new(types);
        let attr_value: AttributeValue = feature_types.into();
        
        match attr_value {
            AttributeValue::Map(map) => {
                assert!(map.contains_key("bldg"));
                match map.get("bldg") {
                    Some(AttributeValue::Array(arr)) => {
                        assert_eq!(arr.len(), 1);
                    }
                    _ => panic!("Expected array"),
                }
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_object_list_value_default() {
        let value = ObjectListValue::default();
        assert!(value.required.is_empty());
        assert!(value.target.is_empty());
        assert!(value.conditional.is_empty());
    }

    #[test]
    fn test_object_list_value_from_attribute_value() {
        let mut map = HashMap::new();
        map.insert(
            "required".to_string(),
            AttributeValue::Array(vec![
                AttributeValue::String("attr1".to_string()),
                AttributeValue::String("attr2".to_string()),
            ]),
        );
        map.insert(
            "target".to_string(),
            AttributeValue::Array(vec![
                AttributeValue::String("attr3".to_string()),
            ]),
        );
        map.insert(
            "conditional".to_string(),
            AttributeValue::Array(vec![]),
        );

        let attr_value = AttributeValue::Map(map);
        let obj_list_value = ObjectListValue::from(attr_value);
        
        assert_eq!(obj_list_value.required.len(), 2);
        assert_eq!(obj_list_value.target.len(), 1);
        assert_eq!(obj_list_value.conditional.len(), 0);
    }

    #[test]
    fn test_object_list_value_to_attribute_value() {
        let value = ObjectListValue {
            required: vec!["attr1".to_string(), "attr2".to_string()],
            target: vec!["attr3".to_string()],
            conditional: vec![],
        };

        let attr_value: AttributeValue = value.into();
        
        match attr_value {
            AttributeValue::Map(map) => {
                assert!(map.contains_key("required"));
                assert!(map.contains_key("target"));
                assert!(map.contains_key("conditional"));
            }
            _ => panic!("Expected map"),
        }
    }

    #[test]
    fn test_object_list_from_records() {
        let records = vec![
            Record {
                feature_prefix: "bldg".to_string(),
                feature_type: "bldg:Building".to_string(),
                category: "主題".to_string(),
                xpath: "gml:name".to_string(),
                required: true,
            },
            Record {
                feature_prefix: "bldg".to_string(),
                feature_type: "bldg:Building".to_string(),
                category: "主題".to_string(),
                xpath: "bldg:usage".to_string(),
                required: false,
            },
            Record {
                feature_prefix: "bldg".to_string(),
                feature_type: "bldg:BuildingPart".to_string(),
                category: "主題".to_string(),
                xpath: "gml:id".to_string(),
                required: true,
            },
        ];

        let object_list = ObjectList::from(records);
        
        // Should have two feature types
        let feature_types = object_list.get_feature_types();
        assert_eq!(feature_types.len(), 2);
        assert!(feature_types.contains(&"bldg:Building".to_string()));
        assert!(feature_types.contains(&"bldg:BuildingPart".to_string()));
        
        // Check building attributes
        let building_value = object_list.get("bldg:Building").unwrap();
        assert_eq!(building_value.required.len(), 1);
        assert_eq!(building_value.target.len(), 1);
        assert_eq!(building_value.required[0], "gml:name");
        assert_eq!(building_value.target[0], "bldg:usage");
        
        // Check building part attributes
        let building_part_value = object_list.get("bldg:BuildingPart").unwrap();
        assert_eq!(building_part_value.required.len(), 1);
        assert_eq!(building_part_value.required[0], "gml:id");
    }

    #[test]
    fn test_object_list_map_empty() {
        let map = ObjectListMap::new(HashMap::new());
        assert!(map.is_empty());
    }

    #[test]
    fn test_object_list_map_get_mut() {
        let mut inner_map = HashMap::new();
        inner_map.insert(
            "bldg".to_string(),
            ObjectList::new(HashMap::new()),
        );

        let mut map = ObjectListMap::new(inner_map);
        
        assert!(map.get_mut("bldg").is_some());
        assert!(map.get_mut("tun").is_none());
    }

    #[test]
    fn test_object_list_map_from_attribute_value() {
        let mut inner_map = HashMap::new();
        
        let mut value_map = HashMap::new();
        value_map.insert("required".to_string(), AttributeValue::Array(vec![]));
        value_map.insert("target".to_string(), AttributeValue::Array(vec![]));
        value_map.insert("conditional".to_string(), AttributeValue::Array(vec![]));
        
        let mut obj_map = HashMap::new();
        obj_map.insert("bldg:Building".to_string(), AttributeValue::Map(value_map));
        
        inner_map.insert("bldg".to_string(), AttributeValue::Map(obj_map));
        
        let attr_value = AttributeValue::Map(inner_map);
        let object_list_map = ObjectListMap::from(attr_value);
        
        assert!(!object_list_map.is_empty());
        assert!(object_list_map.get("bldg").is_some());
    }

    #[test]
    fn test_object_list_keys() {
        let mut inner_map = HashMap::new();
        
        let mut building_value = HashMap::new();
        building_value.insert("bldg:Building".to_string(), ObjectListValue::default());
        
        let object_list = ObjectList::new(building_value);
        inner_map.insert("bldg".to_string(), object_list);
        
        let map = ObjectListMap::new(inner_map);
        let keys: Vec<_> = map.into_iter().map(|(k, _)| k).collect();
        
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"bldg".to_string()));
    }
}


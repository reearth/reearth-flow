#[cfg(test)]
mod tests {
    use crate::object_list::*;
    use reearth_flow_types::AttributeValue;
    use std::collections::HashMap;


    #[test]
    fn test_record_lifecycle_with_complex_xpath() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "uro:buildingIDAttribute".to_string(),
            "uro:buildingID".to_string(),
            "".to_string(),
            "".to_string(),
            "主題".to_string(),
            "".to_string(),
            "create".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "required".to_string(),
        ];

        let record = Record::from(columns);
        
        assert_eq!(record.feature_prefix, "bldg");
        assert_eq!(record.feature_type, "bldg:Building");
        assert_eq!(record.category, "主題");
        assert_eq!(record.xpath, "uro:buildingIDAttribute/uro:buildingID");
        assert_eq!(record.required, true);
    }

    #[test]
    fn test_record_with_all_attribute_levels() {
        let columns = vec![
            "tun".to_string(),
            "tun:Tunnel".to_string(),
            "level1".to_string(),
            "level2".to_string(),
            "level3".to_string(),
            "level4".to_string(),
            "主題".to_string(),
            "".to_string(),
            "create".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ];

        let record = Record::from(columns);
        assert_eq!(record.xpath, "level1/level2/level3/level4");
        assert_eq!(record.required, false);
    }

    #[test]
    fn test_attribute_state_hierarchical_updates() {
        let mut state = AttributeState::default();
        

        state.update_feature_type("bldg:Building");
        state.update_attribute(1, "uro:buildingIDAttribute");
        state.update_attribute(2, "uro:buildingID");
        state.update_attribute(3, "uro:branchID");
        
        let attrs = state.get_attributes();
        assert_eq!(attrs[0], "uro:buildingIDAttribute");
        assert_eq!(attrs[1], "uro:buildingID");
        assert_eq!(attrs[2], "uro:branchID");
        assert_eq!(attrs[3], "");
        

        state.update_attribute(2, "uro:newAttribute");
        let attrs = state.get_attributes();
        assert_eq!(attrs[0], "uro:buildingIDAttribute");
        assert_eq!(attrs[1], "uro:newAttribute");
        assert_eq!(attrs[2], "");
        assert_eq!(attrs[3], "");
    }

    #[test]
    fn test_attribute_state_feature_type_change_clears_all() {
        let mut state = AttributeState::default();
        
        state.update_feature_type("bldg:Building");
        state.update_attribute(1, "attr1");
        state.update_attribute(2, "attr2");
        state.update_attribute(3, "attr3");
        state.update_attribute(4, "attr4");
        

        state.update_feature_type("tun:Tunnel");
        
        let attrs = state.get_attributes();
        assert_eq!(attrs, vec!["", "", "", ""]);
        assert_eq!(state.feature_type, Some("tun:Tunnel".to_string()));
    }

    #[test]
    fn test_expand_row_flood_risk_attributes() {
        let row = vec![
            "fld/test".to_string(),
            "bldg:Building".to_string(),
            "uro:floodingRiskAttribute".to_string(),
            "uro:rank".to_string(),
        ];

        let expanded = expand_row_for_special_prefix(row.clone());
        

        assert_eq!(expanded.len(), 5);
        
        let prefixes: Vec<String> = expanded.iter().map(|r| r[0].clone()).collect();
        assert!(prefixes.contains(&"fld".to_string()));
        assert!(prefixes.contains(&"tnm".to_string()));
        assert!(prefixes.contains(&"htd".to_string()));
        assert!(prefixes.contains(&"ifld".to_string()));
        assert!(prefixes.contains(&"rfld".to_string()));
        

        for expanded_row in &expanded {
            assert_eq!(expanded_row[1], "bldg:Building");
            assert_eq!(expanded_row[2], "uro:floodingRiskAttribute");
            assert_eq!(expanded_row[3], "uro:rank");
        }
    }

    #[test]
    fn test_object_list_from_multiple_records_grouping() {
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
                required: true,
            },
            Record {
                feature_prefix: "bldg".to_string(),
                feature_type: "bldg:Building".to_string(),
                category: "主題".to_string(),
                xpath: "bldg:yearOfConstruction".to_string(),
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
        

        let building_value = object_list.get("bldg:Building").unwrap();
        assert_eq!(building_value.required.len(), 2);
        assert_eq!(building_value.target.len(), 1);
        assert!(building_value.required.contains(&"gml:name".to_string()));
        assert!(building_value.required.contains(&"bldg:usage".to_string()));
        assert!(building_value.target.contains(&"bldg:yearOfConstruction".to_string()));
        

        let building_part_value = object_list.get("bldg:BuildingPart").unwrap();
        assert_eq!(building_part_value.required.len(), 1);
        assert_eq!(building_part_value.target.len(), 0);
        assert!(building_part_value.required.contains(&"gml:id".to_string()));
    }

    #[test]
    fn test_object_list_value_serialization_roundtrip() {
        let original = ObjectListValue {
            required: vec!["attr1".to_string(), "attr2".to_string()],
            target: vec!["attr3".to_string(), "attr4".to_string()],
            conditional: vec!["attr5".to_string()],
        };


        let attr_value: AttributeValue = original.clone().into();
        

        let roundtrip = ObjectListValue::from(attr_value);
        
        assert_eq!(roundtrip.required, original.required);
        assert_eq!(roundtrip.target, original.target);
        assert_eq!(roundtrip.conditional, original.conditional);
    }

    #[test]
    fn test_object_list_map_iteration() {
        let mut inner_map = HashMap::new();
        
        let mut building_attrs = HashMap::new();
        building_attrs.insert(
            "bldg:Building".to_string(),
            ObjectListValue {
                required: vec!["attr1".to_string()],
                target: vec!["attr2".to_string()],
                conditional: vec![],
            },
        );
        
        let mut tunnel_attrs = HashMap::new();
        tunnel_attrs.insert(
            "tun:Tunnel".to_string(),
            ObjectListValue {
                required: vec!["attr3".to_string()],
                target: vec![],
                conditional: vec![],
            },
        );
        
        inner_map.insert("bldg".to_string(), ObjectList::new(building_attrs));
        inner_map.insert("tun".to_string(), ObjectList::new(tunnel_attrs));
        
        let object_list_map = ObjectListMap::new(inner_map);
        
        let keys: Vec<String> = object_list_map.into_iter().map(|(k, _)| k).collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"bldg".to_string()));
        assert!(keys.contains(&"tun".to_string()));
    }

    #[test]
    fn test_feature_types_conversion_complex() {
        let mut types = HashMap::new();
        types.insert(
            "bldg".to_string(),
            vec![
                "bldg:Building".to_string(),
                "bldg:BuildingPart".to_string(),
                "bldg:BuildingInstallation".to_string(),
            ],
        );
        types.insert(
            "tran".to_string(),
            vec![
                "tran:Road".to_string(),
                "tran:Railway".to_string(),
            ],
        );

        let feature_types = FeatureTypes::new(types.clone());
        let attr_value: AttributeValue = feature_types.clone().into();
        

        match attr_value {
            AttributeValue::Map(map) => {
                assert_eq!(map.len(), 2);
                
                if let Some(AttributeValue::Array(bldg_types)) = map.get("bldg") {
                    assert_eq!(bldg_types.len(), 3);
                }
                
                if let Some(AttributeValue::Array(tran_types)) = map.get("tran") {
                    assert_eq!(tran_types.len(), 2);
                }
            }
            _ => panic!("Expected Map"),
        }
    }

    #[test]
    fn test_should_process_row_with_related_role_category() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "関連役割".to_string(),
            "".to_string(),
            "create".to_string(),
        ];

        assert_eq!(should_process_row(&columns), true);
    }

    #[test]
    fn test_should_process_row_edge_cases() {

        let columns1 = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "create".to_string(),
        ];
        assert_eq!(should_process_row(&columns1), false);
        

        let columns2 = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "その他".to_string(),
            "".to_string(),
            "create".to_string(),
        ];
        assert_eq!(should_process_row(&columns2), false);
        

        let columns3 = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "attr1".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "主題".to_string(),
            "".to_string(),
            "".to_string(),
        ];
        assert_eq!(should_process_row(&columns3), false);
    }

    #[test]
    fn test_record_from_row_with_state_inheritance() {
        let mut state = AttributeState::default();
        state.update_feature_type("bldg:Building");
        state.update_attribute(1, "uro:buildingIDAttribute");
        state.update_attribute(2, "uro:buildingID");
        
        let row = vec![
            "bldg".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "uro:prefecture".to_string(),
            "".to_string(),
            "主題".to_string(),
        ];

        let record = Record::from_row_with_state(row, &state);
        
        assert_eq!(record.feature_type, "bldg:Building");


        assert_eq!(record.xpath, "uro:buildingIDAttribute/uro:buildingID");
    }

    #[test]
    fn test_object_list_empty_checks() {
        let empty_map = ObjectListMap::new(HashMap::new());
        assert!(empty_map.is_empty());
        
        let mut non_empty_map = HashMap::new();
        non_empty_map.insert("bldg".to_string(), ObjectList::new(HashMap::new()));
        let non_empty = ObjectListMap::new(non_empty_map);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_object_list_get_feature_types() {
        let mut types_map = HashMap::new();
        types_map.insert("bldg:Building".to_string(), ObjectListValue::default());
        types_map.insert("bldg:BuildingPart".to_string(), ObjectListValue::default());
        types_map.insert("bldg:BuildingInstallation".to_string(), ObjectListValue::default());
        
        let object_list = ObjectList::new(types_map);
        let feature_types = object_list.get_feature_types();
        
        assert_eq!(feature_types.len(), 3);
        assert!(feature_types.contains(&"bldg:Building".to_string()));
        assert!(feature_types.contains(&"bldg:BuildingPart".to_string()));
        assert!(feature_types.contains(&"bldg:BuildingInstallation".to_string()));
    }

    #[test]
    fn test_xpath_special_characters_handling() {
        let columns = vec![
            "bldg".to_string(),
            "bldg:Building".to_string(),
            "(uro:buildingIDAttribute)".to_string(),
            "uro:buildingID.value".to_string(),
            "".to_string(),
            "".to_string(),
            "主題".to_string(),
        ];

        let record = Record::from(columns);
        

        assert_eq!(record.xpath, "uro:buildingIDAttribute/uro:buildingID/value");
    }

    #[test]
    fn test_object_list_value_all_empty() {
        let empty_value = ObjectListValue::default();
        
        assert!(empty_value.required.is_empty());
        assert!(empty_value.target.is_empty());
        assert!(empty_value.conditional.is_empty());
    }

    #[test]
    fn test_object_list_value_from_empty_attribute_value() {
        let empty_map = AttributeValue::Map(HashMap::new());
        let obj_list_value = ObjectListValue::from(empty_map);
        
        assert!(obj_list_value.required.is_empty());
        assert!(obj_list_value.target.is_empty());
        assert!(obj_list_value.conditional.is_empty());
    }

    #[test]
    fn test_object_list_keys_iteration() {
        let mut types_map = HashMap::new();
        types_map.insert("bldg:Building".to_string(), ObjectListValue::default());
        types_map.insert("tun:Tunnel".to_string(), ObjectListValue::default());
        
        let object_list = ObjectList::new(types_map);
        let keys = object_list.keys();
        
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"bldg:Building".to_string()));
        assert!(keys.contains(&"tun:Tunnel".to_string()));
    }
}


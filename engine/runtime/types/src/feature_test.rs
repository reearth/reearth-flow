#[cfg(test)]
mod tests {
    use crate::attribute::{Attribute, AttributeValue};
    use crate::feature::Feature;
    use crate::geometry::Geometry;
    use crate::metadata::Metadata;
    use indexmap::IndexMap;
    use std::collections::HashMap;

    #[test]
    fn test_feature_new() {
        let feature = Feature::new();
        assert!(feature.attributes.is_empty());
        assert!(feature.geometry.is_empty());
    }

    #[test]
    fn test_feature_default() {
        let feature = Feature::default();
        assert!(feature.attributes.is_empty());
    }

    #[test]
    fn test_feature_equality_by_id() {
        let feature1 = Feature::new();
        let feature2 = feature1.clone();
        let feature3 = Feature::new();
        
        assert_eq!(feature1, feature2);
        assert_ne!(feature1, feature3);
    }

    #[test]
    fn test_feature_from_index_map_string() {
        let mut map = IndexMap::new();
        map.insert("key1".to_string(), AttributeValue::String("value1".to_string()));
        map.insert("key2".to_string(), AttributeValue::Number(serde_json::Number::from(42)));
        
        let feature = Feature::from(map);
        
        assert_eq!(feature.attributes.len(), 2);
        assert_eq!(
            feature.get(&"key1"),
            Some(&AttributeValue::String("value1".to_string()))
        );
    }

    #[test]
    fn test_feature_from_index_map_attribute() {
        let mut map = IndexMap::new();
        map.insert(Attribute::new("attr1"), AttributeValue::Bool(true));
        map.insert(Attribute::new("attr2"), AttributeValue::Null);
        
        let feature = Feature::from(map);
        
        assert_eq!(feature.attributes.len(), 2);
        assert_eq!(feature.get(&"attr1"), Some(&AttributeValue::Bool(true)));
    }

    #[test]
    fn test_feature_from_geometry() {
        let geometry = Geometry::default();
        let feature = Feature::from(geometry);
        
        assert!(feature.geometry.is_empty());
    }

    #[test]
    fn test_feature_insert() {
        let mut feature = Feature::new();
        feature.insert("test_key", AttributeValue::String("test_value".to_string()));
        
        assert_eq!(feature.attributes.len(), 1);
        assert_eq!(
            feature.get(&"test_key"),
            Some(&AttributeValue::String("test_value".to_string()))
        );
    }

    #[test]
    fn test_feature_insert_overwrite() {
        let mut feature = Feature::new();
        feature.insert("key", AttributeValue::String("old".to_string()));
        feature.insert("key", AttributeValue::String("new".to_string()));
        
        assert_eq!(feature.attributes.len(), 1);
        assert_eq!(
            feature.get(&"key"),
            Some(&AttributeValue::String("new".to_string()))
        );
    }

    #[test]
    fn test_feature_get() {
        let mut feature = Feature::new();
        feature.insert("exists", AttributeValue::Bool(true));
        
        assert!(feature.get(&"exists").is_some());
        assert!(feature.get(&"nonexistent").is_none());
    }

    #[test]
    fn test_feature_remove() {
        let mut feature = Feature::new();
        feature.insert("to_remove", AttributeValue::Number(serde_json::Number::from(123)));
        
        assert!(feature.get(&"to_remove").is_some());
        
        feature.remove(&Attribute::new("to_remove"));
        
        assert!(feature.get(&"to_remove").is_none());
    }

    #[test]
    fn test_feature_extend() {
        let mut feature = Feature::new();
        feature.insert("original", AttributeValue::String("value".to_string()));
        
        let mut extension = HashMap::new();
        extension.insert(Attribute::new("new1"), AttributeValue::Bool(true));
        extension.insert(Attribute::new("new2"), AttributeValue::Number(serde_json::Number::from(99)));
        
        feature.extend(extension);
        
        assert_eq!(feature.attributes.len(), 3);
        assert!(feature.get(&"original").is_some());
        assert!(feature.get(&"new1").is_some());
        assert!(feature.get(&"new2").is_some());
    }

    #[test]
    fn test_feature_clone() {
        let mut feature1 = Feature::new();
        feature1.insert("key", AttributeValue::String("value".to_string()));
        
        let feature2 = feature1.clone();
        
        assert_eq!(feature1.id, feature2.id);
        assert_eq!(feature1.attributes.len(), feature2.attributes.len());
    }

    #[test]
    fn test_feature_display() {
        let feature = Feature::new();
        let display_string = format!("{}", feature);
        
        assert!(display_string.contains("-"));
    }

    #[test]
    fn test_feature_hash() {
        use std::collections::HashSet;
        
        let feature1 = Feature::new();
        let feature2 = feature1.clone();
        let feature3 = Feature::new();
        
        let mut set = HashSet::new();
        set.insert(feature1.clone());
        set.insert(feature2);
        set.insert(feature3);
        
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_feature_with_complex_attributes() {
        let mut feature = Feature::new();
        
        let mut nested_map = HashMap::new();
        nested_map.insert("nested_key".to_string(), AttributeValue::String("nested_value".to_string()));
        
        feature.insert("simple", AttributeValue::String("simple_value".to_string()));
        feature.insert("number", AttributeValue::Number(serde_json::Number::from(42)));
        feature.insert("bool", AttributeValue::Bool(true));
        feature.insert("null", AttributeValue::Null);
        feature.insert("map", AttributeValue::Map(nested_map));
        feature.insert("array", AttributeValue::Array(vec![
            AttributeValue::String("item1".to_string()),
            AttributeValue::String("item2".to_string()),
        ]));
        
        assert_eq!(feature.attributes.len(), 6);
    }

    #[test]
    fn test_feature_metadata() {
        let mut feature = Feature::new();
        feature.metadata = Metadata {
            feature_id: Some("feat_123".to_string()),
            feature_type: Some("Building".to_string()),
            lod: None,
        };
        
        assert_eq!(feature.metadata.feature_id, Some("feat_123".to_string()));
        assert_eq!(feature.metadata.feature_type, Some("Building".to_string()));
    }

    #[test]
    fn test_feature_attributes_ordering() {
        let mut feature = Feature::new();
        feature.insert("z_last", AttributeValue::String("last".to_string()));
        feature.insert("a_first", AttributeValue::String("first".to_string()));
        feature.insert("m_middle", AttributeValue::String("middle".to_string()));
        
        let keys: Vec<String> = feature.attributes.keys().map(|k| k.to_string()).collect();
        assert_eq!(keys, vec!["z_last", "a_first", "m_middle"]);
    }
}


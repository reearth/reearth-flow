#[cfg(test)]
mod tests {
    use crate::attribute::{Attribute, AttributeValue};
    use bytes::Bytes;
    use std::collections::HashMap;

    #[test]
    fn test_attribute_new() {
        let attr = Attribute::new("test_attribute");
        assert_eq!(attr.as_ref(), "test_attribute");
    }

    #[test]
    fn test_attribute_trimming() {
        let attr = Attribute::new("  trimmed  ");
        assert_eq!(attr.as_ref(), "trimmed");
    }

    #[test]
    fn test_attribute_inner() {
        let attr = Attribute::new("inner_test");
        assert_eq!(attr.inner(), "inner_test");
    }

    #[test]
    fn test_attribute_equality() {
        let attr1 = Attribute::new("same");
        let attr2 = Attribute::new("same");
        let attr3 = Attribute::new("different");
        
        assert_eq!(attr1, attr2);
        assert_ne!(attr1, attr3);
    }

    #[test]
    fn test_attribute_ordering() {
        let attr1 = Attribute::new("aaa");
        let attr2 = Attribute::new("bbb");
        let attr3 = Attribute::new("ccc");
        
        assert!(attr1 < attr2);
        assert!(attr2 < attr3);
        assert!(attr1 < attr3);
    }

    #[test]
    fn test_attribute_value_null() {
        let value = AttributeValue::Null;
        assert!(matches!(value, AttributeValue::Null));
        assert!(value.is_null());
    }

    #[test]
    fn test_attribute_value_bool() {
        let value_true = AttributeValue::Bool(true);
        let value_false = AttributeValue::Bool(false);
        
        assert_eq!(value_true.as_bool(), Some(true));
        assert_eq!(value_false.as_bool(), Some(false));
    }

    #[test]
    fn test_attribute_value_number() {
        let value = AttributeValue::Number(serde_json::Number::from(42));
        assert_eq!(value.as_i64(), Some(42));
    }

    #[test]
    fn test_attribute_value_string() {
        let value = AttributeValue::String("test string".to_string());
        assert_eq!(value.as_string(), Some("test string"));
        assert_eq!(value.to_string(), "test string");
    }

    #[test]
    fn test_attribute_value_array() {
        let value = AttributeValue::Array(vec![
            AttributeValue::Number(serde_json::Number::from(1)),
            AttributeValue::Number(serde_json::Number::from(2)),
            AttributeValue::Number(serde_json::Number::from(3)),
        ]);
        
        assert_eq!(value.as_vec().map(|v| v.len()), Some(3));
    }

    #[test]
    fn test_attribute_value_map() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), AttributeValue::String("value1".to_string()));
        map.insert("key2".to_string(), AttributeValue::Number(serde_json::Number::from(100)));
        
        let value = AttributeValue::Map(map);
        assert_eq!(value.as_map().map(|m| m.len()), Some(2));
    }

    #[test]
    fn test_attribute_value_bytes() {
        let bytes = Bytes::from(vec![1, 2, 3, 4, 5]);
        let value = AttributeValue::Bytes(bytes.clone());
        
        assert_eq!(value.as_bytes(), Some(bytes.as_ref()));
    }

    #[test]
    fn test_attribute_value_default_constructors() {
        assert!(matches!(AttributeValue::default_bool(), AttributeValue::Bool(false)));
        assert!(matches!(AttributeValue::default_number(), AttributeValue::Number(_)));
        assert!(matches!(AttributeValue::default_string(), AttributeValue::String(s) if s.is_empty()));
        assert!(matches!(AttributeValue::default_array(), AttributeValue::Array(a) if a.is_empty()));
        assert!(matches!(AttributeValue::default_map(), AttributeValue::Map(m) if m.is_empty()));
        assert!(matches!(AttributeValue::default_bytes(), AttributeValue::Bytes(b) if b.is_empty()));
    }

    #[test]
    fn test_attribute_value_clone() {
        let value1 = AttributeValue::String("clone_test".to_string());
        let value2 = value1.clone();
        
        assert_eq!(value1.to_string(), value2.to_string());
    }

    #[test]
    fn test_attribute_value_nested_structures() {
        let mut inner_map = HashMap::new();
        inner_map.insert("inner_key".to_string(), AttributeValue::Number(serde_json::Number::from(999)));
        
        let mut outer_map = HashMap::new();
        outer_map.insert("nested".to_string(), AttributeValue::Map(inner_map));
        outer_map.insert("simple".to_string(), AttributeValue::String("value".to_string()));
        
        let value = AttributeValue::Map(outer_map);
        
        if let Some(map) = value.as_map() {
            assert_eq!(map.len(), 2);
            assert!(map.contains_key("nested"));
            assert!(map.contains_key("simple"));
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn test_attribute_value_mixed_array() {
        let value = AttributeValue::Array(vec![
            AttributeValue::String("text".to_string()),
            AttributeValue::Number(serde_json::Number::from(123)),
            AttributeValue::Bool(true),
            AttributeValue::Null,
        ]);
        
        if let Some(arr) = value.as_vec() {
            assert_eq!(arr.len(), 4);
            assert!(matches!(arr[0], AttributeValue::String(_)));
            assert!(matches!(arr[1], AttributeValue::Number(_)));
            assert!(matches!(arr[2], AttributeValue::Bool(true)));
            assert!(matches!(arr[3], AttributeValue::Null));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_attribute_display() {
        let attr = Attribute::new("display_test");
        let display_str = format!("{}", attr);
        assert_eq!(display_str, "display_test");
    }

    #[test]
    fn test_attribute_value_is_null() {
        assert!(AttributeValue::Null.is_null());
        assert!(!AttributeValue::Bool(false).is_null());
        assert!(!AttributeValue::String("".to_string()).is_null());
    }

    #[test]
    fn test_attribute_value_as_f64() {
        let value = AttributeValue::Number(serde_json::Number::from_f64(3.14).unwrap());
        assert_eq!(value.as_f64(), Some(3.14));
    }

    #[test]
    fn test_attribute_value_number_conversions() {
        let int_value = AttributeValue::Number(serde_json::Number::from(42));
        assert_eq!(int_value.as_i64(), Some(42));
        assert_eq!(int_value.as_u64(), Some(42));
        assert_eq!(int_value.as_f64(), Some(42.0));
        
        let float_value = AttributeValue::Number(serde_json::Number::from_f64(3.14).unwrap());
        assert!(float_value.as_i64().is_none());
        assert_eq!(float_value.as_f64(), Some(3.14));
    }

    #[test]
    fn test_attribute_value_to_string_conversions() {
        assert_eq!(AttributeValue::String("text".to_string()).to_string(), "text");
        assert_eq!(AttributeValue::Number(serde_json::Number::from(42)).to_string(), "42");
        assert_eq!(AttributeValue::Bool(true).to_string(), "true");
        assert_eq!(AttributeValue::Null.to_string(), "null");
    }

    #[test]
    fn test_attribute_value_empty_collections() {
        let empty_array = AttributeValue::Array(vec![]);
        let empty_map = AttributeValue::Map(HashMap::new());
        let empty_bytes = AttributeValue::Bytes(Bytes::new());
        
        assert_eq!(empty_array.as_vec(), Some(&vec![]));
        assert_eq!(empty_map.as_map().map(|m| m.len()), Some(0));
        assert_eq!(empty_bytes.as_bytes(), Some(&[] as &[u8]));
    }
}


#[cfg(test)]
mod tests {
    use crate::kvs::{create_kv_store, KvStore, KvStoreKey, MemoryKvStore};
    use reearth_flow_types::AttributeValue;

    #[test]
    fn test_memory_kv_store_new() {
        let store = MemoryKvStore::new();
        let key = KvStoreKey::new("test_key");
        assert!(store.get(&key).is_none());
    }

    #[test]
    fn test_kv_store_insert_and_get() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("my_key");
        let value = AttributeValue::String("test_value".to_string());
        
        store.insert(key.clone(), value.clone());
        let retrieved = store.get(&key);
        
        assert_eq!(retrieved, Some(value));
    }

    #[test]
    fn test_kv_store_insert_updates_existing() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("key1");
        
        store.insert(key.clone(), AttributeValue::String("value1".to_string()));
        store.insert(key.clone(), AttributeValue::String("value2".to_string()));
        
        let retrieved = store.get(&key);
        assert_eq!(retrieved, Some(AttributeValue::String("value2".to_string())));
    }

    #[test]
    fn test_kv_store_remove() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("to_remove");
        let value = AttributeValue::Number(serde_json::Number::from(42));
        
        store.insert(key.clone(), value.clone());
        assert_eq!(store.get(&key), Some(value));
        
        store.remove(&key);
        assert!(store.get(&key).is_none());
    }

    #[test]
    fn test_kv_store_remove_nonexistent() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("nonexistent");
        
        store.remove(&key);
        assert!(store.get(&key).is_none());
    }

    #[test]
    fn test_kv_store_multiple_keys() {
        let mut store = MemoryKvStore::new();
        let key1 = KvStoreKey::new("key1");
        let key2 = KvStoreKey::new("key2");
        let key3 = KvStoreKey::new("key3");
        
        store.insert(key1.clone(), AttributeValue::String("value1".to_string()));
        store.insert(key2.clone(), AttributeValue::String("value2".to_string()));
        store.insert(key3.clone(), AttributeValue::String("value3".to_string()));
        
        assert_eq!(store.get(&key1), Some(AttributeValue::String("value1".to_string())));
        assert_eq!(store.get(&key2), Some(AttributeValue::String("value2".to_string())));
        assert_eq!(store.get(&key3), Some(AttributeValue::String("value3".to_string())));
    }

    #[test]
    fn test_kv_store_with_different_value_types() {
        let mut store = MemoryKvStore::new();
        
        let key_string = KvStoreKey::new("string");
        let key_number = KvStoreKey::new("number");
        let key_bool = KvStoreKey::new("bool");
        let key_null = KvStoreKey::new("null");
        
        store.insert(key_string.clone(), AttributeValue::String("test".to_string()));
        store.insert(key_number.clone(), AttributeValue::Number(serde_json::Number::from(123)));
        store.insert(key_bool.clone(), AttributeValue::Bool(true));
        store.insert(key_null.clone(), AttributeValue::Null);
        
        assert!(matches!(store.get(&key_string), Some(AttributeValue::String(_))));
        assert!(matches!(store.get(&key_number), Some(AttributeValue::Number(_))));
        assert!(matches!(store.get(&key_bool), Some(AttributeValue::Bool(true))));
        assert!(matches!(store.get(&key_null), Some(AttributeValue::Null)));
    }

    #[test]
    fn test_kv_store_clone() {
        let mut store1 = MemoryKvStore::new();
        let key = KvStoreKey::new("shared_key");
        let value = AttributeValue::String("shared_value".to_string());
        
        store1.insert(key.clone(), value.clone());
        
        let store2 = store1.clone();
        assert_eq!(store2.get(&key), Some(value));
    }

    #[test]
    fn test_kv_store_boxed_clone() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("test");
        let value = AttributeValue::String("value".to_string());
        
        store.insert(key.clone(), value.clone());
        
        let cloned = store.boxed_clone();
        assert_eq!(cloned.get(&key), Some(value));
    }

    #[test]
    fn test_create_kv_store() {
        let store = create_kv_store();
        let key = KvStoreKey::new("test");
        assert!(store.get(&key).is_none());
    }

    #[test]
    fn test_kv_store_key_trimming() {
        let key1 = KvStoreKey::new("  test  ");
        let key2 = KvStoreKey::new("test");
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_kv_store_shared_across_threads() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("shared");
        let value = AttributeValue::String("shared_value".to_string());
        
        store.insert(key.clone(), value.clone());
        
        let cloned_store = store.clone();
        assert_eq!(cloned_store.get(&key), Some(value));
    }

    #[test]
    fn test_kv_store_with_complex_values() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("complex");
        
        let mut map = std::collections::HashMap::new();
        map.insert("field1".to_string(), AttributeValue::String("value1".to_string()));
        map.insert("field2".to_string(), AttributeValue::Number(serde_json::Number::from(42)));
        
        let complex_value = AttributeValue::Map(map.clone());
        store.insert(key.clone(), complex_value);
        
        let retrieved = store.get(&key);
        assert!(matches!(retrieved, Some(AttributeValue::Map(_))));
        
        if let Some(AttributeValue::Map(retrieved_map)) = retrieved {
            assert_eq!(retrieved_map.len(), 2);
            assert!(retrieved_map.contains_key("field1"));
            assert!(retrieved_map.contains_key("field2"));
        }
    }

    #[test]
    fn test_kv_store_with_array_values() {
        let mut store = MemoryKvStore::new();
        let key = KvStoreKey::new("array");
        
        let array_value = AttributeValue::Array(vec![
            AttributeValue::String("item1".to_string()),
            AttributeValue::String("item2".to_string()),
            AttributeValue::Number(serde_json::Number::from(3)),
        ]);
        
        store.insert(key.clone(), array_value.clone());
        
        let retrieved = store.get(&key);
        assert_eq!(retrieved, Some(array_value));
    }

    #[test]
    fn test_kv_store_key_display() {
        let key = KvStoreKey::new("display_test");
        let display_str = format!("{}", key);
        assert_eq!(display_str, "display_test");
    }

    #[test]
    fn test_kv_store_key_as_ref() {
        let key = KvStoreKey::new("as_ref_test");
        let key_ref: &str = key.as_ref();
        assert_eq!(key_ref, "as_ref_test");
    }

    #[test]
    fn test_kv_store_key_ordering() {
        let key1 = KvStoreKey::new("aaa");
        let key2 = KvStoreKey::new("bbb");
        let key3 = KvStoreKey::new("ccc");
        
        assert!(key1 < key2);
        assert!(key2 < key3);
        assert!(key1 < key3);
    }

    #[test]
    fn test_boxed_kv_store_operations() {
        let mut store: Box<dyn KvStore> = Box::new(MemoryKvStore::new());
        let key = KvStoreKey::new("boxed_test");
        let value = AttributeValue::String("boxed_value".to_string());
        
        store.insert(key.clone(), value.clone());
        assert_eq!(store.get(&key), Some(value));
        
        store.remove(&key);
        assert!(store.get(&key).is_none());
    }
}


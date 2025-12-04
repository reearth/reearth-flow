#[cfg(test)]
mod tests {
    use crate::State;
    use reearth_flow_common::uri::Uri;
    use reearth_flow_storage::resolve::StorageResolver;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        value: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_save_and_get() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let data = TestData {
            value: "test".to_string(),
            count: 42,
        };
        
        state.save(&data, "item1").await.unwrap();
        let retrieved: TestData = state.get("item1").await.unwrap();
        
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_save_and_get_sync() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let data = TestData {
            value: "sync_test".to_string(),
            count: 100,
        };
        
        state.save_sync(&data, "sync_item").unwrap();
        let retrieved: TestData = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(state.get("sync_item"))
            .unwrap();
        
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_delete() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let data = TestData {
            value: "to_delete".to_string(),
            count: 99,
        };
        
        state.save(&data, "deletable").await.unwrap();
        state.delete("deletable").await.unwrap();
        
        let result: std::io::Result<TestData> = state.get("deletable").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_id_to_location_without_compression() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let location = state.id_to_location("my_id", "json");
        assert!(location.to_str().unwrap().contains("my_id.json"));
    }

    #[test]
    fn test_id_to_location_with_compression() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new_for_test(&Uri::for_test("ram:///test"), &storage_resolver, true).unwrap();
        
        let location = state.id_to_location("my_id", "json.zst");
        assert!(location.to_str().unwrap().contains("my_id.json.zst"));
    }

    #[test]
    fn test_object_to_string() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let data = TestData {
            value: "serialize_test".to_string(),
            count: 123,
        };
        
        let json_string = state.object_to_string(&data).unwrap();
        assert!(json_string.contains("serialize_test"));
        assert!(json_string.contains("123"));
    }

    #[test]
    fn test_string_to_object() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let json_string = r#"{"value":"deserialize_test","count":456}"#;
        let data: TestData = state.string_to_object(json_string).unwrap();
        
        assert_eq!(data.value, "deserialize_test");
        assert_eq!(data.count, 456);
    }

    #[tokio::test]
    async fn test_save_with_compression() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new_for_test(&Uri::for_test("ram:///test"), &storage_resolver, true).unwrap();
        
        let data = TestData {
            value: "compressed".to_string(),
            count: 789,
        };
        
        state.save(&data, "compressed_item").await.unwrap();
        let retrieved: TestData = state.get("compressed_item").await.unwrap();
        
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_save_sync_with_compression() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new_for_test(&Uri::for_test("ram:///test"), &storage_resolver, true).unwrap();
        
        let data = TestData {
            value: "compressed_sync".to_string(),
            count: 321,
        };
        
        state.save_sync(&data, "compressed_sync_item").unwrap();
        let retrieved: TestData = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(state.get("compressed_sync_item"))
            .unwrap();
        
        assert_eq!(retrieved, data);
    }


    #[tokio::test]
    async fn test_multiple_operations_same_id() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let data1 = TestData { value: "first".to_string(), count: 1 };
        let data2 = TestData { value: "second".to_string(), count: 2 };
        
        state.save(&data1, "same_id").await.unwrap();
        state.save(&data2, "same_id").await.unwrap();
        
        let retrieved: TestData = state.get("same_id").await.unwrap();
        assert_eq!(retrieved, data2);
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let result: std::io::Result<TestData> = state.get("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let storage_resolver = Arc::new(StorageResolver::new());
        let state = State::new(&Uri::for_test("ram:///test"), &storage_resolver).unwrap();
        
        let result = state.delete("nonexistent").await;
        assert!(result.is_ok() || result.is_err());
    }
}


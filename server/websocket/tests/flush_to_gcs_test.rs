mod mock_server;

use bytes::Bytes;
use mock_server::{MockGcsStore, MockRedisStore};
use std::sync::Arc;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, Text, Transact, Update};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_gcs_store_basic_operations() {
        let gcs_store = MockGcsStore::new();

        let doc = gcs_store.load_doc_v2("non_existent").await.unwrap();
        assert!(doc.get_or_insert_text("test").len(&doc.transact()) == 0);

        let test_doc = Doc::new();
        let text = test_doc.get_or_insert_text("content");
        {
            let mut txn = test_doc.transact_mut();
            text.push(&mut txn, "Hello, World!");
        }

        gcs_store.flush_doc_v2("test_doc", &test_doc).await.unwrap();

        let loaded_doc = gcs_store.load_doc_v2("test_doc").await.unwrap();
        let loaded_text = loaded_doc.get_or_insert_text("content");
        let content = loaded_text.get_string(&loaded_doc.transact());

        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_mock_gcs_store_error_handling() {
        let gcs_store = MockGcsStore::new();

        let result = gcs_store.load_doc_v2("test").await;
        assert!(result.is_ok());

        gcs_store.set_should_fail(true);
        let result = gcs_store.load_doc_v2("test").await;
        assert!(result.is_err());

        gcs_store.set_should_fail(false);
        let result = gcs_store.load_doc_v2("test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_redis_store_lock_operations() {
        let redis_store = MockRedisStore::new();

        let lock_acquired = redis_store
            .acquire_doc_lock("test_lock", "instance_1")
            .await
            .unwrap();
        assert!(lock_acquired);

        let lock_acquired = redis_store
            .acquire_doc_lock("test_lock", "instance_2")
            .await
            .unwrap();
        assert!(!lock_acquired);

        redis_store
            .release_doc_lock("test_lock", "instance_1")
            .await
            .unwrap();

        let lock_acquired = redis_store
            .acquire_doc_lock("test_lock", "instance_2")
            .await
            .unwrap();
        assert!(lock_acquired);
    }

    #[tokio::test]
    async fn test_mock_redis_store_lock_failure() {
        let redis_store = MockRedisStore::new();

        let result = redis_store
            .acquire_doc_lock("test", "instance")
            .await
            .unwrap();
        assert!(result);

        redis_store.set_lock_should_fail(true);
        let result = redis_store
            .acquire_doc_lock("test2", "instance")
            .await
            .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_mock_redis_store_stream_operations() {
        let redis_store = MockRedisStore::new();

        let data = redis_store
            .read_all_stream_data("empty_stream")
            .await
            .unwrap();
        assert!(data.is_empty());

        redis_store.add_stream_data("test_stream", Bytes::from("update1"));
        redis_store.add_stream_data("test_stream", Bytes::from("update2"));

        let data = redis_store
            .read_all_stream_data("test_stream")
            .await
            .unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], Bytes::from("update1"));
        assert_eq!(data[1], Bytes::from("update2"));

        redis_store.clear_stream("test_stream");
        let data = redis_store
            .read_all_stream_data("test_stream")
            .await
            .unwrap();
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_flush_to_gcs_scenario_simulation() {
        let gcs_store = MockGcsStore::new();
        let redis_store = MockRedisStore::new();

        let doc_id = "test_document";

        redis_store.set_lock_should_fail(true);
        let lock_acquired = redis_store
            .acquire_doc_lock(&format!("gcs:lock:{}", doc_id), "test_instance")
            .await
            .unwrap();
        assert!(!lock_acquired, "Lock should fail when configured to fail");

        redis_store.set_lock_should_fail(false);
        let lock_acquired = redis_store
            .acquire_doc_lock(&format!("gcs:lock:{}", doc_id), "test_instance")
            .await
            .unwrap();
        assert!(
            lock_acquired,
            "Lock should succeed when configured to succeed"
        );

        let initial_doc = Doc::new();
        let text = initial_doc.get_or_insert_text("content");
        {
            let mut txn = initial_doc.transact_mut();
            text.push(&mut txn, "Initial content");
        }
        gcs_store.flush_doc_v2(doc_id, &initial_doc).await.unwrap();

        let update_doc = Doc::new();
        let update_text = update_doc.get_or_insert_text("content");
        {
            let mut txn = update_doc.transact_mut();
            update_text.push(&mut txn, "Redis update");
            let update = txn.encode_update_v1();
            redis_store.add_stream_data(doc_id, Bytes::from(update));
        }

        let stream_updates = redis_store.read_all_stream_data(doc_id).await.unwrap();
        assert!(!stream_updates.is_empty(), "Should have Redis updates");

        let gcs_doc = gcs_store.load_doc_v2(doc_id).await.unwrap();
        {
            let mut txn = gcs_doc.transact_mut();
            for update_bytes in stream_updates {
                if let Ok(update) = Update::decode_v1(&update_bytes) {
                    let _ = txn.apply_update(update);
                }
            }
        }

        let final_text = gcs_doc.get_or_insert_text("content");
        let content = final_text.get_string(&gcs_doc.transact());
        assert!(
            content.contains("Initial content"),
            "Should contain initial content"
        );
        assert!(
            content.contains("Redis update"),
            "Should contain Redis update"
        );

        gcs_store.flush_doc_v2(doc_id, &gcs_doc).await.unwrap();

        redis_store
            .release_doc_lock(&format!("gcs:lock:{}", doc_id), "test_instance")
            .await
            .unwrap();

        let lock_acquired = redis_store
            .acquire_doc_lock(&format!("gcs:lock:{}", doc_id), "different_instance")
            .await
            .unwrap();
        assert!(lock_acquired, "Lock should be available after release");
    }

    #[tokio::test]
    async fn test_concurrent_lock_operations() {
        let redis_store = Arc::new(MockRedisStore::new());
        let doc_id = "concurrent_test";
        let lock_id = format!("gcs:lock:{}", doc_id);

        // Test concurrent lock acquisition
        let redis1 = redis_store.clone();
        let redis2 = redis_store.clone();
        let lock_id1 = lock_id.clone();
        let lock_id2 = lock_id.clone();

        let task1 =
            tokio::spawn(async move { redis1.acquire_doc_lock(&lock_id1, "instance1").await });

        let task2 =
            tokio::spawn(async move { redis2.acquire_doc_lock(&lock_id2, "instance2").await });

        let (result1, result2) = tokio::join!(task1, task2);
        let acquired1 = result1.unwrap().unwrap();
        let acquired2 = result2.unwrap().unwrap();

        // Only one should succeed
        assert!(
            acquired1 != acquired2,
            "Only one lock acquisition should succeed"
        );
        assert!(
            acquired1 || acquired2,
            "At least one lock acquisition should succeed"
        );
    }

    #[tokio::test]
    async fn test_error_recovery_scenarios() {
        let gcs_store = MockGcsStore::new();
        let redis_store = MockRedisStore::new();

        gcs_store.set_should_fail(true);
        let result = gcs_store.load_doc_v2("test").await;
        assert!(result.is_err(), "Should fail when configured to fail");

        gcs_store.set_should_fail(false);
        let result = gcs_store.load_doc_v2("test").await;
        assert!(result.is_ok(), "Should recover after configuration reset");

        redis_store.set_lock_should_fail(true);
        let result = redis_store
            .acquire_doc_lock("test", "instance")
            .await
            .unwrap();
        assert!(!result, "Lock should fail when configured to fail");

        redis_store.set_lock_should_fail(false);
        let result = redis_store
            .acquire_doc_lock("test", "instance")
            .await
            .unwrap();
        assert!(result, "Lock should succeed after recovery");
    }
}

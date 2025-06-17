mod mock_server;

use bytes::Bytes;
use mock_server::{MockGcsStore, MockRedisStore};
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, Text, Transact, Update};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flush_to_gcs() {
        let gcs_store = MockGcsStore::new();
        let redis_store = MockRedisStore::new();

        let doc_id = "test_document";

        let lock_acquired = redis_store
            .acquire_doc_lock(&format!("gcs:lock:{}", doc_id), "test_instance")
            .await
            .unwrap();
        assert!(lock_acquired, "Lock should be acquired successfully");

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
}

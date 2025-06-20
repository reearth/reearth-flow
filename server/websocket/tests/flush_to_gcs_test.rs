mod mock_server;

use bytes::Bytes;
use mock_server::{MockGcsStore, MockRedisStore};
use std::sync::Arc;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, Text, Transact, Update};

struct TestBroadcastPool {
    gcs_store: Arc<MockGcsStore>,
    redis_store: Arc<MockRedisStore>,
}

impl TestBroadcastPool {
    fn new(gcs_store: Arc<MockGcsStore>, redis_store: Arc<MockRedisStore>) -> Self {
        Self {
            gcs_store,
            redis_store,
        }
    }

    async fn test_flush_to_gcs_scenario_simulation(&self, doc_id: &str) -> anyhow::Result<()> {
        let lock_id = format!("gcs:lock:{}", doc_id);
        let instance_id = "test-instance-123".to_string();

        let lock_acquired = self
            .redis_store
            .acquire_doc_lock(&lock_id, &instance_id)
            .await?;

        if !lock_acquired {
            return Ok(());
        }

        let doc = self.gcs_store.load_doc_v2(doc_id).await?;
        let mut txn = doc.transact_mut();

        let updates = self.redis_store.read_all_stream_data(doc_id).await?;

        for update_bytes in updates {
            if let Ok(update) = Update::decode_v1(&update_bytes) {
                if let Err(e) = txn.apply_update(update) {
                    eprintln!("Failed to apply Redis update: {}", e);
                }
            }
        }

        drop(txn);

        self.gcs_store.flush_doc_v2(doc_id, &doc).await?;

        if let Err(e) = self
            .redis_store
            .release_doc_lock(&lock_id, &instance_id)
            .await
        {
            eprintln!("Failed to release GCS lock: {}", e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flush_to_gcs() {
        let gcs_store = Arc::new(MockGcsStore::new());
        let redis_store = Arc::new(MockRedisStore::new());
        let pool = TestBroadcastPool::new(gcs_store.clone(), redis_store.clone());

        let doc_id = "test_document";

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
            update_text.push(&mut txn, " Redis update");
            let update = txn.encode_update_v1();
            redis_store.add_stream_data(doc_id, Bytes::from(update));
        }

        pool.test_flush_to_gcs_scenario_simulation(doc_id)
            .await
            .unwrap();

        let final_doc = gcs_store.load_doc_v2(doc_id).await.unwrap();
        let final_text = final_doc.get_or_insert_text("content");
        let content = final_text.get_string(&final_doc.transact());

        assert!(
            content.contains("Initial content"),
            "Should contain initial content, got: {}",
            content
        );
        assert!(
            content.contains("Redis update"),
            "Should contain Redis update, got: {}",
            content
        );

        println!("Final content: {}", content);
    }
}

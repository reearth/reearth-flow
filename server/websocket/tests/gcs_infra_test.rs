mod gcs_test_utils;

use websocket::application::usecases::kv::DocOps;
use websocket::domain::repositories::kv::KVStore;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};

#[tokio::test]
async fn test_raw_kv_roundtrip() {
    let infra = gcs_test_utils::TestInfra::start().await;

    // Direct KVStore get/upsert
    let key = b"test-key";
    let value = b"test-value";
    infra
        .gcs_store
        .upsert(key, value)
        .await
        .expect("upsert should succeed");

    let result = infra
        .gcs_store
        .get(key)
        .await
        .expect("get should succeed");
    assert_eq!(result.unwrap(), value.to_vec());
}

#[tokio::test]
async fn test_doc_v2_roundtrip() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Write a document via flush_doc_v2
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "hello from fake-gcs");
    }
    {
        let txn = doc.transact();
        store
            .flush_doc_v2("test-doc", &txn)
            .await
            .expect("flush_doc_v2 should succeed");
    }

    // Read it back via load_doc_v2
    let loaded = Doc::new();
    {
        let mut load_txn = loaded.transact_mut();
        store
            .load_doc_v2("test-doc", &mut load_txn)
            .await
            .expect("load_doc_v2 should succeed");
    }

    let loaded_text = loaded.get_or_insert_text("content");
    let content = loaded_text.get_string(&loaded.transact());
    assert_eq!(content, "hello from fake-gcs");
}

#[tokio::test]
async fn test_push_update_and_list() {
    let infra = gcs_test_utils::TestInfra::start().await;

    // Push several updates
    for i in 0..5 {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.push(&mut txn, &format!("update-{}", i));
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);

        infra
            .gcs_store
            .push_update("test-doc", &update.into(), &infra.redis_store)
            .await
            .expect("push_update should succeed");
    }

    // List updates metadata
    let metadata = infra
        .gcs_store
        .get_updates_metadata("test-doc")
        .await
        .expect("get_updates_metadata should succeed");

    assert_eq!(metadata.len(), 5, "should have 5 updates");

    // Verify clock values are sequential (descending in result)
    let clocks: Vec<u32> = metadata.iter().map(|(c, _)| *c).collect();
    for window in clocks.windows(2) {
        assert_eq!(
            window[0],
            window[1] + 1,
            "clocks should be sequential descending"
        );
    }
}

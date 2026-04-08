mod gcs_test_utils;

use websocket::application::usecases::kv::DocOps;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};

// ---------------------------------------------------------------------------
// Phase 4: Shutdown correctness — doc_v2 vs v1 state comparison
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_change_session_produces_no_spurious_update() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Create a document and save it as doc_v2
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "initial content");
    }
    {
        let txn = doc.transact();
        store
            .flush_doc_v2("test-doc", &txn)
            .await
            .expect("flush_doc_v2");
    }

    // Simulate session start: load doc_v2 into a new awareness doc
    let awareness_doc = Doc::new();
    {
        let mut txn = awareness_doc.transact_mut();
        store
            .load_doc_v2("test-doc", &mut txn)
            .await
            .expect("load_doc_v2");
    }

    // Simulate shutdown: compare awareness doc against doc_v2 (the FIXED path)
    let gcs_doc = Doc::new();
    {
        let mut txn = gcs_doc.transact_mut();
        store
            .load_doc_v2("test-doc", &mut txn)
            .await
            .expect("load_doc_v2 for comparison");
    }

    let gcs_state = gcs_doc.transact().state_vector();
    let awareness_txn = awareness_doc.transact();
    let diff = awareness_txn.encode_diff_v1(&gcs_state);
    let awareness_state = awareness_txn.state_vector();

    // The diff should be empty (no changes)
    let is_empty = diff.is_empty()
        || (diff.len() == 2 && diff[0] == 0 && diff[1] == 0)
        || awareness_state == gcs_state;

    assert!(
        is_empty,
        "No-change session should produce empty diff, got {} bytes",
        diff.len()
    );
}

#[tokio::test]
async fn changed_session_produces_exactly_one_update() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Create and save initial document
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "initial content");
    }
    {
        let txn = doc.transact();
        store
            .flush_doc_v2("test-doc", &txn)
            .await
            .expect("flush_doc_v2");
    }

    // Simulate session start: load into awareness doc
    let awareness_doc = Doc::new();
    {
        let mut txn = awareness_doc.transact_mut();
        store
            .load_doc_v2("test-doc", &mut txn)
            .await
            .expect("load_doc_v2");
    }

    // Make actual changes during the session
    let awareness_text = awareness_doc.get_or_insert_text("content");
    {
        let mut txn = awareness_doc.transact_mut();
        awareness_text.push(&mut txn, " MODIFIED");
    }

    // Simulate shutdown: compare awareness doc against doc_v2
    let gcs_doc = Doc::new();
    {
        let mut txn = gcs_doc.transact_mut();
        store
            .load_doc_v2("test-doc", &mut txn)
            .await
            .expect("load_doc_v2 for comparison");
    }

    let gcs_state = gcs_doc.transact().state_vector();
    let awareness_txn = awareness_doc.transact();
    let diff = awareness_txn.encode_diff_v1(&gcs_state);
    let awareness_state = awareness_txn.state_vector();

    let is_empty = diff.is_empty()
        || (diff.len() == 2 && diff[0] == 0 && diff[1] == 0)
        || awareness_state == gcs_state;

    assert!(
        !is_empty,
        "Changed session should produce non-empty diff"
    );

    // Push the update and flush
    let diff_bytes = bytes::Bytes::from(diff);
    store
        .push_update("test-doc", &diff_bytes, &infra.redis_store)
        .await
        .expect("push_update");
    store
        .flush_doc_v2("test-doc", &awareness_txn)
        .await
        .expect("flush_doc_v2");
    drop(awareness_txn);

    // Verify the saved content is correct
    let verify_doc = Doc::new();
    {
        let mut txn = verify_doc.transact_mut();
        store
            .load_doc_v2("test-doc", &mut txn)
            .await
            .expect("load_doc_v2 verify");
    }
    let verify_text = verify_doc.get_or_insert_text("content");
    let content = verify_text.get_string(&verify_doc.transact());
    assert!(
        content.contains("initial content") && content.contains("MODIFIED"),
        "Saved doc should contain both initial and modified content, got: {}",
        content
    );
}

// ---------------------------------------------------------------------------
// Phase 5: Snapshot cleanup
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cleanup_keeps_most_recent_n_updates() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Push 15 updates
    for i in 0..15 {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.push(&mut txn, &format!("update-{}", i));
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);

        store
            .push_update("test-doc", &update.into(), &infra.redis_store)
            .await
            .expect("push_update");
    }

    // Verify we have 15 updates
    let before = store.get_updates_metadata("test-doc").await.unwrap();
    assert_eq!(before.len(), 15, "should have 15 updates before cleanup");

    // Cleanup, keeping only 10
    let deleted = store
        .cleanup_old_updates("test-doc", 10)
        .await
        .expect("cleanup_old_updates");
    assert_eq!(deleted, 5, "should delete 5 updates");

    // Verify we have exactly 10 updates remaining
    let after = store.get_updates_metadata("test-doc").await.unwrap();
    assert_eq!(after.len(), 10, "should have 10 updates after cleanup");
}

#[tokio::test]
async fn cleanup_preserves_document_state() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Create 15 updates with distinct content
    for i in 0..15 {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.push(&mut txn, &format!("v{} ", i));
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);

        store
            .push_update("test-doc", &update.into(), &infra.redis_store)
            .await
            .expect("push_update");
    }

    // Load full state via v1 path (all updates) before cleanup
    let before_doc = Doc::new();
    {
        let mut txn = before_doc.transact_mut();
        store.load_doc("test-doc", &mut txn).await.expect("load_doc");
    }
    let before_text = before_doc.get_or_insert_text("content");
    let before_content = before_text.get_string(&before_doc.transact());

    // Cleanup
    store
        .cleanup_old_updates("test-doc", 10)
        .await
        .expect("cleanup_old_updates");

    // Load full state after cleanup — should be identical
    let after_doc = Doc::new();
    {
        let mut txn = after_doc.transact_mut();
        store.load_doc("test-doc", &mut txn).await.expect("load_doc");
    }
    let after_text = after_doc.get_or_insert_text("content");
    let after_content = after_text.get_string(&after_doc.transact());

    assert_eq!(
        before_content, after_content,
        "Document state should be preserved after cleanup"
    );
}

#[tokio::test]
async fn cleanup_fewer_than_max_is_noop() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Push only 5 updates
    for i in 0..5 {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.push(&mut txn, &format!("update-{}", i));
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);

        store
            .push_update("test-doc", &update.into(), &infra.redis_store)
            .await
            .expect("push_update");
    }

    let deleted = store
        .cleanup_old_updates("test-doc", 10)
        .await
        .expect("cleanup_old_updates");
    assert_eq!(deleted, 0, "should not delete anything when under max");
}

#[tokio::test]
async fn cleanup_nonexistent_doc_is_noop() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    let deleted = store
        .cleanup_old_updates("nonexistent-doc", 10)
        .await
        .expect("cleanup_old_updates");
    assert_eq!(deleted, 0, "should return 0 for nonexistent doc");
}

#[tokio::test]
async fn cleanup_200_updates_completes_successfully() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Push 200 updates
    for i in 0..200 {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.push(&mut txn, &format!("u{} ", i));
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);

        store
            .push_update("test-doc", &update.into(), &infra.redis_store)
            .await
            .expect("push_update");
    }

    let deleted = store
        .cleanup_old_updates("test-doc", 10)
        .await
        .expect("cleanup_old_updates");
    assert_eq!(deleted, 190, "should delete 190 updates");

    let remaining = store.get_updates_metadata("test-doc").await.unwrap();
    assert_eq!(remaining.len(), 10, "should have exactly 10 remaining");
}

// ---------------------------------------------------------------------------
// Phase 6: Document deletion
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_removes_all_doc_data() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Create and save a doc_v2
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "document to delete");
    }
    {
        let txn = doc.transact();
        store
            .flush_doc_v2("test-doc", &txn)
            .await
            .expect("flush_doc_v2");
    }

    // Push 3 updates (enough to create OID + updates, no compaction trigger)
    for i in 0..3 {
        let update_doc = Doc::new();
        let t = update_doc.get_or_insert_text("content");
        let mut txn = update_doc.transact_mut();
        t.push(&mut txn, &format!("update-{}", i));
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);
        store
            .push_update("test-doc", &update.into(), &infra.redis_store)
            .await
            .expect("push_update");
    }

    // Verify updates exist
    let metadata = store.get_updates_metadata("test-doc").await.unwrap();
    assert!(!metadata.is_empty(), "updates should exist before delete");

    // Delete everything
    store
        .delete_all_doc_data("test-doc")
        .await
        .expect("delete_all_doc_data");

    // Verify updates are gone
    let metadata = store.get_updates_metadata("test-doc").await.unwrap();
    assert!(
        metadata.is_empty(),
        "updates should be empty after delete, got {} entries",
        metadata.len()
    );
}

#[tokio::test]
async fn delete_clears_doc_v2_snapshot() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Write doc_v2
    let doc = Doc::new();
    let text = doc.get_or_insert_text("content");
    {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "to be deleted");
    }
    {
        let txn = doc.transact();
        store
            .flush_doc_v2("test-doc", &txn)
            .await
            .expect("flush_doc_v2");
    }

    // Delete
    store
        .delete_all_doc_data("test-doc")
        .await
        .expect("delete_all_doc_data");

    // Verify doc_v2 is gone (loads empty)
    let check_doc = Doc::new();
    {
        let mut txn = check_doc.transact_mut();
        store
            .load_doc_v2("test-doc", &mut txn)
            .await
            .expect("load_doc_v2 after delete");
    }
    let check_text = check_doc.get_or_insert_text("content");
    let content = check_text.get_string(&check_doc.transact());
    assert!(content.is_empty(), "doc_v2 should be empty after delete");
}

// ---------------------------------------------------------------------------
// Phase 7: Batch cleanup (migration)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cleanup_all_documents_processes_multiple_docs() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Create 3 documents, each with 15 updates
    for doc_idx in 0..3 {
        let doc_id = format!("doc-{}", doc_idx);
        for i in 0..15 {
            let doc = Doc::new();
            let text = doc.get_or_insert_text("content");
            let mut txn = doc.transact_mut();
            text.push(&mut txn, &format!("d{}-u{} ", doc_idx, i));
            let update = txn.encode_state_as_update_v1(&StateVector::default());
            drop(txn);

            store
                .push_update(&doc_id, &update.into(), &infra.redis_store)
                .await
                .expect("push_update");
        }
    }

    // Run batch cleanup (keep 10 per doc)
    let (docs_processed, total_deleted) = store
        .cleanup_all_documents(10)
        .await
        .expect("cleanup_all_documents");

    assert_eq!(docs_processed, 3, "should process 3 docs");
    assert_eq!(
        total_deleted, 15,
        "should delete 5 updates from each of 3 docs = 15"
    );

    // Verify each doc has exactly 10 updates
    for doc_idx in 0..3 {
        let doc_id = format!("doc-{}", doc_idx);
        let metadata = store.get_updates_metadata(&doc_id).await.unwrap();
        assert_eq!(
            metadata.len(),
            10,
            "doc-{} should have 10 updates, got {}",
            doc_idx,
            metadata.len()
        );
    }
}

#[tokio::test]
async fn delete_nonexistent_doc_is_idempotent() {
    let infra = gcs_test_utils::TestInfra::start().await;
    let store = infra.gcs_store.as_ref();

    // Should not error
    let result = store.delete_all_doc_data("nonexistent-doc").await;
    assert!(result.is_ok(), "delete of nonexistent doc should succeed");
}

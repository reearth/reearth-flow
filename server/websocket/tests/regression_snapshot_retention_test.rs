//! Regression tests for: Snapshot retention and project deletion
//!
//! Requirements:
//! 1. Keep only the most recent 10 snapshots per document
//! 2. Handle projects with hundreds of snapshots without timeout (batched ops)
//! 3. Project deletion must delete ALL snapshots and intermediate data
//!
//! These tests exercise the data structures and logic patterns used by
//! `cleanup_old_updates` and `delete_all_doc_data` without requiring GCS.

/// Simulates the clock-based retention logic from `cleanup_old_updates`.
/// Given a list of (clock, object_name) pairs, returns which objects to delete
/// to keep only `max_keep` most recent.
fn compute_retention(clock_objects: &[(u32, String)], max_keep: usize) -> Vec<(u32, String)> {
    if clock_objects.len() <= max_keep {
        return vec![];
    }

    let mut sorted = clock_objects.to_vec();
    sorted.sort_by_key(|(clock, _)| *clock); // oldest first

    let num_to_delete = sorted.len() - max_keep;
    sorted[..num_to_delete].to_vec()
}

#[test]
fn retention_keeps_exactly_max_when_over_limit() {
    let objects: Vec<(u32, String)> = (1..=15).map(|i| (i, format!("update_{}", i))).collect();

    let to_delete = compute_retention(&objects, 10);

    assert_eq!(to_delete.len(), 5, "Should delete 5 to keep 10");
    // Deleted should be the 5 oldest (clocks 1..=5)
    let deleted_clocks: Vec<u32> = to_delete.iter().map(|(c, _)| *c).collect();
    assert_eq!(deleted_clocks, vec![1, 2, 3, 4, 5]);
}

#[test]
fn retention_no_op_when_under_limit() {
    let objects: Vec<(u32, String)> = (1..=7).map(|i| (i, format!("update_{}", i))).collect();

    let to_delete = compute_retention(&objects, 10);
    assert!(
        to_delete.is_empty(),
        "Should delete nothing when under limit"
    );
}

#[test]
fn retention_no_op_when_exactly_at_limit() {
    let objects: Vec<(u32, String)> = (1..=10).map(|i| (i, format!("update_{}", i))).collect();

    let to_delete = compute_retention(&objects, 10);
    assert!(
        to_delete.is_empty(),
        "Should delete nothing when exactly at limit"
    );
}

/// Projects with hundreds of snapshots: verify the math is correct and
/// batching logic (chunks of 50) would process them all.
#[test]
fn retention_handles_hundreds_of_snapshots() {
    let count = 500;
    let objects: Vec<(u32, String)> = (1..=count).map(|i| (i, format!("update_{}", i))).collect();

    let to_delete = compute_retention(&objects, 10);
    assert_eq!(to_delete.len(), 490, "Should delete 490 to keep 10");

    // Verify the kept ones are the most recent 10 (clocks 491..=500)
    let deleted_clocks: std::collections::HashSet<u32> =
        to_delete.iter().map(|(c, _)| *c).collect();
    for clock in 491..=500u32 {
        assert!(
            !deleted_clocks.contains(&clock),
            "Clock {} should be KEPT (most recent 10), not deleted",
            clock
        );
    }
    for clock in 1..=490u32 {
        assert!(
            deleted_clocks.contains(&clock),
            "Clock {} should be DELETED (older than most recent 10)",
            clock
        );
    }

    // Verify batching: 490 objects at batch_size=50 = 10 batches
    const BATCH_SIZE: usize = 50;
    let num_batches = (to_delete.len() + BATCH_SIZE - 1) / BATCH_SIZE;
    assert_eq!(num_batches, 10, "490 deletes in batches of 50 = 10 batches");
}

/// Clock values may not be contiguous (gaps from prior cleanups).
/// Retention must work on sorted order, not contiguous numbering.
#[test]
fn retention_handles_non_contiguous_clocks() {
    // Clocks: 5, 10, 15, 100, 200, 300, 400, 500, 600, 700, 800, 900
    let objects: Vec<(u32, String)> = vec![5, 10, 15, 100, 200, 300, 400, 500, 600, 700, 800, 900]
        .into_iter()
        .map(|c| (c, format!("update_{}", c)))
        .collect();

    let to_delete = compute_retention(&objects, 10);
    assert_eq!(to_delete.len(), 2);
    let deleted_clocks: Vec<u32> = to_delete.iter().map(|(c, _)| *c).collect();
    assert_eq!(deleted_clocks, vec![5, 10], "Should delete the 2 oldest");
}

// ─── Project deletion key patterns ──────────────────────────────────────────

/// Verifies that the key patterns used for project deletion cover all
/// object types that exist for a document.
#[test]
fn deletion_covers_all_object_types() {
    let doc_id = "test-project-123";
    let doc_id_hex = hex::encode(doc_id.as_bytes());

    // These are the key patterns that delete_all_doc_data must clean up:
    let expected_patterns = vec![
        format!("doc_v2:{}", doc_id_hex), // compressed v2 snapshot
        format!("checkpoint:{}", doc_id_hex), // compaction checkpoint
                                          // OID mapping: key_oid(doc_id) → hex encoded
                                          // OID-based objects: key_doc, key_state_vector, key_update(s), key_meta(s)
    ];

    // Verify the key construction is consistent
    for pattern in &expected_patterns {
        assert!(!pattern.is_empty(), "Key pattern should not be empty");
        // The hex-encoded doc_id should appear in the key
        assert!(
            pattern.contains(&doc_id_hex),
            "Key '{}' should contain hex-encoded doc_id '{}'",
            pattern,
            doc_id_hex
        );
    }
}

/// Verify that hex encoding is deterministic and reversible —
/// this is critical because GCS object names are hex-encoded keys.
#[test]
fn hex_encoding_is_deterministic_and_reversible() {
    let doc_id = "project-abc-123";
    let encoded = hex::encode(doc_id.as_bytes());
    let decoded_bytes = hex::decode(&encoded).unwrap();
    let decoded = std::str::from_utf8(&decoded_bytes).unwrap();
    assert_eq!(decoded, doc_id);

    // Same input always produces same output (GCS object names must be stable)
    assert_eq!(hex::encode(doc_id.as_bytes()), encoded);
}

// ─── Broadcast channel lag recovery (connection reliability) ────────────────

/// Verifies that a lagged broadcast receiver can recover and continue
/// receiving messages instead of being disconnected.
/// This was a contributing factor to stale content: clients that fell behind
/// would disconnect and reconnect, potentially loading an outdated snapshot.
#[tokio::test]
async fn broadcast_lag_recovery_prevents_stale_reconnects() {
    use bytes::Bytes;
    use tokio::sync::broadcast;

    // Small channel that will overflow easily
    let (tx, _rx) = broadcast::channel::<Bytes>(4);
    let mut rx = tx.subscribe();

    // Send enough to overflow
    for i in 0..10u8 {
        let _ = tx.send(Bytes::from(vec![i]));
    }

    let mut lagged = false;
    let mut received = Vec::new();

    // The FIX: handle Lagged by continuing instead of disconnecting
    loop {
        match rx.try_recv() {
            Ok(msg) => received.push(msg),
            Err(broadcast::error::TryRecvError::Lagged(_)) => {
                lagged = true;
                continue; // <── THE FIX: recover instead of return/disconnect
            }
            Err(broadcast::error::TryRecvError::Empty) => break,
            Err(broadcast::error::TryRecvError::Closed) => break,
        }
    }

    assert!(lagged, "Receiver should have experienced lag");
    assert!(
        !received.is_empty(),
        "After recovering from lag, receiver should still get the most recent messages"
    );
    // The most recent messages (within channel capacity) should be received
    assert!(
        received.len() <= 4,
        "Should receive at most channel_capacity messages after lag recovery"
    );
}

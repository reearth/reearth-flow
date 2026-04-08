//! Regression tests for: "Many versions being created even though no changes have occurred"
//!
//! Root causes:
//! 1. The no-change guard was missing or insufficient — on session close, a diff
//!    was computed against GCS state but the "empty diff" check didn't catch all
//!    cases where no real changes existed.
//! 2. Connection counter leaked on handler errors, leaving "ghost connections"
//!    that kept the BroadcastGroup alive and triggered extra save cycles.
//!
//! The fix:
//! - Shutdown now checks: `update_bytes.is_empty() || update_bytes == [0,0] || awareness_state == gcs_state`
//! - Connection counter decrement is guaranteed via the inner-function pattern.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};

// ─── Bug reproduction: no-change guard ──────────────────────────────────────

/// Simulates the shutdown save-decision logic from `BroadcastGroup::shutdown()`.
/// Returns true if a save WOULD be triggered.
fn should_save(awareness_doc: &Doc, gcs_doc: &Doc) -> bool {
    let gcs_txn = gcs_doc.transact();
    let gcs_state = gcs_txn.state_vector();
    drop(gcs_txn);

    let awareness_txn = awareness_doc.transact();
    let update = awareness_txn.encode_diff_v1(&gcs_state);
    let awareness_state = awareness_txn.state_vector();
    let update_bytes = bytes::Bytes::from(update);

    // This is the exact guard from broadcast_group.rs:593-597
    !(update_bytes.is_empty()
        || (update_bytes.len() == 2 && update_bytes[0] == 0 && update_bytes[1] == 0)
        || awareness_state == gcs_state)
}

/// BUG: Opening and closing a session without making any edits should NOT
/// create a new version. Both docs have identical state → save must be skipped.
#[test]
fn bug_no_edits_must_not_create_version() {
    // Simulate: doc loaded from GCS into both awareness and GCS reference
    let source = Doc::new();
    {
        let txt = source.get_or_insert_text("content");
        let mut txn = source.transact_mut();
        txt.push(&mut txn, "Hello, existing content");
    }

    // GCS doc: the state as it was when last saved
    let gcs_doc = Doc::new();
    {
        let txn = source.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        let mut gcs_txn = gcs_doc.transact_mut();
        let decoded = yrs::Update::decode_v1(&update).unwrap();
        gcs_txn.apply_update(decoded).unwrap();
    }

    // Awareness doc: loaded from the same source (no edits made during session)
    let awareness_doc = Doc::new();
    {
        let txn = source.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        let mut awareness_txn = awareness_doc.transact_mut();
        let decoded = yrs::Update::decode_v1(&update).unwrap();
        awareness_txn.apply_update(decoded).unwrap();
    }

    assert!(
        !should_save(&awareness_doc, &gcs_doc),
        "BUG REGRESSION: A save was triggered even though no changes occurred. \
         This creates spurious versions in the history."
    );
}

/// Verify that when real edits ARE made, the save IS triggered.
#[test]
fn fix_real_edits_do_trigger_save() {
    let source = Doc::new();
    {
        let txt = source.get_or_insert_text("content");
        let mut txn = source.transact_mut();
        txt.push(&mut txn, "original");
    }

    // GCS doc: original state
    let gcs_doc = Doc::new();
    {
        let txn = source.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        let mut gcs_txn = gcs_doc.transact_mut();
        gcs_txn
            .apply_update(yrs::Update::decode_v1(&update).unwrap())
            .unwrap();
    }

    // Awareness doc: has additional edits
    let awareness_doc = Doc::new();
    {
        let txn = source.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        let mut awareness_txn = awareness_doc.transact_mut();
        awareness_txn
            .apply_update(yrs::Update::decode_v1(&update).unwrap())
            .unwrap();
    }
    {
        let txt = awareness_doc.get_or_insert_text("content");
        let mut txn = awareness_doc.transact_mut();
        txt.push(&mut txn, " — edited!");
    }

    assert!(
        should_save(&awareness_doc, &gcs_doc),
        "A save SHOULD be triggered when the awareness doc has real edits"
    );
}

/// Edge case: both docs are completely empty (brand new document, no edits).
#[test]
fn fix_two_empty_docs_must_not_save() {
    let awareness_doc = Doc::new();
    let gcs_doc = Doc::new();

    assert!(
        !should_save(&awareness_doc, &gcs_doc),
        "Two empty docs should not trigger a save"
    );
}

/// Edge case: the "empty diff" is exactly [0, 0] (Yrs encoding for no-ops).
#[test]
fn fix_empty_diff_encoding_is_caught() {
    let doc = Doc::new();
    let txn = doc.transact();

    // Encoding a diff against the doc's own state vector should be a no-op
    let self_state = txn.state_vector();
    let diff = txn.encode_diff_v1(&self_state);
    let diff_bytes = bytes::Bytes::from(diff);

    assert!(
        diff_bytes.is_empty()
            || (diff_bytes.len() == 2 && diff_bytes[0] == 0 && diff_bytes[1] == 0),
        "A self-diff should be recognized as empty by the guard. Got: {:?}",
        diff_bytes.as_ref()
    );
}

// ─── Bug reproduction: connection counter leak ──────────────────────────────

/// BUG: If `handle_connection_inner` returned an error (e.g. auth failure,
/// stream error), the old code path could skip `decrement_connections_count`.
/// This left a ghost connection that prevented cleanup and caused spurious saves.
///
/// The fix: decrement is in the outer function, after the inner function returns
/// (regardless of Ok or Err).
///
/// This test simulates the pattern using an atomic counter.
#[tokio::test]
async fn bug_connection_counter_must_decrement_on_error() {
    let counter = Arc::new(AtomicU32::new(0));

    // Simulate the FIXED pattern from websocket.rs handle_connection:
    // increment → run inner → decrement (always)
    async fn handle_connection(
        counter: Arc<AtomicU32>,
        should_error: bool,
    ) -> Result<(), &'static str> {
        counter.fetch_add(1, Ordering::SeqCst); // increment

        let result = async {
            if should_error {
                Err("connection error")
            } else {
                Ok(())
            }
        }
        .await;

        // This decrement is OUTSIDE the inner function — it always runs.
        // The old code had it INSIDE, where early returns could skip it.
        counter.fetch_sub(1, Ordering::SeqCst); // decrement (guaranteed)

        result
    }

    // Successful connection: counter returns to 0
    let _ = handle_connection(counter.clone(), false).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Counter should be 0 after successful connection"
    );

    // Failed connection: counter MUST still return to 0
    let _ = handle_connection(counter.clone(), true).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "BUG REGRESSION: Counter leaked on error — ghost connection prevents cleanup \
         and causes spurious version saves on next session close"
    );

    // Multiple concurrent failures: counter must be 0 after all complete
    let mut handles = vec![];
    for _ in 0..10 {
        let c = counter.clone();
        handles.push(tokio::spawn(async move {
            let _ = handle_connection(c, true).await;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "Counter must be 0 after 10 concurrent failed connections"
    );
}

/// Demonstrates the OLD buggy pattern where decrement was inside the inner
/// function and could be skipped by early returns.
#[tokio::test]
async fn demonstration_old_pattern_leaks_on_early_return() {
    let counter = Arc::new(AtomicU32::new(0));

    // OLD buggy pattern (simplified):
    // The decrement was after the connection result processing,
    // but early returns in the middle could skip it.
    async fn old_handle_connection(
        counter: Arc<AtomicU32>,
        fail_early: bool,
    ) -> Result<(), &'static str> {
        counter.fetch_add(1, Ordering::SeqCst);

        if fail_early {
            return Err("early auth failure"); // <── BUG: skips decrement!
        }

        // ... connection logic ...

        counter.fetch_sub(1, Ordering::SeqCst); // only reached on success path
        Ok(())
    }

    let _ = old_handle_connection(counter.clone(), true).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1, // LEAKED! Counter is 1 instead of 0
        "This demonstrates the old bug: early return skipped decrement"
    );
}

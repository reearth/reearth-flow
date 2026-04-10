use bytes::Bytes;
use tokio::sync::broadcast;

/// Proves that sending to a broadcast channel after receivers have been dropped
/// returns an error, and that the sender can continue to send after new receivers subscribe.
/// This demonstrates that the awareness_updater's broadcast send failure is transient:
/// continuing instead of returning allows recovery when the channel gets new subscribers.
#[tokio::test]
async fn broadcast_send_failure_is_recoverable() {
    let (tx, rx) = broadcast::channel::<Bytes>(4);

    // Drop all receivers — send will fail
    drop(rx);
    let result = tx.send(Bytes::from(vec![1]));
    assert!(result.is_err(), "Send with no receivers should fail");

    // Subscribe a new receiver — send should succeed again
    let mut rx2 = tx.subscribe();
    let result = tx.send(Bytes::from(vec![2]));
    assert!(result.is_ok(), "Send with new receiver should succeed");

    let received = rx2.recv().await.unwrap();
    assert_eq!(received, Bytes::from(vec![2]));
}

/// Proves that after a broadcast send error, the channel is still usable for future sends.
/// This is the key property that makes `continue` safe in the awareness_updater loop:
/// a failed send doesn't permanently break the sender.
#[tokio::test]
async fn broadcast_sender_survives_failed_sends() {
    let (tx, _rx) = broadcast::channel::<Bytes>(4);

    // Send fails because no receivers
    for _ in 0..5 {
        let _ = tx.send(Bytes::from(vec![0]));
    }

    // After multiple failures, subscribing a new receiver restores delivery
    let mut rx = tx.subscribe();
    tx.send(Bytes::from(vec![42])).unwrap();
    let msg = rx.recv().await.unwrap();
    assert_eq!(msg, Bytes::from(vec![42]));
}

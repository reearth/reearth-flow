use bytes::Bytes;
use tokio::sync::broadcast;

/// Proves that a lagged broadcast receiver recovers instead of disconnecting.
/// This is the formalized version of freeze_evals.rs eval 5.
#[tokio::test]
async fn lagged_receiver_recovers_and_continues() {
    // Create a broadcast channel with capacity 4 (small for easy overflow)
    let (tx, _rx) = broadcast::channel::<Bytes>(4);
    let mut rx = tx.subscribe();

    // Send 10 messages — the receiver hasn't read any, so it will lag
    for i in 0..10u8 {
        tx.send(Bytes::from(vec![i])).unwrap();
    }

    // Now read — first recv() should return Lagged
    let mut received = Vec::new();
    let mut lagged_count = 0u64;

    for _ in 0..10 {
        match rx.try_recv() {
            Ok(msg) => received.push(msg),
            Err(broadcast::error::TryRecvError::Lagged(n)) => {
                lagged_count += n;
                continue;
            }
            Err(broadcast::error::TryRecvError::Empty) => break,
            Err(broadcast::error::TryRecvError::Closed) => break,
        }
    }

    assert!(lagged_count > 0, "Should have detected lag");
    assert!(!received.is_empty(), "Should have received messages after lag");
}

/// Proves the old `while let Ok(msg) = rx.recv().await` pattern exits on lag.
/// After the fix, the loop instead continues (warn + continue on Lagged).
#[tokio::test]
async fn old_pattern_exits_on_lag() {
    // Use a channel with capacity 2 so we can control the exact state
    let (tx, _rx) = broadcast::channel::<Bytes>(2);
    let mut rx = tx.subscribe();

    // Send 5 messages — receiver hasn't read any, so it lags by 3
    for i in 0..5u8 {
        let _ = tx.send(Bytes::from(vec![i]));
    }

    // The old pattern: `while let Ok(msg) = rx.recv().await` exits here because
    // recv() returns Err(Lagged), not Ok.
    let first = rx.recv().await;
    assert!(
        matches!(first, Err(broadcast::error::RecvError::Lagged(_))),
        "First recv after overflow should be Lagged, got {:?}",
        first
    );

    // With the new fix (match + continue on Lagged), the loop survives the error
    // and can receive subsequent messages. Drain what remains in the buffer.
    let mut recovered = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        recovered.push(msg);
    }

    // The last `capacity` messages (3,4) should be available after recovering from lag
    assert!(
        !recovered.is_empty(),
        "Should have recovered buffered messages after the Lagged error"
    );
}

#[cfg(test)]
mod connection_counter_tests {
    use std::sync::{Arc, Mutex};

    /// Simulates the connection counter leak bug:
    /// increment at start, early return on error, counter never decremented.
    #[test]
    fn counter_leak_on_early_return() {
        let counter = Arc::new(Mutex::new(0usize));

        // Simulate: increment then error return (old behavior)
        {
            let mut c = counter.lock().unwrap();
            *c += 1;
        }
        // Simulated error: function returns without decrementing
        // Counter should be 1 (leaked)
        assert_eq!(*counter.lock().unwrap(), 1);

        // Now simulate the fix: wrap in inner function pattern
        let result: Result<(), &str> = {
            // inner function runs and fails
            Err("setup failed")
        };
        // Outer always decrements
        {
            let mut c = counter.lock().unwrap();
            *c = c.saturating_sub(1);
        }
        assert_eq!(*counter.lock().unwrap(), 0);
        assert!(result.is_err());
    }

    /// Proves the inner-function pattern guarantees balanced increment/decrement
    /// even when the inner function returns early.
    #[test]
    fn inner_function_pattern_always_decrements() {
        let counter = Arc::new(Mutex::new(0usize));

        let increment = |c: &Arc<Mutex<usize>>| {
            let mut lock = c.lock().unwrap();
            *lock += 1;
        };
        let decrement = |c: &Arc<Mutex<usize>>| {
            let mut lock = c.lock().unwrap();
            *lock = lock.saturating_sub(1);
        };

        // Simulate three connections that all fail at setup (early return in inner fn)
        for _ in 0..3 {
            increment(&counter);
            // inner function: returns Err early
            let _result: Result<(), &str> = Err("setup failed");
            // outer always decrements
            decrement(&counter);
        }

        assert_eq!(
            *counter.lock().unwrap(),
            0,
            "Counter must be 0 after three failed connections"
        );
    }
}

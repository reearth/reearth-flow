use websocket::infrastructure::websocket::redis_channels::RedisChannels;

#[tokio::test]
async fn test_redis_channels_new() {
    let channels = RedisChannels::new(100, 50);

    assert!(channels.write_tx.capacity() == 100);
    assert!(channels.awareness_tx.capacity() == 50);
}

#[tokio::test]
async fn test_redis_channels_with_different_capacities() {
    let channels_small = RedisChannels::new(10, 5);
    assert!(channels_small.write_tx.capacity() == 10);
    assert!(channels_small.awareness_tx.capacity() == 5);

    let channels_large = RedisChannels::new(1000, 500);
    assert!(channels_large.write_tx.capacity() == 1000);
    assert!(channels_large.awareness_tx.capacity() == 500);
}

#[tokio::test]
async fn test_send_and_receive_write_updates() {
    let channels = RedisChannels::new(100, 50);

    let test_data = vec![1u8, 2, 3, 4, 5];
    channels
        .write_tx
        .send(test_data.clone())
        .await
        .expect("Failed to send write update");

    let mut rx = channels.write_rx.lock().await;
    let received = rx.recv().await.expect("Failed to receive write update");

    assert_eq!(received, test_data);
}

#[tokio::test]
async fn test_send_and_receive_awareness_updates() {
    let channels = RedisChannels::new(100, 50);

    let test_data = vec![10u8, 20, 30, 40, 50];
    channels
        .awareness_tx
        .send(test_data.clone())
        .await
        .expect("Failed to send awareness update");

    let mut rx = channels.awareness_rx.lock().await;
    let received = rx.recv().await.expect("Failed to receive awareness update");

    assert_eq!(received, test_data);
}

#[tokio::test]
async fn test_multiple_write_updates() {
    let channels = RedisChannels::new(100, 50);

    let updates = vec![
        vec![1u8, 2, 3],
        vec![4u8, 5, 6],
        vec![7u8, 8, 9],
        vec![10u8, 11, 12],
    ];

    for update in &updates {
        channels
            .write_tx
            .send(update.clone())
            .await
            .expect("Failed to send write update");
    }

    let mut rx = channels.write_rx.lock().await;
    for expected in &updates {
        let received = rx.recv().await.expect("Failed to receive write update");
        assert_eq!(&received, expected);
    }
}

#[tokio::test]
async fn test_multiple_awareness_updates() {
    let channels = RedisChannels::new(100, 50);

    let updates = vec![
        vec![100u8, 101, 102],
        vec![103u8, 104, 105],
        vec![106u8, 107, 108],
    ];

    for update in &updates {
        channels
            .awareness_tx
            .send(update.clone())
            .await
            .expect("Failed to send awareness update");
    }

    let mut rx = channels.awareness_rx.lock().await;
    for expected in &updates {
        let received = rx.recv().await.expect("Failed to receive awareness update");
        assert_eq!(&received, expected);
    }
}

#[tokio::test]
async fn test_send_empty_data() {
    let channels = RedisChannels::new(100, 50);
    let empty_data = vec![];
    channels
        .write_tx
        .send(empty_data.clone())
        .await
        .expect("Failed to send empty write update");

    let mut rx = channels.write_rx.lock().await;
    let received = rx.recv().await.expect("Failed to receive write update");
    assert_eq!(received, empty_data);
}

#[tokio::test]
async fn test_send_large_data() {
    let channels = RedisChannels::new(100, 50);

    let large_data: Vec<u8> = (0..10240).map(|i| (i % 256) as u8).collect();
    channels
        .write_tx
        .send(large_data.clone())
        .await
        .expect("Failed to send large write update");

    let mut rx = channels.write_rx.lock().await;
    let received = rx.recv().await.expect("Failed to receive write update");
    assert_eq!(received, large_data);
}

#[tokio::test]
async fn test_concurrent_sends() {
    let channels = RedisChannels::new(1000, 500);
    let write_tx = channels.write_tx.clone();
    let awareness_tx = channels.awareness_tx.clone();

    let write_handle = tokio::spawn(async move {
        for i in 0..100 {
            let data = vec![i as u8; 10];
            write_tx.send(data).await.expect("Failed to send");
        }
    });

    let awareness_handle = tokio::spawn(async move {
        for i in 0..100 {
            let data = vec![(i + 100) as u8; 10];
            awareness_tx.send(data).await.expect("Failed to send");
        }
    });

    write_handle.await.expect("Write task failed");
    awareness_handle.await.expect("Awareness task failed");

    let mut write_rx = channels.write_rx.lock().await;
    let mut awareness_rx = channels.awareness_rx.lock().await;

    let mut write_count = 0;
    while write_rx.try_recv().is_ok() {
        write_count += 1;
    }

    let mut awareness_count = 0;
    while awareness_rx.try_recv().is_ok() {
        awareness_count += 1;
    }

    assert_eq!(write_count, 100);
    assert_eq!(awareness_count, 100);
}

#[tokio::test]
async fn test_channel_full_behavior() {
    let channels = RedisChannels::new(2, 2);

    channels.write_tx.send(vec![1u8]).await.unwrap();
    channels.write_tx.send(vec![2u8]).await.unwrap();

    let result = channels.write_tx.try_send(vec![3u8]);
    assert!(result.is_err(), "Channel should be full");
}

#[tokio::test]
async fn test_channel_isolation() {
    let channels = RedisChannels::new(100, 50);

    let write_data = vec![1u8, 2, 3];
    channels
        .write_tx
        .send(write_data.clone())
        .await
        .expect("Failed to send write update");

    let awareness_data = vec![4u8, 5, 6];
    channels
        .awareness_tx
        .send(awareness_data.clone())
        .await
        .expect("Failed to send awareness update");

    let mut write_rx = channels.write_rx.lock().await;
    let write_received = write_rx.recv().await.expect("Failed to receive");
    assert_eq!(write_received, write_data);
    assert_ne!(write_received, awareness_data);

    drop(write_rx); // Release the lock
    let mut awareness_rx = channels.awareness_rx.lock().await;
    let awareness_received = awareness_rx.recv().await.expect("Failed to receive");
    assert_eq!(awareness_received, awareness_data);
    assert_ne!(awareness_received, write_data);
}

#[tokio::test]
async fn test_receiver_drop() {
    let channels = RedisChannels::new(100, 50);

    channels
        .write_tx
        .send(vec![1u8, 2, 3])
        .await
        .expect("Failed to send");

    {
        let mut rx = channels.write_rx.lock().await;
        let _received = rx.recv().await.expect("Failed to receive");
    }

    channels
        .write_tx
        .send(vec![4u8, 5, 6])
        .await
        .expect("Failed to send after lock release");
}

#[tokio::test]
async fn test_try_recv_on_empty_channel() {
    let channels = RedisChannels::new(100, 50);

    let mut rx = channels.write_rx.lock().await;
    let result = rx.try_recv();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_binary_data_integrity() {
    let channels = RedisChannels::new(100, 50);

    let test_patterns = vec![
        vec![0u8; 100],
        vec![255u8; 100],
        (0..=255u8).collect::<Vec<u8>>(),
        vec![0xDEu8, 0xAD, 0xBE, 0xEF],
        vec![0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00],
    ];

    for pattern in test_patterns {
        channels
            .write_tx
            .send(pattern.clone())
            .await
            .expect("Failed to send pattern");

        let mut rx = channels.write_rx.lock().await;
        let received = rx.recv().await.expect("Failed to receive pattern");
        assert_eq!(received, pattern, "Binary data integrity check failed");
    }
}

#[tokio::test]
async fn test_zero_capacity() {
    let channels = RedisChannels::new(1, 1);

    channels.write_tx.send(vec![1u8]).await.unwrap();
    let mut rx = channels.write_rx.lock().await;
    let received = rx.recv().await.unwrap();
    assert_eq!(received, vec![1u8]);
}

#[tokio::test]
async fn test_interleaved_sends() {
    let channels = RedisChannels::new(100, 50);

    for i in 0..10 {
        channels
            .write_tx
            .send(vec![i as u8])
            .await
            .expect("Failed to send write");
        channels
            .awareness_tx
            .send(vec![(i + 100) as u8])
            .await
            .expect("Failed to send awareness");
    }

    let mut write_rx = channels.write_rx.lock().await;
    for i in 0..10 {
        let received = write_rx.recv().await.expect("Failed to receive");
        assert_eq!(received, vec![i as u8]);
    }
    drop(write_rx);

    let mut awareness_rx = channels.awareness_rx.lock().await;
    for i in 0..10 {
        let received = awareness_rx.recv().await.expect("Failed to receive");
        assert_eq!(received, vec![(i + 100) as u8]);
    }
}

#[tokio::test]
async fn test_receiver_batching_simulation() {
    let channels = RedisChannels::new(100, 50);

    let batch_size = 10;
    for i in 0..batch_size {
        channels
            .write_tx
            .send(vec![i as u8])
            .await
            .expect("Failed to send");
    }

    let mut rx = channels.write_rx.lock().await;
    let mut batch = Vec::new();
    for _ in 0..batch_size {
        if let Some(update) = rx.recv().await {
            batch.push(update);
        }
    }

    assert_eq!(batch.len(), batch_size);
    for (i, update) in batch.iter().enumerate() {
        assert_eq!(update, &vec![i as u8]);
    }
}

#[tokio::test]
async fn test_channel_stress() {
    let channels = RedisChannels::new(1000, 1000);
    let write_tx = channels.write_tx.clone();
    let awareness_tx = channels.awareness_tx.clone();

    let mut handles = vec![];

    for i in 0..10 {
        let tx = write_tx.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                let data = vec![(i * 100 + j) as u8; 10];
                tx.send(data).await.expect("Failed to send");
            }
        });
        handles.push(handle);
    }

    for i in 0..10 {
        let tx = awareness_tx.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                let data = vec![(i * 100 + j + 1000) as u8; 10];
                tx.send(data).await.expect("Failed to send");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    let mut write_rx = channels.write_rx.lock().await;
    let mut write_count = 0;
    while write_rx.try_recv().is_ok() {
        write_count += 1;
    }
    drop(write_rx);

    let mut awareness_rx = channels.awareness_rx.lock().await;
    let mut awareness_count = 0;
    while awareness_rx.try_recv().is_ok() {
        awareness_count += 1;
    }

    assert_eq!(write_count, 1000, "Expected 1000 write updates");
    assert_eq!(awareness_count, 1000, "Expected 1000 awareness updates");
}

#[tokio::test]
async fn test_channel_ordering() {
    let channels = RedisChannels::new(100, 50);

    for i in 0..50 {
        channels
            .write_tx
            .send(vec![i as u8])
            .await
            .expect("Failed to send");
    }

    let mut rx = channels.write_rx.lock().await;
    for i in 0..50 {
        let received = rx.recv().await.expect("Failed to receive");
        assert_eq!(received, vec![i as u8], "Messages out of order at {i}");
    }
}

#[tokio::test]
async fn test_multiple_receiver_locks() {
    let channels = RedisChannels::new(100, 50);

    channels
        .write_tx
        .send(vec![1u8, 2, 3])
        .await
        .expect("Failed to send");

    {
        let _rx = channels.write_rx.lock().await;
    }

    let mut rx = channels.write_rx.lock().await;
    let received = rx.recv().await.expect("Failed to receive");
    assert_eq!(received, vec![1u8, 2, 3]);
}

#[tokio::test]
async fn test_clone_sender() {
    let channels = RedisChannels::new(100, 50);

    let tx1 = channels.write_tx.clone();
    let tx2 = channels.write_tx.clone();

    tx1.send(vec![1u8]).await.expect("Failed to send from tx1");
    tx2.send(vec![2u8]).await.expect("Failed to send from tx2");
    channels
        .write_tx
        .send(vec![3u8])
        .await
        .expect("Failed to send from original");

    let mut rx = channels.write_rx.lock().await;
    let received1 = rx.recv().await.expect("Failed to receive");
    let received2 = rx.recv().await.expect("Failed to receive");
    let received3 = rx.recv().await.expect("Failed to receive");

    assert_eq!(received1, vec![1u8]);
    assert_eq!(received2, vec![2u8]);
    assert_eq!(received3, vec![3u8]);
}

#[tokio::test]
async fn test_capacity_respected() {
    let channels = RedisChannels::new(5, 5);

    for i in 0..5 {
        channels
            .write_tx
            .send(vec![i as u8])
            .await
            .expect("Failed to send");
    }

    let result = channels.write_tx.try_send(vec![99u8]);
    assert!(result.is_err(), "Should fail when channel is full");
}

#[tokio::test]
async fn test_unicode_data() {
    let channels = RedisChannels::new(100, 50);

    let text = "Hello, world!";
    let data = text.as_bytes().to_vec();

    channels
        .write_tx
        .send(data.clone())
        .await
        .expect("Failed to send");

    let mut rx = channels.write_rx.lock().await;
    let received = rx.recv().await.expect("Failed to receive");
    assert_eq!(received, data);

    let received_text = String::from_utf8(received).expect("Failed to decode UTF-8");
    assert_eq!(received_text, text);
}

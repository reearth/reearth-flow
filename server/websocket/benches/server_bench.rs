use criterion::{criterion_group, criterion_main, Criterion};
use futures_util::{SinkExt, StreamExt};
use tokio::runtime::Builder;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use yrs::{updates::decoder::Decode, Doc, ReadTxn, StateVector, Text, Transact, Update};

// Reduce timeout duration
const TIMEOUT_DURATION: Duration = Duration::from_millis(50);
// Shorter delays for connection operations
const CONNECTION_DELAY: Duration = Duration::from_millis(10);
const CLOSE_DELAY: Duration = Duration::from_millis(20);

/// Helper function to verify CRDT message reception
async fn verify_message_reception(
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    expected_doc: &Doc,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = tokio::time::timeout(TIMEOUT_DURATION, read.next())
        .await
        .map_err(|_| "Timeout waiting for response")?
        .ok_or("Connection closed unexpectedly")?
        .map_err(|e| format!("WebSocket error: {e}"))?;

    match msg {
        Message::Binary(data) => {
            // Try to apply the update to verify it's valid CRDT data
            let update =
                Update::decode_v1(&data).map_err(|e| format!("Invalid CRDT update: {e}"))?;
            let mut txn = expected_doc.transact_mut();
            txn.apply_update(update)
                .map_err(|e| format!("Failed to apply update: {e}"))?;
            Ok(())
        }
        Message::Close(_) => Err("Received unexpected close frame".into()),
        _ => Err("Received unexpected message type".into()),
    }
}

async fn connect_client(
    doc_id: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let url = format!("ws://127.0.0.1:8080/{doc_id}");
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    ws_stream
}

async fn close_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    // Send close frame and wait for acknowledgment
    if let Err(e) = ws_stream.close(None).await {
        tracing::warn!("Error sending close frame: {}", e);
    }

    // Wait for close frame acknowledgment with shorter timeout
    let mut received_close = false;
    while let Ok(Some(msg)) = tokio::time::timeout(CONNECTION_DELAY, ws_stream.next()).await {
        if let Ok(Message::Close(_)) = msg {
            received_close = true;
            break;
        }
    }

    if !received_close {
        tracing::warn!("Did not receive close frame acknowledgment");
    }

    // Shorter delay for connection cleanup
    sleep(CLOSE_DELAY).await;
}

fn bench_websocket_connection(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("websocket_connection", |b| {
        b.iter(|| {
            rt.block_on(async {
                let ws_stream = connect_client("bench-doc").await;
                // Properly close the connection
                close_connection(ws_stream).await;
                // Small delay between connections
                sleep(Duration::from_millis(10)).await;
            });
        })
    });
}

fn bench_doc_sync(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("doc_sync", |b| {
        b.iter(|| {
            rt.block_on(async {
                let ws_stream = connect_client("bench-doc-sync").await;
                let (mut write, mut read) = ws_stream.split();

                // Create a document and make some changes
                let doc = Doc::new();
                let text = doc.get_or_insert_text("test");
                {
                    let mut txn = doc.transact_mut();
                    text.push(&mut txn, "Hello, World!");
                }

                // Send update message
                let txn = doc.transact();
                let update = txn.encode_diff_v1(&StateVector::default());
                let msg = Message::Binary(update.into());
                write.send(msg).await.expect("Failed to send message");
                write.flush().await.expect("Failed to flush");

                // Verify message reception
                if let Err(e) = verify_message_reception(&mut read, &doc).await {
                    tracing::warn!("Failed to verify message reception: {}", e);
                }

                // Properly close the connection
                write
                    .send(Message::Close(None))
                    .await
                    .expect("Failed to send close frame");
                write.flush().await.expect("Failed to flush close frame");
                sleep(Duration::from_millis(50)).await;
            });
        })
    });
}

fn bench_text_operations(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("text_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let ws_stream = connect_client("bench-doc-text").await;
                let (mut write, mut read) = ws_stream.split();

                // Create a document and perform multiple text operations
                let doc = Doc::new();
                let text = doc.get_or_insert_text("test");
                {
                    let mut txn = doc.transact_mut();
                    // Insert operations
                    text.push(&mut txn, "First line\n");
                    text.push(&mut txn, "Second line\n");
                    text.push(&mut txn, "Third line\n");
                    text.push(&mut txn, "Fourth line\n");
                    text.push(&mut txn, "Fifth line\n");
                }

                // Send update message
                let txn = doc.transact();
                let update = txn.encode_diff_v1(&StateVector::default());
                let msg = Message::Binary(update.into());
                write.send(msg).await.expect("Failed to send message");
                write.flush().await.expect("Failed to flush");

                // Verify message reception
                if let Err(e) = verify_message_reception(&mut read, &doc).await {
                    tracing::warn!("Failed to verify message reception: {}", e);
                }

                // Properly close the connection
                write
                    .send(Message::Close(None))
                    .await
                    .expect("Failed to send close frame");
                write.flush().await.expect("Failed to flush close frame");
                sleep(Duration::from_millis(50)).await;
            });
        })
    });
}

fn bench_concurrent_clients(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    const NUM_CLIENTS: usize = 5;

    let mut group = c.benchmark_group("concurrent_clients");
    group.sample_size(30);
    group.bench_function("concurrent_clients", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::with_capacity(NUM_CLIENTS);

                // Spawn multiple clients with delay between each
                for i in 0..NUM_CLIENTS {
                    let handle = tokio::spawn(async move {
                        let ws_stream = connect_client("bench-doc-concurrent").await;
                        let (mut write, mut read) = ws_stream.split();

                        // Create and send update
                        let doc = Doc::new();
                        let text = doc.get_or_insert_text("test");
                        {
                            let mut txn = doc.transact_mut();
                            text.push(&mut txn, &format!("Change from client {i}\n"));
                        }
                        let txn = doc.transact();
                        let update = txn.encode_diff_v1(&StateVector::default());

                        write
                            .send(Message::Binary(update.into()))
                            .await
                            .expect("Failed to send message");
                        write.flush().await.expect("Failed to flush");

                        // Verify message reception
                        if let Err(e) = verify_message_reception(&mut read, &doc).await {
                            tracing::warn!("Failed to verify message reception: {}", e);
                        }

                        // Properly close the connection
                        write
                            .send(Message::Close(None))
                            .await
                            .expect("Failed to send close frame");
                        write.flush().await.expect("Failed to flush close frame");
                        sleep(Duration::from_millis(50)).await;
                    });
                    handles.push(handle);
                    sleep(Duration::from_millis(10)).await; // Delay between client spawns
                }

                // Wait for all clients
                for handle in handles {
                    handle.await.expect("Task failed");
                }

                // Delay between iterations
                sleep(Duration::from_millis(100)).await;
            });
        })
    });
}

fn bench_broadcast(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    const NUM_RECEIVERS: usize = 3;

    let mut group = c.benchmark_group("broadcast");
    group.sample_size(10);
    group.bench_function("broadcast", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create multiple receiver connections
                let mut receivers = Vec::with_capacity(NUM_RECEIVERS);
                let mut receiver_reads = Vec::with_capacity(NUM_RECEIVERS);
                for _ in 0..NUM_RECEIVERS {
                    let ws_stream = connect_client("bench-doc-broadcast").await;
                    let (write, read) = ws_stream.split();
                    receivers.push(write);
                    receiver_reads.push(read);
                }

                // Create sender connection
                let ws_stream = connect_client("bench-doc-broadcast").await;
                let (mut sender_write, mut sender_read) = ws_stream.split();

                // Create and send the message
                let doc = Doc::new();
                let text = doc.get_or_insert_text("test");
                {
                    let mut txn = doc.transact_mut();
                    for i in 0..20 {
                        text.push(&mut txn, &format!("Broadcast message line {i}\n"));
                    }
                }

                let txn = doc.transact();
                let update = txn.encode_diff_v1(&StateVector::default());
                let msg = Message::Binary(update.into());

                sender_write
                    .send(msg)
                    .await
                    .expect("Failed to send message");
                sender_write.flush().await.expect("Failed to flush");

                // Verify message reception on sender
                if let Err(e) = verify_message_reception(&mut sender_read, &doc).await {
                    tracing::warn!("Failed to verify sender message reception: {}", e);
                }

                // Verify message reception on all receivers
                for (i, read) in receiver_reads.iter_mut().enumerate() {
                    if let Err(e) = verify_message_reception(read, &doc).await {
                        tracing::warn!("Failed to verify receiver {} message reception: {}", i, e);
                    }
                }

                // Close all connections
                sender_write
                    .send(Message::Close(None))
                    .await
                    .expect("Failed to send close frame");
                for mut write in receivers {
                    write
                        .send(Message::Close(None))
                        .await
                        .expect("Failed to send close frame");
                }
            });
        })
    });
    group.finish();
}

fn bench_large_update(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("large_update", |b| {
        b.iter(|| {
            rt.block_on(async {
                let ws_stream = connect_client("bench-doc-large").await;
                let (mut write, mut read) = ws_stream.split();

                // Create a large document update
                let doc = Doc::new();
                let text = doc.get_or_insert_text("test");
                {
                    let mut txn = doc.transact_mut();
                    for i in 0..100 {
                        text.push(&mut txn, &format!("Large update line {i}\n"));
                    }
                }

                let txn = doc.transact();
                let update = txn.encode_diff_v1(&StateVector::default());
                let msg = Message::Binary(update.into());
                write.send(msg).await.expect("Failed to send message");
                write.flush().await.expect("Failed to flush");

                // Verify message reception
                if let Err(e) = verify_message_reception(&mut read, &doc).await {
                    tracing::warn!("Failed to verify message reception: {}", e);
                }

                // Close connection
                write
                    .send(Message::Close(None))
                    .await
                    .expect("Failed to send close frame");
            });
        })
    });
}

fn bench_state_vector_sync(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("state_vector_sync", |b| {
        let doc = Doc::new();
        let text = doc.get_or_insert_text("test");
        {
            let mut txn = doc.transact_mut();
            text.push(&mut txn, "Initial content\n");
        }
        let state_vector = StateVector::default();

        b.iter(|| {
            rt.block_on(async {
                let ws_stream = connect_client("bench-doc-sv").await;
                let (mut write, mut read) = ws_stream.split();

                // Send update with current state
                let txn = doc.transact();
                let update = txn.encode_diff_v1(&state_vector);
                let msg = Message::Binary(update.into());
                write.send(msg).await.expect("Failed to send message");
                write.flush().await.expect("Failed to flush");

                // Verify message reception
                if let Err(e) = verify_message_reception(&mut read, &doc).await {
                    tracing::warn!("Failed to verify message reception: {}", e);
                }

                // Close connection
                write
                    .send(Message::Close(None))
                    .await
                    .expect("Failed to send close frame");
            });
        })
    });
}

fn bench_concurrent_clients_variable(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    let mut group = c.benchmark_group("concurrent_clients_variable");
    group.sample_size(10); // Reduce sample size
    for num_clients in [2, 4, 8].iter() {
        // Remove 16 clients test to reduce time
        group.bench_with_input(
            format!("clients_{num_clients}"),
            num_clients,
            |b, &num_clients| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = Vec::with_capacity(num_clients);

                        for i in 0..num_clients {
                            let handle = tokio::spawn(async move {
                                let mut ws_stream =
                                    connect_client("bench-doc-concurrent-var").await;

                                let doc = Doc::new();
                                let text = doc.get_or_insert_text("test");
                                {
                                    let mut txn = doc.transact_mut();
                                    text.push(&mut txn, &format!("Change from client {i}\n"));
                                }
                                let txn = doc.transact();
                                let update = txn.encode_diff_v1(&StateVector::default());

                                ws_stream
                                    .send(Message::Binary(update.into()))
                                    .await
                                    .expect("Failed to send message");
                                sleep(CONNECTION_DELAY).await;
                                ws_stream.close(None).await.expect("Failed to close");
                            });
                            handles.push(handle);
                            sleep(CONNECTION_DELAY).await;
                        }

                        for handle in handles {
                            handle.await.expect("Task failed");
                        }

                        sleep(Duration::from_millis(100)).await;
                    });
                })
            },
        );
    }
    group.finish();
}

fn bench_long_connection(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    c.bench_function("long_connection", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut ws_stream = connect_client("bench-doc-long").await;

                // Reduce number of updates and delay between them
                for i in 0..5 {
                    let doc = Doc::new();
                    let text = doc.get_or_insert_text("test");
                    {
                        let mut txn = doc.transact_mut();
                        text.push(&mut txn, &format!("Update {i} in long connection\n"));
                    }
                    let txn = doc.transact();
                    let update = txn.encode_diff_v1(&StateVector::default());

                    ws_stream
                        .send(Message::Binary(update.into()))
                        .await
                        .expect("Failed to send message");
                    sleep(CONNECTION_DELAY).await;
                }

                close_connection(ws_stream).await;
            });
        })
    });
}

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(1))
        .warm_up_time(Duration::from_millis(500))
        .output_directory(std::path::Path::new("target/criterion"))
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = bench_websocket_connection,
        bench_doc_sync,
        bench_text_operations,
        bench_concurrent_clients,
        bench_broadcast,
        bench_large_update,
        bench_state_vector_sync,
        bench_concurrent_clients_variable,
        bench_long_connection
}
criterion_main!(benches);

use axum::{routing::get, Router};
use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use std::sync::Arc;
use thrift::server::TProcessor;
use tokio::net::TcpListener;
use tracing::info;

use crate::{doc::DocumentHandler, storage::gcs::GcsStore, ws::ws_handler, AppState};

pub async fn ensure_bucket(client: &Client, bucket_name: &str) -> Result<(), anyhow::Error> {
    let bucket = BucketCreationConfig {
        location: "US".to_string(),
        ..Default::default()
    };
    let request = InsertBucketRequest {
        name: bucket_name.to_string(),
        bucket,
        ..Default::default()
    };

    match client.insert_bucket(&request).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("already exists") => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn start_server(state: Arc<AppState>, port: &str) -> Result<(), anyhow::Error> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Starting WebSocket server on {}", addr);

    let app = Router::new()
        .route("/{doc_id}", get(ws_handler))
        .with_state(state.clone());

    // Start the WebSocket server
    let axum_server = axum::serve(listener, app);

    // Start the Thrift server
    let thrift_port = port.parse::<u16>().unwrap_or(8080) + 1;
    let thrift_addr = format!("0.0.0.0:{}", thrift_port);

    info!("Starting Thrift server on {}", thrift_addr);

    // Get the storage from the pool
    let store = state.pool.get_store();

    // Spawn the Thrift server in a separate task
    tokio::spawn(async move {
        if let Err(e) = start_thrift_server(store, &thrift_addr).await {
            tracing::error!("Thrift server error: {}", e);
        }
    });

    // Wait for the WebSocket server to complete
    axum_server.await?;

    Ok(())
}

async fn start_thrift_server(store: Arc<GcsStore>, addr: &str) -> Result<(), anyhow::Error> {
    use std::net::TcpListener;
    use thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol};
    use thrift::transport::{TFramedReadTransport, TFramedWriteTransport};

    // Create a TCP listener
    let listener = TcpListener::bind(addr)?;
    info!("Thrift server running on {}", addr);

    // Create the document handler
    let handler = DocumentHandler::new(store);
    let processor = handler.processor();

    // Accept connections and process them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Create the input and output protocols
                let i_trans = TFramedReadTransport::new(stream.try_clone()?);
                let o_trans = TFramedWriteTransport::new(stream);

                let mut i_prot = TBinaryInputProtocol::new(i_trans, true);
                let mut o_prot = TBinaryOutputProtocol::new(o_trans, true);

                // Process the request
                if let Err(e) = processor.process(&mut i_prot, &mut o_prot) {
                    tracing::error!("Error processing Thrift request: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}

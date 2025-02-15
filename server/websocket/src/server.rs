use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

use crate::{
    grpc::{document::document_service_server::DocumentServiceServer, DocumentServiceImpl},
    AppState, BUCKET_NAME, PORT,
};

pub async fn ensure_bucket(client: &Client) -> Result<(), anyhow::Error> {
    let bucket = BucketCreationConfig {
        location: "US".to_string(),
        ..Default::default()
    };
    let request = InsertBucketRequest {
        name: BUCKET_NAME.to_string(),
        bucket,
        ..Default::default()
    };

    match client.insert_bucket(&request).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("already exists") => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn start_server(state: Arc<AppState>) -> Result<(), anyhow::Error> {
    let addr = format!("0.0.0.0:{}", PORT).parse()?;
    let document_service = DocumentServiceImpl::new(state);

    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(DocumentServiceServer::new(document_service))
        .serve(addr)
        .await?;

    Ok(())
}

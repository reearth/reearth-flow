use anyhow::{Context, Result};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GcsClient {
    client: Client,
    bucket: String,
}

impl GcsClient {
    pub async fn new(bucket: String) -> Result<Self> {
        let config = ClientConfig::default()
            .with_auth()
            .await
            .context("Failed to create ClientConfig with authentication")?;
        let client = Client::new(config);
        Ok(GcsClient { client, bucket })
    }

    pub async fn upload<T: Serialize>(&self, path: String, data: &T) -> Result<()> {
        let upload_type = UploadType::Simple(Media::new(path.clone()));
        let bytes = serde_json::to_string(data).context("Failed to serialize data")?;
        self.client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                bytes,
                &upload_type,
            )
            .await
            .context(format!("Failed to upload object to path: {}", path))?;
        Ok(())
    }

    pub async fn download<T: for<'de> Deserialize<'de>>(&self, path: String) -> Result<T> {
        let bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: path.clone(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .context(format!("Failed to download object from path: {}", path))?;
        let src = String::from_utf8(bytes).context("Failed to convert bytes to string")?;
        let data = serde_json::from_str(&src).context("Failed to deserialize data")?;
        Ok(data)
    }
}

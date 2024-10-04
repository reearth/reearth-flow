use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GcsError {
    #[error(transparent)]
    Auth(#[from] google_cloud_storage::client::google_cloud_auth::error::Error),
    #[error(transparent)]
    Http(#[from] google_cloud_storage::http::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Clone)]
pub struct GcsClient {
    client: Client,
    bucket: String,
}

impl GcsClient {
    pub async fn new(bucket: String) -> Result<Self, GcsError> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(GcsClient { client, bucket })
    }

    pub async fn upload<T: Serialize>(&self, path: String, data: &T) -> Result<(), GcsError> {
        let upload_type = UploadType::Simple(Media::new(path));
        let bytes = serde_json::to_string(data)?;
        let _uploaded = self
            .client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                bytes,
                &upload_type,
            )
            .await?;
        Ok(())
    }

    pub async fn download<T: for<'de> Deserialize<'de>>(
        &self,
        path: String,
    ) -> Result<T, GcsError> {
        let bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: path,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;
        let src = String::from_utf8(bytes)?;
        let data = serde_json::from_str(&src)?;
        Ok(data)
    }

    pub async fn delete(&self, path: String) -> Result<(), GcsError> {
        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path,
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}

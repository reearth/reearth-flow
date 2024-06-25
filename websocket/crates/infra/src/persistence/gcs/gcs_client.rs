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
    pub async fn new(bucket: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(GcsClient { client, bucket })
    }

    pub async fn upload<T: Serialize>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
    ) -> Result<T, Box<dyn std::error::Error>> {
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
}

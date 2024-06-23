use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

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

    pub async fn upload(
        &mut self,
        path: String,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let upload_type = UploadType::Simple(Media::new(path));
        let _uploaded = self
            .client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                data,
                &upload_type,
            )
            .await?;
        Ok(())
    }

    pub async fn download(&self, path: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = self
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
        Ok(data)
    }
}

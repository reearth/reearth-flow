use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use std::fs;
use std::io::Read;
use std::path::Path;
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
    pub async fn new(bucket: &str) -> Result<Self, GcsError> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(GcsClient {
            client,
            bucket: bucket.to_string(),
        })
    }
    pub async fn upload_directory(
        self,
        local_directory: &str,
        gcs_directory: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let local_path = Path::new(local_directory);
        let dir_entries = fs::read_dir(local_path)?;

        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let new_local_dir = path.to_str().unwrap();
                let new_gcs_dir = format!(
                    "{}/{}",
                    gcs_directory,
                    path.file_name().unwrap().to_str().unwrap()
                );
                let future = self.clone().upload_directory(new_local_dir, &new_gcs_dir);
                Box::pin(future).await?;
            } else {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let gcs_path = format!("{}/{}", gcs_directory, file_name);
                let mut file = fs::File::open(&path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                let upload_type = UploadType::Simple(Media::new(gcs_path.clone()));
                let upload_request = UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                };

                self.client
                    .upload_object(&upload_request, buffer, &upload_type)
                    .await?;

                println!("Uploaded: {}", gcs_path);
            }
        }

        Ok(())
    }
}

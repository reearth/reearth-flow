use crate::domain::repository::kv::KVEntry;
use crate::domain::repository::kv::KVStore;
use anyhow::Result;
use futures::future::join_all;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::delete::DeleteObjectRequest,
    http::objects::download::Range,
    http::objects::get::GetObjectRequest,
    http::objects::list::ListObjectsRequest,
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
    http::objects::Object,
};
use hex;
use time::OffsetDateTime;
use tracing::debug;

use crate::domain::entity::gcs::GcsConfig;

const BATCH_SIZE: usize = 50;

fn find_common_prefix(a: &str, b: &str) -> String {
    let min_len = std::cmp::min(a.len(), b.len());
    let mut common_len = 0;

    for i in 0..min_len {
        if a.chars().nth(i) == b.chars().nth(i) {
            common_len += 1;
        } else {
            break;
        }
    }

    if common_len == 0 {
        if !a.is_empty() {
            a.chars().take(1).collect()
        } else {
            String::new()
        }
    } else {
        a.chars().take(common_len).collect()
    }
}

pub struct GcsStore {
    #[allow(dead_code)]
    pub client: Client,
    pub bucket: String,
}

impl std::fmt::Debug for GcsStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GcsStore")
            .field("bucket", &self.bucket)
            .finish_non_exhaustive()
    }
}

impl GcsStore {
    pub async fn new(bucket: String) -> Result<Self, google_cloud_storage::http::Error> {
        let config = ClientConfig::default();
        let client = Client::new(config);
        Ok(Self { client, bucket })
    }

    pub async fn new_with_config(config: GcsConfig) -> Result<Self> {
        let client_config = if let Some(endpoint) = &config.endpoint {
            let mut client_config = ClientConfig::default().anonymous();
            client_config.storage_endpoint = endpoint.clone();
            client_config
        } else {
            ClientConfig::default().with_auth().await?
        };

        let client = Client::new(client_config);

        Ok(Self {
            client,
            bucket: config.bucket_name,
        })
    }

    pub async fn with_client(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

#[async_trait::async_trait]
impl KVStore for GcsStore {
    type Error = google_cloud_storage::http::Error;
    type Cursor = GcsRange;
    type Entry = GcsEntry;
    type Return = Vec<u8>;

    async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error> {
        let key_hex = hex::encode(key);
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key_hex,
            ..Default::default()
        };

        match self
            .client
            .download_object(&request, &Range::default())
            .await
        {
            Ok(data) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }

    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let key_hex = hex::encode(key);
        let upload_type = UploadType::Simple(Media::new(key_hex.clone()));
        self.client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                value.to_vec(),
                &upload_type,
            )
            .await?;
        Ok(())
    }

    async fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        let key_hex = hex::encode(key);
        let request = DeleteObjectRequest {
            bucket: self.bucket.clone(),
            object: key_hex,
            ..Default::default()
        };

        self.client.delete_object(&request).await?;
        Ok(())
    }

    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error> {
        let from_hex = hex::encode(from);
        let to_hex = hex::encode(to);

        let common_prefix = find_common_prefix(&from_hex, &to_hex);

        let mut all_objects = Vec::new();
        let mut page_token = None;

        loop {
            let request = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: Some(common_prefix.clone()),
                page_token: page_token.clone(),
                ..Default::default()
            };

            let response = self.client.list_objects(&request).await?;
            let items = response.items.unwrap_or_default();

            let filtered_items = items.into_iter().filter(|obj| {
                obj.name.as_str() >= from_hex.as_str() && obj.name.as_str() <= to_hex.as_str()
            });

            all_objects.extend(filtered_items);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        let delete_futures = all_objects.into_iter().map(|obj| {
            let bucket = self.bucket.clone();
            async move {
                let delete_request = DeleteObjectRequest {
                    bucket,
                    object: obj.name.clone(),
                    ..Default::default()
                };
                self.client.delete_object(&delete_request).await
            }
        });

        let _results = join_all(delete_futures).await;

        Ok(())
    }

    async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error> {
        let from_hex: String = hex::encode(from);
        let to_hex: String = hex::encode(to);

        let common_prefix: String = find_common_prefix(&from_hex, &to_hex);

        let mut all_objects: Vec<Object> = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let request: ListObjectsRequest = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: Some(common_prefix.clone()),
                page_token: page_token.clone(),
                ..Default::default()
            };

            let response: google_cloud_storage::http::objects::list::ListObjectsResponse =
                self.client.list_objects(&request).await?;
            let items: Vec<Object> = response.items.unwrap_or_default();

            let filtered_items = items.into_iter().filter(|obj| {
                obj.name.as_str() >= from_hex.as_str() && obj.name.as_str() <= to_hex.as_str()
            });

            all_objects.extend(filtered_items);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        all_objects.sort_by(|a: &Object, b: &Object| a.name.cmp(&b.name));

        let mut all_values: Vec<Option<Vec<u8>>> = Vec::with_capacity(all_objects.len());

        for chunk in all_objects.chunks(BATCH_SIZE) {
            let chunk_futures = chunk.iter().map(|obj| {
                let bucket: String = self.bucket.clone();
                let object: String = obj.name.clone();
                async move {
                    let request: GetObjectRequest = GetObjectRequest {
                        bucket,
                        object,
                        ..Default::default()
                    };
                    (
                        obj.name.clone(),
                        self.client
                            .download_object(&request, &Range::default())
                            .await,
                    )
                }
            });

            let batch_results: Vec<(
                String,
                std::result::Result<Vec<u8>, google_cloud_storage::http::Error>,
            )> = join_all(chunk_futures).await;

            let mut result_map: std::collections::HashMap<String, Option<Vec<u8>>> =
                std::collections::HashMap::new();
            for (name, result) in batch_results {
                if let Ok(data) = result {
                    result_map.insert(name, Some(data));
                } else {
                    result_map.insert(name, None);
                }
            }

            for obj in chunk {
                all_values.push(result_map.remove(&obj.name).unwrap_or(None));
            }
        }

        Ok(GcsRange {
            objects: all_objects,
            values: all_values,
            current: 0,
        })
    }

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
        let key_hex = hex::encode(key);

        let prefix = if key_hex.len() > 2 {
            key_hex.chars().take(2).collect::<String>()
        } else {
            key_hex.clone()
        };

        let mut all_objects = Vec::new();
        let mut page_token = None;

        loop {
            let request = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: Some(prefix.clone()),
                page_token: page_token.clone(),
                ..Default::default()
            };

            let response = self.client.list_objects(&request).await?;
            let items = response.items.unwrap_or_default();

            let filtered_items = items
                .into_iter()
                .filter(|obj| obj.name.as_str() < key_hex.as_str());

            all_objects.extend(filtered_items);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        all_objects.sort_by(|a, b| a.name.cmp(&b.name));

        if let Some(obj) = all_objects.pop() {
            let get_request = GetObjectRequest {
                bucket: self.bucket.clone(),
                object: obj.name.clone(),
                ..Default::default()
            };

            let value = self
                .client
                .download_object(&get_request, &Range::default())
                .await?;

            Ok(Some(GcsEntry {
                key: hex::decode(&obj.name).unwrap_or_default(),
                value,
            }))
        } else {
            debug!("No objects found for peek_back with key: {:?}", key);
            Ok(None)
        }
    }

    async fn get_metadata(
        &self,
        from: &[u8],
        to: &[u8],
    ) -> Result<Option<Vec<(u32, OffsetDateTime)>>, Self::Error> {
        let from_hex = hex::encode(from);
        let to_hex = hex::encode(to);

        let common_prefix: String = find_common_prefix(&from_hex, &to_hex);

        let mut all_objects = Vec::new();
        let mut page_token = None;

        loop {
            let request = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: Some(common_prefix.clone()),
                page_token: page_token.clone(),
                ..Default::default()
            };

            let response = self.client.list_objects(&request).await?;
            let items = response.items.unwrap_or_default();
            all_objects.extend(items);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        let mut metadata: Vec<(u32, OffsetDateTime)> = Vec::new();
        for obj in all_objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into().unwrap_or_default();
                    let clock = u32::from_be_bytes(clock_bytes);
                    let timestamp = obj.updated.unwrap_or_else(OffsetDateTime::now_utc);

                    metadata.push((clock, timestamp));
                }
            }
        }

        metadata.sort_by_key(|(clock, _)| std::cmp::Reverse(*clock));

        Ok(Some(metadata))
    }
}

pub struct GcsRange {
    objects: Vec<Object>,
    values: Vec<Option<Vec<u8>>>,
    current: usize,
}

impl Iterator for GcsRange {
    type Item = GcsEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.objects.len() {
            return None;
        }
        let obj = &self.objects[self.current];
        let value = self.values[self.current].clone()?;
        self.current += 1;

        Some(GcsEntry {
            key: obj.name.clone().into_bytes(),
            value,
        })
    }
}

pub struct GcsEntry {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl KVEntry for GcsEntry {
    fn key(&self) -> &[u8] {
        &self.key
    }
    fn value(&self) -> &[u8] {
        &self.value
    }
}

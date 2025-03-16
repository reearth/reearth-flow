//! **yrs-gcs** is a persistence layer allowing to store [Yrs](https://docs.rs/yrs/latest/yrs/index.html)
//! documents and providing convenient utility functions to work with them, using Google Cloud Storage for
//! persistent backend.

pub use super::kv as store;
use super::kv::keys::{KEYSPACE_DOC, SUB_UPDATE, V1};
use super::kv::{get_oid, DocOps, KVEntry, KVStore};
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
use serde::Deserialize;
use time::OffsetDateTime;
use tracing::debug;
use yrs::{updates::decoder::Decode, Doc, Transact, Update};

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

/// Type wrapper around GCS Client struct. Used to extend GCS with [DocOps]
/// methods used for convenience when working with Yrs documents.
pub struct GcsStore {
    #[allow(dead_code)]
    pub client: Client,
    pub bucket: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GcsConfig {
    pub bucket_name: String,
    pub endpoint: Option<String>,
}

impl std::fmt::Debug for GcsStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GcsStore")
            .field("bucket", &self.bucket)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct UpdateInfo {
    pub update: Update,
    pub clock: u32,
    pub timestamp: OffsetDateTime,
}

impl GcsStore {
    pub async fn new(bucket: String) -> Result<Self, google_cloud_storage::http::Error> {
        let config = ClientConfig::default();
        let client = Client::new(config);
        Ok(Self { client, bucket })
    }

    pub async fn new_with_config(config: GcsConfig) -> Result<Self, anyhow::Error> {
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

    pub async fn get_updates(&self, doc_id: &str) -> Result<Vec<UpdateInfo>, anyhow::Error> {
        // Get the OID for this document
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(Vec::new()),
        };

        // Create prefix that matches all updates for this OID
        let prefix_bytes = [V1, KEYSPACE_DOC]
            .iter()
            .chain(&oid.to_be_bytes())
            .chain(&[SUB_UPDATE])
            .copied()
            .collect::<Vec<_>>();
        let prefix_str = hex::encode(&prefix_bytes);

        // List objects with the specified prefix
        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            prefix: Some(prefix_str),
            ..Default::default()
        };

        let objects = self
            .client
            .list_objects(&request)
            .await?
            .items
            .unwrap_or_default();

        let mut updates = Vec::new();
        for obj in objects {
            let request = GetObjectRequest {
                bucket: self.bucket.clone(),
                object: obj.name.clone(),
                ..Default::default()
            };

            if let Ok(data) = self
                .client
                .download_object(&request, &Range::default())
                .await
            {
                if let Ok(update) = Update::decode_v1(&data) {
                    // Extract clock from object name
                    if let Ok(key_bytes) = hex::decode(&obj.name) {
                        if key_bytes.len() >= 12 {
                            let clock_bytes: [u8; 4] = key_bytes[7..11].try_into().unwrap();
                            let clock = u32::from_be_bytes(clock_bytes);

                            // Get timestamp from object metadata or use current time
                            let timestamp = obj.updated.unwrap_or_else(OffsetDateTime::now_utc);

                            updates.push(UpdateInfo {
                                clock,
                                timestamp,
                                update,
                            });
                        }
                    }
                }
            } else {
                tracing::error!("Failed to download update from {}", obj.name);
            }
        }

        updates.sort_unstable_by_key(|info| info.clock);

        Ok(updates)
    }

    pub async fn rollback_to(&self, doc_id: &str, target_clock: u32) -> anyhow::Result<Doc> {
        // Get the OID for this document
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => anyhow::bail!("Document not found"),
        };

        let prefix_bytes = [V1, KEYSPACE_DOC]
            .iter()
            .chain(&oid.to_be_bytes())
            .chain(&[SUB_UPDATE])
            .copied()
            .collect::<Vec<_>>();
        let prefix_str = hex::encode(&prefix_bytes);

        let mut all_objects = Vec::new();
        let mut page_token = None;

        loop {
            let request = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: Some(prefix_str.clone()),
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

        debug!(
            "Found {} objects for rollback for doc: {}",
            all_objects.len(),
            doc_id
        );

        // Create a new document
        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        // Download and apply updates up to target_clock
        for obj in all_objects {
            // Extract clock from object name
            let key_bytes = hex::decode(&obj.name)
                .map_err(|e| anyhow::anyhow!("Failed to decode hex: {}", e))?;

            if key_bytes.len() < 12 {
                continue;
            }

            let clock_bytes: [u8; 4] = key_bytes[7..11].try_into().unwrap();
            let clock = u32::from_be_bytes(clock_bytes);

            if clock > target_clock {
                continue;
            }

            // Download and apply update
            let request = GetObjectRequest {
                bucket: self.bucket.clone(),
                object: obj.name,
                ..Default::default()
            };

            if let Ok(data) = self
                .client
                .download_object(&request, &Range::default())
                .await
            {
                if let Ok(update) = Update::decode_v1(&data) {
                    debug!("Applying update with clock: {} for doc: {}", clock, doc_id);
                    let _ = txn.apply_update(update);
                }
            }
        }

        drop(txn);
        Ok(doc)
    }

    pub async fn get_latest_update_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Option<(u32, OffsetDateTime)>, anyhow::Error> {
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(None),
        };

        let prefix_bytes = [V1, KEYSPACE_DOC]
            .iter()
            .chain(&oid.to_be_bytes())
            .chain(&[SUB_UPDATE])
            .copied()
            .collect::<Vec<_>>();
        let prefix_str = hex::encode(&prefix_bytes);

        let request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            prefix: Some(prefix_str),
            ..Default::default()
        };

        let objects = self
            .client
            .list_objects(&request)
            .await?
            .items
            .unwrap_or_default();

        if objects.is_empty() {
            return Ok(None);
        }

        let mut latest_clock = 0u32;
        let mut latest_timestamp = OffsetDateTime::now_utc();

        for obj in objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into().unwrap();
                    let clock = u32::from_be_bytes(clock_bytes);

                    if clock > latest_clock {
                        latest_clock = clock;
                        latest_timestamp = obj.updated.unwrap_or_else(OffsetDateTime::now_utc);
                    }
                }
            }
        }

        Ok(Some((latest_clock, latest_timestamp)))
    }
}

impl DocOps<'_> for GcsStore {}

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

        debug!("Getting from GCS storage - key: {:?}", key);

        match self
            .client
            .download_object(&request, &Range::default())
            .await
        {
            Ok(data) => {
                debug!("Got from GCS storage - key: {:?}", key);
                debug!("Value length: {} bytes", data.len());
                debug!("Value: {:?}", data);
                Ok(Some(data))
            }
            // 404 is returned when the object is not found
            Err(_) => Ok(None),
        }
    }

    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let key_hex = hex::encode(key);
        debug!("Writing to GCS storage - key: {:?}, hex: {}", key, key_hex);
        debug!("Value length: {} bytes", value.len());

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

        match self.client.delete_object(&request).await {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error> {
        let from_hex = hex::encode(from);
        let to_hex = hex::encode(to);

        let common_prefix = find_common_prefix(&from_hex, &to_hex);
        debug!(
            "Remove range from: {:?} to: {:?} with prefix: {}",
            from_hex, to_hex, common_prefix
        );

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

        debug!("Removing {} objects in range", all_objects.len());

        let delete_futures = all_objects.into_iter().map(|obj| {
            let bucket = self.bucket.clone();
            async move {
                let delete_request = DeleteObjectRequest {
                    bucket,
                    object: obj.name.clone(),
                    ..Default::default()
                };
                debug!("Deleting object: {}", obj.name);
                self.client.delete_object(&delete_request).await
            }
        });

        let _results = join_all(delete_futures).await;

        Ok(())
    }

    async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error> {
        let from_hex = hex::encode(from);
        let to_hex = hex::encode(to);

        let common_prefix = find_common_prefix(&from_hex, &to_hex);
        debug!(
            "Range query from: {:?} to: {:?} with prefix: {}",
            from_hex, to_hex, common_prefix
        );

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
                debug!("Checking object: {:?}", obj.name.as_str());
                obj.name.as_str() >= from_hex.as_str() && obj.name.as_str() <= to_hex.as_str()
            });

            all_objects.extend(filtered_items);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        all_objects.sort_by(|a, b| a.name.cmp(&b.name));

        let results = join_all(all_objects.iter().map(|obj| {
            let bucket = self.bucket.clone();
            let object = obj.name.clone();
            async move {
                let request = GetObjectRequest {
                    bucket,
                    object,
                    ..Default::default()
                };
                self.client
                    .download_object(&request, &Range::default())
                    .await
            }
        }))
        .await;

        debug!(
            "Got range from GCS storage - from: {:?}, to: {:?}, count: {}",
            from,
            to,
            all_objects.len()
        );

        let values = results.into_iter().map(|r| r.ok()).collect();

        Ok(GcsRange {
            objects: all_objects,
            values,
            current: 0,
        })
    }

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
        let key_hex = hex::encode(key);
        debug!(
            "Peeking back in GCS storage - key: {:?}, hex: {}",
            key, key_hex
        );

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

            debug!("Found peek_back object: {}", obj.name);

            Ok(Some(GcsEntry {
                key: hex::decode(&obj.name).unwrap_or_default(),
                value,
            }))
        } else {
            debug!("No objects found for peek_back with key: {:?}", key);
            Ok(None)
        }
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

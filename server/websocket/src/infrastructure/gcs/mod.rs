use crate::application::usecases::kv::{get_oid, get_or_create_oid, DocOps};
use crate::domain::repositories::kv::KVEntry;
use crate::domain::repositories::kv::KVStore;
use crate::domain::value_objects::keys::{
    key_doc, key_state_vector, key_update, KEYSPACE_DOC, KEYSPACE_OID, SUB_DOC, SUB_STATE_VEC,
    SUB_UPDATE, V1,
};

use crate::infrastructure::redis::RedisStore;
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
use serde::Deserialize;
use time::OffsetDateTime;
use tracing::{debug, error, info};
use yrs::{
    updates::decoder::Decode, updates::encoder::Encode, Doc, ReadTxn, StateVector, Transact, Update,
};

use crate::shared::utils::first_zero_bit;

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

    pub async fn push_update(
        &self,
        doc_id: &str,
        update: &bytes::Bytes,
        redis: &RedisStore,
    ) -> Result<u32> {
        let oid = get_oid(self, doc_id.as_bytes()).await?;
        let oid = match oid {
            Some(oid) => oid,
            None => get_or_create_oid(self, doc_id.as_bytes(), redis).await?,
        };

        let last_clock = {
            let end = key_update(oid, u32::MAX)?;
            if let Some(e) = self.peek_back(&end).await? {
                let last_key = e.key();
                let len = last_key.len();
                let last_clock = &last_key[(len - 5)..(len - 1)]; // update key scheme: 01{name:n}1{clock:4}0
                u32::from_be_bytes(last_clock.try_into().unwrap())
            } else {
                0
            }
        };
        let clock = last_clock + 1;
        let update_key = key_update(oid, clock)?;
        self.upsert(&update_key, update).await?;

        if clock % 10 == 0 {
            let doc = Doc::new();
            let mut txn = doc.transact_mut();

            let doc_key = key_doc(oid)?;
            if let Some(doc_state) = self.get(&doc_key).await? {
                if let Ok(base_update) = Update::decode_v1(doc_state.as_ref()) {
                    let _ = txn.apply_update(base_update);
                }
            }

            let update_range_start = key_update(oid, 0)?;
            let update_range_end = key_update(oid, clock)?;
            let mut updates: Vec<Update> = Vec::new();
            for entry in self
                .iter_range(&update_range_start, &update_range_end)
                .await?
            {
                let value = entry.value();
                if let Ok(update) = Update::decode_v1(value) {
                    updates.push(update);
                }
            }

            for update in updates {
                let _ = txn.apply_update(update);
            }

            let doc_state = txn.encode_state_as_update_v1(&StateVector::default());
            let state_vector = txn.state_vector().encode_v1();

            self.upsert(&doc_key, &doc_state).await?;
            let sv_key = key_state_vector(oid)?;
            self.upsert(&sv_key, &state_vector).await?;

            let checkpoint_key = format!("checkpoint:{}", hex::encode(doc_id.as_bytes()));
            self.upsert(checkpoint_key.as_bytes(), &clock.to_be_bytes())
                .await?;
        }

        Ok(clock)
    }

    pub async fn get_last_checkpoint(&self, doc_id: &str) -> Result<Option<u32>> {
        let checkpoint_key = format!("checkpoint:{}", hex::encode(doc_id.as_bytes()));
        if let Some(data) = self.get(checkpoint_key.as_bytes()).await? {
            let data_ref: &[u8] = data.as_ref();
            if data_ref.len() >= 4 {
                let clock_bytes: [u8; 4] = data_ref[..4].try_into().unwrap();
                Ok(Some(u32::from_be_bytes(clock_bytes)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn rollback_to(&self, doc_id: &str, target_clock: u32) -> Result<Doc> {
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => anyhow::bail!("Document not found"),
        };

        // Try to find the nearest checkpoint before target_clock
        let last_checkpoint = self.get_last_checkpoint(doc_id).await?;
        let start_clock = if let Some(checkpoint_clock) = last_checkpoint {
            if checkpoint_clock <= target_clock {
                // Load from checkpoint
                let doc_key = key_doc(oid)?;
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                if let Some(doc_state) = self.get(&doc_key).await? {
                    if let Ok(update) = Update::decode_v1(doc_state.as_ref()) {
                        let _ = txn.apply_update(update);
                    }
                }
                drop(txn);
                checkpoint_clock
            } else {
                0
            }
        } else {
            0
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

        let mut filtered_objects = Vec::new();
        for obj in all_objects {
            let key_bytes = hex::decode(&obj.name)?;

            if key_bytes.len() < 12 {
                continue;
            }

            let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
            let clock = u32::from_be_bytes(clock_bytes);

            if clock > start_clock && clock <= target_clock {
                filtered_objects.push((obj, clock));
            }
        }

        filtered_objects.sort_by_key(|(_, clock)| *clock);

        let doc = if start_clock == 0 {
            Doc::new()
        } else {
            let doc_key = key_doc(oid)?;
            let doc = Doc::new();
            let mut txn = doc.transact_mut();

            if let Some(doc_state) = self.get(&doc_key).await? {
                if let Ok(update) = Update::decode_v1(doc_state.as_ref()) {
                    let _ = txn.apply_update(update);
                }
            }
            drop(txn);
            doc
        };

        let mut txn = doc.transact_mut();

        for chunk in filtered_objects.chunks(BATCH_SIZE) {
            let chunk_futures = chunk.iter().map(|(obj, clock)| {
                let bucket = self.bucket.clone();
                let object = obj.name.clone();
                let clock = *clock;

                async move {
                    let request = GetObjectRequest {
                        bucket,
                        object: object.clone(),
                        ..Default::default()
                    };

                    match self
                        .client
                        .download_object(&request, &Range::default())
                        .await
                    {
                        Ok(data) => {
                            if let Ok(update) = Update::decode_v1(&data) {
                                Some((clock, update))
                            } else {
                                error!("Failed to decode update from object: {}", object);
                                None
                            }
                        }
                        Err(e) => {
                            error!("Failed to download object {}: {:?}", object, e);
                            None
                        }
                    }
                }
            });

            let batch_results = join_all(chunk_futures).await;

            for result in batch_results.into_iter().flatten() {
                let (_, update) = result;
                let _ = txn.apply_update(update);
            }
        }

        drop(txn);
        Ok(doc)
    }

    /// Delete all GCS update objects with clock > target_clock.
    /// Used after rollback to remove stale "future" versions so that
    /// `get_updates()` / `get_history()` only return valid versions.
    pub async fn delete_updates_after(&self, doc_id: &str, target_clock: u32) -> Result<usize> {
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(0),
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

        let mut to_delete = Vec::new();
        for obj in &all_objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    if let Ok(clock_bytes) = key_bytes[7..11].try_into() {
                        let clock = u32::from_be_bytes(clock_bytes);
                        if clock > target_clock {
                            to_delete.push(obj.name.clone());
                        }
                    }
                }
            }
        }

        let count = to_delete.len();
        for chunk in to_delete.chunks(BATCH_SIZE) {
            let delete_futures = chunk.iter().map(|obj_name| {
                let bucket = self.bucket.clone();
                let object = obj_name.clone();
                async move {
                    self.client
                        .delete_object(&DeleteObjectRequest {
                            bucket,
                            object,
                            ..Default::default()
                        })
                        .await
                }
            });
            let _ = join_all(delete_futures).await;
        }

        if count > 0 {
            info!(
                "Deleted {} update objects after clock {} for doc '{}'",
                count, target_clock, doc_id
            );
        }

        Ok(count)
    }

    pub async fn get_updates(&self, doc_id: &str) -> Result<Vec<UpdateInfo>> {
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(Vec::new()),
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

        let mut updates = Vec::new();
        for obj in objects {
            let request = GetObjectRequest {
                bucket: self.bucket.clone(),
                object: obj.name.clone(),
                ..Default::default()
            };

            let data = self
                .client
                .download_object(&request, &Range::default())
                .await?;

            if let Ok(update) = Update::decode_v1(&data) {
                if let Ok(key_bytes) = hex::decode(&obj.name) {
                    if key_bytes.len() >= 12 {
                        let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
                        let clock = u32::from_be_bytes(clock_bytes);

                        let timestamp = obj.updated.unwrap_or_else(OffsetDateTime::now_utc);

                        updates.push(UpdateInfo {
                            clock,
                            timestamp,
                            update,
                        });
                    }
                }
            }
        }

        updates.sort_unstable_by_key(|info| std::cmp::Reverse(info.clock));

        Ok(updates)
    }

    pub async fn get_updates_by_version(
        &self,
        doc_id: &str,
        version: u32,
    ) -> Result<Option<UpdateInfo>> {
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

        for obj in objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into().unwrap();
                    let clock = u32::from_be_bytes(clock_bytes);

                    if clock == version {
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
                                let timestamp = obj.updated.unwrap_or_else(OffsetDateTime::now_utc);
                                return Ok(Some(UpdateInfo {
                                    clock,
                                    timestamp,
                                    update,
                                }));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn get_latest_update_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Option<(u32, OffsetDateTime)>> {
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
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
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

    pub async fn get_updates_metadata(&self, doc_id: &str) -> Result<Vec<(u32, OffsetDateTime)>> {
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(Vec::new()),
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

        let mut metadata = Vec::new();
        for obj in all_objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
                    let clock = u32::from_be_bytes(clock_bytes);
                    let timestamp = obj.updated.unwrap_or_else(OffsetDateTime::now_utc);

                    metadata.push((clock, timestamp));
                }
            }
        }

        metadata.sort_by_key(|(clock, _)| std::cmp::Reverse(*clock));

        Ok(metadata)
    }

    pub async fn trim_updates_logarithmic(
        &self,
        doc_id: &str,
        density_shift: u32,
    ) -> Result<Option<Doc>> {
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

        let mut clocks = Vec::new();
        for obj in &objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
                    let clock = u32::from_be_bytes(clock_bytes);
                    clocks.push((clock, obj.name.clone()));
                }
            }
        }

        if clocks.is_empty() {
            return Ok(None);
        }

        clocks.sort_by_key(|(clock, _)| *clock);
        let n = clocks.last().unwrap().0;

        let to_delete = if n > 0 {
            let bit = first_zero_bit(n);
            let delete_offset = bit << density_shift;
            n.saturating_sub(delete_offset)
        } else {
            0
        };

        if to_delete == 0 {
            return Ok(None);
        }

        let mut to_delete_obj = None;
        for (clock, name) in clocks {
            if clock == to_delete {
                to_delete_obj = Some(name);
                break;
            }
        }

        let Some(obj_to_delete) = to_delete_obj else {
            return Ok(None);
        };

        let doc = Doc::new();
        let mut found = false;

        let doc_key_bytes = [V1, KEYSPACE_DOC]
            .iter()
            .chain(&oid.to_be_bytes())
            .chain(&[SUB_DOC])
            .copied()
            .collect::<Vec<_>>();
        let doc_key_str = hex::encode(&doc_key_bytes);

        let doc_request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: doc_key_str,
            ..Default::default()
        };

        if let Ok(data) = self
            .client
            .download_object(&doc_request, &Range::default())
            .await
        {
            if let Ok(update) = Update::decode_v1(&data) {
                let _ = doc.transact_mut().apply_update(update);
                found = true;
            }
        }

        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: obj_to_delete.clone(),
            ..Default::default()
        };

        if let Ok(data) = self
            .client
            .download_object(&request, &Range::default())
            .await
        {
            if let Ok(update) = Update::decode_v1(&data) {
                let _ = doc.transact_mut().apply_update(update);
                found = true;
            }
        }

        let delete_request = DeleteObjectRequest {
            bucket: self.bucket.clone(),
            object: obj_to_delete,
            ..Default::default()
        };

        self.client.delete_object(&delete_request).await?;

        if found {
            let doc_state;
            let state_vector;
            {
                let txn = doc.transact();
                doc_state = txn.encode_state_as_update_v1(&StateVector::default());
                state_vector = txn.state_vector().encode_v1();
            }

            let doc_state_key = hex::encode(&doc_key_bytes);
            let upload_type = UploadType::Simple(Media::new(doc_state_key.clone()));
            self.client
                .upload_object(
                    &UploadObjectRequest {
                        bucket: self.bucket.clone(),
                        ..Default::default()
                    },
                    doc_state.to_vec(),
                    &upload_type,
                )
                .await?;

            let sv_key_bytes = [V1, KEYSPACE_DOC]
                .iter()
                .chain(&oid.to_be_bytes())
                .chain(&[SUB_STATE_VEC])
                .copied()
                .collect::<Vec<_>>();
            let sv_key = hex::encode(&sv_key_bytes);

            let sv_upload_type = UploadType::Simple(Media::new(sv_key.clone()));
            self.client
                .upload_object(
                    &UploadObjectRequest {
                        bucket: self.bucket.clone(),
                        ..Default::default()
                    },
                    state_vector.to_vec(),
                    &sv_upload_type,
                )
                .await?;

            return Ok(Some(doc));
        }

        Ok(None)
    }

    pub async fn create_snapshot_from_version(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Option<Doc>> {
        let target_version = version as u32;

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

        let mut filtered_objects = Vec::new();
        for obj in all_objects {
            let key_bytes = hex::decode(&obj.name)?;

            if key_bytes.len() < 12 {
                continue;
            }

            let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
            let clock = u32::from_be_bytes(clock_bytes);

            if clock <= target_version {
                filtered_objects.push((obj, clock));
            }
        }

        if filtered_objects.is_empty() {
            return Ok(None);
        }

        filtered_objects.sort_by_key(|(_, clock)| *clock);

        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        let mut updates_applied = false;

        for chunk in filtered_objects.chunks(BATCH_SIZE) {
            let chunk_futures = chunk.iter().map(|(obj, _)| {
                let bucket = self.bucket.clone();
                let object = obj.name.clone();

                async move {
                    let request = GetObjectRequest {
                        bucket,
                        object: object.clone(),
                        ..Default::default()
                    };

                    match self
                        .client
                        .download_object(&request, &Range::default())
                        .await
                    {
                        Ok(data) => {
                            if let Ok(update) = Update::decode_v1(&data) {
                                Some(update)
                            } else {
                                error!("Failed to decode update from object: {}", object);
                                None
                            }
                        }
                        Err(e) => {
                            error!("Failed to download object {}: {:?}", object, e);
                            None
                        }
                    }
                }
            });

            let batch_results = join_all(chunk_futures).await;

            for update in batch_results.into_iter().flatten() {
                let _ = txn.apply_update(update);
                updates_applied = true;
            }
        }

        drop(txn);

        if updates_applied {
            Ok(Some(doc))
        } else {
            Ok(None)
        }
    }

    /// Keep only the most recent `max_keep` individual update objects for a document.
    /// Older updates are merged into the compacted v1 doc state, then deleted.
    /// Operates in batches to handle documents with hundreds of updates.
    pub async fn cleanup_old_updates(&self, doc_id: &str, max_keep: usize) -> Result<usize> {
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(0),
        };

        // List all update objects (paginated)
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

        // Parse clock from each object key
        let mut clock_objects: Vec<(u32, String)> = Vec::new();
        for obj in &all_objects {
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                if key_bytes.len() >= 12 {
                    let clock_bytes: [u8; 4] = key_bytes[7..11].try_into()?;
                    let clock = u32::from_be_bytes(clock_bytes);
                    clock_objects.push((clock, obj.name.clone()));
                }
            }
        }

        if clock_objects.len() <= max_keep {
            return Ok(0);
        }

        // Sort ascending by clock — oldest first
        clock_objects.sort_by_key(|(clock, _)| *clock);

        let num_to_delete = clock_objects.len() - max_keep;
        let to_delete: Vec<(u32, String)> = clock_objects[..num_to_delete].to_vec();
        let highest_deleted_clock = to_delete.last().map(|(c, _)| *c).unwrap_or(0);

        // Merge deleted updates into the compacted v1 doc state
        let doc = Doc::new();
        let doc_key = key_doc(oid)?;

        {
            let mut txn = doc.transact_mut();
            // Load existing compacted state
            if let Some(doc_state) = self.get(&doc_key).await? {
                if let Ok(update) = Update::decode_v1(doc_state.as_ref()) {
                    let _ = txn.apply_update(update);
                }
            }

            // Download and apply updates being deleted (in batches).
            // If any download or decode fails, abort cleanup to avoid data loss.
            for chunk in to_delete.chunks(BATCH_SIZE) {
                let chunk_futures = chunk.iter().map(|(_, obj_name)| {
                    let bucket = self.bucket.clone();
                    let object = obj_name.clone();

                    async move {
                        let request = GetObjectRequest {
                            bucket,
                            object: object.clone(),
                            ..Default::default()
                        };
                        self.client
                            .download_object(&request, &Range::default())
                            .await
                            .map_err(|err| {
                                anyhow::anyhow!(
                                    "failed to download update object '{}' for compaction: {}",
                                    object,
                                    err
                                )
                            })
                    }
                });

                let results = join_all(chunk_futures).await;
                for data in results {
                    let data = data?;
                    let update = Update::decode_v1(&data).map_err(|err| {
                        anyhow::anyhow!(
                            "failed to decode update while compacting document '{}': {}",
                            doc_id,
                            err
                        )
                    })?;
                    txn.apply_update(update).map_err(|err| {
                        anyhow::anyhow!(
                            "failed to apply update while compacting document '{}': {}",
                            doc_id,
                            err
                        )
                    })?;
                }
            }
        }

        // Write merged state
        {
            let txn = doc.transact();
            let doc_state = txn.encode_state_as_update_v1(&StateVector::default());
            let state_vector = txn.state_vector().encode_v1();

            self.upsert(&doc_key, &doc_state).await?;
            let sv_key = key_state_vector(oid)?;
            self.upsert(&sv_key, &state_vector).await?;
        }

        // Update checkpoint, but never move it backwards relative to the
        // already-persisted compacted doc state.
        let checkpoint_key = format!("checkpoint:{}", hex::encode(doc_id.as_bytes()));
        let current_checkpoint = self
            .get(checkpoint_key.as_bytes())
            .await?
            .and_then(|bytes| {
                if bytes.len() == 4 {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(bytes.as_ref());
                    Some(u32::from_be_bytes(arr))
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let new_checkpoint = current_checkpoint.max(highest_deleted_clock);
        self.upsert(checkpoint_key.as_bytes(), &new_checkpoint.to_be_bytes())
            .await?;

        // Delete old update objects in batches
        for chunk in to_delete.chunks(BATCH_SIZE) {
            let delete_futures = chunk.iter().map(|(_, obj_name)| {
                let bucket = self.bucket.clone();
                let object = obj_name.clone();
                async move {
                    let request = DeleteObjectRequest {
                        bucket,
                        object,
                        ..Default::default()
                    };
                    self.client.delete_object(&request).await
                }
            });
            let _ = join_all(delete_futures).await;
        }

        Ok(num_to_delete)
    }

    /// Delete ALL data for a document: doc_v2, v1 state, state vector, all updates,
    /// checkpoint, and OID mapping. Used for project deletion.
    pub async fn delete_all_doc_data(&self, doc_id: &str) -> Result<()> {
        // Delete doc_v2 snapshot
        let doc_v2_key = format!("doc_v2:{}", hex::encode(doc_id.as_bytes()));
        let doc_v2_hex = hex::encode(doc_v2_key.as_bytes());
        let _ = self
            .client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: doc_v2_hex,
                ..Default::default()
            })
            .await;

        // Delete checkpoint
        let checkpoint_key = format!("checkpoint:{}", hex::encode(doc_id.as_bytes()));
        let checkpoint_hex = hex::encode(checkpoint_key.as_bytes());
        let _ = self
            .client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: checkpoint_hex,
                ..Default::default()
            })
            .await;

        // Delete all OID-based objects using direct GCS listing
        let oid = match get_oid(self, doc_id.as_bytes()).await? {
            Some(oid) => oid,
            None => return Ok(()),
        };

        // Delete OID mapping
        let oid_key = crate::domain::value_objects::keys::key_oid(doc_id.as_bytes())?;
        let oid_hex = hex::encode(oid_key.as_ref());
        let _ = self
            .client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: oid_hex,
                ..Default::default()
            })
            .await;

        // List and delete all objects in the document's key range.
        // Use [V1, KEYSPACE_DOC, oid] as prefix (without sub-key tag) to match
        // all sub-keys: doc state, state vector, updates, and metadata.
        let doc_prefix_bytes = [V1, KEYSPACE_DOC]
            .iter()
            .chain(&oid.to_be_bytes())
            .copied()
            .collect::<Vec<_>>();
        let end_bytes = doc_prefix_bytes
            .iter()
            .chain(&[0xFF])
            .copied()
            .collect::<Vec<_>>();
        let prefix = hex::encode(&doc_prefix_bytes);

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

            let end_hex = hex::encode(&end_bytes);
            let filtered = items
                .into_iter()
                .filter(|obj| obj.name.as_str() <= end_hex.as_str());
            all_objects.extend(filtered);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        // Batch delete
        for chunk in all_objects.chunks(BATCH_SIZE) {
            let delete_futures = chunk.iter().map(|obj| {
                let bucket = self.bucket.clone();
                let object = obj.name.clone();
                async move {
                    let request = DeleteObjectRequest {
                        bucket,
                        object,
                        ..Default::default()
                    };
                    self.client.delete_object(&request).await
                }
            });
            let _ = join_all(delete_futures).await;
        }

        Ok(())
    }

    /// Run cleanup on ALL documents in the bucket: for each document with an OID,
    /// run `cleanup_old_updates` to keep at most `max_keep` versions.
    /// Returns (docs_processed, total_updates_deleted).
    pub async fn cleanup_all_documents(&self, max_keep: usize) -> Result<(usize, usize)> {
        // List all OID entries (prefix "00" in hex = KEYSPACE_OID)
        let oid_prefix = hex::encode([V1, KEYSPACE_OID]);
        let doc_end_prefix = hex::encode([V1, KEYSPACE_DOC]);

        let mut all_oid_objects = Vec::new();
        let mut page_token = None;

        loop {
            let request = ListObjectsRequest {
                bucket: self.bucket.clone(),
                prefix: Some(oid_prefix.clone()),
                page_token: page_token.clone(),
                ..Default::default()
            };

            let response = self.client.list_objects(&request).await?;
            let items = response.items.unwrap_or_default();

            // Filter to only OID keyspace entries (< doc keyspace)
            let filtered = items
                .into_iter()
                .filter(|obj| obj.name.as_str() < doc_end_prefix.as_str());
            all_oid_objects.extend(filtered);

            if let Some(token) = response.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        let mut docs_processed = 0;
        let mut total_deleted = 0;

        for obj in &all_oid_objects {
            // Decode the OID key to extract the document name
            if let Ok(key_bytes) = hex::decode(&obj.name) {
                // Key format: [V1, KEYSPACE_OID, ...doc_name_bytes..., TERMINATOR]
                if key_bytes.len() > 3 {
                    let doc_name_bytes = &key_bytes[2..key_bytes.len() - 1];
                    if let Ok(doc_name) = std::str::from_utf8(doc_name_bytes) {
                        match self.cleanup_old_updates(doc_name, max_keep).await {
                            Ok(deleted) => {
                                if deleted > 0 {
                                    tracing::info!(
                                        "Cleaned up {} old updates for doc '{}'",
                                        deleted,
                                        doc_name
                                    );
                                }
                                total_deleted += deleted;
                            }
                            Err(e) => {
                                tracing::warn!("Failed to cleanup doc '{}': {}", doc_name, e);
                            }
                        }
                        docs_processed += 1;
                    }
                }
            }
        }

        Ok((docs_processed, total_deleted))
    }
}

impl DocOps<'_> for GcsStore {}

/// Returns true if the given GCS error represents a "not found" (404) response.
/// All other errors (permission denied, server errors, network failures) return false
/// so that they propagate to the caller instead of being silently swallowed.
fn is_not_found(err: &google_cloud_storage::http::Error) -> bool {
    match err {
        google_cloud_storage::http::Error::Response(resp) => resp.code == 404,
        google_cloud_storage::http::Error::HttpClient(reqwest_err) => {
            reqwest_err.status().is_some_and(|s| s.as_u16() == 404)
        }
        _ => false,
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
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(e),
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

        all_objects.sort_by(|a, b| a.name.cmp(&b.name));

        let mut all_values = Vec::with_capacity(all_objects.len());

        for chunk in all_objects.chunks(BATCH_SIZE) {
            let chunk_futures = chunk.iter().map(|obj| {
                let bucket = self.bucket.clone();
                let object = obj.name.clone();
                async move {
                    let request = GetObjectRequest {
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

            let batch_results = join_all(chunk_futures).await;

            let mut result_map = std::collections::HashMap::new();
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

#[cfg(test)]
mod tests {
    use super::is_not_found;
    use google_cloud_storage::http::error::ErrorResponse;
    use google_cloud_storage::http::Error as GcsError;

    fn make_response_error(code: u16, message: &str) -> GcsError {
        GcsError::Response(ErrorResponse {
            code,
            errors: vec![],
            message: message.to_string(),
        })
    }

    #[test]
    fn gcs_404_is_not_found() {
        let err = make_response_error(404, "Not Found");
        assert!(
            is_not_found(&err),
            "404 Response error should be identified as not-found"
        );
    }

    #[test]
    fn gcs_500_is_not_not_found() {
        let err = make_response_error(500, "Internal Server Error");
        assert!(
            !is_not_found(&err),
            "500 Response error should not be identified as not-found"
        );
    }

    #[test]
    fn gcs_403_is_not_not_found() {
        let err = make_response_error(403, "Forbidden");
        assert!(
            !is_not_found(&err),
            "403 Response error should not be identified as not-found"
        );
    }

    #[test]
    fn gcs_401_is_not_not_found() {
        let err = make_response_error(401, "Unauthorized");
        assert!(
            !is_not_found(&err),
            "401 Response error should not be identified as not-found"
        );
    }
}

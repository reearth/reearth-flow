//! In order to do so, persistent unit of transaction should define set of basic operations via
//! [KVStore] trait implementation. Once this is done it can implement [DocOps]. Latter offers a
//! set of useful operations like document metadata management options, document and update merging
//! etc. They are implemented automatically as long struct has correctly implemented [KVStore].
//!
//! ## Internal representation
//!
//! yrs-kvstore operates around few key spaces. All keys inserted via [DocOps] are prefixed with
//! [V1] constant. Later on the key space is further divided into:
//!
//! - [KEYSPACE_OID] used for object ID (OID) index mapping. Whenever the new document is being
//!   inserted, a new OID number is generated for it. While document names can be any kind of strings
//!   OIDs are guaranteed to have constant size. Internally all of the document contents are referred
//!   to via their OID identifiers.
//! - [KEYSPACE_DOC] used to store [document state](crate::keys::SUB_DOC), its
//!   [state vector](crate::keys::SUB_STATE_VEC), corresponding series of
//!   [updates](crate::keys::SUB_UPDATE) and [metadata](crate::keys::SUB_META). Document state and
//!   state vector may not represent full system knowledge about the document, as they don't reflect
//!   information inside document updates. Updates can be stored separately to avoid big document
//!   binary read/parse/merge/store cycles of every update. It's a good idea to insert updates as they
//!   come and every once in a while call [DocOps::flush_doc] or [DocOps::flush_doc_with] to merge
//!   them into document state itself.
//!
//! The variants and schemas of byte keys in use could be summarized as:
//!
//! ```nocompile
//! 00{doc_name:N}0      - OID key pattern
//! 01{oid:4}0           - document key pattern
//! 01{oid:4}1           - state vector key pattern
//! 01{oid:4}2{seqNr:4}0 - document update key pattern
//! 01{oid:4}3{name:M}0  - document meta key pattern
//! ```

use crate::domain::entity::keys::{Error, SUB_UPDATE};

use crate::domain::entity::keys::{
    doc_oid_name, key_doc, key_doc_end, key_doc_start, key_meta, key_meta_end, key_meta_start,
    key_oid, key_state_vector, key_update, Key, KEYSPACE_DOC, KEYSPACE_OID, OID, V1,
};
use crate::domain::repository::kv::KVEntry;
use crate::domain::repository::kv::KVStore;
use crate::infrastructure::RedisStore;
use crate::tools::{compress_brotli, decompress_brotli};
use anyhow;
use anyhow::Result;
use async_trait::async_trait;
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use hex;
use std::convert::TryInto;
use time::OffsetDateTime;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Transaction, TransactionMut, Update};

use super::first_zero_bit;

#[async_trait]
pub trait DocOps<'a>: KVStore + Sized
where
    Error: From<<Self as KVStore>::Error>,
{
    async fn insert_doc<K: AsRef<[u8]> + ?Sized + Sync, T: ReadTxn + Sync>(
        &self,
        name: &K,
        txn: &T,
        redis: &RedisStore,
    ) -> Result<(), Error> {
        let doc_state = txn.encode_diff_v1(&StateVector::default());
        let state_vector = txn.state_vector().encode_v1();
        self.insert_doc_raw_v1(name.as_ref(), &doc_state, &state_vector, redis)
            .await
    }

    async fn insert_doc_raw_v1(
        &self,
        name: &[u8],
        doc_state_v1: &[u8],
        doc_sv_v1: &[u8],
        redis: &RedisStore,
    ) -> Result<()> {
        let oid = get_or_create_oid(self, name, redis).await?;
        insert_inner_v1(self, oid, doc_state_v1, doc_sv_v1).await?;
        Ok(())
    }

    async fn load_doc<'doc, K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
        txn: &mut TransactionMut<'doc>,
    ) -> Result<bool> {
        if let Some(oid) = get_oid(self, name.as_ref()).await? {
            let loaded = load_doc(self, oid, txn).await?;
            Ok(loaded != 0)
        } else {
            Ok(false)
        }
    }

    async fn flush_doc<K: AsRef<[u8]> + ?Sized + Sync>(&self, name: &K) -> Result<Option<Doc>> {
        self.flush_doc_with(name, yrs::Options::default()).await
    }

    async fn trim_updates<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
        density_shift: u32,
    ) -> Result<Option<Doc>> {
        if let Some(oid) = get_oid(self, name.as_ref()).await? {
            let update_range_start = key_update(oid, 0)?;
            let update_range_end = key_update(oid, u32::MAX)?;
            let mut updates = Vec::new();

            for entry in self
                .iter_range(&update_range_start, &update_range_end)
                .await?
            {
                let key = entry.key();
                let len = key.len();
                let seq_bytes = &key[(len - 5)..(len - 1)];
                let seq_nr = u32::from_be_bytes(seq_bytes.try_into().unwrap());
                updates.push(seq_nr);
            }

            if updates.is_empty() {
                return Ok(None);
            }

            updates.sort_unstable();
            let n = *updates.last().unwrap();

            let to_delete = if n > 0 {
                let bit = first_zero_bit(n);
                let delete_offset = bit << density_shift;
                n.saturating_sub(delete_offset)
            } else {
                0
            };

            if to_delete > 0 && updates.contains(&to_delete) {
                let doc = Doc::new();
                let mut found = false;

                {
                    let doc_key = key_doc(oid)?;
                    if let Some(doc_state) = self.get(&doc_key).await? {
                        let update = Update::decode_v1(doc_state.as_ref())?;
                        let _ = doc.transact_mut().apply_update(update);
                        found = true;
                    }
                }

                let update_key = key_update(oid, to_delete)?;
                if let Some(data) = self.get(&update_key).await? {
                    let update = Update::decode_v1(data.as_ref())?;
                    let _ = doc.transact_mut().apply_update(update);
                    found = true;
                }

                self.remove(&update_key).await?;

                if found {
                    let txn = doc.transact();
                    let doc_state = txn.encode_state_as_update_v1(&StateVector::default());
                    let state_vec = txn.state_vector().encode_v1();
                    drop(txn);

                    insert_inner_v1(self, oid, &doc_state, &state_vec).await?;
                    return Ok(Some(doc));
                }
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }

    async fn flush_doc_with<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
        options: yrs::Options,
    ) -> Result<Option<Doc>> {
        if let Some(oid) = get_oid(self, name.as_ref()).await? {
            let doc = flush_doc(self, oid, options).await?;
            Ok(doc)
        } else {
            Ok(None)
        }
    }

    async fn get_state_vector<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
    ) -> Result<(Option<StateVector>, bool)> {
        if let Some(oid) = get_oid(self, name.as_ref()).await? {
            let key = key_state_vector(oid)?;
            let data = self.get(&key).await?;
            let sv = if let Some(data) = data {
                let state_vector = StateVector::decode_v1(data.as_ref())?;
                Some(state_vector)
            } else {
                None
            };
            let update_range_start = key_update(oid, 0)?;
            let update_range_end = key_update(oid, u32::MAX)?;
            let mut iter = self
                .iter_range(&update_range_start, &update_range_end)
                .await?;
            let up_to_date = iter.next().is_none();
            Ok((sv, up_to_date))
        } else {
            Ok((None, true))
        }
    }

    async fn push_update<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
        update: &[u8],
        redis: &RedisStore,
    ) -> Result<u32> {
        let oid = get_or_create_oid(self, name.as_ref(), redis).await?;
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
        Ok(clock)
    }

    async fn get_diff<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
        sv: &StateVector,
    ) -> Result<Option<Vec<u8>>> {
        let doc = Doc::new();
        let found = {
            let mut txn = doc.transact_mut();
            self.load_doc(name, &mut txn).await?
        };
        if found {
            Ok(Some(doc.transact().encode_diff_v1(sv)))
        } else {
            Ok(None)
        }
    }

    async fn clear_doc<K: AsRef<[u8]> + ?Sized + Sync>(&self, name: &K) -> Result<()> {
        let oid_key = key_oid(name.as_ref())?;
        if let Some(oid) = self.get(&oid_key).await? {
            // all document related elements are stored within bounds [0,1,..oid,0]..[0,1,..oid,255]
            let oid: [u8; 4] = oid.as_ref().try_into().unwrap();
            let oid = OID::from_be_bytes(oid);
            self.remove(&oid_key).await?;
            let start = key_doc_start(oid)?;
            let end = key_doc_end(oid)?;
            for v in self.iter_range(&start, &end).await? {
                let key: &[u8] = v.key();
                if key > end.as_ref() {
                    break; //TODO: for some reason key range doesn't always work
                }
                self.remove(key).await?;
            }
        }
        Ok(())
    }

    async fn get_meta<K1: AsRef<[u8]> + ?Sized + Sync, K2: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K1,
        meta_key: &K2,
    ) -> Result<Option<Self::Return>> {
        if let Some(oid) = get_oid(self, name.as_ref()).await? {
            let key = key_meta(oid, meta_key.as_ref())?;
            Ok(self.get(&key).await?)
        } else {
            Ok(None)
        }
    }

    async fn insert_meta<K1: AsRef<[u8]> + ?Sized + Sync, K2: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K1,
        meta_key: &K2,
        meta: &[u8],
        redis: &RedisStore,
    ) -> Result<()> {
        let oid = get_or_create_oid(self, name.as_ref(), redis).await?;
        let key = key_meta(oid, meta_key.as_ref())?;
        self.upsert(&key, meta).await?;
        Ok(())
    }

    async fn remove_meta<K1: AsRef<[u8]> + ?Sized + Sync, K2: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K1,
        meta_key: &K2,
    ) -> Result<()> {
        if let Some(oid) = get_oid(self, name.as_ref()).await? {
            let key = key_meta(oid, meta_key.as_ref())?;
            self.remove(&key).await?;
        }
        Ok(())
    }

    async fn iter_docs(&self) -> Result<DocsNameIter<Self::Cursor, Self::Entry>> {
        let start = Key::from_const([V1, KEYSPACE_OID]);
        let end = Key::from_const([V1, KEYSPACE_DOC]);
        let cursor = self.iter_range(&start, &end).await?;
        Ok(DocsNameIter { cursor })
    }

    async fn iter_meta<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        doc_name: &K,
    ) -> Result<MetadataIter<Self::Cursor, Self::Entry>> {
        if let Some(oid) = get_oid(self, doc_name.as_ref()).await? {
            let start = key_meta_start(oid)?.to_vec();
            let end = key_meta_end(oid)?.to_vec();
            let cursor = self.iter_range(&start, &end).await?;
            Ok(MetadataIter(Some((cursor, start, end))))
        } else {
            Ok(MetadataIter(None))
        }
    }

    async fn load_doc_v2<K: AsRef<[u8]> + ?Sized + Sync>(&self, name: &K) -> Result<Doc> {
        let doc_key = format!("doc_v2:{}", hex::encode(name.as_ref()));
        let doc_key_bytes = doc_key.as_bytes();

        match self.get(doc_key_bytes).await? {
            Some(data) => {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                let decompressed_data = decompress_brotli(data.as_ref())?;
                if let Ok(update) = Update::decode_v2(&decompressed_data) {
                    txn.apply_update(update)?;
                }
                drop(txn);
                Ok(doc)
            }

            None => Err(anyhow::anyhow!(
                "Document not found: {}",
                hex::encode(name.as_ref())
            )),
        }
    }

    async fn flush_doc_v2<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        name: &K,
        txn: &Transaction,
    ) -> Result<()> {
        let doc_key = format!("doc_v2:{}", hex::encode(name.as_ref()));
        let doc_key_bytes = doc_key.as_bytes();

        let state = txn.encode_state_as_update_v2(&StateVector::default());

        let compressed_data = compress_brotli(&state, 4, 22)?;

        self.upsert(doc_key_bytes, &compressed_data).await?;
        Ok(())
    }

    async fn create_snapshot_from_version<K: AsRef<[u8]> + ?Sized + Sync>(
        &self,
        doc_id: &K,
        version: u64,
    ) -> Result<Option<Doc>> {
        let target_version = version as u32;

        let oid = match get_oid(self, doc_id.as_ref()).await? {
            Some(oid) => oid,
            None => return Ok(None),
        };

        let prefix_bytes: Vec<u8> = [V1, KEYSPACE_DOC]
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

    async fn get_last_checkpoint(&self, doc_id: &str) -> Result<Option<u32>, Error> {
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

    async fn rollback_to(&self, doc_id: &str, target_clock: u32) -> Result<Doc> {
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

    async fn get_latest_update_metadata(
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
}

pub async fn get_oid<'a, DB: DocOps<'a>>(db: &DB, name: &[u8]) -> Result<Option<OID>, Error>
where
    Error: From<<DB as KVStore>::Error>,
{
    let key = key_oid(name)?;
    let value = db.get(&key).await?;
    if let Some(value) = value {
        let bytes: [u8; 4] = value.as_ref().try_into().unwrap();
        let oid = OID::from_be_bytes(bytes);
        Ok(Some(oid))
    } else {
        Ok(None)
    }
}

pub async fn get_or_create_oid<'a, DB: DocOps<'a>>(
    db: &DB,
    name: &[u8],
    redis: &RedisStore,
) -> Result<OID, Error>
where
    Error: From<<DB as KVStore>::Error>,
{
    if let Some(oid) = get_oid(db, name).await? {
        return Ok(oid);
    }

    let mut lock_value = None;
    let max_retries = 10;
    let retry_delay_ms = 500;

    for attempt in 0..max_retries {
        match redis.acquire_oid_lock(10).await {
            Ok(value) => {
                lock_value = Some(value);
                break;
            }
            Err(e) => {
                if attempt < max_retries - 1 {
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;

                    if let Some(oid) = get_oid(db, name).await? {
                        return Ok(oid);
                    }
                } else {
                    return Err(anyhow::anyhow!("Failed to acquire OID lock: {}", e));
                }
            }
        }
    }

    let lock_value = lock_value.ok_or_else(|| {
        anyhow::anyhow!("Failed to acquire OID lock after {} attempts", max_retries)
    })?;

    if let Some(oid) = get_oid(db, name).await? {
        let _ = redis.release_oid_lock(&lock_value).await;
        return Ok(oid);
    }

    let last_oid_key = b"system:last_oid".to_vec();
    let new_oid = match db.get(&last_oid_key).await? {
        Some(last_oid_data) => {
            if last_oid_data.as_ref().len() >= 4 {
                let bytes: [u8; 4] = last_oid_data.as_ref()[..4].try_into().unwrap();
                let last_oid = OID::from_be_bytes(bytes);
                last_oid + 1
            } else {
                let last_oid = if let Some(e) = db.peek_back([V1, KEYSPACE_DOC].as_ref()).await? {
                    let value = e.value();
                    OID::from_be_bytes(value.try_into().unwrap())
                } else {
                    0
                };
                last_oid + 1
            }
        }
        None => {
            let last_oid = if let Some(e) = db.peek_back([V1, KEYSPACE_DOC].as_ref()).await? {
                let value = e.value();
                OID::from_be_bytes(value.try_into().unwrap())
            } else {
                0
            };
            last_oid + 1
        }
    };

    let key = key_oid(name)?;
    let key_ref = key.as_ref();
    let new_oid_bytes = new_oid.to_be_bytes();
    let batch = [
        (key_ref, &new_oid_bytes[..]),
        (last_oid_key.as_ref(), &new_oid_bytes[..]),
    ];
    db.batch_upsert(&batch).await?;

    let _ = redis.release_oid_lock(&lock_value).await;

    Ok(new_oid)
}

async fn load_doc<'doc, 'a, DB: DocOps<'a>>(
    db: &DB,
    oid: OID,
    txn: &mut TransactionMut<'doc>,
) -> Result<u32, Error>
where
    Error: From<<DB as KVStore>::Error>,
{
    let mut found = false;
    {
        let doc_key = key_doc(oid)?;
        if let Some(doc_state) = db.get(&doc_key).await? {
            let update = Update::decode_v1(doc_state.as_ref())?;
            let _ = txn.apply_update(update);
            found = true;
        }
    }
    let mut update_count = 0;
    {
        let update_key_start = key_update(oid, 0)?;
        let update_key_end = key_update(oid, u32::MAX)?;
        let iter = db.iter_range(&update_key_start, &update_key_end).await?;
        for e in iter {
            let value = e.value();
            let update = Update::decode_v1(value)?;
            let _ = txn.apply_update(update);
            update_count += 1;
        }
    }
    if found {
        update_count |= 1 << 31;
    }
    Ok(update_count)
}

async fn delete_updates<'a, DB: DocOps<'a>>(db: &DB, oid: OID) -> Result<(), Error>
where
    Error: From<<DB as KVStore>::Error>,
{
    let start = key_update(oid, 0)?;
    let end = key_update(oid, u32::MAX)?;
    db.remove_range(&start, &end).await?;
    Ok(())
}

async fn flush_doc<'a, DB: DocOps<'a>>(
    db: &DB,
    oid: OID,
    options: yrs::Options,
) -> Result<Option<Doc>, Error>
where
    Error: From<<DB as KVStore>::Error>,
{
    let doc = Doc::with_options(options);
    let found = load_doc(db, oid, &mut doc.transact_mut()).await?;
    if found & !(1 << 31) != 0 {
        let txn = doc.transact();
        let doc_state = txn.encode_state_as_update_v1(&StateVector::default());
        let state_vec = txn.state_vector().encode_v1();
        drop(txn);

        insert_inner_v1(db, oid, &doc_state, &state_vec).await?;
        delete_updates(db, oid).await?;
        Ok(Some(doc))
    } else {
        Ok(None)
    }
}

async fn insert_inner_v1<'a, DB: DocOps<'a>>(
    db: &DB,
    oid: OID,
    doc_state_v1: &[u8],
    doc_sv_v1: &[u8],
) -> Result<(), Error>
where
    Error: From<<DB as KVStore>::Error>,
{
    let key_doc = key_doc(oid)?;
    let key_sv = key_state_vector(oid)?;
    db.upsert(&key_doc, doc_state_v1).await?;
    db.upsert(&key_sv, doc_sv_v1).await?;
    Ok(())
}

pub struct DocsNameIter<I, E>
where
    I: Iterator<Item = E>,
    E: KVEntry,
{
    cursor: I,
}

impl<I, E> Iterator for DocsNameIter<I, E>
where
    I: Iterator<Item = E>,
    E: KVEntry,
{
    type Item = Box<[u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        let e = self.cursor.next()?;
        Some(doc_oid_name(e.key()).into())
    }
}

pub struct MetadataIter<I, E>(Option<(I, Vec<u8>, Vec<u8>)>)
where
    I: Iterator<Item = E>,
    E: KVEntry;

impl<I, E> Iterator for MetadataIter<I, E>
where
    I: Iterator<Item = E>,
    E: KVEntry,
{
    type Item = (Box<[u8]>, Box<[u8]>);

    fn next(&mut self) -> Option<Self::Item> {
        let (cursor, _, _) = self.0.as_mut()?;
        let v = cursor.next()?;
        let key = v.key();
        let value = v.value();
        let meta_key = &key[7..key.len() - 1];
        Some((meta_key.into(), value.into()))
    }
}

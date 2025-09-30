#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use websocket::domain::repositories::kv::{KVEntry, KVStore};
    use websocket::domain::value_objects::keys::{
        key_doc, key_oid, key_state_vector, key_update, OID,
    };
    use yrs::updates::encoder::Encode;
    use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};

    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use websocket::application::kv::*;

    struct MockEntry {
        key: Vec<u8>,
        value: Vec<u8>,
    }

    impl KVEntry for MockEntry {
        fn key(&self) -> &[u8] {
            &self.key
        }

        fn value(&self) -> &[u8] {
            &self.value
        }
    }

    struct MockCursor {
        entries: Vec<MockEntry>,
        index: usize,
    }

    impl Iterator for MockCursor {
        type Item = MockEntry;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.entries.len() {
                let entry = MockEntry {
                    key: self.entries[self.index].key.clone(),
                    value: self.entries[self.index].value.clone(),
                };
                self.index += 1;
                Some(entry)
            } else {
                None
            }
        }
    }

    #[derive(Clone)]
    struct MockRedisStore {}

    impl MockRedisStore {
        fn new() -> Self {
            Self {}
        }

        // async fn acquire_oid_lock(&self, _timeout: u64) -> Result<String, error::Error> {
        //     Ok("mock_lock".to_string())
        // }

        // async fn release_oid_lock(&self, _lock_value: &str) -> Result<(), error::Error> {
        //     Ok(())
        // }
    }

    #[derive(Debug)]
    struct MockError(String);

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for MockError {}

    #[derive(Clone)]
    struct MockStore {
        data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(BTreeMap::new())),
            }
        }
    }

    #[async_trait]
    impl KVStore for MockStore {
        type Error = MockError;
        type Cursor = MockCursor;
        type Entry = MockEntry;
        type Return = Vec<u8>;

        async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error> {
            let data = self.data.lock().unwrap();
            Ok(data.get(key).cloned())
        }

        async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_vec(), value.to_vec());
            Ok(())
        }

        async fn batch_upsert(&self, entries: &[(&[u8], &[u8])]) -> Result<(), Self::Error> {
            let mut data = self.data.lock().unwrap();
            for (key, value) in entries {
                data.insert(key.to_vec(), value.to_vec());
            }
            Ok(())
        }

        async fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
            let mut data = self.data.lock().unwrap();
            data.remove(key);
            Ok(())
        }

        async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error> {
            let mut data = self.data.lock().unwrap();
            let keys_to_remove: Vec<Vec<u8>> = data
                .range(from.to_vec()..=to.to_vec())
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys_to_remove {
                data.remove(&key);
            }
            Ok(())
        }

        async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error> {
            let data = self.data.lock().unwrap();
            let entries: Vec<MockEntry> = data
                .range(from.to_vec()..=to.to_vec())
                .map(|(k, v)| MockEntry {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect();

            Ok(MockCursor { entries, index: 0 })
        }

        async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
            let data = self.data.lock().unwrap();
            if let Some(entry) = data.range(..=key.to_vec()).next_back() {
                return Ok(Some(MockEntry {
                    key: entry.0.clone(),
                    value: entry.1.clone(),
                }));
            }
            Ok(None)
        }
    }

    impl DocOps<'_> for MockStore {}

    async fn test_insert_doc(
        store: &MockStore,
        name: &[u8],
        txn: &impl ReadTxn,
        _redis: &MockRedisStore,
    ) -> Result<(), anyhow::Error> {
        let doc_state = txn.encode_diff_v1(&StateVector::default());
        let state_vector = txn.state_vector().encode_v1();

        let oid = if let Some(oid) = get_oid(store, name).await? {
            oid
        } else {
            let last_oid_key = b"system:last_oid".to_vec();
            let new_oid = match store.get(&last_oid_key).await.unwrap() {
                Some(last_oid_data) => {
                    if last_oid_data.len() >= 4 {
                        let bytes: [u8; 4] = last_oid_data[..4].try_into().unwrap();
                        let last_oid = OID::from_be_bytes(bytes);
                        last_oid + 1
                    } else {
                        1
                    }
                }
                None => 1,
            };

            let key = key_oid(name)?;
            let key_ref = key.as_ref();
            let new_oid_bytes = new_oid.to_be_bytes();
            let batch = [
                (key_ref, &new_oid_bytes[..]),
                (last_oid_key.as_ref(), &new_oid_bytes[..]),
            ];
            store.batch_upsert(&batch).await.unwrap();
            new_oid
        };

        let key_doc = key_doc(oid)?;
        let key_sv = key_state_vector(oid)?;
        store.upsert(&key_doc, &doc_state).await.unwrap();
        store.upsert(&key_sv, &state_vector).await.unwrap();
        Ok(())
    }

    async fn test_push_update(
        store: &MockStore,
        name: &[u8],
        update: &[u8],
        _redis: &MockRedisStore,
    ) -> Result<u32, anyhow::Error> {
        if let Some(oid) = get_oid(store, name).await? {
            let last_clock = {
                let end = key_update(oid, u32::MAX)?;
                if let Some(e) = store.peek_back(&end).await.unwrap() {
                    let last_key = e.key();
                    let len = last_key.len();
                    let last_clock = &last_key[(len - 5)..(len - 1)];
                    u32::from_be_bytes(last_clock.try_into().unwrap())
                } else {
                    0
                }
            };
            let clock = last_clock + 1;
            let update_key = key_update(oid, clock)?;
            store.upsert(&update_key, update).await.unwrap();
            Ok(clock)
        } else {
            test_insert_doc(store, name, &Doc::new().transact(), _redis).await?;
            Box::pin(test_push_update(store, name, update, _redis)).await
        }
    }

    #[tokio::test]
    async fn test_insert_and_load_doc() {
        let store = MockStore::new();
        let redis = MockRedisStore::new();
        let doc_name = b"test_doc";

        let doc = Doc::new();
        let text = doc.get_or_insert_text("test");
        {
            let mut txn = doc.transact_mut();
            text.push(&mut txn, "Hello, world!");
        }

        test_insert_doc(&store, doc_name, &doc.transact(), &redis)
            .await
            .unwrap();

        let loaded_doc = Doc::new();
        {
            let mut txn = loaded_doc.transact_mut();
            let loaded = store.load_doc(doc_name, &mut txn).await.unwrap();
            assert!(loaded);
        }

        let loaded_text = loaded_doc.get_or_insert_text("test");
        let txn = loaded_doc.transact();
        let content = loaded_text.get_string(&txn);
        assert_eq!(content, "Hello, world!");
    }

    #[tokio::test]
    async fn test_push_update_and_flush() {
        let store = MockStore::new();
        let redis = MockRedisStore::new();
        let doc_name = b"test_doc";

        let doc = Doc::new();
        let text = doc.get_or_insert_text("test");
        {
            let mut txn = doc.transact_mut();
            text.push(&mut txn, "Initial content");
        }

        test_insert_doc(&store, doc_name, &doc.transact(), &redis)
            .await
            .unwrap();

        {
            let mut txn = doc.transact_mut();
            text.insert(&mut txn, 16, " + update");
            let update = txn.encode_update_v1();

            test_push_update(&store, doc_name, &update, &redis)
                .await
                .unwrap();
        }

        let flushed_doc = store.flush_doc(doc_name).await.unwrap();
        assert!(flushed_doc.is_some());

        let flushed_doc = flushed_doc.unwrap();
        let flushed_text = flushed_doc.get_or_insert_text("test");
        let txn = flushed_doc.transact();
        let content = flushed_text.get_string(&txn);
        assert_eq!(content, "Initial content + update");
    }

    #[tokio::test]
    async fn test_direct_v2_doc_storage() {
        let store = MockStore::new();
        let doc_name = b"test_doc_v2";

        let doc = Doc::new();
        let text = doc.get_or_insert_text("test");
        {
            let mut txn = doc.transact_mut();
            text.push(&mut txn, "V2 content");
        }

        let txn = doc.transact();
        store.flush_doc_v2(doc_name, &txn).await.unwrap();

        let loaded_doc = Doc::new();
        let mut txn = loaded_doc.transact_mut();
        store.load_doc_v2(doc_name, &mut txn).await.unwrap();
        drop(txn);

        let loaded_text = loaded_doc.get_or_insert_text("test");
        let txn = loaded_doc.transact();
        let content = loaded_text.get_string(&txn);
        assert_eq!(content, "V2 content");
    }
}

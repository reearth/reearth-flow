use std::{collections::HashMap, fmt::Debug, sync::Arc};

use reearth_flow_types::AttributeValue;

#[nutype::nutype(
    sanitize(trim),
    derive(
        Debug,
        Clone,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        AsRef,
        Serialize,
        Deserialize,
        Hash,
        Display
    )
)]
pub struct KvStoreKey(String);

/// key-value store.
pub trait KvStore: Send + Sync + Debug {
    /// Fetch entry with given key.
    fn get(&self, key: &KvStoreKey) -> Option<AttributeValue>;

    /// Update entry with given key.
    ///
    /// If the database did not have this key present, [`None`] is returned.
    ///
    /// If the database did have this key present, the value is updated, and the old value is
    /// returned.
    ///
    /// [`None`]: std::option::Option
    fn insert(&mut self, key: KvStoreKey, value: AttributeValue);

    /// Remove entry with given key, returning the value at the key if the key was previously
    /// in the database.
    fn remove(&mut self, key: &KvStoreKey);

    /// Clone the KvStore.
    fn boxed_clone(&self) -> Box<dyn KvStore>;
}

impl<T: ?Sized + KvStore> KvStore for Box<T> {
    fn get(&self, key: &KvStoreKey) -> Option<AttributeValue> {
        KvStore::get(&**self, key)
    }

    fn insert(&mut self, key: KvStoreKey, value: AttributeValue) {
        KvStore::insert(&mut **self, key, value)
    }

    fn remove(&mut self, key: &KvStoreKey) {
        KvStore::remove(&mut **self, key)
    }

    fn boxed_clone(&self) -> Box<dyn KvStore> {
        KvStore::boxed_clone(&**self)
    }
}

pub fn create_kv_store() -> Box<dyn KvStore> {
    Box::new(MemoryKvStore::new())
}

#[derive(Debug, Default, Clone)]
pub struct MemoryKvStore(Arc<parking_lot::RwLock<HashMap<KvStoreKey, AttributeValue>>>);

impl MemoryKvStore {
    pub fn new() -> Self {
        MemoryKvStore(Arc::new(parking_lot::RwLock::new(HashMap::new())))
    }
}

impl KvStore for MemoryKvStore {
    fn get(&self, key: &KvStoreKey) -> Option<AttributeValue> {
        self.0.read().get(key).cloned()
    }

    fn insert(&mut self, key: KvStoreKey, value: AttributeValue) {
        self.0.write().insert(key, value);
    }

    fn remove(&mut self, key: &KvStoreKey) {
        self.0.write().remove(key);
    }

    fn boxed_clone(&self) -> Box<dyn KvStore> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
#[path = "kvs_test.rs"]
mod kvs_test;

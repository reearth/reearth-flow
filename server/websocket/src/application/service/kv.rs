use crate::{domain::repository::kv::KVStore, infrastructure::persistence::kv::DocOps};

pub struct KVService<T: KVStore> {
    store: T,
}

impl<T: KVStore> KVService<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }
}

// Implement KVStore trait to delegate to the underlying store
impl<T: KVStore> KVStore for KVService<T> {
    type Error = T::Error;
    type Return = T::Return;
    type Cursor = T::Cursor;
    type Entry = T::Entry;

    async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error> {
        self.store.get(key).await
    }

    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        self.store.upsert(key, value).await
    }

    async fn batch_upsert(&self, entries: &[(&[u8], &[u8])]) -> Result<(), Self::Error> {
        self.store.batch_upsert(entries).await
    }

    async fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        self.store.remove(key).await
    }

    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error> {
        self.store.remove_range(from, to).await
    }

    async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error> {
        self.store.iter_range(from, to).await
    }

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error> {
        self.store.peek_back(key).await
    }
}

impl<'a, T: KVStore> DocOps<'a> for KVService<T> where
    crate::application::service::kv::Error: From<T::Error>
{
}

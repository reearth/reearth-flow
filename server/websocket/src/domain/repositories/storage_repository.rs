use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait KVStore: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type Cursor: Iterator<Item = Self::Entry> + Send;
    type Entry: KVEntry + Send;
    type Return: AsRef<[u8]> + Send;

    async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error>;

    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    async fn batch_upsert(&self, entries: &[(&[u8], &[u8])]) -> Result<(), Self::Error> {
        for (key, value) in entries {
            self.upsert(key, value).await?;
        }
        Ok(())
    }

    async fn remove(&self, key: &[u8]) -> Result<(), Self::Error>;

    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error>;

    async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error>;

    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error>;
}

pub trait KVEntry {
    fn key(&self) -> &[u8];
    fn value(&self) -> &[u8];
}

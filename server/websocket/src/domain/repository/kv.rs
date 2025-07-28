use async_trait::async_trait;

/// Trait used by [KVStore] to define key-value entry tuples returned by cursor iterators.
pub trait KVEntry {
    /// Returns a key of current entry.
    fn key(&self) -> &[u8];
    /// Returns a value of current entry.
    fn value(&self) -> &[u8];
}

#[async_trait]
pub trait KVStore: Send + Sync {
    /// Error type returned from the implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Cursor type used to iterate over the ordered range of key-value entries.
    type Cursor: Iterator<Item = Self::Entry> + Send;
    /// Entry type returned by cursor.
    type Entry: KVEntry + Send;
    /// Type returned from the implementation.
    type Return: AsRef<[u8]> + Send;

    /// Return a value stored under given `key` or `None` if key was not found.
    async fn get(&self, key: &[u8]) -> Result<Option<Self::Return>, Self::Error>;

    /// Insert a new `value` under given `key` or replace an existing value with new one if
    /// entry with that `key` already existed.
    async fn upsert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    /// Batch insert or update multiple key-value pairs.
    async fn batch_upsert(&self, entries: &[(&[u8], &[u8])]) -> Result<(), Self::Error> {
        // Default implementation processes entries one by one
        for (key, value) in entries {
            self.upsert(key, value).await?;
        }
        Ok(())
    }

    /// Return a value stored under the given `key` if it exists.
    async fn remove(&self, key: &[u8]) -> Result<(), Self::Error>;

    /// Remove all keys between `from`..=`to` range of keys.
    async fn remove_range(&self, from: &[u8], to: &[u8]) -> Result<(), Self::Error>;

    /// Return an iterator over all entries between `from`..=`to` range of keys.
    async fn iter_range(&self, from: &[u8], to: &[u8]) -> Result<Self::Cursor, Self::Error>;

    /// Looks into the last entry value prior to a given key. The provided key parameter may not
    /// exist and it's used only to establish cursor position in ordered key collection.
    ///
    /// In example: in a key collection of `{1,2,5,7}`, this method with the key parameter of `4`
    /// should return value of `2`.
    async fn peek_back(&self, key: &[u8]) -> Result<Option<Self::Entry>, Self::Error>;
}

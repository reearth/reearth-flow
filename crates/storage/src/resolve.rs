use reearth_flow_common::uri::Uri;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::operator::resolve_operator;
use crate::storage::Storage;

#[derive(Debug, Default, Clone)]
pub struct StorageResolver {
    storages: Arc<RwLock<HashMap<Uri, Arc<Storage>>>>,
}

impl StorageResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves the given URI.
    pub fn resolve(&self, uri: &Uri) -> anyhow::Result<Arc<Storage>> {
        let storages = self.storages.read().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(storage) = storages.get(&uri.root_uri()) {
            return Ok(Arc::clone(storage));
        }
        drop(storages);
        let mut storages = self
            .storages
            .write()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let op = resolve_operator(uri)?;
        let storage = Arc::new(Storage::new(uri.root_uri(), op));
        storages.insert(uri.root_uri(), Arc::clone(&storage));
        Ok(storage)
    }
}

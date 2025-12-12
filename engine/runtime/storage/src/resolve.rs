use reearth_flow_common::uri::Uri;
use std::collections::HashMap;
use std::sync::Arc;

use crate::operator::resolve_operator;
use crate::storage::Storage;

#[derive(Debug, Default, Clone)]
pub struct StorageResolver {
    storages: Arc<parking_lot::RwLock<HashMap<Uri, Arc<Storage>>>>,
}

impl StorageResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves the given URI.
    pub fn resolve(&self, uri: &Uri) -> crate::Result<Arc<Storage>> {
        let storages = self.storages.read();
        if let Some(storage) = storages.get(&uri.root_uri()) {
            return Ok(Arc::clone(storage));
        }
        drop(storages);
        let mut storages = self.storages.write();
        let op = resolve_operator(uri).map_err(|e| crate::Error::Resolve(format!("{e}")))?;
        let storage = Arc::new(Storage::new(uri.root_uri(), op));
        storages.insert(uri.root_uri(), Arc::clone(&storage));
        Ok(storage)
    }
}

#[cfg(test)]
#[path = "resolve_test.rs"]
mod resolve_test;

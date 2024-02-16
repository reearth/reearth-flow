use reearth_flow_common::uri::Uri;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::operator::resolve_operator;
use crate::storage::Storage;

#[derive(Debug, Default, Clone)]
pub struct StorageResolver {
    storages: Arc<Mutex<HashMap<Uri, Arc<Storage>>>>,
}

impl StorageResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves the given URI.
    pub fn resolve(&self, uri: &Uri) -> anyhow::Result<Arc<Storage>> {
        let mut storages = self.storages.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(storage) = storages.get(&uri.root_uri()) {
            return Ok(Arc::clone(storage));
        }
        let op = resolve_operator(uri)?;
        let storage = Arc::new(Storage::new(uri.root_uri(), op));
        storages.insert(uri.root_uri(), Arc::clone(&storage));
        Ok(storage)
    }
}

use crate::domain::services::kv::DocOps;
use std::sync::Arc;
pub struct AwarenessServer<D>
where
    D: for<'a> DocOps<'a>,
{
    storage: Arc<D>,
}

impl<D: for<'a> DocOps<'a>> AwarenessServer<D> {
    pub fn new(storage: Arc<D>) -> Self {
        Self { storage }
    }
    pub fn storage(&self) -> &Arc<D> {
        &self.storage
    }
}

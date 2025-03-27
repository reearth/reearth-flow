use std::sync::Arc;

use crate::BroadcastGroup;

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub storage_enabled: bool,
    pub doc_name: Option<String>,
}

#[derive(Debug)]
pub struct BroadcastGroupContext {
    pub group: Arc<BroadcastGroup>,
}

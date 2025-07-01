use crate::infrastructure::BroadcastPool;
use std::sync::Arc;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
    pub instance_id: String,
}

impl AppState {
    pub fn new(pool: Arc<BroadcastPool>, instance_id: String) -> Self {
        Self { pool, instance_id }
    }
}

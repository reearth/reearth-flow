use serde::Deserialize;

/// 回滚请求
#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub clock: u32,
}

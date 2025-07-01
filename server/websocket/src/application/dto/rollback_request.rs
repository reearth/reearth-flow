use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub clock: u32,
}

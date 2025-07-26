use serde::Deserialize;
use time::OffsetDateTime;
use yrs::Update;

#[derive(Debug)]
pub struct UpdateInfo {
    pub update: Update,
    pub clock: u32,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GcsConfig {
    pub bucket_name: String,
    pub endpoint: Option<String>,
}

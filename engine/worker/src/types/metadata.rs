use std::collections::HashMap;

use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub job_id: Uuid,
    pub assets: Asset,
    pub timestamps: Timestamp,
    pub tags: Option<Vec<String>>,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub(crate) base_url: String,
    pub(crate) files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetFile {
    name: String,
    path: String,
    checksum: Option<String>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Timestamp {
    created: chrono::DateTime<Utc>,
    updated: Option<chrono::DateTime<Utc>>,
}

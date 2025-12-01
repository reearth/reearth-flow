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
    pub artifact_base_url: String,
    pub timestamps: Timestamp,
    pub tags: Option<Vec<String>>,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub base_url: String,
    pub files: Vec<String>,
}

impl Asset {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetFile {
    pub name: String,
    pub path: String,
    pub checksum: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Timestamp {
    pub created: chrono::DateTime<Utc>,
    pub updated: Option<chrono::DateTime<Utc>>,
}

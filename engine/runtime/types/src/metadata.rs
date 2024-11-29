use serde::{Deserialize, Serialize};

use crate::lod::LodMask;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub feature_id: Option<String>,
    pub feature_type: Option<String>,
    pub lod: Option<LodMask>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }
}

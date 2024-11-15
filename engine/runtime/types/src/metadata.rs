use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub feature_id: Option<String>,
    pub feature_type: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self::default()
    }
}

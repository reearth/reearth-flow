use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct SchemaFeature {
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) min_occurs: String,
    pub(super) max_occurs: String,
    pub(super) flag: Option<String>,
    pub(super) children: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct Schema {
    pub(super) features: HashMap<String, Vec<SchemaFeature>>,
    pub(super) complex_types: HashMap<String, Vec<SchemaFeature>>,
}

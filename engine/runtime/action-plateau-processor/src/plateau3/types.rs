use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SchemaFeature {
    pub(crate) name: String,
    pub(crate) r#type: String,
    pub(crate) min_occurs: String,
    pub(crate) max_occurs: String,
    pub(crate) flag: Option<String>,
    pub(crate) children: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Schema {
    pub(crate) features: HashMap<String, Vec<SchemaFeature>>,
    pub(crate) complex_types: HashMap<String, Vec<SchemaFeature>>,
}

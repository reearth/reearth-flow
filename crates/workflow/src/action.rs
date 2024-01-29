use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum Action {
    #[serde(rename = "featureReader")]
    FeatureReader,
}

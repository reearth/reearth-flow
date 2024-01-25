use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum Action {
    #[serde(rename = "featureReader")]
    FeatureReader,
}

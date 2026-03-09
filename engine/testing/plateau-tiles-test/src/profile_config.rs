use crate::cast_config::CastConfigValue;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ConvFlowSource {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct ConvMvtEntry {
    #[serde(default)]
    pub flow: Option<ConvFlowSource>,
    #[serde(default)]
    pub fme_path: Option<String>,
    #[serde(default)]
    pub truth_path: Option<String>,
    #[serde(default)]
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

#[derive(Debug, Deserialize)]
pub struct ConvMvtPngEntry {
    #[serde(default)]
    pub flow: Option<ConvFlowSource>,
    #[serde(default)]
    pub fme_path: Option<String>,
    pub truth_path: String,
    #[serde(default)]
    pub tiles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Convs {
    #[serde(default)]
    pub mvt: HashMap<String, ConvMvtEntry>,
    #[serde(default)]
    pub mvt_png: HashMap<String, ConvMvtPngEntry>,
}

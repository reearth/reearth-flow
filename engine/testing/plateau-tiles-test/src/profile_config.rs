use crate::cast_config::CastConfigValue;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ConvMvtEntry {
    pub path: String,
    pub truth_path: String,
    #[serde(default)]
    pub fme: bool,
    #[serde(default)]
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

#[derive(Debug, Deserialize)]
pub struct ConvMvtPngEntry {
    pub path: String,
    pub truth_path: String,
    #[serde(default)]
    pub fme: bool,
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

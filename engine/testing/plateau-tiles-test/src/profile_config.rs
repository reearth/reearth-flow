use crate::cast_config::CastConfigValue;
use crate::rasterize::{RasterSize, DEFAULT_STROKE};
use serde::Deserialize;
use std::collections::HashMap;

fn default_stroke() -> f64 {
    DEFAULT_STROKE
}

#[derive(Debug, Deserialize)]
pub struct ConvMvtEntry {
    pub path: String,
    pub truth_path: String,
    pub generate_truth: bool,
    #[serde(default)]
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

#[derive(Debug, Deserialize)]
pub struct ConvMvtPngEntry {
    pub path: String,
    pub truth_path: String,
    pub generate_truth: bool,
    #[serde(default)]
    pub tiles: Option<Vec<String>>,
    #[serde(default)]
    pub size: RasterSize,
    #[serde(default = "default_stroke")]
    pub stroke: f64,
}

#[derive(Debug, Deserialize)]
pub struct ConvJsonEntry {
    pub flow_path: String,
    pub output_path: String,
    pub generate_truth: bool,
    #[serde(default)]
    pub json_path: Option<String>,
    #[serde(default)]
    pub casts: HashMap<String, CastConfigValue>,
}

#[derive(Debug, Deserialize)]
pub struct ConvCesiumStatisticsEntry {
    /// Path to 3DTiles directory, relative to output_dir
    pub path: String,
    /// Path to truth JSON file, relative to testcase dir
    pub truth_path: String,
    pub generate_truth: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Convs {
    #[serde(default)]
    pub mvt_attributes: HashMap<String, ConvMvtEntry>,
    #[serde(default)]
    pub mvt_png: HashMap<String, ConvMvtPngEntry>,
    #[serde(default)]
    pub json: HashMap<String, ConvJsonEntry>,
    #[serde(default)]
    pub cesium_statistics: HashMap<String, ConvCesiumStatisticsEntry>,
}

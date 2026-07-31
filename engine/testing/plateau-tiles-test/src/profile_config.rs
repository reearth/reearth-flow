use crate::cast_config::CastConfigValue;
use crate::rasterize::{RasterSize, DEFAULT_STROKE, RASTER3D_SIZE};
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
pub struct ConvCesiumEntry {
    /// Path to 3DTiles directory, relative to flow_extracted
    pub path: String,
    /// Path to truth JSON file, relative to flow_extracted
    pub truth_path: String,
    pub generate_truth: bool,
    #[serde(default)]
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    /// ECEF xyz, meters.
    pub position: [f64; 3],
    /// ECEF xyz, meters.
    pub look_at: [f64; 3],
    /// ECEF xyz direction. Defaults to the local zenith (`position` normalized)
    /// if omitted.
    #[serde(default)]
    pub up: Option<[f64; 3]>,
    pub fov_y_deg: f64,
    pub near: f64,
    pub far: f64,
}

fn default_raster3d_size() -> RasterSize {
    RasterSize::Square(RASTER3D_SIZE)
}

#[derive(Debug, Deserialize)]
pub struct ConvRaster3dEntry {
    /// Path to 3D Tiles directory, relative to flow_extracted/truth_extracted.
    pub path: String,
    /// Path to rendered-depth-PNG output directory, relative to flow_extracted/truth.
    pub truth_path: String,
    pub generate_truth: bool,
    #[serde(default = "default_raster3d_size")]
    pub size: RasterSize,
    pub cameras: HashMap<String, CameraConfig>,
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
    pub cesium_attributes: HashMap<String, ConvCesiumEntry>,
    #[serde(default)]
    pub cesium_statistics: HashMap<String, ConvCesiumStatisticsEntry>,
    #[serde(default)]
    pub raster3d: HashMap<String, ConvRaster3dEntry>,
}

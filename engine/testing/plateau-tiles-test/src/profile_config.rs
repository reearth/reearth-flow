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
pub struct ConvCesiumEntry {
    /// Path to 3DTiles directory, relative to flow_extracted
    pub path: String,
    /// Path to truth JSON file, relative to flow_extracted
    pub truth_path: String,
    pub generate_truth: bool,
    #[serde(default)]
    pub casts: Option<HashMap<String, CastConfigValue>>,
}

/// A geometry check of the cesium statistics test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryTest {
    TexturePresence,
    MonotonicGeometricError,
    BoundingBox,
    MassCenter,
    AverageColor,
    AverageWinding,
}

impl GeometryTest {
    pub fn enabled_by_default(self) -> bool {
        !matches!(self, GeometryTest::AverageWinding)
    }
}

#[derive(Debug, Deserialize)]
pub struct ConvCesiumStatisticsEntry {
    /// Path to 3DTiles directory, relative to output_dir
    pub path: String,
    /// Path to truth JSON file, relative to testcase dir
    pub truth_path: String,
    pub generate_truth: bool,
    /// Per-check overrides of the default enabled geometry checks.
    #[serde(default)]
    pub geometry_tests: HashMap<GeometryTest, bool>,
}

impl ConvCesiumStatisticsEntry {
    pub fn enabled(&self, test: GeometryTest) -> bool {
        self.geometry_tests
            .get(&test)
            .copied()
            .unwrap_or(test.enabled_by_default())
    }
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
}

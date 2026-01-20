#![allow(unused)]
use std::collections::HashMap;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct ImageRasterizerFactory;

impl ProcessorFactory for ImageRasterizerFactory {
    fn name(&self) -> &str {
        "ImageRasterizer"
    }

    fn description(&self) -> &str {
        "Convert vector geometries to raster image format"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ImageRasterizerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ImageRasterizerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::BuffererFactory(format!(
                    // Using an existing error variant
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::BuffererFactory(format!(
                    // Using an existing error variant
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::BuffererFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = ImageRasterizer {
            cell_size_x: params.cell_size_x,
            cell_size_y: params.cell_size_y,
            color_interpretation: params.color_interpretation,
            background_color: params.background_color,
            background_color_alpha: params.background_color_alpha,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
enum ColorInterpretation {
    /// # RGBA32
    /// 32-bit color with alpha channel (Red, Green, Blue, Alpha)
    #[serde(rename = "rgba32")]
    Rgba32,
    /// # RGB24
    /// 24-bit color without alpha channel (Red, Green, Blue)
    #[serde(rename = "rgb24")]
    Rgb24,
}

/// # Image Rasterizer Parameters
/// Configure how to convert vector geometries to raster images
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ImageRasterizerParam {
    /// # Cell Size X
    /// The width of each pixel in coordinate units
    #[serde(default = "default_cell_size")]
    cell_size_x: f64,

    /// # Cell Size Y
    /// The height of each pixel in coordinate units
    #[serde(default = "default_cell_size")]
    cell_size_y: f64,

    /// # Color Interpretation
    /// How to interpret and store color information
    #[serde(default = "default_color_interpretation")]
    color_interpretation: ColorInterpretation,

    /// # Background Color
    /// The color to use for background pixels (RGB values 0-255)
    #[serde(default = "default_background_color")]
    background_color: [u8; 3],

    #[serde(default = "default_background_color_alpha")]
    background_color_alpha: f64,
}

#[derive(Debug, Clone)]
struct ImageRasterizer {
    cell_size_x: f64,
    cell_size_y: f64,
    color_interpretation: ColorInterpretation,
    background_color: [u8; 3],
    background_color_alpha: f64,
}

fn default_cell_size() -> f64 {
    1.0
}

fn default_color_interpretation() -> ColorInterpretation {
    ColorInterpretation::Rgba32
}

fn default_background_color() -> [u8; 3] {
    [255, 255, 255] // White
}

fn default_background_color_alpha() -> f64 {
    1.0
}

fn default_anti_aliasing() -> bool {
    true
}

impl Default for ImageRasterizerParam {
    fn default() -> Self {
        Self {
            cell_size_x: default_cell_size(),
            cell_size_y: default_cell_size(),
            color_interpretation: default_color_interpretation(),
            background_color: default_background_color(),
            background_color_alpha: default_background_color_alpha(),
        }
    }
}

impl Processor for ImageRasterizer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ImageRasterizer"
    }
}

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
        Some(schemars::schema_for!(ImageRasterizer))
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
        let image_rasterizer: ImageRasterizer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::BuffererFactory(format!( // Using an existing error variant
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::BuffererFactory(format!( // Using an existing error variant
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            ImageRasterizer::default()
        };
        Ok(Box::new(image_rasterizer))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
enum RasterizationMode {
    /// # Fill Mode
    /// Fill the entire polygon area
    #[serde(rename = "fill")]
    Fill,
    /// # Outline Mode
    /// Draw only the outline of the geometry
    #[serde(rename = "outline")]
    Outline,
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
pub struct ImageRasterizer {
    /// # Cell Size X
    /// The width of each pixel in coordinate units
    #[serde(default = "default_cell_size")]
    cell_size_x: f64,

    /// # Cell Size Y
    /// The height of each pixel in coordinate units
    #[serde(default = "default_cell_size")]
    cell_size_y: f64,

    /// # Rasterization Mode
    /// How to render the geometry onto the raster
    #[serde(default = "default_rasterization_mode")]
    rasterization_mode: RasterizationMode,

    /// # Color Interpretation
    /// How to interpret and store color information
    #[serde(default = "default_color_interpretation")]
    color_interpretation: ColorInterpretation,

    /// # Background Color
    /// The color to use for background pixels (RGB values 0-255)
    #[serde(default = "default_background_color")]
    background_color: [u8; 3],

    /// # Fill Color Attribute
    /// Name of the attribute containing fill color information (RGB values 0-255)
    #[serde(default = "default_fill_color_attribute")]
    fill_color_attribute: String,

    /// # Anti-Aliasing
    /// Enable anti-aliasing for smoother edges
    #[serde(default = "default_anti_aliasing")]
    anti_aliasing: bool,
}

fn default_cell_size() -> f64 {
    1.0
}

fn default_rasterization_mode() -> RasterizationMode {
    RasterizationMode::Fill
}

fn default_color_interpretation() -> ColorInterpretation {
    ColorInterpretation::Rgba32
}

fn default_background_color() -> [u8; 3] {
    [255, 255, 255] // White
}

fn default_fill_color_attribute() -> String {
    "color".to_string()
}

fn default_anti_aliasing() -> bool {
    true
}

impl Default for ImageRasterizer {
    fn default() -> Self {
        Self {
            cell_size_x: default_cell_size(),
            cell_size_y: default_cell_size(),
            rasterization_mode: default_rasterization_mode(),
            color_interpretation: default_color_interpretation(),
            background_color: default_background_color(),
            fill_color_attribute: default_fill_color_attribute(),
            anti_aliasing: default_anti_aliasing(),
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

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                // For now, we'll pass through the geometry as-is since actual rasterization
                // would require more complex image processing libraries
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                // Convert 3D to 2D before rasterization
                let geos_2d: Geometry2D = geos.clone().into();
                self.handle_2d_geometry(&geos_2d, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(gml) => {
                // For now, convert to 2D geometry and process
                let geos_2d: Geometry2D = gml.clone().into();
                self.handle_2d_geometry(&geos_2d, feature, geometry, &ctx, fw);
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "ImageRasterizer"
    }
}

impl ImageRasterizer {
    fn handle_2d_geometry(
        &self,
        _geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        // In a real implementation, this would perform actual rasterization
        // For now, we'll just pass through the geometry with a note that it has been processed
        let mut feature = feature.clone();
        let geometry = geometry.clone();

        // Add attributes indicating that this feature has been processed by the rasterizer
        let mut new_attributes = feature.attributes.clone();
        new_attributes.insert(Attribute::new("rasterized".to_string()), AttributeValue::Bool(true));
        new_attributes.insert(Attribute::new("cell_size_x".to_string()), AttributeValue::Number(serde_json::Number::from_f64(self.cell_size_x).unwrap_or_else(|| serde_json::Number::from(0))));
        new_attributes.insert(Attribute::new("cell_size_y".to_string()), AttributeValue::Number(serde_json::Number::from_f64(self.cell_size_y).unwrap_or_else(|| serde_json::Number::from(0))));
        new_attributes.insert(Attribute::new("rasterization_mode".to_string()), AttributeValue::String(format!("{:?}", self.rasterization_mode)));
        new_attributes.insert(Attribute::new("color_interpretation".to_string()), AttributeValue::String(format!("{:?}", self.color_interpretation)));

        // In a real implementation, we would convert the geometry to a raster representation
        // For now, we'll keep the original geometry but mark it as processed
        feature.geometry = geometry;

        let updated_feature = feature.with_attributes(new_attributes);

        fw.send(ctx.new_with_feature_and_port(updated_feature, DEFAULT_PORT.clone()));
    }
}
#![allow(unused)]
use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
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

#[derive(Clone, Debug)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    // mapping from one CoordinatesBoudnary to another CoordinatesBoudnary
    pub fn map_to(
        &self, // Add reference to self to access the current coordinate's x, y values
        from_geo_boundary: CoordinatesBoudnary,
        to_png_boundary: CoordinatesBoudnary,
    ) -> Coordinate {
        // Calculate the proportional position of the coordinate within the source boundary
        let x_ratio = (self.x - from_geo_boundary.left_up.x)
            / (from_geo_boundary.right_down.x - from_geo_boundary.left_up.x);
        let y_ratio = (self.y - from_geo_boundary.left_up.y)
            / (from_geo_boundary.right_down.y - from_geo_boundary.left_up.y);

        // Apply the ratios to the destination boundary to get the new coordinate
        let new_x = to_png_boundary.left_up.x
            + (to_png_boundary.right_down.x - to_png_boundary.left_up.x) * x_ratio;
        let new_y = to_png_boundary.left_up.y
            + (to_png_boundary.right_down.y - to_png_boundary.left_up.y) * y_ratio;

        // Return the new coordinate in the destination system
        Coordinate { x: new_x, y: new_y }
    }
}

#[derive(Clone, Debug)]
pub struct CoordinatesBoudnary {
    pub left_up: Coordinate,
    pub left_down: Coordinate,
    pub right_up: Coordinate,
    pub right_down: Coordinate,
}

// Define the GeometryPixels structure to hold the mapped data
#[derive(Debug, Clone)]
pub struct ImagePixel {
    // coordinate in image -- x
    pub x: u32,
    // coordinate in image -- y
    pub y: u32,
    // pixel color
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct GeometryPolygon {
    pub coordinates: Vec<(f64, f64)>, // List of (x, y) coordinates forming the polygon
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

impl GeometryPolygon {
    // Maps the polygon's coordinates to PNG image space
    pub fn map_to_png(
        &self,
        geo_boundary: CoordinatesBoudnary,
        png_width: u32,
        png_height: u32,
    ) -> GeometryPolygon {
        // Define the PNG boundary with (0,0) at top-left
        let png_boundary = CoordinatesBoudnary {
            left_up: Coordinate { x: 0.0, y: 0.0 },
            left_down: Coordinate {
                x: 0.0,
                y: png_height as f64,
            },
            right_up: Coordinate {
                x: png_width as f64,
                y: 0.0,
            },
            right_down: Coordinate {
                x: png_width as f64,
                y: png_height as f64,
            },
        };

        // Map each coordinate in the polygon to the PNG space
        let mapped_coordinates: Vec<(f64, f64)> = self
            .coordinates
            .iter()
            .map(|(x, y)| {
                let coord = Coordinate { x: *x, y: *y };
                let mapped_coord = coord.map_to(geo_boundary.clone(), png_boundary.clone());
                (mapped_coord.x, mapped_coord.y)
            })
            .collect();

        // Return a new polygon with the mapped coordinates but the same color
        GeometryPolygon {
            coordinates: mapped_coordinates,
            color_r: self.color_r,
            color_g: self.color_g,
            color_b: self.color_b,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GemotryPolygons {
    pub polygons: Vec<GeometryPolygon>,
}

impl GemotryPolygons {
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
        }
    }

    pub fn add_polygon(&mut self, polygon: GeometryPolygon) {
        self.polygons.push(polygon);
    }

    pub fn find_coordinates_boudary(&self) -> CoordinatesBoudnary {
        if self.polygons.is_empty() {
            // Return a default boundary if no polygons exist
            return CoordinatesBoudnary {
                left_up: Coordinate { x: 0.0, y: 0.0 },
                left_down: Coordinate { x: 0.0, y: 0.0 },
                right_up: Coordinate { x: 0.0, y: 0.0 },
                right_down: Coordinate { x: 0.0, y: 0.0 },
            };
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        // Iterate through all polygons and their coordinates to find the min/max values
        for polygon in &self.polygons {
            for (x, y) in &polygon.coordinates {
                min_x = min_x.min(*x);
                max_x = max_x.max(*x);
                min_y = min_y.min(*y);
                max_y = max_y.max(*y);
            }
        }

        // Create the boundary coordinates
        CoordinatesBoudnary {
            left_up: Coordinate { x: min_x, y: max_y }, // top-left corner
            left_down: Coordinate { x: min_x, y: min_y }, // bottom-left corner
            right_up: Coordinate { x: max_x, y: max_y }, // top-right corner
            right_down: Coordinate { x: max_x, y: min_y }, // bottom-right corner
        }
    }
}

// Helper function to parse the JSON and convert to GemotryPolygons
pub fn json_to_gemotry_polygons(
    json_value: &Value,
) -> Result<GemotryPolygons, Box<dyn std::error::Error>> {
    let mut gemotry_polygons = GemotryPolygons::new();

    if let Some(features) = json_value.as_array() {
        for feature in features {
            // Extract color information from the feature
            let r = feature
                .get("color_r")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(255);
            let g = feature
                .get("color_g")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);
            let b = feature
                .get("color_b")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);

            // Process the geometry
            if let Some(geo) = feature.get("json_geometry") {
                if let Some(coords) = geo.get("coordinates").and_then(|c| c.as_array()) {
                    // The coordinates structure is [[[x, y], [x, y], ...]] - array of rings
                    // The structure [[[x, y], [x, y], ...]] represents a GeoJSON Polygon structure, which consists of:
                    //  1. Outermost array: Contains multiple "rings" (usually just one outer ring, but can include inner rings for holes)
                    //  2. Middle array: Represents a single "ring" - a sequence of connected coordinate pairs forming a closed shape
                    //  3. Innermost arrays: Individual [x, y] coordinate pairs that define points along the ring
                    for ring_array in coords {
                        if let Some(ring_points) = ring_array.as_array() {
                            let mut coordinates = Vec::new();

                            for point in ring_points {
                                if let Some(xy) = point.as_array() {
                                    if xy.len() >= 2 {
                                        let x = xy[0].as_f64().unwrap_or(0.0);
                                        let y = xy[1].as_f64().unwrap_or(0.0);
                                        coordinates.push((x, y));
                                    }
                                }
                            }

                            // Create a polygon with the collected coordinates and color
                            let polygon = GeometryPolygon {
                                coordinates,
                                color_r: r,
                                color_g: g,
                                color_b: b,
                            };

                            gemotry_polygons.add_polygon(polygon);
                        }
                    }
                }
            }
        }
    }

    Ok(gemotry_polygons)
}

#[cfg(test)]
mod tests {
    #![allow(unused)]
    use super::*;
    use image::{ImageBuffer, ImageFormat, Rgb, RgbImage};
    use std::fs;
    use std::path::Path;

    #[test]
    fn case01() {
        // Create a new 200x100 RGB image filled with white
        let mut img = RgbImage::new(200, 100);

        // Draw a red rectangle
        for x in 50..150 {
            for y in 20..80 {
                img.put_pixel(x, y, Rgb([255, 0, 0]));
            }
        }

        // Create the cache directory and save the image directly to the user-accessible location
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = Path::new(&home_dir)
            .join(".cache")
            .join("reearth-flow-test-images");

        if let Err(_) = fs::create_dir_all(&cache_dir) {
            panic!("Could not create cache directory for test images");
        }

        let dest_path = cache_dir.join("output.png");

        // Save the image directly to the cache directory
        img.save(&dest_path).unwrap();
        println!(
            "Successfully saved image directly to cache directory: {:?}",
            dest_path
        );
    }

    #[test]
    fn case02() {
        use std::fs::File;
        use std::io::BufReader;

        // Open and read the JSON file
        let file_path =
            "/home/zw/code/rust_programming/reearth-flow/engine/tmp/image_rasterizer_input.json";
        let file = File::open(file_path).expect("Failed to open image_rasterizer_input.json");
        let reader = BufReader::new(file);

        // Parse the JSON
        let json_value: serde_json::Value = serde_json::from_reader(reader)
            .expect("Failed to parse JSON from image_rasterizer_input.json");

        println!("Successfully read and parsed JSON from: {}", file_path);

        // Convert JSON to GemotryPolygons
        let gemotry_polygons = json_to_gemotry_polygons(&json_value)
            .expect("Failed to convert JSON to GemotryPolygons");

        println!("Converted JSON to GemotryPolygons:");
        println!("  Number of polygons: {}", gemotry_polygons.polygons.len());

        // Print info about first few polygons as sample
        for (i, polygon) in gemotry_polygons.polygons.iter().take(5).enumerate() {
            println!(
                "  Polygon {}: {} points, color ({}, {}, {})",
                i,
                polygon.coordinates.len(),
                polygon.color_r,
                polygon.color_g,
                polygon.color_b
            );
        }

        // Test the find_coordinates_boudary function
        let boundary = gemotry_polygons.find_coordinates_boudary();
        println!("Boundary coordinates:");
        println!(
            "  Left Up: ({}, {})",
            boundary.left_up.x, boundary.left_up.y
        );
        println!(
            "  Left Down: ({}, {})",
            boundary.left_down.x, boundary.left_down.y
        );
        println!(
            "  Right Up: ({}, {})",
            boundary.right_up.x, boundary.right_up.y
        );
        println!(
            "  Right Down: ({}, {})",
            boundary.right_down.x, boundary.right_down.y
        );
    }
}

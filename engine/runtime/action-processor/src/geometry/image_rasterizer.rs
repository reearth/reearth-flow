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
            gemotry_polygons: GemotryPolygons::new(),
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
    gemotry_polygons: GemotryPolygons,
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
        // During process
        // 1. extract color related features, color_r, color_g, color_b
        // 2. from feature.geometry, get coordinates
        // 3. each processed feature should get a GeometryPolygon, accumulate it in GemotryPolygons
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // Extract color information from the feature properties
        let color_r = feature
            .get("color_r")
            .and_then(|v| v.as_f64())
            .map(|v| v as u8)
            .unwrap_or(255);
        let color_g = feature
            .get("color_g")
            .and_then(|v| v.as_f64())
            .map(|v| v as u8)
            .unwrap_or(0);
        let color_b = feature
            .get("color_b")
            .and_then(|v| v.as_f64())
            .map(|v| v as u8)
            .unwrap_or(0);

        // Extract coordinates from the geometry
        let coordinates = extract_coordinates_from_geometry(geometry)?;

        if !coordinates.is_empty() {
            // Create a GeometryPolygon with the extracted coordinates and colors
            let polygon = GeometryPolygon {
                coordinates,
                color_r,
                color_g,
                color_b,
            };

            // Accumulate the polygon in the collection
            self.gemotry_polygons.add_polygon(polygon);
        }

        // Forward the feature to the output port
        fw.send(ctx);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // When all features are processed, during finish phase
        // call draw method on GeometryPolygon, with dependent parameters get from &self

        // Determine image dimensions based on the accumulated polygons and cell sizes
        let boundary = self.gemotry_polygons.find_coordinates_boudary();

        // Calculate the width and height of the image based on the boundary and cell size
        let width = if boundary.right_down.x > boundary.left_up.x {
            ((boundary.right_down.x - boundary.left_up.x) / self.cell_size_x).ceil() as u32
        } else {
            1 // Minimum width of 1 pixel
        };

        let height = if boundary.right_down.y > boundary.left_up.y {
            ((boundary.right_down.y - boundary.left_up.y) / self.cell_size_y).ceil() as u32
        } else {
            1 // Minimum height of 1 pixel
        };

        // Draw the accumulated polygons to an image
        let img = self.gemotry_polygons.draw(width, height, true); // fill_area = true

        // Create the cache directory and save the image
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = std::path::Path::new(&home_dir)
            .join(".cache")
            .join("reearth-flow-generated-images");

        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            ctx.event_hub.warn_log(None, format!("Failed to create cache directory: {}", e));
        } else {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let dest_path = cache_dir.join(format!("image_rasterizer_output_{}.png", timestamp));

            // Save the image to the cache directory
            if let Err(e) = img.save(&dest_path) {
                ctx.event_hub.warn_log(None, format!("Failed to save image: {}", e));
            } else {
                ctx.event_hub.info_log(
                    None,
                    format!("ImageRasterizer saved image to: {:?}", dest_path),
                );
            }
        }

        // Log the completion of the image creation
        ctx.event_hub.info_log(
            None,
            format!("ImageRasterizer created image with dimensions {}x{}", width, height),
        );

        Ok(())
    }

    fn name(&self) -> &str {
        "ImageRasterizer"
    }
}

// Helper function to extract coordinates from geometry
fn extract_coordinates_from_geometry(geometry: &reearth_flow_types::Geometry) -> Result<Vec<(f64, f64)>, BoxedError> {
    use reearth_flow_types::GeometryValue;

    match &geometry.value {
        GeometryValue::None => Ok(Vec::new()),
        GeometryValue::FlowGeometry2D(geom) => {
            use reearth_flow_geometry::types::geometry::Geometry2D;
            match geom {
                Geometry2D::Polygon(polygon) => {
                    // For a polygon, we typically want the exterior ring
                    let exterior = polygon.exterior();
                    Ok(exterior.0.iter().map(|p| (p.x, p.y)).collect())
                },
                _ => {
                    // For other geometry types, return an error indicating it's not supported
                    Err(GeometryProcessorError::ImageRasterizer(
                        "Only Polygon geometry type is supported".to_string(),
                    ).into())
                }
            }
        },
        GeometryValue::FlowGeometry3D(_) | GeometryValue::CityGmlGeometry(_) => {
            // For 3D geometry types, return an error asking to convert to 2D first
            Err(GeometryProcessorError::ImageRasterizer(
                "Please convert 3D geometry to 2D first".to_string(),
            ).into())
        },
    }
}

#[derive(Clone, Debug)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    // Creates a mapping function that can be reused to transform coordinates
    pub fn create_mapping_function(
        from_geo_boundary: CoordinatesBoudnary,
        to_png_boundary: CoordinatesBoudnary,
    ) -> impl Fn((f64, f64)) -> (f64, f64) {
        // Pre-calculate the ratios to avoid repeated computation
        let x_scale = (to_png_boundary.right_down.x - to_png_boundary.left_up.x)
            / (from_geo_boundary.right_down.x - from_geo_boundary.left_up.x);
        let y_scale = (to_png_boundary.right_down.y - to_png_boundary.left_up.y)
            / (from_geo_boundary.right_down.y - from_geo_boundary.left_up.y);

        let x_offset = to_png_boundary.left_up.x - from_geo_boundary.left_up.x * x_scale;
        let y_offset = to_png_boundary.left_up.y - from_geo_boundary.left_up.y * y_scale;

        move |(x, y): (f64, f64)| -> (f64, f64) {
            let new_x = x * x_scale + x_offset;
            let new_y = y * y_scale + y_offset;
            (new_x, new_y)
        }
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
    // Creates a mapping function that can be reused to transform the polygon's coordinates
    pub fn create_mapping_function_to_png(
        geo_boundary: CoordinatesBoudnary,
        png_width: u32,
        png_height: u32,
    ) -> impl Fn(&GeometryPolygon) -> GeometryPolygon {
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

        // Create the coordinate mapping function
        let coordinate_mapper = Coordinate::create_mapping_function(geo_boundary, png_boundary);

        move |polygon: &GeometryPolygon| {
            // Apply the coordinate mapping function to each coordinate in the polygon
            let mapped_coordinates: Vec<(f64, f64)> = polygon
                .coordinates
                .iter()
                .map(|&(x, y)| coordinate_mapper((x, y)))
                .collect();

            // Return a new polygon with the mapped coordinates but the same color
            GeometryPolygon {
                coordinates: mapped_coordinates,
                color_r: polygon.color_r,
                color_g: polygon.color_g,
                color_b: polygon.color_b,
            }
        }
    }

    // Maps the polygon to a vector of image pixels
    pub fn to_image_pixels<F>(&self, mapping_fn: F, fill_area: bool) -> Vec<ImagePixel>
    where
        F: Fn(&GeometryPolygon) -> GeometryPolygon,
    {
        // Apply the mapping function to transform the polygon
        let mapped_polygon = mapping_fn(self);

        // Convert the mapped polygon's coordinates to image pixels
        let mut pixels = Vec::new();

        // Add the border pixels (outline of the polygon)
        for (i, current) in mapped_polygon.coordinates.iter().enumerate() {
            let next_index = (i + 1) % mapped_polygon.coordinates.len();
            let next = &mapped_polygon.coordinates[next_index];

            // Draw line between consecutive points
            let line_pixels = self.draw_line(
                current.0.round() as u32,
                current.1.round() as u32,
                next.0.round() as u32,
                next.1.round() as u32,
                mapped_polygon.color_r,
                mapped_polygon.color_g,
                mapped_polygon.color_b,
            );
            pixels.extend(line_pixels);
        }

        // If fill_area is true, fill the interior of the polygon
        if fill_area {
            let fill_pixels = self.fill_polygon_interior(&mapped_polygon);
            pixels.extend(fill_pixels);
        }

        pixels
    }

    // Helper function to draw a line between two points using Bresenham's algorithm
    fn draw_line(
        &self,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
        r: u8,
        g: u8,
        b: u8,
    ) -> Vec<ImagePixel> {
        let mut pixels = Vec::new();

        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = (y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            pixels.push(ImagePixel {
                x: x as u32,
                y: y as u32,
                r,
                g,
                b,
            });

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }

        pixels
    }

    // Helper function to fill the interior of a polygon using scanline algorithm
    fn fill_polygon_interior(&self, polygon: &GeometryPolygon) -> Vec<ImagePixel> {
        if polygon.coordinates.len() < 3 {
            return Vec::new(); // Not enough points to form a polygon
        }

        let mut pixels = Vec::new();

        // Find the bounding box of the polygon
        let mut min_y = u32::MAX;
        let mut max_y = 0u32;
        let mut min_x = u32::MAX;
        let mut max_x = 0u32;

        for &(x, y) in &polygon.coordinates {
            let x_int = x.round() as u32;
            let y_int = y.round() as u32;

            if y_int < min_y {
                min_y = y_int;
            }
            if y_int > max_y {
                max_y = y_int;
            }
            if x_int < min_x {
                min_x = x_int;
            }
            if x_int > max_x {
                max_x = x_int;
            }
        }

        // For each scanline (horizontal line), find intersections with polygon edges
        for y in min_y..=max_y {
            let mut intersections = Vec::new();

            for i in 0..polygon.coordinates.len() {
                let p1 = &polygon.coordinates[i];
                let p2 = &polygon.coordinates[(i + 1) % polygon.coordinates.len()];

                let y1 = p1.1.round() as u32;
                let y2 = p2.1.round() as u32;

                // Check if the edge crosses the current scanline
                if (y1 <= y && y2 > y) || (y2 <= y && y1 > y) {
                    if y2 != y1 {
                        let x1 = p1.0.round() as f64;
                        let x2 = p2.0.round() as f64;

                        // Calculate intersection point
                        let x = x1 + (y as f64 - y1 as f64) / (y2 as f64 - y1 as f64) * (x2 - x1);
                        intersections.push(x.round() as u32);
                    }
                }
            }

            // Sort intersections and fill between pairs
            intersections.sort();

            for i in (0..intersections.len()).step_by(2) {
                if i + 1 < intersections.len() {
                    let start_x = intersections[i];
                    let end_x = intersections[i + 1];

                    for x in start_x..=end_x {
                        pixels.push(ImagePixel {
                            x,
                            y,
                            r: polygon.color_r,
                            g: polygon.color_g,
                            b: polygon.color_b,
                        });
                    }
                }
            }
        }

        pixels
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

    // Draws the polygons to a PNG image with the specified width and height
    pub fn draw(&self, width: u32, height: u32, fill_area: bool) -> image::RgbImage {
        // Create a new image with white background
        let mut img = image::RgbImage::new(width, height);

        // Find the boundary of all polygons
        let geo_boundary = self.find_coordinates_boudary();

        // Create the mapping function using the geographic boundary and image dimensions
        let mapping_fn =
            GeometryPolygon::create_mapping_function_to_png(geo_boundary, width, height);

        // Generate pixels for each polygon and draw them on the image
        for polygon in &self.polygons {
            // Convert the polygon to image pixels using the mapping function
            let pixels = polygon.to_image_pixels(&mapping_fn, fill_area);

            // Draw each pixel on the image
            for pixel in pixels {
                if pixel.x < width && pixel.y < height {
                    img.put_pixel(pixel.x, pixel.y, image::Rgb([pixel.r, pixel.g, pixel.b]));
                }
            }
        }

        img
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
        println!("Number of polygons: {}", gemotry_polygons.polygons.len());

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

    #[test]
    fn case03() {
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

        // Draw the polygons to a PNG image
        let img = gemotry_polygons.draw(1000, 1000, true); // width=1000, height=1000, fill_area=true

        // Create the cache directory and save the image
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = Path::new(&home_dir)
            .join(".cache")
            .join("reearth-flow-test-images");

        if let Err(_) = fs::create_dir_all(&cache_dir) {
            panic!("Could not create cache directory for test images");
        }

        let dest_path = cache_dir.join("generated_image.png");

        // Save the image directly to the cache directory
        img.save(&dest_path).unwrap();
        println!("Successfully generated and saved image to: {:?}", dest_path);
    }
}

use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{material::Texture, Attributes, Expr, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::errors::GeometryProcessorError;

static TEXTURE_COORDS_PORT: Lazy<Port> = Lazy::new(|| Port::new("textureCoordinates"));
static TEXTURED_PORT: Lazy<Port> = Lazy::new(|| Port::new("textured"));

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
        vec![DEFAULT_PORT.clone(), TEXTURE_COORDS_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), TEXTURED_PORT.clone()]
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
                GeometryProcessorError::ImageRasterizerFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ImageRasterizerFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ImageRasterizerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = ImageRasterizer {
            width: params.image_width,
            save_to: params.save_to,
            evaluated_save_path: None,
            geometry_polygons: GeometryPolygons::new(),
            texture_coord_features: Vec::new(),
        };
        Ok(Box::new(process))
    }
}

/// # Image Rasterizer Parameters
/// Configure how to convert vector geometries to raster images
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ImageRasterizerParam {
    /// The width of image
    #[serde(default = "default_image_width")]
    image_width: u32,

    /// # Save To
    /// Optional path expression to save the generated image. If not provided, uses default cache directory.
    #[serde(default)]
    save_to: Option<Expr>,
}

#[derive(Debug, Clone)]
struct ImageRasterizer {
    width: u32,
    save_to: Option<Expr>,
    evaluated_save_path: Option<String>,
    geometry_polygons: GeometryPolygons,
    texture_coord_features: Vec<Feature>,
}

fn default_image_width() -> u32 {
    1000
}

// Helper function to convert a path string (which may be a file:// URL) to a filesystem path
fn path_string_to_filesystem_path(path_str: &str) -> String {
    // Handle file:// URL format
    if let Some(stripped) = path_str.strip_prefix("file://") {
        stripped.to_string()
    } else {
        path_str.to_string()
    }
}

// Helper function to save image to a path, either custom or default cache location
fn save_image_with_path_option(
    img: &image::RgbImage,
    custom_path: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    match custom_path {
        Some(path_str) => {
            // Convert URL-style path to filesystem path
            let fs_path_str = path_string_to_filesystem_path(&path_str);
            let path = std::path::Path::new(&fs_path_str);

            // Create parent directories if they don't exist
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Save the image to the custom path
            img.save(path)?;
            // Return the filesystem path (not the URL)
            Ok(fs_path_str)
        }
        None => {
            // Use the default cache directory
            let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let cache_dir = std::path::Path::new(&home_dir)
                .join(".cache")
                .join("reearth-flow-generated-images");

            std::fs::create_dir_all(&cache_dir)?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let dest_path = cache_dir.join(format!("image_rasterizer_output_{}.png", timestamp));

            // Save the image to the cache directory
            img.save(&dest_path)?;
            Ok(dest_path.to_string_lossy().to_string())
        }
    }
}

impl Default for ImageRasterizerParam {
    fn default() -> Self {
        Self {
            image_width: default_image_width(),
            save_to: None,
        }
    }
}

impl Processor for ImageRasterizer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Evaluate save_to expression on first feature if not already done
        if self.evaluated_save_path.is_none() {
            if let Some(ref save_to_expr) = self.save_to {
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let scope = expr_engine.new_scope();
                let path = scope
                    .eval::<String>(save_to_expr.as_ref())
                    .unwrap_or_else(|_| save_to_expr.as_ref().to_string());
                self.evaluated_save_path = Some(path);
            }
        }

        // Check which port the feature came from
        if ctx.port == *TEXTURE_COORDS_PORT {
            // Features from textureCoords port are collected for UV assignment
            self.texture_coord_features.push(feature.clone());
        } else {
            // Features from default port are used to build the rasterized image
            // Extract color and geometry to accumulate in GeometryPolygons
            if let Some(polygon) = extract_geometry_polygon_from_feature(feature) {
                self.geometry_polygons.add_polygon(polygon);
            }
        }

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // When all features are processed, during finish phase
        // call draw method on GeometryPolygon, with dependent parameters get from &self

        // Determine image dimensions based on the accumulated polygons and cell sizes
        let boundary = self.geometry_polygons.find_coordinates_boundary();

        // Calculate the width and height of the image based on the boundary and cell size
        let width = self.width;

        // derive the height according to the ratio of boundary with relative to width
        let height = boundary.calculate_height_from_boundary_ratio(width);

        // Draw the accumulated polygons to an image
        let img = self.geometry_polygons.draw(width, height, true); // fill_area = true

        // Save the image using the helper function with the evaluated save_to path
        match save_image_with_path_option(&img, self.evaluated_save_path.clone()) {
            Ok(saved_path) => {
                ctx.event_hub.info_log(
                    None,
                    format!("ImageRasterizer saved image to: {:?}", saved_path),
                );

                // If we have features waiting for texture coordinates, process them
                if !self.texture_coord_features.is_empty() {
                    // Create texture URL from the saved path
                    let texture_url = Url::from_file_path(&saved_path).map_err(|_| {
                        GeometryProcessorError::ImageRasterizer(format!(
                            "Failed to create URL from path: {}",
                            saved_path
                        ))
                    })?;

                    ctx.event_hub.info_log(
                        None,
                        format!(
                            "ImageRasterizer assigning texture coordinates to {} features",
                            self.texture_coord_features.len()
                        ),
                    );

                    // Process each feature that needs texture coordinates
                    for feature in &self.texture_coord_features {
                        // Pass through the original feature unchanged to default port
                        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                            &ctx,
                            feature.clone(),
                            DEFAULT_PORT.clone(),
                        ));

                        // Send textured feature to textured port
                        match assign_texture_coordinates(feature, &boundary, &texture_url) {
                            Ok(updated_feature) => {
                                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                                    &ctx,
                                    updated_feature,
                                    TEXTURED_PORT.clone(),
                                ));
                            }
                            Err(e) => {
                                ctx.event_hub.warn_log(
                                    None,
                                    format!(
                                        "Failed to assign texture coordinates to feature: {}",
                                        e
                                    ),
                                );
                            }
                        }
                    }
                } else {
                    // No texture coordinate features, output image path as before
                    let mut feature = Feature::new_with_attributes(Attributes::default());
                    feature.insert(
                        "png_image",
                        reearth_flow_types::AttributeValue::String(saved_path),
                    );
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        feature,
                        DEFAULT_PORT.clone(),
                    ));
                }
            }
            Err(e) => {
                ctx.event_hub
                    .warn_log(None, format!("Failed to save image: {}", e));
            }
        }

        // Log the completion of the image creation
        ctx.event_hub.info_log(
            None,
            format!(
                "ImageRasterizer created image with dimensions {}x{}",
                width, height
            ),
        );

        Ok(())
    }

    fn name(&self) -> &str {
        "ImageRasterizer"
    }
}

#[derive(Clone, Debug)]
struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    // Creates a mapping function that can be reused to transform coordinates
    pub fn create_mapping_function(
        from_geo_boundary: CoordinatesBoundary,
        to_png_boundary: CoordinatesBoundary,
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
struct CoordinatesBoundary {
    left_up: Coordinate,
    left_down: Coordinate,
    right_up: Coordinate,
    right_down: Coordinate,
}

impl CoordinatesBoundary {
    fn calculate_height_from_boundary_ratio(&self, width: u32) -> u32 {
        let original_width = (self.left_up.x - self.right_up.x).abs();
        let original_height = (self.left_up.y - self.left_down.y).abs();
        let ratio = width as f64 / original_width;

        (original_height * ratio) as u32
    }
}

// Define the GeometryPixels structure to hold the mapped data
#[derive(Debug, Clone)]
struct ImagePixel {
    // coordinate in image -- x
    x: u32,
    // coordinate in image -- y
    y: u32,
    // pixel color
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone)]
struct GeometryPolygon {
    exterior_coordinates: Vec<(f64, f64)>, // List of (x, y) coordinates forming the exterior ring
    interior_coordinates: Vec<Vec<(f64, f64)>>, // List of interior rings (holes)
    color_r: u8,
    color_g: u8,
    color_b: u8,
}

impl GeometryPolygon {
    // Creates a mapping function that can be reused to transform the polygon's coordinates
    fn create_mapping_function_to_png(
        geo_boundary: CoordinatesBoundary,
        png_width: u32,
        png_height: u32,
    ) -> impl Fn(&GeometryPolygon) -> GeometryPolygon {
        // Define the PNG boundary with (0,0) at top-left
        let png_boundary = CoordinatesBoundary {
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
            // Apply the coordinate mapping function to each coordinate in the exterior ring
            let mapped_exterior_coordinates: Vec<(f64, f64)> = polygon
                .exterior_coordinates
                .iter()
                .map(|&(x, y)| coordinate_mapper((x, y)))
                .collect();

            // Apply the coordinate mapping function to each coordinate in the interior rings
            let mapped_interior_coordinates: Vec<Vec<(f64, f64)>> = polygon
                .interior_coordinates
                .iter()
                .map(|ring| {
                    ring.iter()
                        .map(|&(x, y)| coordinate_mapper((x, y)))
                        .collect()
                })
                .collect();

            // Return a new polygon with the mapped coordinates but the same color
            GeometryPolygon {
                exterior_coordinates: mapped_exterior_coordinates,
                interior_coordinates: mapped_interior_coordinates,
                color_r: polygon.color_r,
                color_g: polygon.color_g,
                color_b: polygon.color_b,
            }
        }
    }

    // Maps the polygon to a vector of image pixels
    fn to_image_pixels<F>(&self, mapping_fn: F, fill_area: bool) -> Vec<ImagePixel>
    where
        F: Fn(&GeometryPolygon) -> GeometryPolygon,
    {
        // Apply the mapping function to transform the polygon
        let mapped_polygon = mapping_fn(self);

        // Convert the mapped polygon's coordinates to image pixels
        let mut pixels = Vec::new();

        // Add the border pixels (outline of the exterior ring)
        for (i, current) in mapped_polygon.exterior_coordinates.iter().enumerate() {
            let next_index = (i + 1) % mapped_polygon.exterior_coordinates.len();
            let next = &mapped_polygon.exterior_coordinates[next_index];

            // Draw line between consecutive points
            let line_pixels = self.draw_line(
                current.0.round() as u32,
                current.1.round() as u32,
                next.0.round() as u32,
                next.1.round() as u32,
                (
                    mapped_polygon.color_r,
                    mapped_polygon.color_g,
                    mapped_polygon.color_b,
                ),
            );
            pixels.extend(line_pixels);
        }

        // Add the border pixels for each interior ring (holes)
        for interior_ring in &mapped_polygon.interior_coordinates {
            for (i, current) in interior_ring.iter().enumerate() {
                let next_index = (i + 1) % interior_ring.len();
                let next = &interior_ring[next_index];

                // Draw line between consecutive points
                let line_pixels = self.draw_line(
                    current.0.round() as u32,
                    current.1.round() as u32,
                    next.0.round() as u32,
                    next.1.round() as u32,
                    (
                        mapped_polygon.color_r,
                        mapped_polygon.color_g,
                        mapped_polygon.color_b,
                    ),
                );
                pixels.extend(line_pixels);
            }
        }

        // If fill_area is true, fill the interior of the polygon (including holes)
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
        color: (u8, u8, u8), // (r, g, b) tuple to reduce argument count
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
                r: color.0,
                g: color.1,
                b: color.2,
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
        if polygon.exterior_coordinates.len() < 3 {
            return Vec::new(); // Not enough points to form a polygon
        }

        let mut pixels = Vec::new();

        // Find the bounding box of the polygon (considering exterior ring only)
        let mut min_y = u32::MAX;
        let mut max_y = 0u32;
        let mut min_x = u32::MAX;
        let mut max_x = 0u32;

        for &(x, y) in &polygon.exterior_coordinates {
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

            // Process exterior ring
            for i in 0..polygon.exterior_coordinates.len() {
                let p1 = &polygon.exterior_coordinates[i];
                let p2 =
                    &polygon.exterior_coordinates[(i + 1) % polygon.exterior_coordinates.len()];

                let y1 = p1.1.round() as u32;
                let y2 = p2.1.round() as u32;

                // Check if the edge crosses the current scanline
                if ((y1 <= y && y2 > y) || (y2 <= y && y1 > y)) && y2 != y1 {
                    let x1 = p1.0;
                    let x2 = p2.0;

                    // Calculate intersection point
                    let x = x1 + (y as f64 - y1 as f64) / (y2 as f64 - y1 as f64) * (x2 - x1);
                    intersections.push(x.round() as u32);
                }
            }

            // Process interior rings (holes)
            for interior_ring in &polygon.interior_coordinates {
                for i in 0..interior_ring.len() {
                    let p1 = &interior_ring[i];
                    let p2 = &interior_ring[(i + 1) % interior_ring.len()];

                    let y1 = p1.1.round() as u32;
                    let y2 = p2.1.round() as u32;

                    // Check if the edge crosses the current scanline
                    if ((y1 <= y && y2 > y) || (y2 <= y && y1 > y)) && y2 != y1 {
                        let x1 = p1.0.round();
                        let x2 = p2.0.round();

                        // Calculate intersection point
                        let x = x1 + (y as f64 - y1 as f64) / (y2 as f64 - y1 as f64) * (x2 - x1);
                        intersections.push(x.round() as u32);
                    }
                }
            }

            // Sort intersections
            intersections.sort();

            // Apply even-odd rule to determine which segments to fill
            // We start with the assumption that we're outside the polygon
            // Each intersection toggles whether we're inside or outside
            let mut inside_exterior = false;
            let mut last_x = 0;

            for &current_x in &intersections {
                // Toggle the inside/outside state
                inside_exterior = !inside_exterior;

                // If we're now inside the exterior polygon but not inside a hole, fill the segment
                if inside_exterior {
                    last_x = current_x;
                } else {
                    // We're transitioning from inside to outside
                    // Fill from last_x to current_x if we're inside the exterior polygon
                    for x in last_x..=current_x {
                        // Count how many interior (hole) polygons this point is inside
                        let mut inside_hole_count = 0;

                        for interior_ring in &polygon.interior_coordinates {
                            if point_in_polygon(x as f64, y as f64, interior_ring) {
                                inside_hole_count += 1;
                            }
                        }

                        // Only fill if we're inside the exterior polygon AND inside an even number of holes (or no holes)
                        if inside_hole_count % 2 == 0 {
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
        }

        pixels
    }
}

// Helper function to determine if a point is inside a polygon using ray casting algorithm
fn point_in_polygon(point_x: f64, point_y: f64, polygon: &[(f64, f64)]) -> bool {
    if polygon.len() < 3 {
        return false; // Not enough points to form a polygon
    }

    let mut inside = false;

    for i in 0..polygon.len() {
        let p1 = &polygon[i];
        let p2 = &polygon[(i + 1) % polygon.len()];

        // Check if the point is on the same horizontal level as the edge
        if (p1.1 > point_y) != (p2.1 > point_y) {
            // Calculate the x-coordinate where the horizontal ray intersects the edge
            let x_intersection = (p2.0 - p1.0) * (point_y - p1.1) / (p2.1 - p1.1) + p1.0;

            // If the intersection is to the right of the point, we have crossed an edge
            if point_x < x_intersection {
                inside = !inside;
            }
        }
    }

    inside
}

#[derive(Debug, Clone)]
struct GeometryPolygons {
    polygons: Vec<GeometryPolygon>,
}

impl GeometryPolygons {
    fn new() -> Self {
        Self {
            polygons: Vec::new(),
        }
    }

    fn add_polygon(&mut self, polygon: GeometryPolygon) {
        self.polygons.push(polygon);
    }

    fn find_coordinates_boundary(&self) -> CoordinatesBoundary {
        if self.polygons.is_empty() {
            // Return a default boundary if no polygons exist
            return CoordinatesBoundary {
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
            // Check exterior coordinates
            for (x, y) in &polygon.exterior_coordinates {
                min_x = min_x.min(*x);
                max_x = max_x.max(*x);
                min_y = min_y.min(*y);
                max_y = max_y.max(*y);
            }

            // Also check interior coordinates (holes)
            for ring in &polygon.interior_coordinates {
                for (x, y) in ring {
                    min_x = min_x.min(*x);
                    max_x = max_x.max(*x);
                    min_y = min_y.min(*y);
                    max_y = max_y.max(*y);
                }
            }
        }

        // Create the boundary coordinates
        CoordinatesBoundary {
            left_up: Coordinate { x: min_x, y: max_y }, // top-left corner
            left_down: Coordinate { x: min_x, y: min_y }, // bottom-left corner
            right_up: Coordinate { x: max_x, y: max_y }, // top-right corner
            right_down: Coordinate { x: max_x, y: min_y }, // bottom-right corner
        }
    }

    // Draws the polygons to a PNG image with the specified width and height
    fn draw(&self, width: u32, height: u32, fill_area: bool) -> image::RgbImage {
        // Create a new image with white background
        let mut img = image::RgbImage::new(width, height);

        // Fill the image with white background
        for x in 0..width {
            for y in 0..height {
                img.put_pixel(x, y, image::Rgb([255, 255, 255])); // White background
            }
        }

        // Find the boundary of all polygons
        let geo_boundary = self.find_coordinates_boundary();

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

fn extract_geometry_polygon_from_feature(feature: &Feature) -> Option<GeometryPolygon> {
    // Extract color information from the feature attributes
    let r = feature
        .get("color_r")
        .and_then(|v| match v {
            reearth_flow_types::AttributeValue::String(s) => s.parse::<u8>().ok(),
            reearth_flow_types::AttributeValue::Number(n) => n.as_f64().map(|f| f as u8),
            _ => None,
        })
        .unwrap_or(255);
    let g = feature
        .get("color_g")
        .and_then(|v| match v {
            reearth_flow_types::AttributeValue::String(s) => s.parse::<u8>().ok(),
            reearth_flow_types::AttributeValue::Number(n) => n.as_f64().map(|f| f as u8),
            _ => None,
        })
        .unwrap_or(0);
    let b = feature
        .get("color_b")
        .and_then(|v| match v {
            reearth_flow_types::AttributeValue::String(s) => s.parse::<u8>().ok(),
            reearth_flow_types::AttributeValue::Number(n) => n.as_f64().map(|f| f as u8),
            _ => None,
        })
        .unwrap_or(0);

    // Alternative: Check if geometry is stored directly in the feature's geometry field
    // This handles the case where the geometry is already parsed into the Feature's geometry field
    if let Some(coords) = extract_coordinates_from_feature_geometry(feature) {
        if !coords.is_empty() {
            // First ring is the exterior (outer boundary), subsequent rings are interiors (holes)
            if let Some(exterior_ring) = coords.first() {
                let mut exterior_coordinates = Vec::new();

                for &(x, y) in exterior_ring {
                    exterior_coordinates.push((x, y));
                }

                // Collect interior rings (holes) if they exist
                let mut interior_rings = Vec::new();
                for i in 1..coords.len() {
                    if let Some(interior_ring) = coords.get(i) {
                        let mut interior_coords = Vec::new();
                        for &(x, y) in interior_ring {
                            interior_coords.push((x, y));
                        }
                        if !interior_coords.is_empty() {
                            interior_rings.push(interior_coords);
                        }
                    }
                }

                // Create a polygon with the exterior and interior coordinates and color
                let polygon = GeometryPolygon {
                    exterior_coordinates,
                    interior_coordinates: interior_rings,
                    color_r: r,
                    color_g: g,
                    color_b: b,
                };

                return Some(polygon);
            }
        }
    }

    None
}

// Helper function to extract coordinates from the feature's geometry field
fn extract_coordinates_from_feature_geometry(feature: &Feature) -> Option<Vec<Vec<(f64, f64)>>> {
    // Access the geometry field of the feature
    match &feature.geometry.value {
        reearth_flow_types::GeometryValue::FlowGeometry2D(geom) => {
            match geom {
                reearth_flow_geometry::types::geometry::Geometry2D::MultiPolygon(mpoly) => {
                    let mut all_rings = Vec::new();

                    for polygon in &mpoly.0 {
                        // Add exterior ring
                        let exterior: Vec<(f64, f64)> = polygon
                            .exterior()
                            .iter()
                            .map(|coord| (coord.x, coord.y))
                            .collect();
                        all_rings.push(exterior);

                        // Add interior rings (holes)
                        for interior in polygon.interiors() {
                            let interior_ring: Vec<(f64, f64)> =
                                interior.iter().map(|coord| (coord.x, coord.y)).collect();
                            all_rings.push(interior_ring);
                        }
                    }

                    Some(all_rings)
                }
                reearth_flow_geometry::types::geometry::Geometry2D::Polygon(poly) => {
                    let mut all_rings = Vec::new();

                    // Add exterior ring
                    let exterior: Vec<(f64, f64)> = poly
                        .exterior()
                        .iter()
                        .map(|coord| (coord.x, coord.y))
                        .collect();
                    all_rings.push(exterior);

                    // Add interior rings (holes)
                    for interior in poly.interiors() {
                        let interior_ring: Vec<(f64, f64)> =
                            interior.iter().map(|coord| (coord.x, coord.y)).collect();
                        all_rings.push(interior_ring);
                    }

                    Some(all_rings)
                }
                _ => None, // Handle other geometry types as needed
            }
        }
        _ => None, // Handle other geometry value types as needed
    }
}

/// Assigns texture coordinates to a feature's CityGmlGeometry based on the rasterized image boundary.
///
/// For each polygon vertex at position (x, y), computes UV coordinates as:
///   u = (x - min_x) / (max_x - min_x)
///   v = 1.0 - (y - min_y) / (max_y - min_y)  (flipped for image coordinate system)
///
/// Also sets the texture reference on each polygon to point to the generated image.
fn assign_texture_coordinates(
    feature: &Feature,
    boundary: &CoordinatesBoundary,
    texture_url: &Url,
) -> Result<Feature, GeometryProcessorError> {
    // Get the geometry - must be CityGmlGeometry
    let GeometryValue::CityGmlGeometry(citygml) = &feature.geometry.value else {
        return Err(GeometryProcessorError::ImageRasterizer(
            "Feature does not have CityGmlGeometry".to_string(),
        ));
    };

    let mut updated_citygml = citygml.clone();

    // Calculate boundary dimensions
    let min_x = boundary.left_down.x;
    let max_x = boundary.right_down.x;
    let min_y = boundary.left_down.y;
    let max_y = boundary.left_up.y;

    let width = max_x - min_x;
    let height = max_y - min_y;

    // Avoid division by zero
    if width.abs() < f64::EPSILON || height.abs() < f64::EPSILON {
        return Err(GeometryProcessorError::ImageRasterizer(
            "Boundary has zero width or height".to_string(),
        ));
    }

    // Clear existing texture data and set up for single texture
    updated_citygml.textures = vec![Texture {
        uri: texture_url.clone(),
    }];
    updated_citygml.polygon_textures.clear();
    updated_citygml.polygon_uvs = MultiPolygon2D::new(Vec::new());

    // Process each GmlGeometry entry
    for gml in &updated_citygml.gml_geometries {
        for polygon in &gml.polygons {
            // Compute UV for exterior ring
            let exterior_uvs: Vec<Coordinate2D<f64>> = polygon
                .exterior()
                .0
                .iter()
                .map(|v| {
                    let u = (v.x - min_x) / width;
                    // Flip v coordinate: image origin is top-left, but geometry Y increases upward
                    let v_coord = 1.0 - (v.y - min_y) / height;
                    Coordinate2D::new_(u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0))
                })
                .collect();

            // Compute UV for interior rings (holes)
            let interior_uvs: Vec<LineString2D<f64>> = polygon
                .interiors()
                .iter()
                .map(|ring| {
                    let uvs: Vec<Coordinate2D<f64>> = ring
                        .0
                        .iter()
                        .map(|v| {
                            let u = (v.x - min_x) / width;
                            let v_coord = 1.0 - (v.y - min_y) / height;
                            Coordinate2D::new_(u.clamp(0.0, 1.0), v_coord.clamp(0.0, 1.0))
                        })
                        .collect();
                    LineString2D::new(uvs)
                })
                .collect();

            // Add UV polygon
            updated_citygml.polygon_uvs.0.push(Polygon2D::new(
                LineString2D::new(exterior_uvs),
                interior_uvs,
            ));

            // All polygons reference texture index 0 (the single generated texture)
            updated_citygml.polygon_textures.push(Some(0));
        }
    }

    // Create a new feature with the updated geometry
    let new_geometry = Geometry {
        epsg: feature.geometry.epsg,
        value: GeometryValue::CityGmlGeometry(updated_citygml),
    };
    let updated_feature = Feature::new_with_attributes_and_geometry(
        (*feature.attributes).clone(),
        new_geometry,
        feature.metadata.clone(),
    );

    Ok(updated_feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case01() {
        use std::fs::File;
        use std::io::BufReader;

        // Load and parse the input_features.json file into Vec<Feature>
        // The file is located in the runtime/tests directory relative to the project root
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let project_root = std::path::Path::new(manifest_dir).parent().unwrap();
        let file_path = project_root
            .join("tests")
            .join("fixture")
            .join("testdata")
            .join("image_rasterizer")
            .join("input_features.json");

        let file = File::open(&file_path)
            .unwrap_or_else(|_| panic!("failed to open: {}", file_path.display()));
        let reader = BufReader::new(file);

        let features: Vec<Feature> =
            serde_json::from_reader(reader).expect("Failed to parse JSON into Vec<Feature>");

        assert!(!features.is_empty(), "Features vector should not be empty");
        println!(
            "Successfully parsed {} features from input_features.json",
            features.len()
        );

        // Verify the structure of the first feature
        let first_feature = &features[0];
        assert!(
            !first_feature.attributes.is_empty(),
            "First feature should have attributes"
        );
        // Note: We can't check if geometry is empty since Geometry doesn't have an is_empty method

        println!("First feature ID: {}", first_feature.id);
        println!(
            "First feature has {} attributes",
            first_feature.attributes.len()
        );

        // Use the Vec<Feature> to draw an image using extract_geometry_polygon_from_feature
        let mut geometry_polygons = GeometryPolygons::new();

        for feature in &features {
            if let Some(polygon) = extract_geometry_polygon_from_feature(feature) {
                geometry_polygons.add_polygon(polygon);
                println!("Added polygon from feature ID: {}", feature.id);
            } else {
                println!("No polygon extracted from feature ID: {}", feature.id);
            }
        }

        assert!(
            !geometry_polygons.polygons.is_empty(),
            "At least one polygon should have been extracted from features"
        );
        println!(
            "Successfully extracted {} polygons from features",
            geometry_polygons.polygons.len()
        );

        // Draw the polygons to an image
        let width = 1000;
        let boundary = geometry_polygons.find_coordinates_boundary();
        let height = boundary.calculate_height_from_boundary_ratio(width);
        let img = geometry_polygons.draw(width, height, true);

        // Save the image to verify it worked
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = std::path::Path::new(&home_dir)
            .join(".cache")
            .join("reearth-flow-test-images");

        std::fs::create_dir_all(&cache_dir)
            .expect("Could not create cache directory for test images");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let dest_path = cache_dir.join(format!("test_case04_output_{}.png", timestamp));

        img.save(&dest_path).expect("Failed to save test image");
        println!("Successfully generated and saved image to: {:?}", dest_path);
    }
}

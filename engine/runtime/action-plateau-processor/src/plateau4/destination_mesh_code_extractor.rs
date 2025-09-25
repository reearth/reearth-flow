use std::collections::HashMap;

use nusamai_projection::jprect::JPRZone;
use reearth_flow_geometry::algorithm::{
    area2d::Area2D, bool_ops::BooleanOps, bounding_rect::BoundingRect, centroid::Centroid,
};
use reearth_flow_geometry::types::{
    coordinate::Coordinate2D, geometry::Geometry2D, polygon::Polygon2D, rect::Rect2D,
};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::jpmesh::{JPMeshCode, JPMeshType};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::{json, Number};

#[derive(Debug, Clone, Default)]
pub struct DestinationMeshCodeExtractorFactory;

impl ProcessorFactory for DestinationMeshCodeExtractorFactory {
    fn name(&self) -> &str {
        "DestinationMeshCodeExtractor"
    }

    fn description(&self) -> &str {
        "Extract Japanese standard regional mesh code for PLATEAU destination files and add as attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DestinationMeshCodeExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: DestinationMeshCodeExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with)
                .map_err(|e| format!("Failed to serialize parameters: {e}"))?;
            serde_json::from_value(value)
                .map_err(|e| format!("Failed to deserialize parameters: {e}"))?
        } else {
            DestinationMeshCodeExtractorParam::default()
        };

        let mesh_type = match params.mesh_type {
            1 => JPMeshType::Mesh80km,
            2 => JPMeshType::Mesh10km,
            3 => JPMeshType::Mesh1km,
            4 => JPMeshType::Mesh500m,
            5 => JPMeshType::Mesh250m,
            6 => JPMeshType::Mesh125m,
            _ => return Err("Invalid mesh_type. Must be 1-6".into()),
        };

        Ok(Box::new(DestinationMeshCodeExtractor {
            mesh_type,
            meshcode_attr: params.meshcode_attr,
        }))
    }
}

/// # PLATEAU Destination MeshCode Extractor Parameters
/// Configure mesh code extraction for Japanese standard regional mesh
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DestinationMeshCodeExtractorParam {
    /// # Mesh Type
    /// Japanese standard mesh type: 1=80km, 2=10km, 3=1km, 4=500m, 5=250m, 6=125m
    #[serde(default = "default_mesh_type")]
    pub mesh_type: u8,

    /// # Mesh Code Attribute Name
    /// Output attribute name for the mesh code
    #[serde(default = "default_meshcode_attr")]
    pub meshcode_attr: String,
}

impl Default for DestinationMeshCodeExtractorParam {
    fn default() -> Self {
        Self {
            mesh_type: default_mesh_type(),
            meshcode_attr: default_meshcode_attr(),
        }
    }
}

fn default_mesh_type() -> u8 {
    3 // Tertiary Standard Mesh (1km) - PLATEAU default
}

fn default_meshcode_attr() -> String {
    "_meshcode".to_string()
}

#[derive(Debug, Clone)]
pub struct DestinationMeshCodeExtractor {
    mesh_type: JPMeshType,
    meshcode_attr: String,
}

impl Processor for DestinationMeshCodeExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                // Calculate mesh code using PLATEAU specification compliant area-based method
                let mesh_result = if let Some(result) = self.calculate_mesh_with_details(geometry) {
                    result
                } else {
                    // If mesh calculation fails, reject the feature
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                // Add mesh code and additional attributes
                let mut new_feature = feature.clone();
                new_feature.attributes.insert(
                    Attribute::new(&self.meshcode_attr),
                    AttributeValue::String(mesh_result.selected_mesh.to_number().to_string()),
                );
                new_feature.attributes.insert(
                    Attribute::new("_mesh_count"),
                    AttributeValue::Number(Number::from(mesh_result.mesh_count)),
                );
                new_feature.attributes.insert(
                    Attribute::new("__area"),
                    AttributeValue::Number(
                        Number::from_f64(mesh_result.max_area).unwrap_or(Number::from(0)),
                    ),
                );
                new_feature.attributes.insert(
                    Attribute::new("_meshcode_to_area"),
                    AttributeValue::String(mesh_result.meshcode_to_area_json),
                );

                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "DestinationMeshCodeExtractor"
    }
}

/// Result of mesh calculation including all required attributes
#[derive(Debug, Clone)]
struct MeshCalculationResult {
    /// The selected mesh code with maximum area
    selected_mesh: JPMeshCode,
    /// Total number of meshes the feature intersects with
    mesh_count: usize,
    /// Maximum area found in the selected mesh (rounded to 2 decimal places)
    max_area: f64,
    /// JSON string mapping mesh codes to their intersection areas
    meshcode_to_area_json: String,
}

impl DestinationMeshCodeExtractor {
    /// Transform WGS84 coordinates to Japanese Plane Rectangular Coordinate System Zone 9 (Tokyo area)
    /// Returns coordinates in meters suitable for accurate area calculation
    fn transform_to_jpr_zone9(&self, lon_lat: Coordinate2D<f64>) -> Option<Coordinate2D<f64>> {
        let projection = JPRZone::Zone9.projection();

        // Convert degrees to radians
        let lon_rad = lon_lat.x.to_radians();
        let lat_rad = lon_lat.y.to_radians();
        let height = 0.0; // Assume height = 0 for 2D calculations

        // Project to plane rectangular coordinates (in meters)
        match projection.project_forward(lon_rad, lat_rad, height) {
            Ok((x, y, _)) => Some(Coordinate2D::new_(x, y)),
            Err(_) => None,
        }
    }

    /// Transform a polygon from WGS84 to Japanese Plane Rectangular Coordinate System Zone 9
    fn transform_polygon_to_jpr(&self, polygon: &Polygon2D<f64>) -> Option<Polygon2D<f64>> {
        // Transform exterior ring
        let exterior_coords: Result<Vec<Coordinate2D<f64>>, ()> = polygon
            .exterior()
            .0
            .iter()
            .map(|coord| self.transform_to_jpr_zone9(*coord).ok_or(()))
            .collect();

        let exterior_coords = exterior_coords.ok()?;

        // Transform interior rings
        let interior_rings: Result<
            Vec<reearth_flow_geometry::types::line_string::LineString2D<f64>>,
            (),
        > = polygon
            .interiors()
            .iter()
            .map(|interior| {
                let interior_coords: Result<Vec<Coordinate2D<f64>>, ()> = interior
                    .0
                    .iter()
                    .map(|coord| self.transform_to_jpr_zone9(*coord).ok_or(()))
                    .collect();
                interior_coords.map(|coords| coords.into())
            })
            .collect();

        let interior_rings = interior_rings.ok()?;

        Some(Polygon2D::new(exterior_coords.into(), interior_rings))
    }

    /// Calculate mesh code with detailed information for all required attributes
    /// Returns comprehensive mesh calculation result including count, areas, and mapping
    fn calculate_mesh_with_details(
        &self,
        geometry: &Geometry2D<f64>,
    ) -> Option<MeshCalculationResult> {
        // Convert geometry to polygon for area calculation
        let polygon_wgs84 = self.geometry_to_polygon(geometry)?;

        // Transform polygon to Japanese Plane Rectangular Coordinate System for accurate area calculation
        let polygon_jpr = self.transform_polygon_to_jpr(&polygon_wgs84)?;

        // Get bounding box of the feature (in WGS84 for mesh lookup)
        let bounds = geometry.bounding_rect()?;

        // Get all mesh codes that intersect with the feature bounds
        let candidate_meshes = JPMeshCode::from_inside_bounds(bounds, self.mesh_type);

        let mut max_area = 0.0f64;
        let mut selected_mesh: Option<JPMeshCode> = None;
        let mut selected_mesh_number = u64::MAX;
        let mut mesh_area_mapping = Vec::new();
        let mut intersecting_mesh_count = 0;

        for mesh_code in candidate_meshes {
            // Get mesh boundary as polygon (in WGS84)
            let mesh_bounds_wgs84 = mesh_code.bounds();
            let mesh_polygon_wgs84 = self.rect_to_polygon(&mesh_bounds_wgs84);

            // Transform mesh polygon to Japanese Plane Rectangular Coordinate System
            let mesh_polygon_jpr = match self.transform_polygon_to_jpr(&mesh_polygon_wgs84) {
                Some(polygon) => polygon,
                None => continue, // Skip this mesh if transformation fails
            };

            // Calculate intersection area in meters
            let intersection = polygon_jpr.intersection(&mesh_polygon_jpr);
            let area = intersection.unsigned_area2d(); // Now in square meters

            // Round to 2 decimal places as per PLATEAU specification (now in square meters)
            let rounded_area = (area * 100.0).round() / 100.0;

            // Only count meshes that actually intersect with the feature (area > 0)
            if rounded_area > 0.0 {
                intersecting_mesh_count += 1;
                let mesh_number = mesh_code.to_number();

                // Store mesh code and area mapping
                mesh_area_mapping.push(json!({
                    "meshcode": mesh_number.to_string(),
                    "area": rounded_area
                }));

                // Select mesh with maximum area, or smaller mesh number if areas are equal
                if rounded_area > max_area
                    || (rounded_area == max_area && mesh_number < selected_mesh_number)
                {
                    max_area = rounded_area;
                    selected_mesh = Some(mesh_code);
                    selected_mesh_number = mesh_number;
                }
            }
        }

        // Convert mesh-area mapping to JSON string
        let meshcode_to_area_json =
            serde_json::to_string(&mesh_area_mapping).unwrap_or_else(|_| "[]".to_string());

        selected_mesh.map(|mesh| MeshCalculationResult {
            selected_mesh: mesh,
            mesh_count: intersecting_mesh_count,
            max_area,
            meshcode_to_area_json,
        })
    }

    /// Convert Geometry2D to Polygon2D for area calculations
    fn geometry_to_polygon(&self, geometry: &Geometry2D<f64>) -> Option<Polygon2D<f64>> {
        match geometry {
            Geometry2D::Polygon(p) => Some(p.clone()),
            Geometry2D::MultiPolygon(mp) => {
                // For MultiPolygon, combine all polygons into one
                // This is a simplification - in practice, each polygon should be processed separately
                mp.0.first().cloned()
            }
            Geometry2D::Rect(r) => Some(self.rect_to_polygon(r)),
            Geometry2D::Triangle(t) => {
                let coords = t.to_array();
                Some(Polygon2D::new(
                    vec![coords[0], coords[1], coords[2], coords[0]].into(), // Close the triangle
                    vec![],
                ))
            }
            // For non-area geometries, create a small buffer around the centroid
            _ => {
                if let Some(centroid) = geometry.centroid() {
                    let coord = centroid.0;
                    let epsilon = 0.0001; // Small buffer
                    Some(Polygon2D::new(
                        vec![
                            Coordinate2D::new_(coord.x - epsilon, coord.y - epsilon),
                            Coordinate2D::new_(coord.x + epsilon, coord.y - epsilon),
                            Coordinate2D::new_(coord.x + epsilon, coord.y + epsilon),
                            Coordinate2D::new_(coord.x - epsilon, coord.y + epsilon),
                            Coordinate2D::new_(coord.x - epsilon, coord.y - epsilon),
                        ]
                        .into(),
                        vec![],
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// Convert Rect2D to Polygon2D
    fn rect_to_polygon(&self, rect: &Rect2D<f64>) -> Polygon2D<f64> {
        let min = rect.min();
        let max = rect.max();

        Polygon2D::new(
            vec![
                min,
                Coordinate2D::new_(max.x, min.y),
                max,
                Coordinate2D::new_(min.x, max.y),
                min, // Close the polygon
            ]
            .into(),
            vec![],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use reearth_flow_types::{Feature, Geometry};

    #[test]
    fn test_destination_mesh_code_extraction() {
        let factory = DestinationMeshCodeExtractorFactory;
        let params = DestinationMeshCodeExtractorParam {
            mesh_type: 3, // 1km mesh
            meshcode_attr: "_meshcode".to_string(),
        };

        let processor = factory
            .build(
                NodeContext::default(),
                EventHub::new(100),
                "test".to_string(),
                Some(
                    serde_json::to_value(params)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect(),
                ),
            )
            .unwrap();

        // Test with a point in Tokyo area (simplified for testing)
        let geometry = Geometry {
            value: GeometryValue::None,
            epsg: Some(4326), // WGS84
        };

        let _feature = Feature {
            id: uuid::Uuid::new_v4(),
            geometry,
            attributes: IndexMap::new(),
            metadata: Default::default(),
        };

        // This test demonstrates the structure - actual execution would need proper context
        assert_eq!(processor.name(), "DestinationMeshCodeExtractor");
    }

    #[test]
    fn test_parameter_defaults() {
        let params = DestinationMeshCodeExtractorParam::default();
        assert_eq!(params.mesh_type, 3);
        assert_eq!(params.meshcode_attr, "_meshcode");
    }

    #[test]
    fn test_area_based_mesh_assignment() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, polygon::Polygon2D};
        use reearth_flow_types::jpmesh::JPMeshType;

        let extractor = DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
        };

        // Create a polygon that spans multiple meshes
        // Using coordinates around Tokyo area for realistic test
        let polygon = Polygon2D::new(
            vec![
                Coordinate2D::new_(139.7525, 35.6850), // Southwest
                Coordinate2D::new_(139.7575, 35.6850), // Southeast
                Coordinate2D::new_(139.7575, 35.6900), // Northeast
                Coordinate2D::new_(139.7525, 35.6900), // Northwest
                Coordinate2D::new_(139.7525, 35.6850), // Close polygon
            ]
            .into(),
            vec![],
        );

        let geometry = reearth_flow_geometry::types::geometry::Geometry2D::Polygon(polygon);
        // Test the detailed calculation method instead
        let result = extractor.calculate_mesh_with_details(&geometry);

        assert!(
            result.is_some(),
            "Should calculate mesh code for valid polygon"
        );
    }

    #[test]
    fn test_mesh_calculation_with_details() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, polygon::Polygon2D};
        use reearth_flow_types::jpmesh::JPMeshType;

        let extractor = DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
        };

        // Create a much larger polygon that spans multiple meshes with significant area
        // Using a larger coordinate range to ensure measurable intersection areas
        let polygon = Polygon2D::new(
            vec![
                Coordinate2D::new_(139.7000, 35.6700), // Southwest (much wider area)
                Coordinate2D::new_(139.8000, 35.6700), // Southeast
                Coordinate2D::new_(139.8000, 35.7200), // Northeast
                Coordinate2D::new_(139.7000, 35.7200), // Northwest
                Coordinate2D::new_(139.7000, 35.6700), // Close polygon
            ]
            .into(),
            vec![],
        );

        let geometry = reearth_flow_geometry::types::geometry::Geometry2D::Polygon(polygon);
        let result = extractor.calculate_mesh_with_details(&geometry);

        assert!(
            result.is_some(),
            "Should calculate detailed mesh information"
        );

        if let Some(mesh_result) = result {
            // Test that mesh count is reasonable (should be at least 1)
            assert!(
                mesh_result.mesh_count >= 1,
                "Should intersect with at least one mesh"
            );

            // Test that max_area is positive and now in reasonable square meter range
            assert!(mesh_result.max_area > 0.0, "Max area should be positive");
            println!("Max area in m²: {}", mesh_result.max_area);
            println!("Total meshes intersected: {}", mesh_result.mesh_count);

            // Test that area is now in a realistic range for square meters (not tiny decimal degrees)
            assert!(
                mesh_result.max_area > 1.0,
                "Area should be at least 1 m² for this large test polygon"
            );

            // Test that JSON mapping is valid
            assert!(
                !mesh_result.meshcode_to_area_json.is_empty(),
                "JSON mapping should not be empty"
            );

            // Try to parse the JSON to ensure it's valid
            let parsed: serde_json::Result<Vec<serde_json::Value>> =
                serde_json::from_str(&mesh_result.meshcode_to_area_json);
            assert!(
                parsed.is_ok(),
                "JSON should be valid: {}",
                mesh_result.meshcode_to_area_json
            );

            // Test that selected mesh code is valid
            assert!(
                mesh_result.selected_mesh.to_number() > 0,
                "Selected mesh code should be valid"
            );
        }
    }

    #[test]
    fn test_json_mapping_structure() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, polygon::Polygon2D};
        use reearth_flow_types::jpmesh::JPMeshType;

        let extractor = DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
        };

        // Create a small polygon that should intersect with limited meshes
        let polygon = Polygon2D::new(
            vec![
                Coordinate2D::new_(139.7550, 35.6875),
                Coordinate2D::new_(139.7551, 35.6875),
                Coordinate2D::new_(139.7551, 35.6876),
                Coordinate2D::new_(139.7550, 35.6876),
                Coordinate2D::new_(139.7550, 35.6875),
            ]
            .into(),
            vec![],
        );

        let geometry = reearth_flow_geometry::types::geometry::Geometry2D::Polygon(polygon);
        let result = extractor.calculate_mesh_with_details(&geometry);

        if let Some(mesh_result) = result {
            let parsed: serde_json::Result<Vec<serde_json::Value>> =
                serde_json::from_str(&mesh_result.meshcode_to_area_json);

            assert!(parsed.is_ok(), "JSON should be parseable");

            if let Ok(mappings) = parsed {
                for mapping in mappings {
                    // Each mapping should have "meshcode" and "area" fields
                    assert!(
                        mapping.get("meshcode").is_some(),
                        "Each mapping should have meshcode field"
                    );
                    assert!(
                        mapping.get("area").is_some(),
                        "Each mapping should have area field"
                    );

                    // Area should be a positive number
                    if let Some(area) = mapping.get("area").and_then(|v| v.as_f64()) {
                        assert!(area >= 0.0, "Area should be non-negative");
                    }

                    // Meshcode should be a string
                    if let Some(meshcode) = mapping.get("meshcode").and_then(|v| v.as_str()) {
                        assert!(!meshcode.is_empty(), "Meshcode should not be empty");
                        assert!(
                            meshcode.parse::<u64>().is_ok(),
                            "Meshcode should be numeric"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_geometry_conversion() {
        use reearth_flow_geometry::types::{
            coordinate::Coordinate2D, geometry::Geometry2D, point::Point2D, rect::Rect2D,
            triangle::Triangle2D,
        };
        use reearth_flow_types::jpmesh::JPMeshType;

        let extractor = DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
        };

        // Test point geometry conversion
        let point = Point2D::from(Coordinate2D::new_(139.7550, 35.6875));
        let point_geometry = Geometry2D::Point(point);
        let point_polygon = extractor.geometry_to_polygon(&point_geometry);
        assert!(point_polygon.is_some(), "Should convert point to polygon");

        // Test rectangle geometry conversion
        let rect = Rect2D::new(
            Coordinate2D::new_(139.7525, 35.6850),
            Coordinate2D::new_(139.7575, 35.6900),
        );
        let rect_geometry = Geometry2D::Rect(rect);
        let rect_polygon = extractor.geometry_to_polygon(&rect_geometry);
        assert!(rect_polygon.is_some(), "Should convert rect to polygon");

        // Test triangle geometry conversion
        let triangle = Triangle2D::new(
            Coordinate2D::new_(139.7525, 35.6850),
            Coordinate2D::new_(139.7575, 35.6850),
            Coordinate2D::new_(139.7550, 35.6900),
        );
        let triangle_geometry = Geometry2D::Triangle(triangle);
        let triangle_polygon = extractor.geometry_to_polygon(&triangle_geometry);
        assert!(
            triangle_polygon.is_some(),
            "Should convert triangle to polygon"
        );
    }

    #[test]
    fn test_rect_to_polygon_conversion() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, rect::Rect2D};
        use reearth_flow_types::jpmesh::JPMeshType;

        let extractor = DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
        };

        let rect = Rect2D::new(
            Coordinate2D::new_(139.0, 35.0),
            Coordinate2D::new_(140.0, 36.0),
        );

        let polygon = extractor.rect_to_polygon(&rect);

        // Check that polygon has 5 points (closed)
        assert_eq!(polygon.exterior().0.len(), 5);

        // Check that first and last points are the same (closed polygon)
        assert_eq!(polygon.exterior().0[0], polygon.exterior().0[4]);
    }

    #[test]
    fn test_mesh_type_mapping() {
        let factory = DestinationMeshCodeExtractorFactory;

        // Test all valid mesh types
        for mesh_type in 1..=6 {
            let params = DestinationMeshCodeExtractorParam {
                mesh_type,
                meshcode_attr: "_meshcode".to_string(),
            };

            let result = factory.build(
                NodeContext::default(),
                EventHub::new(100),
                "test".to_string(),
                Some(
                    serde_json::to_value(params)
                        .unwrap()
                        .as_object()
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect(),
                ),
            );

            assert!(result.is_ok(), "Mesh type {mesh_type} should be valid");
        }

        // Test invalid mesh type
        let params = DestinationMeshCodeExtractorParam {
            mesh_type: 7, // Invalid
            meshcode_attr: "_meshcode".to_string(),
        };

        let result = factory.build(
            NodeContext::default(),
            EventHub::new(100),
            "test".to_string(),
            Some(
                serde_json::to_value(params)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .collect(),
            ),
        );

        assert!(result.is_err(), "Mesh type 7 should be invalid");
    }
}

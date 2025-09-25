use std::collections::HashMap;

use nusamai_projection::jprect::JPRZone;
use reearth_flow_geometry::algorithm::transverse_mercator_proj::TransverseMercatorProjection;
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
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

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
                        Number::from_f64(dbg!(mesh_result.max_area)).unwrap_or(Number::from(0)),
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
    /// Calculate mesh code with detailed information for all required attributes
    /// Returns comprehensive mesh calculation result including count, areas, and mapping
    fn calculate_mesh_with_details(
        &self,
        geometry: &Geometry2D<f64>,
    ) -> Option<MeshCalculationResult> {
        // Convert geometry to polygon for area calculation
        let polygon_wgs84 = self.geometry_to_polygon(geometry)?;

        // Transform polygon to fixed EPSG:6675 for accurate area calculation
        let polygon_jpr = self.transform_polygon_to_fixed_epsg6675(&polygon_wgs84)?;

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

            // Transform mesh polygon to fixed EPSG:6675
            let mesh_polygon_jpr =
                match self.transform_polygon_to_fixed_epsg6675(&mesh_polygon_wgs84) {
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

    /// Transform coordinates to Japanese Plane Rectangular Coordinate System Zone 7 (JGD2011, EPSG:6675)
    /// Zone 7 covers: Ishikawa-ken, Toyama-ken, Gifu-ken, Aichi-ken
    /// FIXED: Always uses EPSG:6675 regardless of input coordinate system
    /// Returns coordinates in meters suitable for accurate area calculation
    /// Uses Flow's TransverseMercatorProjection for accurate results
    fn transform_to_fixed_epsg6675(
        &self,
        mut coord: Coordinate2D<f64>,
    ) -> Option<Coordinate2D<f64>> {
        // Fixed EPSG:6675 (JGD2011 / Japan Plane Rectangular CS VII)
        let projection = JPRZone::from_epsg(6675)?.projection();

        // Use Flow's TransverseMercatorProjection trait
        // This will treat input coordinates as geographic (lat/lon in degrees)
        // and transform them to EPSG:6675 projected coordinates (meters)
        coord.project_forward(&projection, false).ok()?;

        Some(coord)
    }

    /// Transform a polygon to fixed EPSG:6675 (Japanese Plane Rectangular Coordinate System Zone 7)
    /// FIXED: Always transforms to EPSG:6675 regardless of input coordinate system
    fn transform_polygon_to_fixed_epsg6675(
        &self,
        polygon: &Polygon2D<f64>,
    ) -> Option<Polygon2D<f64>> {
        // Transform exterior ring using fixed EPSG:6675
        let exterior_coords: Result<Vec<Coordinate2D<f64>>, ()> = polygon
            .exterior()
            .0
            .iter()
            .map(|coord| self.transform_to_fixed_epsg6675(*coord).ok_or(()))
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
                    .map(|coord| self.transform_to_fixed_epsg6675(*coord).ok_or(()))
                    .collect();
                interior_coords.map(|coords| coords.into())
            })
            .collect();

        let interior_rings = interior_rings.ok()?;

        Some(Polygon2D::new(exterior_coords.into(), interior_rings))
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
            Geometry2D::LineString(ls) => {
                // Convert closed LineString to Polygon
                if self.is_closed_linestring(ls) {
                    Some(Polygon2D::new(ls.clone(), vec![]))
                } else {
                    // For non-closed LineString, create a small buffer around the centroid
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
            // For other non-area geometries, create a small buffer around the centroid
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

    /// Check if a LineString is closed (first and last points are the same)
    fn is_closed_linestring(
        &self,
        linestring: &reearth_flow_geometry::types::line_string::LineString2D<f64>,
    ) -> bool {
        if linestring.0.len() < 4 {
            return false; // A polygon needs at least 4 points (including closing point)
        }
        let first = linestring.0.first();
        let last = linestring.0.last();
        match (first, last) {
            (Some(f), Some(l)) => {
                // Check if first and last coordinates are approximately equal
                (f.x - l.x).abs() < f64::EPSILON && (f.y - l.y).abs() < f64::EPSILON
            }
            _ => false,
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

    #[test]
    fn test_real_gml_coordinates_area_calculation() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, polygon::Polygon2D};
        use reearth_flow_types::jpmesh::JPMeshType;

        let extractor = DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
        };

        // Real coordinates from GML posList data
        // Note: GML coordinates are in lat,lon order, but we need lon,lat for Coordinate2D
        let polygon_wgs84 = Polygon2D::new(
            vec![
                Coordinate2D::new_(137.07022204628032, 36.65423985231743),
                Coordinate2D::new_(137.07018801464667, 36.65426289610968),
                Coordinate2D::new_(137.0701714804155, 36.654247021162256),
                Coordinate2D::new_(137.07020551204704, 36.65422397737467),
                Coordinate2D::new_(137.07022204628032, 36.65423985231743),
            ]
            .into(),
            vec![],
        );

        // Transform to fixed EPSG:6675
        let polygon_jpr = extractor
            .transform_polygon_to_fixed_epsg6675(&polygon_wgs84)
            .expect("Polygon transformation failed");
        let area_jpr = polygon_jpr.unsigned_area2d();
        // Round to 4 decimal places for comparison
        let rounded_area = (area_jpr * 10000.0).round() / 10000.0;

        assert!(
            (rounded_area - 9.1410).abs() < 0.01,
            "Calculated area {rounded_area} should be close to expected area 9.1410"
        );
    }
}

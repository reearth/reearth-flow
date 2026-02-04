use std::{collections::HashMap, sync::Arc};

use reearth_flow_geometry::algorithm::{
    area2d::Area2D, bool_ops::BooleanOps, bounding_rect::BoundingRect,
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
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, GeometryValue};
use rhai::AST;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;

use crate::plateau4::errors::PlateauProcessorError;

#[derive(Debug, Clone, Default)]
pub struct DestinationMeshCodeExtractorFactory;

impl ProcessorFactory for DestinationMeshCodeExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.DestinationMeshCodeExtractor"
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: DestinationMeshCodeExtractorParam = if let Some(with) = with.as_ref() {
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

        // Compile EPSG code expression for runtime evaluation
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let epsg_code_ast = expr_engine
            .compile(params.epsg_code.as_ref())
            .map_err(|e| format!("Failed to compile epsg_code expression: {e}"))?;

        Ok(Box::new(DestinationMeshCodeExtractor {
            global_params: with,
            mesh_type,
            meshcode_attr: params.meshcode_attr,
            epsg_code_ast,
            cached_epsg_code: None,
            proj_to_epsg: None,
            proj_from_epsg: None,
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

    /// # EPSG Code
    /// Japanese Plane Rectangular Coordinate System EPSG code for area calculation
    #[serde(default = "default_epsg_code")]
    pub epsg_code: Expr,
}

impl Default for DestinationMeshCodeExtractorParam {
    fn default() -> Self {
        Self {
            mesh_type: default_mesh_type(),
            meshcode_attr: default_meshcode_attr(),
            epsg_code: default_epsg_code(),
        }
    }
}

fn default_mesh_type() -> u8 {
    3 // Tertiary Standard Mesh (1km) - PLATEAU default
}

fn default_meshcode_attr() -> String {
    "_meshcode".to_string()
}

fn default_epsg_code() -> Expr {
    Expr::new("6691".to_string()) // JGD2011 / UTM Zone 54N - PLATEAU standard coordinate system
}

#[derive(Debug)]
pub struct DestinationMeshCodeExtractor {
    global_params: Option<HashMap<String, Value>>,
    mesh_type: JPMeshType,
    meshcode_attr: String,
    epsg_code_ast: AST,
    // Cached projection instances for reuse
    cached_epsg_code: Option<String>,
    proj_to_epsg: Option<proj::Proj>,   // 6697 -> target_epsg
    proj_from_epsg: Option<proj::Proj>, // target_epsg -> 6697
}

impl Clone for DestinationMeshCodeExtractor {
    fn clone(&self) -> Self {
        DestinationMeshCodeExtractor {
            global_params: self.global_params.clone(),
            mesh_type: self.mesh_type,
            meshcode_attr: self.meshcode_attr.clone(),
            epsg_code_ast: self.epsg_code_ast.clone(),
            // Proj instances are not cloned - they will be recreated on first use
            // This is safe because the engine only clones processors at build time
            cached_epsg_code: None,
            proj_to_epsg: None,
            proj_from_epsg: None,
        }
    }
}

// SAFETY: `proj::Proj` wraps the PROJ C library which uses raw pointers and doesn't implement
// Send/Sync. However, this is safe because:
// 1. `num_threads()` returns 1, guaranteeing single-threaded execution of this processor
// 2. The processor owns the `Proj` instances exclusively; they're never shared across threads
unsafe impl Send for DestinationMeshCodeExtractor {}
unsafe impl Sync for DestinationMeshCodeExtractor {}

impl Processor for DestinationMeshCodeExtractor {
    fn num_threads(&self) -> usize {
        // DO NOT change this value. The unsafe Send/Sync impls rely on single-threaded execution.
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        // Evaluate EPSG code expression at runtime with feature context
        let epsg_code = self.evaluate_epsg_code(feature, &ctx)?;

        // Update cached proj instances if EPSG code changed
        if self.cached_epsg_code.as_ref() != Some(&epsg_code) {
            self.proj_to_epsg = Some(
                proj::Proj::new_known_crs("EPSG:6697", &format!("EPSG:{}", epsg_code), None)
                    .map_err(|e| {
                        PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                            "Failed to create PROJ transformation from 6697 to {epsg_code}: {e}"
                        ))
                    })?,
            );
            self.proj_from_epsg = Some(
                proj::Proj::new_known_crs(&format!("EPSG:{}", epsg_code), "EPSG:6697", None)
                    .map_err(|e| {
                        PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                            "Failed to create PROJ transformation from {epsg_code} to 6697: {e}"
                        ))
                    })?,
            );
            self.cached_epsg_code = Some(epsg_code.clone());
        }

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
                new_feature.attributes_mut().insert(
                    Attribute::new(&self.meshcode_attr),
                    AttributeValue::String(mesh_result.selected_mesh.to_number().to_string()),
                );
                new_feature.attributes_mut().insert(
                    Attribute::new("_mesh_count"),
                    AttributeValue::Number(Number::from(mesh_result.mesh_count)),
                );
                new_feature.attributes_mut().insert(
                    Attribute::new("__area"),
                    AttributeValue::Number(
                        Number::from_f64(mesh_result.max_area).unwrap_or(Number::from(0)),
                    ),
                );
                new_feature.attributes_mut().insert(
                    Attribute::new("_meshcode_to_area"),
                    AttributeValue::String(
                        serde_json::to_string(&mesh_result.meshcode_to_area)
                            .unwrap_or(String::new()),
                    ),
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
    meshcode_to_area: Vec<MeshCodeToArea>,
}

#[derive(Debug, Clone, Serialize)]
struct MeshCodeToArea {
    mesh_code: u64,
    area: f64,
}

impl DestinationMeshCodeExtractor {
    /// Evaluate the EPSG code expression at runtime with feature context
    fn evaluate_epsg_code(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
    ) -> Result<String, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let epsg_code = scope
            .eval_ast::<rhai::Dynamic>(&self.epsg_code_ast)
            .map_err(|e| {
                PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                    "Failed to evaluate epsg_code expression: {e:?}"
                ))
            })?;

        // Handle both string and integer EPSG codes
        if let Some(s) = epsg_code.clone().try_cast::<String>() {
            Ok(s)
        } else if let Some(i) = epsg_code.clone().try_cast::<i64>() {
            Ok(i.to_string())
        } else {
            Err(PlateauProcessorError::DestinationMeshCodeExtractor(
                "epsg_code expression did not evaluate to a string or integer".to_string(),
            )
            .into())
        }
    }

    /// Calculate mesh code with detailed information for all required attributes
    /// Returns comprehensive mesh calculation result including count, areas, and mapping
    /// Uses cached proj instances for coordinate transformations
    fn calculate_mesh_with_details(
        &self,
        geometry: &Geometry2D<f64>,
    ) -> Option<MeshCalculationResult> {
        // Convert geometry to polygon for area calculation
        let polygon = self.geometry_to_polygon(geometry)?;

        // Get bounding box of the feature (in WGS84 for mesh lookup)
        let bounds = geometry.bounding_rect()?;
        let bounds = self.transform_bounds_to_epsg_inverse(&bounds)?;

        // Get all mesh codes that intersect with the feature bounds
        let candidate_meshes = JPMeshCode::from_inside_bounds(bounds, self.mesh_type);

        let mut max_area = 0.0f64;
        let mut selected_mesh: Option<JPMeshCode> = None;
        let mut selected_mesh_number = u64::MAX;
        let mut mesh_area_mapping = Vec::new();
        let mut intersecting_mesh_count = 0;

        for mesh_code in candidate_meshes {
            // Get mesh boundary as polygon (in WGS84)
            let mesh_bounds = mesh_code.bounds();
            let mesh_polygon = mesh_bounds.to_polygon();

            let mesh_polygon = self.transform_polygon_to_epsg(&mesh_polygon)?;

            // Calculate intersection area in meters
            let intersection = polygon.intersection(&mesh_polygon);
            let area = intersection.unsigned_area2d(); // Now in square meters

            // Round to 2 decimal places as per PLATEAU specification (now in square meters)
            let rounded_area = (area * 100.0).round() / 100.0;

            // Only count meshes that actually intersect with the feature (area > 0)
            if rounded_area > 0.0 {
                intersecting_mesh_count += 1;
                let mesh_number = mesh_code.to_number();

                mesh_area_mapping.push(MeshCodeToArea {
                    mesh_code: mesh_number,
                    area: rounded_area,
                });

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

        selected_mesh.map(|mesh| MeshCalculationResult {
            selected_mesh: mesh,
            mesh_count: intersecting_mesh_count,
            max_area,
            meshcode_to_area: mesh_area_mapping,
        })
    }

    /// Transform a coordinate using a cached proj instance
    fn transform_coord_with_proj(
        coord: Coordinate2D<f64>,
        proj: &proj::Proj,
    ) -> Option<Coordinate2D<f64>> {
        let transformed = proj.convert((coord.x, coord.y)).ok()?;
        Some(Coordinate2D::new_(transformed.0, transformed.1))
    }

    /// Transform a polygon to the configured Japanese Plane Rectangular Coordinate System
    /// Uses the cached proj_to_epsg instance (6697 -> target EPSG)
    fn transform_polygon_to_epsg(&self, polygon: &Polygon2D<f64>) -> Option<Polygon2D<f64>> {
        let proj = self.proj_to_epsg.as_ref()?;

        let exterior_coords: Result<Vec<Coordinate2D<f64>>, ()> = polygon
            .exterior()
            .0
            .iter()
            .map(|coord| Self::transform_coord_with_proj(*coord, proj).ok_or(()))
            .collect();
        let exterior_coords = exterior_coords.ok()?;

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
                    .map(|coord| Self::transform_coord_with_proj(*coord, proj).ok_or(()))
                    .collect();
                interior_coords.map(|coords| coords.into())
            })
            .collect();
        let interior_rings = interior_rings.ok()?;

        Some(Polygon2D::new(exterior_coords.into(), interior_rings))
    }

    /// Transform bounds from target EPSG back to 6697 (WGS84)
    /// Uses the cached proj_from_epsg instance (target EPSG -> 6697)
    fn transform_bounds_to_epsg_inverse(&self, rect: &Rect2D<f64>) -> Option<Rect2D<f64>> {
        let proj = self.proj_from_epsg.as_ref()?;
        let min = Self::transform_coord_with_proj(rect.min(), proj)?;
        let max = Self::transform_coord_with_proj(rect.max(), proj)?;
        Some(Rect2D::new(min, max))
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
            Geometry2D::Rect(r) => Some(r.to_polygon()),
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
                    None
                }
            }
            // Other non-area geometries are invalid for mesh code extraction
            _ => None,
        }
    }

    /// Check if a LineString is closed
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_eval_expr::engine::Engine;

    #[test]
    fn test_real_gml_coordinates_area_calculation() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, polygon::Polygon2D};
        use reearth_flow_types::jpmesh::JPMeshType;

        // Value actually obtained from FME
        const EXPECTED_AREA: f64 = 9.14;

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

        // Create a dummy AST for testing (the actual value doesn't matter for this test)
        let engine = Engine::new();
        let epsg_code_ast = engine.compile("\"6675\"").unwrap();

        // Create proj instance for 6697 -> 6675 transformation
        let proj_to_epsg = proj::Proj::new_known_crs("EPSG:6697", "EPSG:6675", None)
            .expect("Failed to create Proj");

        let extractor = DestinationMeshCodeExtractor {
            global_params: None,
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: "_meshcode".to_string(),
            epsg_code_ast,
            cached_epsg_code: Some("6675".to_string()),
            proj_to_epsg: Some(proj_to_epsg),
            proj_from_epsg: None, // Not needed for this test
        };

        // Transform to EPSG:6675 using cached proj instance
        let polygon_jpr = extractor
            .transform_polygon_to_epsg(&polygon_wgs84)
            .expect("Polygon transformation failed");
        let area_jpr = polygon_jpr.unsigned_area2d();
        let rounded_area = (area_jpr * 100.0).round() / 100.0;

        assert!(
            (rounded_area - EXPECTED_AREA).abs() < 0.0001,
            "Calculated area {rounded_area} should be close to expected area 9.1410"
        );
    }
}

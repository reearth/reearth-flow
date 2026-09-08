use std::{cell::RefCell, collections::HashMap, sync::Arc};

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::{
    area2d::Area2D, bool_ops::BooleanOps, bounding_rect::BoundingRect,
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::{geometry::Geometry2D, polygon::Polygon2D};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    coordinate::{CoordinateFrame, EpsgCode},
    ops::{
        reproject::transform_coords_3d, rings_as_faces_2d, Aabb, BoundingBox, Elevation,
        ReprojectionCache,
    },
    overlay::{overlay_2d, OverlayOp},
    polygon::Polygon2D,
    predicates::view::{flatten_2d, Leaf2D},
    Euclidean2DGeometry, Geometry,
};
// The mesh grid is still described with the pre-migration rectangle types.
use reearth_flow_geometry::types::{coordinate::Coordinate2D, rect::Rect2D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::jpmesh::{JPMeshCode, JPMeshType};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue, Code, CodeType, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use serde_json::Value;

use super::errors::PlateauProcessorError;
use super::PlateauProfile;

// Thread-local cache for PROJ transformations.
// Stores both forward (6697 -> target) and inverse (target -> 6697) projections.
// Each thread maintains its own cache to ensure thread-safety without requiring
// unsafe Send/Sync implementations on types containing proj::Proj.
#[cfg(not(feature = "new-geometry"))]
thread_local! {
    // Cache for forward projections: 6697 -> target EPSG
    static PROJ_TO_CACHE: RefCell<HashMap<String, proj::Proj>> = RefCell::new(HashMap::new());
    // Cache for inverse projections: target EPSG -> 6697
    static PROJ_FROM_CACHE: RefCell<HashMap<String, proj::Proj>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub(crate) struct DestinationMeshCodeExtractorFactory {
    name: String,
}

impl DestinationMeshCodeExtractorFactory {
    pub(crate) fn new(profile: &PlateauProfile) -> Self {
        Self {
            name: profile.action_name("DestinationMeshCodeExtractor"),
        }
    }
}

impl ProcessorFactory for DestinationMeshCodeExtractorFactory {
    fn name(&self) -> &str {
        &self.name
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
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: DestinationMeshCodeExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::DestinationMeshCodeExtractorFactory(format!(
                    "Failed to serialize parameters: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::DestinationMeshCodeExtractorFactory(format!(
                    "Failed to deserialize parameters: {e}"
                ))
            })?
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

        let epsg_code_ast = params
            .epsg_code
            .compile()
            .map_err(|e| format!("Failed to compile epsg_code expression: {e}"))?;

        Ok(Box::new(DestinationMeshCodeExtractor {
            mesh_type,
            meshcode_attr: params.meshcode_attr,
            epsg_code_ast,
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
    pub epsg_code: Code<{ CodeType::FlowExpr as u32 }>,
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

fn default_epsg_code() -> Code<{ CodeType::FlowExpr as u32 }> {
    Code {
        ty: CodeType::FlowExpr,
        value: "6691".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct DestinationMeshCodeExtractor {
    mesh_type: JPMeshType,
    meshcode_attr: String,
    epsg_code_ast: CompiledCode,
}

#[cfg(not(feature = "new-geometry"))]
/// Helper function to ensure proj instances exist in thread-local cache for the given EPSG code.
fn ensure_proj_cached(epsg_code: &str) -> Result<(), BoxedError> {
    use std::collections::hash_map::Entry;

    PROJ_TO_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Entry::Vacant(e) = cache.entry(epsg_code.to_string()) {
            let proj = proj::Proj::new_known_crs("EPSG:6697", &format!("EPSG:{}", epsg_code), None)
                .map_err(|e| {
                    PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                        "Failed to create PROJ transformation from 6697 to {epsg_code}: {e}"
                    ))
                })?;
            e.insert(proj);
        }
        Ok::<_, BoxedError>(())
    })?;

    PROJ_FROM_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Entry::Vacant(e) = cache.entry(epsg_code.to_string()) {
            let proj = proj::Proj::new_known_crs(&format!("EPSG:{}", epsg_code), "EPSG:6697", None)
                .map_err(|e| {
                    PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                        "Failed to create PROJ transformation from {epsg_code} to 6697: {e}"
                    ))
                })?;
            e.insert(proj);
        }
        Ok::<_, BoxedError>(())
    })
}

impl Processor for DestinationMeshCodeExtractor {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        let epsg_code = self.evaluate_epsg_code(feature, ctx.variables.clone())?;

        // Ensure proj instances are cached for this EPSG code
        ensure_proj_cached(&epsg_code)?;

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
                let mesh_result = if let Some(result) =
                    self.calculate_mesh_with_details(geometry, &epsg_code)
                {
                    result
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                let new_feature = self.with_mesh_attributes(feature, &mesh_result);
                fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let epsg_code = self.evaluate_epsg_code(feature, ctx.variables.clone())?;
        let epsg = epsg_code
            .trim()
            .parse::<u16>()
            .map(EpsgCode::new)
            .map_err(|e| {
                PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                    "epsg_code expression ({epsg_code}) is not an EPSG code: {e}"
                ))
            })?;

        // The footprint must already be a flat areal geometry in `epsg`: this
        // action measures intersection areas in that projected space and never
        // reprojects the feature itself.
        let mesh_result = match feature.geometry.as_ref() {
            Geometry::Euclidean2D(geometry) => {
                match areal_operand(geometry, &CoordinateFrame::Crs(epsg)) {
                    Some(area) => self.calculate_mesh(&area, epsg),
                    None => Ok(None),
                }
            }
            _ => Ok(None),
        };
        let mesh_result = match mesh_result {
            Ok(Some(result)) => result,
            Ok(None) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
            Err(e) => {
                ctx.event_hub.warn_log(
                    Some(ctx.error_span()),
                    format!("mesh code extraction failed: {e}"),
                );
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        let new_feature = self.with_mesh_attributes(feature, &mesh_result);
        fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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
    fn evaluate_epsg_code(
        &self,
        feature: &Feature,
        variables: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<String, BoxedError> {
        let epsg_code = self.epsg_code_ast.eval(feature, variables).map_err(|e| {
            PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                "Failed to evaluate epsg_code expression: {e:?}"
            ))
        })?;
        match epsg_code {
            AttributeValue::String(s) => Ok(s),
            AttributeValue::Number(n) => Ok(n.to_string()),
            other => Err(PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                "epsg_code expression ({:?}) did not evaluate to a string or integer",
                other
            ))
            .into()),
        }
    }

    /// A copy of the feature carrying the mesh result: the chosen mesh code,
    /// how many meshes it spans, the area inside the chosen one, and the full
    /// mesh-to-area mapping as JSON.
    fn with_mesh_attributes(&self, feature: &Feature, result: &MeshCalculationResult) -> Feature {
        let mut feature = feature.clone();
        let attributes = feature.attributes_mut();
        attributes.insert(
            Attribute::new(&self.meshcode_attr),
            AttributeValue::String(result.selected_mesh.to_number().to_string()),
        );
        attributes.insert(
            Attribute::new("_mesh_count"),
            AttributeValue::Number(Number::from(result.mesh_count)),
        );
        attributes.insert(
            Attribute::new("__area"),
            AttributeValue::Number(Number::from_f64(result.max_area).unwrap_or(Number::from(0))),
        );
        attributes.insert(
            Attribute::new("_meshcode_to_area"),
            AttributeValue::String(
                serde_json::to_string(&result.meshcode_to_area).unwrap_or_default(),
            ),
        );
        feature
    }
}

#[cfg(not(feature = "new-geometry"))]
impl DestinationMeshCodeExtractor {
    /// Calculate mesh code with detailed information for all required attributes
    /// Returns comprehensive mesh calculation result including count, areas, and mapping
    /// Uses thread-local cached proj instances for coordinate transformations
    fn calculate_mesh_with_details(
        &self,
        geometry: &Geometry2D<f64>,
        epsg_code: &str,
    ) -> Option<MeshCalculationResult> {
        // Convert geometry to polygon for area calculation
        let polygon = self.geometry_to_polygon(geometry)?;

        // Get bounding box of the feature (in WGS84 for mesh lookup)
        let bounds = geometry.bounding_rect()?;
        let bounds = Self::transform_bounds_to_epsg_inverse(&bounds, epsg_code)?;

        // Get all mesh codes that intersect with the feature bounds
        let candidate_meshes = JPMeshCode::from_inside_bounds(bounds, self.mesh_type);

        let mut max_area = 0.0f64;
        let mut selected_mesh: Option<JPMeshCode> = None;
        let mut selected_mesh_number = u64::MAX;
        let mut mesh_area_mapping = Vec::new();
        let mut intersecting_mesh_count = 0;

        for mesh_code in candidate_meshes {
            // Get mesh boundary as polygon (in WGS84) with densified edges.
            // Mesh boundaries are constant-latitude/longitude lines which are curved
            // in Transverse Mercator projections. A 4-corner polygon with straight edges
            // in projected space deviates from the true boundary. Adding intermediate
            // points along each edge before transformation produces a more accurate
            // representation of the curved boundaries.
            let mesh_bounds = mesh_code.bounds();
            let mesh_polygon = Self::densified_rect_polygon(&mesh_bounds);

            let mesh_polygon = Self::transform_polygon_to_epsg(&mesh_polygon, epsg_code)?;

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

    /// Transform a coordinate using a proj instance from thread-local cache
    fn transform_coord_to_epsg(
        coord: Coordinate2D<f64>,
        epsg_code: &str,
    ) -> Option<Coordinate2D<f64>> {
        PROJ_TO_CACHE.with(|cache| {
            let cache = cache.borrow();
            let proj = cache.get(epsg_code)?;
            let transformed = proj.convert((coord.x, coord.y)).ok()?;
            Some(Coordinate2D::new_(transformed.0, transformed.1))
        })
    }

    /// Transform a coordinate using the inverse proj instance from thread-local cache
    fn transform_coord_from_epsg(
        coord: Coordinate2D<f64>,
        epsg_code: &str,
    ) -> Option<Coordinate2D<f64>> {
        PROJ_FROM_CACHE.with(|cache| {
            let cache = cache.borrow();
            let proj = cache.get(epsg_code)?;
            let transformed = proj.convert((coord.x, coord.y)).ok()?;
            Some(Coordinate2D::new_(transformed.0, transformed.1))
        })
    }

    /// Transform a polygon to the configured Japanese Plane Rectangular Coordinate System
    /// Uses the thread-local cached proj_to_epsg instance (6697 -> target EPSG)
    fn transform_polygon_to_epsg(
        polygon: &Polygon2D<f64>,
        epsg_code: &str,
    ) -> Option<Polygon2D<f64>> {
        let exterior_coords: Result<Vec<Coordinate2D<f64>>, ()> = polygon
            .exterior()
            .0
            .iter()
            .map(|coord| Self::transform_coord_to_epsg(*coord, epsg_code).ok_or(()))
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
                    .map(|coord| Self::transform_coord_to_epsg(*coord, epsg_code).ok_or(()))
                    .collect();
                interior_coords.map(|coords| coords.into())
            })
            .collect();
        let interior_rings = interior_rings.ok()?;

        Some(Polygon2D::new(exterior_coords.into(), interior_rings))
    }

    /// Transform bounds from target EPSG back to EPSG:6697
    /// Uses the thread-local cached proj_from_epsg instance (target EPSG -> 6697)
    fn transform_bounds_to_epsg_inverse(
        rect: &Rect2D<f64>,
        epsg_code: &str,
    ) -> Option<Rect2D<f64>> {
        let min = Self::transform_coord_from_epsg(rect.min(), epsg_code)?;
        let max = Self::transform_coord_from_epsg(rect.max(), epsg_code)?;
        Some(Rect2D::new(min, max))
    }

    /// Create a densified polygon from a geographic rectangle by subdividing each edge.
    ///
    /// In Transverse Mercator projections, lines of constant latitude are curved.
    /// A simple 4-corner rectangle transformed to projected coordinates uses straight
    /// lines between corners, which deviates from the true curved mesh boundary.
    /// By adding intermediate points along each edge in geographic coordinates before
    /// transformation, the resulting projected polygon more accurately follows the
    /// true constant-latitude/longitude curves.
    fn densified_rect_polygon(rect: &Rect2D<f64>) -> Polygon2D<f64> {
        const SEGMENTS_PER_EDGE: usize = 16;

        let min_x = rect.min().x;
        let min_y = rect.min().y;
        let max_x = rect.max().x;
        let max_y = rect.max().y;

        let mut coords = Vec::with_capacity(4 * SEGMENTS_PER_EDGE + 1 /* closing point */);

        // Bottom edge: SW → SE (constant lat = min_y, varying lng)
        for i in 0..SEGMENTS_PER_EDGE {
            let t = i as f64 / SEGMENTS_PER_EDGE as f64;
            coords.push(Coordinate2D::new_(min_x + t * (max_x - min_x), min_y));
        }
        // Right edge: SE → NE (constant lng = max_x, varying lat)
        for i in 0..SEGMENTS_PER_EDGE {
            let t = i as f64 / SEGMENTS_PER_EDGE as f64;
            coords.push(Coordinate2D::new_(max_x, min_y + t * (max_y - min_y)));
        }
        // Top edge: NE → NW (constant lat = max_y, varying lng)
        for i in 0..SEGMENTS_PER_EDGE {
            let t = i as f64 / SEGMENTS_PER_EDGE as f64;
            coords.push(Coordinate2D::new_(max_x - t * (max_x - min_x), max_y));
        }
        // Left edge: NW → SW (constant lng = min_x, varying lat)
        for i in 0..SEGMENTS_PER_EDGE {
            let t = i as f64 / SEGMENTS_PER_EDGE as f64;
            coords.push(Coordinate2D::new_(min_x, max_y - t * (max_y - min_y)));
        }
        // Close the polygon
        coords.push(coords[0]);

        Polygon2D::new(coords.into(), vec![])
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
                if Self::is_closed_linestring(ls) {
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

/// The geographic CRS the Japanese mesh grid is described in: the 2D horizontal
/// counterpart of EPSG:6697, sharing its datum and its `[latitude, longitude]`
/// axis order. [`JPMeshCode`], in contrast, works in `(x = longitude,
/// y = latitude)`, so coordinates are swapped at every crossing between them.
#[cfg(feature = "new-geometry")]
const GEOGRAPHIC_EPSG: EpsgCode = EpsgCode::new(6668);

/// How many pieces each edge of a mesh cell is cut into before it is projected.
#[cfg(feature = "new-geometry")]
const SEGMENTS_PER_EDGE: usize = 16;

#[cfg(feature = "new-geometry")]
thread_local! {
    /// The live PROJ transforms, kept off the `Processor` (which must be
    /// `Send + Sync + Clone`) because they wrap non-shareable PROJ pointers.
    /// One per direction: a cache holds a single `(from, to)` pair, so sharing
    /// one between the two directions would rebuild the transform every call.
    static TO_GEOGRAPHIC: RefCell<ReprojectionCache> = RefCell::new(ReprojectionCache::new());
    static FROM_GEOGRAPHIC: RefCell<ReprojectionCache> = RefCell::new(ReprojectionCache::new());
}

#[cfg(feature = "new-geometry")]
impl DestinationMeshCodeExtractor {
    /// The mesh the footprint belongs to, together with the per-mesh
    /// intersection areas behind that choice. `None` when it meets no mesh.
    ///
    /// Areas are measured in `epsg`, the projected CRS the footprint is already
    /// expressed in, so they come out in square metres.
    fn calculate_mesh(
        &self,
        area: &Euclidean2DGeometry,
        epsg: EpsgCode,
    ) -> Result<Option<MeshCalculationResult>, BoxedError> {
        let Ok(Aabb::D2 { min, max }) = area.bounding_box() else {
            return Ok(None);
        };
        let bounds = geographic_bounds(min, max, epsg)?;
        let frame = CoordinateFrame::Crs(epsg);

        let mut max_area = 0.0f64;
        let mut selected_mesh: Option<JPMeshCode> = None;
        let mut selected_number = u64::MAX;
        let mut meshcode_to_area = Vec::new();
        let mut mesh_count = 0usize;

        for mesh_code in JPMeshCode::from_inside_bounds(bounds, self.mesh_type) {
            let mesh = mesh_face(&mesh_code.bounds(), epsg, &frame)?;
            let mesh = Euclidean2DGeometry::Polygon(Box::new(mesh));
            let number = mesh_code.to_number();
            let pieces = overlay_2d(area, &mesh, OverlayOp::Intersection).map_err(|e| {
                PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                    "Failed to intersect the footprint with mesh {number}: {e}"
                ))
            })?;
            let intersection: f64 = pieces.iter().map(Polygon2D::area).sum();
            // Two decimal places, as the PLATEAU specification reports them.
            let area = (intersection * 100.0).round() / 100.0;
            if area <= 0.0 {
                continue;
            }

            mesh_count += 1;
            meshcode_to_area.push(MeshCodeToArea {
                mesh_code: number,
                area,
            });
            // The largest area wins; the smaller mesh code breaks a tie.
            if area > max_area || (area == max_area && number < selected_number) {
                max_area = area;
                selected_mesh = Some(mesh_code);
                selected_number = number;
            }
        }

        Ok(selected_mesh.map(|mesh| MeshCalculationResult {
            selected_mesh: mesh,
            mesh_count,
            max_area,
            meshcode_to_area,
        }))
    }
}

/// The geometry as an areal operand, or `None` when it is not one: an empty
/// geometry, a leaf outside `frame`, a leaf raised to an elevation, or a leaf
/// that encloses no area.
#[cfg(feature = "new-geometry")]
fn areal_operand(
    geometry: &Euclidean2DGeometry,
    frame: &CoordinateFrame,
) -> Option<Euclidean2DGeometry> {
    let mut leaves = Vec::new();
    flatten_2d(geometry, &mut leaves);
    if leaves.is_empty() {
        return None;
    }
    for leaf in &leaves {
        if leaf.frame() != frame || leaf.elevation().is_some() {
            return None;
        }
        match leaf {
            Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {}
            Leaf2D::Line(line) if line.is_closed_ring() => {}
            _ => return None,
        }
    }
    Some(rings_as_faces_2d(geometry))
}

/// A projected bounding box carried back to the mesh grid's geographic space,
/// as the `(longitude, latitude)` rectangle [`JPMeshCode`] expects.
#[cfg(feature = "new-geometry")]
fn geographic_bounds(
    min: [f64; 2],
    max: [f64; 2],
    epsg: EpsgCode,
) -> Result<Rect2D<f64>, BoxedError> {
    let mut corners = [[min[0], min[1], 0.0], [max[0], max[1], 0.0]];
    TO_GEOGRAPHIC
        .with(|cache| {
            transform_coords_3d(&mut cache.borrow_mut(), epsg, GEOGRAPHIC_EPSG, &mut corners)
        })
        .map_err(|e| {
            PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                "Failed to transform the footprint bounds from EPSG:{epsg} to EPSG:{GEOGRAPHIC_EPSG}: {e}"
            ))
        })?;
    Ok(Rect2D::new(
        Coordinate2D::new_(corners[0][1], corners[0][0]),
        Coordinate2D::new_(corners[1][1], corners[1][0]),
    ))
}

/// One mesh cell as a face in `frame`, its edges subdivided before projection.
///
/// A cell is bounded by lines of constant latitude and longitude, which curve
/// under a Transverse Mercator projection. Straight edges between the four
/// corners would cut into the cell, so intermediate vertices are added while
/// the ring is still geographic.
#[cfg(feature = "new-geometry")]
fn mesh_face(
    bounds: &Rect2D<f64>,
    epsg: EpsgCode,
    frame: &CoordinateFrame,
) -> Result<Polygon2D, BoxedError> {
    let (min_lng, min_lat) = (bounds.min().x, bounds.min().y);
    let (max_lng, max_lat) = (bounds.max().x, bounds.max().y);
    let step = |i: usize| i as f64 / SEGMENTS_PER_EDGE as f64;

    // South, east, north then west edge: counter-clockwise seen in
    // (East, North). Vertices are held as [latitude, longitude], the geographic
    // CRS's declared axis order, ready for the transform.
    let mut ring = Vec::with_capacity(4 * SEGMENTS_PER_EDGE + 1);
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([min_lat, min_lng + step(i) * (max_lng - min_lng), 0.0]);
    }
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([min_lat + step(i) * (max_lat - min_lat), max_lng, 0.0]);
    }
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([max_lat, max_lng - step(i) * (max_lng - min_lng), 0.0]);
    }
    for i in 0..SEGMENTS_PER_EDGE {
        ring.push([max_lat - step(i) * (max_lat - min_lat), min_lng, 0.0]);
    }
    ring.push(ring[0]);

    FROM_GEOGRAPHIC
        .with(|cache| {
            transform_coords_3d(&mut cache.borrow_mut(), GEOGRAPHIC_EPSG, epsg, &mut ring)
        })
        .map_err(|e| {
            PlateauProcessorError::DestinationMeshCodeExtractor(format!(
                "Failed to transform a mesh cell from EPSG:{GEOGRAPHIC_EPSG} to EPSG:{epsg}: {e}"
            ))
        })?;

    Ok(Polygon2D::from_rings(
        frame.clone(),
        ring.iter().map(|c| [c[0], c[1]]),
        Vec::<Vec<[f64; 2]>>::new(),
    ))
}
#[cfg(all(test, not(feature = "new-geometry")))]
mod tests {
    use super::*;

    #[test]
    fn test_real_gml_coordinates_area_calculation() {
        use reearth_flow_geometry::types::{coordinate::Coordinate2D, polygon::Polygon2D};

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

        let epsg_code = "6675";

        // Ensure proj is cached for this test
        ensure_proj_cached(epsg_code).expect("Failed to cache Proj");

        // Transform to EPSG:6675 using thread-local cached proj instance
        let polygon_jpr =
            DestinationMeshCodeExtractor::transform_polygon_to_epsg(&polygon_wgs84, epsg_code)
                .expect("Polygon transformation failed");
        let area_jpr = polygon_jpr.unsigned_area2d();
        let rounded_area = (area_jpr * 100.0).round() / 100.0;

        assert!(
            (rounded_area - EXPECTED_AREA).abs() < 0.0001,
            "Calculated area {rounded_area} should be close to expected area 9.1410"
        );
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;

    /// Real coordinates from the `Z-bldg-03_meshcode-extractor_01` fixture, as
    /// `(longitude, latitude)`. Its expected CSV row is mesh 54377085 covering
    /// 9.14 m², a value originally obtained from the original implementation.
    const FOOTPRINT: [(f64, f64); 5] = [
        (137.07022204628032, 36.65423985231743),
        (137.07018801464667, 36.65426289610968),
        (137.0701714804155, 36.654247021162256),
        (137.07020551204704, 36.65422397737467),
        (137.07022204628032, 36.65423985231743),
    ];

    const PRCS: EpsgCode = EpsgCode::new(6675);

    /// A ring of `(longitude, latitude)` pairs as a footprint projected into
    /// EPSG:6675, the shape the workflow hands this action.
    fn footprint_in_prcs(ring: &[(f64, f64)]) -> Euclidean2DGeometry {
        let mut coords: Vec<[f64; 3]> = ring.iter().map(|&(lng, lat)| [lat, lng, 0.0]).collect();
        FROM_GEOGRAPHIC
            .with(|cache| {
                transform_coords_3d(&mut cache.borrow_mut(), GEOGRAPHIC_EPSG, PRCS, &mut coords)
            })
            .expect("projection into EPSG:6675");
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            CoordinateFrame::Crs(PRCS),
            coords.iter().map(|c| [c[0], c[1]]),
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    fn extractor() -> DestinationMeshCodeExtractor {
        DestinationMeshCodeExtractor {
            mesh_type: JPMeshType::Mesh1km,
            meshcode_attr: default_meshcode_attr(),
            epsg_code_ast: default_epsg_code().compile().unwrap(),
        }
    }

    #[test]
    fn coordinates_keep_each_crs_declared_axis_order() {
        let geometry = footprint_in_prcs(&FOOTPRINT);
        let Ok(Aabb::D2 { min, max }) = geometry.bounding_box() else {
            panic!("the footprint has no 2D bounding box");
        };
        // EPSG:6675's origin is 36°N 137°10′E, so this footprint sits ~72.6 km
        // north and ~8.6 km west of it, stored [northing, easting].
        assert!((min[0] - 72_580.0).abs() < 200.0, "northing was {}", min[0]);
        assert!((min[1] + 8_620.0).abs() < 200.0, "easting was {}", min[1]);

        // Back in geographic space the rectangle is (longitude, latitude),
        // which is what JPMeshCode reads.
        let bounds = geographic_bounds(min, max, PRCS).expect("inverse projection");
        assert!((bounds.min().x - 137.0701714804155).abs() < 1e-7);
        assert!((bounds.min().y - 36.65422397737467).abs() < 1e-7);
    }

    #[test]
    fn selects_the_mesh_the_footprint_lies_in() {
        let result = extractor()
            .calculate_mesh(&footprint_in_prcs(&FOOTPRINT), PRCS)
            .expect("mesh calculation")
            .expect("the footprint meets a mesh");

        assert_eq!(result.selected_mesh.to_number(), 54377085);
        assert_eq!(result.mesh_count, 1);
        assert_eq!(result.max_area, 9.14);
        assert_eq!(result.meshcode_to_area.len(), 1);
    }
}

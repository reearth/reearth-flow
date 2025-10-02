use std::collections::HashMap;

use parking_lot::Mutex;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;

#[derive(Debug, Clone, Default)]
pub struct BuildingPartConnectivityCheckerFactory;

impl ProcessorFactory for BuildingPartConnectivityCheckerFactory {
    fn name(&self) -> &str {
        "PLATEAU4.BuildingPartConnectivityChecker"
    }

    fn description(&self) -> &str {
        "Check connectivity between BuildingParts within the same Building using 3D boundary surface matching"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(BuildingPartConnectivityCheckerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature", "PLATEAU"]
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
        let params: BuildingPartConnectivityCheckerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::BuildingPartConnectivityChecker(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::BuildingPartConnectivityChecker(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            BuildingPartConnectivityCheckerParam::default()
        };

        Ok(Box::new(BuildingPartConnectivityChecker {
            params,
            buffer: Mutex::new(HashMap::new()),
        }))
    }
}

/// # BuildingPartConnectivityChecker Parameters
/// Configure how to check connectivity between BuildingParts
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingPartConnectivityCheckerParam {
    /// # Building ID Attribute
    /// Attribute containing the parent Building ID (default: "gmlId")
    #[serde(default = "default_building_id_attr")]
    pub building_id_attribute: Attribute,

    /// # Part ID Attribute
    /// Attribute containing the BuildingPart ID (default: "featureId")
    #[serde(default = "default_part_id_attr")]
    pub part_id_attribute: Attribute,

    /// # LOD Attribute
    /// Attribute containing the Level of Detail (default: "lod")
    #[serde(default = "default_lod_attr")]
    pub lod_attribute: Attribute,

    /// # File Index Attribute
    /// Attribute containing the file index (default: "fileIndex")
    #[serde(default = "default_file_index_attr")]
    pub file_index_attribute: Attribute,
}

fn default_building_id_attr() -> Attribute {
    Attribute::new("gmlId")
}

fn default_part_id_attr() -> Attribute {
    Attribute::new("featureId")
}

fn default_lod_attr() -> Attribute {
    Attribute::new("lod")
}

fn default_file_index_attr() -> Attribute {
    Attribute::new("fileIndex")
}

impl Default for BuildingPartConnectivityCheckerParam {
    fn default() -> Self {
        Self {
            building_id_attribute: default_building_id_attr(),
            part_id_attribute: default_part_id_attr(),
            lod_attribute: default_lod_attr(),
            file_index_attribute: default_file_index_attr(),
        }
    }
}

#[derive(Debug)]
pub struct BuildingPartConnectivityChecker {
    params: BuildingPartConnectivityCheckerParam,
    buffer: Mutex<HashMap<GroupKey, Vec<BuildingPartInfo>>>,
}

impl Clone for BuildingPartConnectivityChecker {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            buffer: Mutex::new(self.buffer.lock().clone()),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct GroupKey {
    building_id: String,
    lod: String,
    file_index: String,
}

#[derive(Debug, Clone)]
struct BuildingPartInfo {
    part_id: String,
    geometry: reearth_flow_types::Geometry,
    feature: Feature,
}

/// Union-Find data structure for tracking connected components
#[derive(Debug, Clone)]
struct UnionFind {
    parent: HashMap<String, String>,
    rank: HashMap<String, usize>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    fn make_set(&mut self, x: String) {
        if !self.parent.contains_key(&x) {
            self.parent.insert(x.clone(), x.clone());
            self.rank.insert(x, 0);
        }
    }

    fn find(&mut self, x: &str) -> String {
        let parent = self.parent.get(x).cloned().unwrap_or_else(|| x.to_string());
        if parent != x {
            let root = self.find(&parent);
            self.parent.insert(x.to_string(), root.clone());
            root
        } else {
            parent
        }
    }

    fn union(&mut self, a: &str, b: &str) {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return;
        }

        let rank_a = *self.rank.get(&root_a).unwrap_or(&0);
        let rank_b = *self.rank.get(&root_b).unwrap_or(&0);

        if rank_a < rank_b {
            self.parent.insert(root_a, root_b);
        } else if rank_a > rank_b {
            self.parent.insert(root_b, root_a.clone());
        } else {
            self.parent.insert(root_b, root_a.clone());
            *self.rank.entry(root_a).or_insert(0) += 1;
        }
    }

    fn get_connected_components(&mut self) -> HashMap<String, Vec<String>> {
        let mut components: HashMap<String, Vec<String>> = HashMap::new();
        for part_id in self.parent.keys().cloned().collect::<Vec<_>>() {
            let root = self.find(&part_id);
            components.entry(root).or_default().push(part_id);
        }
        components
    }
}

impl Processor for BuildingPartConnectivityChecker {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Extract grouping key
        let building_id = feature
            .get(&self.params.building_id_attribute)
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                PlateauProcessorError::BuildingPartConnectivityChecker(format!(
                    "Building ID attribute not found: {}",
                    self.params.building_id_attribute
                ))
            })?
            .to_string();

        let lod = feature
            .get(&self.params.lod_attribute)
            .ok_or_else(|| {
                PlateauProcessorError::BuildingPartConnectivityChecker(format!(
                    "LOD attribute not found: {}",
                    self.params.lod_attribute
                ))
            })?
            .to_string();

        let file_index = feature
            .get(&self.params.file_index_attribute)
            .ok_or_else(|| {
                PlateauProcessorError::BuildingPartConnectivityChecker(format!(
                    "File index attribute not found: {}",
                    self.params.file_index_attribute
                ))
            })?
            .to_string();

        let part_id = feature
            .get(&self.params.part_id_attribute)
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                PlateauProcessorError::BuildingPartConnectivityChecker(format!(
                    "Part ID attribute not found: {}",
                    self.params.part_id_attribute
                ))
            })?
            .to_string();

        let geometry = feature.geometry.clone();
        if matches!(geometry.value, reearth_flow_types::GeometryValue::None) {
            return Ok(());
        }

        let group_key = GroupKey {
            building_id,
            lod,
            file_index,
        };

        // Buffer BuildingParts by group
        let mut buffer = self.buffer.lock();
        let parts = buffer.entry(group_key.clone()).or_default();
        parts.push(BuildingPartInfo {
            part_id,
            geometry,
            feature: feature.clone(),
        });

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        self.flush_buffer(ctx.as_context(), fw)?;
        Ok(())
    }

    fn name(&self) -> &str {
        "BuildingPartConnectivityChecker"
    }
}

impl BuildingPartConnectivityChecker {
    fn flush_buffer(&self, ctx: Context, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let buffer = self.buffer.lock();
        for parts in buffer.values() {
            if parts.is_empty() {
                continue;
            }

            // Check connectivity within the group
            let results = check_connectivity(parts)?;

            // Output features with connectivity status
            for (part_id, status, connected_id, connected_parts) in results {
                if let Some(part_info) = parts.iter().find(|p| p.part_id == part_id) {
                    let mut feature = part_info.feature.clone();

                    feature.attributes.insert(
                        Attribute::new("_status"),
                        AttributeValue::String(status.clone()),
                    );
                    feature.attributes.insert(
                        Attribute::new("_connected_id"),
                        AttributeValue::Number(serde_json::Number::from(connected_id)),
                    );
                    feature.attributes.insert(
                        Attribute::new("_connected_parts"),
                        AttributeValue::Number(serde_json::Number::from(connected_parts)),
                    );

                    fw.send(ExecutorContext::new_with_context_feature_and_port(
                        &ctx,
                        feature,
                        DEFAULT_PORT.clone(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Check connectivity between BuildingParts
/// Returns: Vec<(part_id, status, connected_id, connected_parts)>
fn check_connectivity(
    parts: &[BuildingPartInfo],
) -> Result<Vec<(String, String, usize, usize)>, BoxedError> {
    let mut uf = UnionFind::new();

    // Initialize Union-Find
    for part in parts {
        uf.make_set(part.part_id.clone());
    }

    // Check boundary surface sharing between all pairs
    for i in 0..parts.len() {
        for j in (i + 1)..parts.len() {
            let shares = shares_boundary_surface(&parts[i].geometry, &parts[j].geometry)?;
            if shares {
                uf.union(&parts[i].part_id, &parts[j].part_id);
            }
        }
    }

    // Get connected components
    let components = uf.get_connected_components();

    // Sort components by size (descending)
    let mut sorted_components: Vec<_> = components.into_iter().collect();
    sorted_components.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    // Assign connected_id and determine status
    let mut results = Vec::new();
    let total_parts = parts.len();

    for (connected_id, (_, component_parts)) in sorted_components.iter().enumerate() {
        let connected_parts = component_parts.len();

        for part_id in component_parts {
            // Single BuildingPart is always "alone" (error condition)
            let status = if total_parts == 1 {
                "alone".to_string()
            } else if sorted_components.len() == 1 && connected_parts == total_parts {
                // All BuildingParts are connected (valid)
                "full".to_string()
            } else if connected_parts == 1 {
                // Isolated BuildingPart (error)
                "alone".to_string()
            } else {
                // Multiple disconnected groups (error)
                "partial".to_string()
            };

            results.push((part_id.clone(), status, connected_id, connected_parts));
        }
    }

    Ok(results)
}

/// Check if two polygons share the same boundary (same coordinates, possibly in reverse order)
fn polygons_share_boundary(
    poly_a: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    poly_b: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
) -> bool {
    use reearth_flow_geometry::types::coordinate::Coordinate;

    let exterior_a = poly_a.exterior();
    let exterior_b = poly_b.exterior();

    let coords_a: Vec<&Coordinate<f64>> = exterior_a.0.iter().collect();
    let coords_b: Vec<&Coordinate<f64>> = exterior_b.0.iter().collect();

    // Skip if different number of points
    if coords_a.len() != coords_b.len() || coords_a.is_empty() {
        return false;
    }

    const EPSILON: f64 = 1e-9;

    fn coords_equal(a: &Coordinate<f64>, b: &Coordinate<f64>) -> bool {
        (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON && (a.z - b.z).abs() < EPSILON
    }

    // Polygon rings can start at any vertex and can be in forward or reverse direction
    // Check all possible rotations in both directions

    // Remove duplicate last point for comparison (GML polygons repeat first point as last)
    let n = coords_a.len() - 1;

    // Check forward direction (same orientation)
    for offset in 0..n {
        let matches = (0..n).all(|i| {
            let a_idx = i;
            let b_idx = (i + offset) % n;
            coords_equal(coords_a[a_idx], coords_b[b_idx])
        });
        if matches {
            return true;
        }
    }

    // Check reverse direction (opposite orientation - shared face)
    for offset in 0..n {
        let matches = (0..n).all(|i| {
            let a_idx = i;
            let b_idx = (n - 1 - i + offset) % n;
            coords_equal(coords_a[a_idx], coords_b[b_idx])
        });
        if matches {
            return true;
        }
    }

    false
}

/// Check if two CityGmlGeometry share boundary surfaces
fn shares_citygml_boundary_surface(
    citygml_a: &reearth_flow_types::CityGmlGeometry,
    citygml_b: &reearth_flow_types::CityGmlGeometry,
) -> Result<bool, BoxedError> {
    use reearth_flow_geometry::types::multi_polygon::MultiPolygon;

    // Convert CityGmlGeometry to Geometry3D::MultiPolygon
    // Collect all polygons from all gml_geometries
    let mut all_polygons_a = Vec::new();
    for gml_geom in &citygml_a.gml_geometries {
        all_polygons_a.extend(gml_geom.polygons.iter().cloned());
    }

    let mut all_polygons_b = Vec::new();
    for gml_geom in &citygml_b.gml_geometries {
        all_polygons_b.extend(gml_geom.polygons.iter().cloned());
    }

    let mp_a = MultiPolygon(all_polygons_a);
    let mp_b = MultiPolygon(all_polygons_b);

    // Compare all polygon pairs
    for poly_a in mp_a.0.iter() {
        for poly_b in mp_b.0.iter() {
            if polygons_share_boundary(poly_a, poly_b) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Check if two Solid geometries share boundary surfaces (faces)
fn shares_solid_boundary_surface(
    solid_a: &reearth_flow_geometry::types::solid::Solid3D<f64>,
    solid_b: &reearth_flow_geometry::types::solid::Solid3D<f64>,
) -> Result<bool, BoxedError> {
    use reearth_flow_geometry::types::line_string::LineString;
    use reearth_flow_geometry::types::polygon::Polygon;

    // Get all faces from both solids
    let faces_a = solid_a.all_faces();
    let faces_b = solid_b.all_faces();

    // Check if any pair of faces share a boundary (same coordinates, possibly in reverse order)
    for face_a in &faces_a {
        for face_b in &faces_b {
            // Convert Face to Polygon for comparison
            let coords_a: Vec<_> = face_a.0.iter().map(|c| (c.x, c.y, c.z)).collect();
            let coords_b: Vec<_> = face_b.0.iter().map(|c| (c.x, c.y, c.z)).collect();

            if coords_a.is_empty() || coords_b.is_empty() {
                continue;
            }

            let line_a = LineString::from(coords_a);
            let line_b = LineString::from(coords_b);

            let poly_a = Polygon::new(line_a, vec![]);
            let poly_b = Polygon::new(line_b, vec![]);

            if polygons_share_boundary(&poly_a, &poly_b) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Check if two geometries share boundary surfaces
fn shares_boundary_surface(
    geom_a: &reearth_flow_types::Geometry,
    geom_b: &reearth_flow_types::Geometry,
) -> Result<bool, BoxedError> {
    use reearth_flow_types::GeometryValue;

    // Handle CityGmlGeometry by converting to FlowGeometry3D
    match (&geom_a.value, &geom_b.value) {
        (GeometryValue::CityGmlGeometry(citygml_a), GeometryValue::CityGmlGeometry(citygml_b)) => {
            return shares_citygml_boundary_surface(citygml_a, citygml_b);
        }
        (GeometryValue::None, _) | (_, GeometryValue::None) => return Ok(false),
        (GeometryValue::FlowGeometry2D(_), _) | (_, GeometryValue::FlowGeometry2D(_)) => {
            return Ok(false)
        }
        (GeometryValue::CityGmlGeometry(_), _) | (_, GeometryValue::CityGmlGeometry(_)) => {
            return Ok(false);
        }
        (GeometryValue::FlowGeometry3D(_), GeometryValue::FlowGeometry3D(_)) => {
            // Continue to handle FlowGeometry3D
        }
    }

    let geom3d_a = match &geom_a.value {
        GeometryValue::FlowGeometry3D(g) => g,
        _ => unreachable!(),
    };

    let geom3d_b = match &geom_b.value {
        GeometryValue::FlowGeometry3D(g) => g,
        _ => unreachable!(),
    };

    // Handle FlowGeometry3D types
    match (geom3d_a, geom3d_b) {
        // Solid: Check if any faces share boundaries
        (Geometry3D::Solid(a), Geometry3D::Solid(b)) => shares_solid_boundary_surface(a, b),
        // For other geometry types, not implemented yet
        _ => Ok(false),
    }
}

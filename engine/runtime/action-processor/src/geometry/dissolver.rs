use std::collections::HashMap;
#[cfg(feature = "new-geometry")]
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    cache::executor_cache_subdir,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::{
    algorithm::{bool_ops::BooleanOps, tolerance::glue_vertices_closer_than},
    types::multi_polygon::MultiPolygon2D,
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    collection::Collection2D,
    coordinate::CoordinateFrame,
    overlay::dissolve_leaves,
    polygon::Polygon2D,
    predicates::view::{flatten_2d, Leaf2D},
    Euclidean2DGeometry, Geometry,
};

use super::errors::GeometryProcessorError;
use crate::ACCUMULATOR_BUFFER_BYTE_THRESHOLD;

/// Executor-specific engine cache folder for accumulating processors
fn engine_cache_dir(executor_id: uuid::Uuid) -> PathBuf {
    executor_cache_subdir(executor_id, "processors")
}

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));

/// # Attribute Accumulation Strategy
/// Defines how attributes should be handled when dissolving multiple features into one
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AttributeAccumulationStrategy {
    /// # Drop Incoming Attributes
    /// No attributes from any incoming features will be preserved in the output (except group_by attributes if specified)
    DropAttributes,
    /// # Merge Incoming Attributes
    /// The output feature will merge all input attributes. When multiple features have the same attribute with different values, all values are collected into an array
    MergeAttributes,
    /// # Use Attributes From One Feature
    /// The output inherits the attributes of one representative feature of the group
    #[default]
    UseOneFeature,
}

#[derive(Debug, Clone, Default)]
pub struct DissolverFactory;

impl ProcessorFactory for DissolverFactory {
    fn name(&self) -> &str {
        "Dissolver"
    }

    fn description(&self) -> &str {
        "Dissolve Features by Grouping Attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DissolverParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![AREA_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: DissolverParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::DissolverFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::DissolverFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::DissolverFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = Dissolver {
            group_by: param.group_by,
            // Default tolerance to 0.0 if not specified.
            // TODO: This default value is to not break existing behavior, but should be changed in the future once we have more unit tests.
            tolerance: param.tolerance.unwrap_or(0.0),
            attribute_accumulation: param.attribute_accumulation,
            group_map: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            group_frame: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            reported_groups: HashSet::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        };

        Ok(Box::new(process))
    }
}

/// # Dissolver Parameters
/// Configure how to dissolve features by grouping them based on shared attributes
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DissolverParam {
    /// # Group By Attributes
    /// List of attribute names to group features by before dissolving. Features with the same values for these attributes will be dissolved together
    group_by: Option<Vec<Attribute>>,
    /// # Tolerance
    /// Geometric tolerance. Vertices closer than this distance will be considered identical during the dissolve operation.
    tolerance: Option<f64>,
    /// # Attribute Accumulation
    /// Strategy for handling attributes when dissolving features
    #[serde(default)]
    attribute_accumulation: AttributeAccumulationStrategy,
}

pub struct Dissolver {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    attribute_accumulation: AttributeAccumulationStrategy,
    // Disk-backed state
    group_map: HashMap<AttributeValue, usize>,
    /// The coordinate frame each group's members must share, fixed by the
    /// group's first feature. The dissolve merges point sets, which is only
    /// meaningful within one frame.
    #[cfg(feature = "new-geometry")]
    group_frame: HashMap<usize, CoordinateFrame>,
    /// Groups whose frame mismatch has already been reported, so the same
    /// upstream problem is logged once rather than once per feature.
    #[cfg(feature = "new-geometry")]
    reported_groups: HashSet<usize>,
    group_count: usize,
    temp_dir: Option<PathBuf>,
    // In-memory buffer: group_idx -> compressed zstd bytes (concatenated frames)
    buffer: HashMap<usize, Vec<u8>>,
    buffer_bytes: usize,
    /// Executor ID for cache isolation, set on first process() call
    executor_id: Option<uuid::Uuid>,
}

impl std::fmt::Debug for Dissolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dissolver")
            .field("group_count", &self.group_count)
            .finish_non_exhaustive()
    }
}

impl Clone for Dissolver {
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            tolerance: self.tolerance,
            attribute_accumulation: self.attribute_accumulation.clone(),
            group_map: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            group_frame: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            reported_groups: HashSet::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: self.executor_id,
        }
    }
}

impl Drop for Dissolver {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl Dissolver {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let executor_id = self.executor_id.unwrap_or_else(uuid::Uuid::nil);
            let dir =
                engine_cache_dir(executor_id).join(format!("dissolver-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir)?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn group_file_path(&self, group_idx: usize) -> PathBuf {
        self.temp_dir
            .as_ref()
            .unwrap()
            .join(format!("group_{group_idx:06}.jsonl.zst"))
    }

    fn write_feature(&mut self, group_idx: usize, feature: &Feature) -> Result<(), BoxedError> {
        let feature_json = serde_json::to_string(feature)?;
        self.buffer_bytes += feature_json.len();
        let mut src = feature_json.into_bytes();
        src.push(b'\n');
        let frame = zstd::encode_all(src.as_slice(), 1)?;
        self.buffer.entry(group_idx).or_default().extend(frame);

        if self.buffer_bytes >= ACCUMULATOR_BUFFER_BYTE_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), BoxedError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        self.ensure_temp_dir()?;
        for (group_idx, bytes) in std::mem::take(&mut self.buffer) {
            let path = self.group_file_path(group_idx);
            let mut file = File::options().create(true).append(true).open(path)?;
            file.write_all(&bytes)?;
        }

        self.buffer_bytes = 0;
        Ok(())
    }

    fn read_features_for_group(&self, group_idx: usize) -> Result<Vec<Feature>, BoxedError> {
        let path = self.group_file_path(group_idx);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(zstd::Decoder::new(file)?);
        let mut features = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                features.push(serde_json::from_str(&line)?);
            }
        }
        Ok(features)
    }

    /// Whether the incoming feature's geometry may join `group_idx`. The legacy
    /// geometry world reads no CRS off the geometry, so every group takes it.
    #[cfg(not(feature = "new-geometry"))]
    fn admit_frame(&mut self, _group_idx: usize, _ctx: &ExecutorContext) -> bool {
        true
    }

    /// Whether the incoming feature's geometry may join `group_idx`: a group's
    /// members must all be in the coordinate frame its first feature fixed,
    /// since the dissolve merges their point sets.
    ///
    /// A frame mismatch is a property of the upstream pipeline rather than of
    /// one feature, and so would repeat for every remaining member: it is
    /// reported once per group.
    #[cfg(feature = "new-geometry")]
    fn admit_frame(&mut self, group_idx: usize, ctx: &ExecutorContext) -> bool {
        let Geometry::Euclidean2D(geom_2d) = ctx.feature.geometry.as_ref() else {
            return false;
        };
        let mut leaves = Vec::new();
        flatten_2d(geom_2d, &mut leaves);
        let Some(frame) = leaves.first().map(Leaf2D::frame) else {
            return false;
        };
        let Some(group_frame) = self.group_frame.get(&group_idx) else {
            self.group_frame.insert(group_idx, frame.clone());
            return true;
        };
        if group_frame == frame {
            return true;
        }

        let message = format!(
            "Dissolver rejected a feature in {frame:?}: its group is in {group_frame:?}. \
             Reproject the input to one coordinate frame upstream."
        );
        if self.reported_groups.insert(group_idx) {
            ctx.event_hub.warn_log(Some(ctx.error_span()), message);
        } else {
            ctx.event_hub.debug_log(Some(ctx.error_span()), message);
        }
        false
    }

    fn dissolve_all_groups(&mut self) -> Result<Vec<Feature>, BoxedError> {
        // Flush buffer before reading files
        self.flush_buffer()?;

        let mut dissolved = Vec::new();

        for &group_idx in self.group_map.values() {
            let features = match self.read_features_for_group(group_idx) {
                Ok(f) => f,
                Err(_) => continue,
            };

            if let Some(feature) = self.dissolve_group(features)? {
                dissolved.push(feature);
            }
        }

        // Clean up all group files
        for &group_idx in self.group_map.values() {
            let path = self.group_file_path(group_idx);
            let _ = std::fs::remove_file(path);
        }

        // Reset state
        self.group_map.clear();
        #[cfg(feature = "new-geometry")]
        self.group_frame.clear();
        #[cfg(feature = "new-geometry")]
        self.reported_groups.clear();
        self.group_count = 0;

        Ok(dissolved)
    }

    /// The attributes carried onto a group's dissolved output, under the
    /// configured strategy. `representative` is the group's stand-in feature for
    /// the strategies that take attributes from a single member.
    fn accumulate_attributes(
        &self,
        features: &[Feature],
        representative: Option<&Feature>,
    ) -> IndexMap<Attribute, AttributeValue> {
        match self.attribute_accumulation {
            AttributeAccumulationStrategy::DropAttributes => {
                // Only keep group_by attributes if specified
                if let (Some(group_by), Some(representative)) = (&self.group_by, representative) {
                    group_by
                        .iter()
                        .filter_map(|attr| {
                            let value = representative.attributes.get(attr).cloned()?;
                            Some((attr.clone(), value))
                        })
                        .collect()
                } else {
                    IndexMap::new()
                }
            }
            AttributeAccumulationStrategy::MergeAttributes => {
                // Merge all attributes from all features
                let mut merged_attributes: IndexMap<Attribute, Vec<AttributeValue>> =
                    IndexMap::new();

                for feature in features {
                    for (key, value) in feature.attributes.iter() {
                        merged_attributes
                            .entry(key.clone())
                            .and_modify(|existing| {
                                // Add value if it's not already in the list
                                if !existing.contains(value) {
                                    existing.push(value.clone());
                                }
                            })
                            .or_insert_with(|| vec![value.clone()]);
                    }
                }

                // Convert single-element vectors to single values
                merged_attributes
                    .into_iter()
                    .map(|(key, values)| {
                        let final_value = if values.len() == 1 {
                            values.into_iter().next().unwrap()
                        } else {
                            AttributeValue::Array(values)
                        };
                        (key, final_value)
                    })
                    .collect()
            }
            AttributeAccumulationStrategy::UseOneFeature => representative
                .map(|feature| (*feature.attributes).clone())
                .unwrap_or_default(),
        }
    }
}

impl Processor for Dissolver {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Capture executor_id on first process call for cache isolation
        if self.executor_id.is_none() {
            self.executor_id = Some(fw.executor_id());
        }

        let feature = &ctx.feature;
        if !accepts(feature.geometry.as_ref()) {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let key = if let Some(group_by) = &self.group_by {
            AttributeValue::Array(
                group_by
                    .iter()
                    .filter_map(|attr| feature.attributes.get(attr).cloned())
                    .collect(),
            )
        } else {
            AttributeValue::Null
        };

        // Get or create group index for this key
        let group_idx = if let Some(&idx) = self.group_map.get(&key) {
            idx
        } else {
            let idx = self.group_count;
            self.group_map.insert(key, idx);
            self.group_count += 1;
            idx
        };

        if !self.admit_frame(group_idx, &ctx) {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        self.write_feature(group_idx, &ctx.feature)?;
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for dissolved in self.dissolve_all_groups()? {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                dissolved,
                AREA_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Dissolver"
    }
}

// --- legacy geometry ---------------------------------------------------------

/// Whether the geometry can take part in a dissolve: any non-empty 2D geometry.
#[cfg(not(feature = "new-geometry"))]
fn accepts(geometry: &Geometry) -> bool {
    !geometry.is_empty() && matches!(geometry.value, GeometryValue::FlowGeometry2D(_))
}

/// The group's representative feature: its last one.
#[cfg(not(feature = "new-geometry"))]
fn representative(features: &[Feature]) -> Option<&Feature> {
    features.last()
}

#[cfg(not(feature = "new-geometry"))]
impl Dissolver {
    fn dissolve_group(&self, features: Vec<Feature>) -> Result<Option<Feature>, BoxedError> {
        let attrs = self.accumulate_attributes(&features, representative(&features));

        // Start with an empty multi-polygon
        let mut multi_polygon_2d = MultiPolygon2D::new(vec![]);
        for feature in features {
            let Some(geometry) = feature.geometry.value.as_flow_geometry_2d() else {
                continue;
            };
            let mut multi_polygon = if let Some(mp) = geometry.as_multi_polygon() {
                mp.clone()
            } else if let Some(polygon) = geometry.as_polygon() {
                MultiPolygon2D::new(vec![polygon.clone()])
            } else {
                continue;
            };
            let mut vertices = multi_polygon_2d.get_vertices_mut();
            vertices.extend(multi_polygon.get_vertices_mut());
            glue_vertices_closer_than(self.tolerance, vertices);
            multi_polygon_2d = multi_polygon_2d.union(&multi_polygon);
        }

        // Only create feature if we accumulated some geometry
        if multi_polygon_2d.is_empty() {
            return Ok(None);
        }

        let geometry = Geometry {
            value: GeometryValue::FlowGeometry2D(multi_polygon_2d.into()),
            ..Default::default()
        };
        Ok(Some(Feature::new_with_attributes_and_geometry(
            attrs, geometry,
        )))
    }
}

// --- new geometry ------------------------------------------------------------

/// Whether the geometry can take part in a dissolve: a planar areal geometry
/// (polygons or meshes) whose leaves share one coordinate frame.
///
/// The dissolve reasons about the plane alone, so a leaf placed at an elevation
/// is refused rather than silently merged with one at a different height. Areas
/// are what dissolve, so a line string is refused.
#[cfg(feature = "new-geometry")]
fn accepts(geometry: &Geometry) -> bool {
    let Geometry::Euclidean2D(geom_2d) = geometry else {
        return false;
    };
    let mut leaves = Vec::new();
    flatten_2d(geom_2d, &mut leaves);
    let Some(frame) = leaves.first().map(Leaf2D::frame) else {
        return false;
    };
    leaves.iter().all(|leaf| {
        leaf.frame() == frame
            && leaf_elevation(leaf).is_none()
            && matches!(
                leaf,
                Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_)
            )
    })
}

/// The elevation a 2D leaf lies at, or `None` when it is planar.
#[cfg(feature = "new-geometry")]
fn leaf_elevation(leaf: &Leaf2D<'_>) -> Option<f64> {
    match leaf {
        Leaf2D::Polygon(p) => p.elevation(),
        Leaf2D::PolygonMesh(m) => m.elevation(),
        Leaf2D::TriangularMesh(m) => m.elevation(),
        Leaf2D::Line(l) => l.elevation(),
        Leaf2D::Point(_) => None,
    }
}

/// The group's representative feature: the one covering the most area. Ties go
/// to the earliest feature of the group.
#[cfg(feature = "new-geometry")]
fn representative(features: &[Feature]) -> Option<&Feature> {
    let mut largest: Option<(&Feature, f64)> = None;
    for feature in features {
        let area = covered_area(feature);
        if largest.is_none_or(|(_, most)| area > most) {
            largest = Some((feature, area));
        }
    }
    largest.map(|(feature, _)| feature)
}

/// The total planar area the feature's geometry covers.
#[cfg(feature = "new-geometry")]
fn covered_area(feature: &Feature) -> f64 {
    let Geometry::Euclidean2D(geom_2d) = feature.geometry.as_ref() else {
        return 0.0;
    };
    let mut leaves = Vec::new();
    flatten_2d(geom_2d, &mut leaves);
    leaves.iter().map(Leaf2D::area).sum()
}

/// Dissolved polygons as one geometry.
#[cfg(feature = "new-geometry")]
fn wrap_polygons(mut polygons: Vec<Polygon2D>) -> Euclidean2DGeometry {
    if polygons.len() == 1 {
        Euclidean2DGeometry::Polygon(Box::new(polygons.remove(0)))
    } else {
        Euclidean2DGeometry::Collection(Collection2D::new(
            polygons
                .into_iter()
                .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
        ))
    }
}

#[cfg(feature = "new-geometry")]
impl Dissolver {
    fn dissolve_group(&self, features: Vec<Feature>) -> Result<Option<Feature>, BoxedError> {
        let attrs = self.accumulate_attributes(&features, representative(&features));

        // The whole group dissolves in one call rather than one union per member:
        // the backend rounds its input to a grid, so unioning one member at a
        // time would round the same coordinates again on every pass.
        let mut leaves = Vec::new();
        for feature in &features {
            if let Geometry::Euclidean2D(geom_2d) = feature.geometry.as_ref() {
                flatten_2d(geom_2d, &mut leaves);
            }
        }
        let polygons = dissolve_leaves(&leaves, self.tolerance).map_err(|e| {
            GeometryProcessorError::Dissolver(format!("Failed to dissolve group: {e}"))
        })?;

        // Only create feature if the group enclosed some area
        if polygons.is_empty() {
            return Ok(None);
        }

        Ok(Some(Feature::new_with_attributes_and_geometry(
            attrs,
            Geometry::Euclidean2D(wrap_polygons(polygons)),
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pretty_assertions::assert_eq;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    // --- shared fixtures -----------------------------------------------------

    /// A closed CCW square ring with the corner at `min` and the given side.
    fn square_ring(min: [f64; 2], side: f64) -> Vec<[f64; 2]> {
        vec![
            [min[0], min[1]],
            [min[0] + side, min[1]],
            [min[0] + side, min[1] + side],
            [min[0], min[1] + side],
            [min[0], min[1]],
        ]
    }

    fn attributes(pairs: &[(&str, &str)]) -> IndexMap<Attribute, AttributeValue> {
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.to_string()),
                    AttributeValue::String(v.to_string()),
                )
            })
            .collect()
    }

    fn with_attributes(mut feature: Feature, pairs: &[(&str, &str)]) -> Feature {
        feature.attributes = Arc::new(attributes(pairs));
        feature
    }

    fn attribute(feature: &Feature, key: &str) -> Option<AttributeValue> {
        feature
            .attributes
            .get(&Attribute::new(key.to_string()))
            .cloned()
    }

    fn dissolver(
        group_by: Option<Vec<Attribute>>,
        tolerance: f64,
        attribute_accumulation: AttributeAccumulationStrategy,
    ) -> Dissolver {
        Dissolver {
            group_by,
            tolerance,
            attribute_accumulation,
            group_map: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            group_frame: HashMap::new(),
            #[cfg(feature = "new-geometry")]
            reported_groups: HashSet::new(),
            group_count: 0,
            temp_dir: None,
            buffer: HashMap::new(),
            buffer_bytes: 0,
            executor_id: None,
        }
    }

    /// Feed `features` through the processor and collect what the `area` and
    /// `rejected` ports received.
    fn run(mut processor: Dissolver, features: Vec<Feature>) -> (Vec<Feature>, Vec<Feature>) {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in &features {
            processor
                .process(create_default_execute_context(feature), &fw)
                .unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let sent = noop.send_features.lock().unwrap().clone();
        let mut area = Vec::new();
        let mut rejected = Vec::new();
        for (port, feature) in ports.iter().zip(sent) {
            if *port == *AREA_PORT {
                area.push(feature);
            } else if *port == *REJECTED_PORT {
                rejected.push(feature);
            } else {
                panic!("unexpected port {port:?}");
            }
        }
        (area, rejected)
    }

    // --- per-world fixtures --------------------------------------------------

    /// A feature carrying a square face with the corner at `min`.
    #[cfg(not(feature = "new-geometry"))]
    fn square(min: [f64; 2], side: f64) -> Feature {
        use reearth_flow_geometry::types::{
            coordinate::Coordinate2D, geometry::Geometry2D, line_string::LineString2D,
            polygon::Polygon2D,
        };

        let ring: Vec<Coordinate2D<f64>> = square_ring(min, side)
            .into_iter()
            .map(|[x, y]| Coordinate2D::new_(x, y))
            .collect();
        let polygon = Polygon2D::new(LineString2D::from(ring), vec![]);
        Feature::from(Geometry::with_value(GeometryValue::FlowGeometry2D(
            Geometry2D::Polygon(polygon),
        )))
    }

    #[cfg(feature = "new-geometry")]
    fn square(min: [f64; 2], side: f64) -> Feature {
        Feature::from(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
            Box::new(Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                square_ring(min, side),
                Vec::<Vec<[f64; 2]>>::new(),
            )),
        )))
    }

    /// The total area the output feature's geometry covers.
    #[cfg(not(feature = "new-geometry"))]
    fn output_area(feature: &Feature) -> f64 {
        use reearth_flow_geometry::algorithm::area2d::Area2D;

        feature
            .geometry
            .value
            .as_flow_geometry_2d()
            .and_then(|g| g.as_multi_polygon().map(|mp| mp.unsigned_area2d()))
            .unwrap_or(0.0)
    }

    #[cfg(feature = "new-geometry")]
    fn output_area(feature: &Feature) -> f64 {
        covered_area(feature)
    }

    /// The number of separate faces the output feature's geometry holds.
    #[cfg(not(feature = "new-geometry"))]
    fn output_faces(feature: &Feature) -> usize {
        feature
            .geometry
            .value
            .as_flow_geometry_2d()
            .and_then(|g| g.as_multi_polygon().map(|mp| mp.0.len()))
            .unwrap_or(0)
    }

    #[cfg(feature = "new-geometry")]
    fn output_faces(feature: &Feature) -> usize {
        let Geometry::Euclidean2D(geom_2d) = feature.geometry.as_ref() else {
            return 0;
        };
        let mut leaves = Vec::new();
        flatten_2d(geom_2d, &mut leaves);
        leaves.len()
    }

    // --- behaviour shared by both geometry worlds ----------------------------

    #[test]
    fn adjacent_squares_in_one_group_dissolve_into_one_feature() {
        let (area, rejected) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![square([0.0, 0.0], 2.0), square([2.0, 0.0], 2.0)],
        );
        assert_eq!(rejected.len(), 0);
        assert_eq!(area.len(), 1);
        assert_eq!(output_faces(&area[0]), 1);
        assert_eq!(output_area(&area[0]), 8.0);
    }

    #[test]
    fn group_by_dissolves_each_group_separately() {
        let features = vec![
            with_attributes(square([0.0, 0.0], 2.0), &[("side", "west")]),
            with_attributes(square([2.0, 0.0], 2.0), &[("side", "west")]),
            with_attributes(square([10.0, 0.0], 2.0), &[("side", "east")]),
        ];
        let (area, rejected) = run(
            dissolver(
                Some(vec![Attribute::new("side".to_string())]),
                0.0,
                AttributeAccumulationStrategy::UseOneFeature,
            ),
            features,
        );
        assert_eq!(rejected.len(), 0);
        assert_eq!(area.len(), 2);
        let mut areas: Vec<f64> = area.iter().map(output_area).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas, vec![4.0, 8.0]);
    }

    #[test]
    fn separate_squares_in_one_group_stay_separate_faces() {
        let (area, _) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![square([0.0, 0.0], 2.0), square([10.0, 0.0], 2.0)],
        );
        assert_eq!(area.len(), 1);
        assert_eq!(output_faces(&area[0]), 2);
        assert_eq!(output_area(&area[0]), 8.0);
    }

    #[test]
    fn merge_attributes_collects_differing_values_into_an_array() {
        let features = vec![
            with_attributes(square([0.0, 0.0], 2.0), &[("name", "a"), ("kind", "road")]),
            with_attributes(square([2.0, 0.0], 2.0), &[("name", "b"), ("kind", "road")]),
        ];
        let (area, _) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::MergeAttributes),
            features,
        );
        assert_eq!(area.len(), 1);
        assert_eq!(
            attribute(&area[0], "name"),
            Some(AttributeValue::Array(vec![
                AttributeValue::String("a".to_string()),
                AttributeValue::String("b".to_string()),
            ]))
        );
        // A value the whole group agrees on stays a single value.
        assert_eq!(
            attribute(&area[0], "kind"),
            Some(AttributeValue::String("road".to_string()))
        );
    }

    #[test]
    fn drop_attributes_keeps_only_the_group_by_attributes() {
        let feature = with_attributes(
            square([0.0, 0.0], 2.0),
            &[("side", "west"), ("name", "dropped")],
        );
        let (area, _) = run(
            dissolver(
                Some(vec![Attribute::new("side".to_string())]),
                0.0,
                AttributeAccumulationStrategy::DropAttributes,
            ),
            vec![feature],
        );
        assert_eq!(area.len(), 1);
        assert_eq!(
            attribute(&area[0], "side"),
            Some(AttributeValue::String("west".to_string()))
        );
        assert_eq!(attribute(&area[0], "name"), None);
    }

    #[test]
    fn an_empty_input_produces_no_output() {
        let (area, rejected) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![],
        );
        assert_eq!((area.len(), rejected.len()), (0, 0));
    }

    #[test]
    fn the_tolerance_merges_squares_whose_shared_edge_nearly_coincides() {
        // The right neighbour's left edge sits 0.001 away from the left one's.
        let nearly_adjacent = || vec![square([0.0, 0.0], 2.0), square([2.001, 0.0], 2.0)];

        let (untouched, _) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            nearly_adjacent(),
        );
        assert_eq!(untouched.len(), 1);
        assert_eq!(output_faces(&untouched[0]), 2);

        let (snapped, _) = run(
            dissolver(None, 0.01, AttributeAccumulationStrategy::UseOneFeature),
            nearly_adjacent(),
        );
        assert_eq!(snapped.len(), 1);
        assert_eq!(output_faces(&snapped[0]), 1);
    }

    // --- legacy geometry -----------------------------------------------------

    /// The legacy world keeps taking single-feature attributes from the group's
    /// last member, where the new world takes them from its largest.
    #[test]
    #[cfg(not(feature = "new-geometry"))]
    fn use_one_feature_takes_the_last_features_attributes() {
        let features = vec![
            with_attributes(square([1.0, 0.0], 4.0), &[("name", "large")]),
            with_attributes(square([0.0, 0.0], 1.0), &[("name", "small")]),
        ];
        let (area, _) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            features,
        );
        assert_eq!(area.len(), 1);
        assert_eq!(
            attribute(&area[0], "name"),
            Some(AttributeValue::String("small".to_string()))
        );
    }

    // --- new geometry --------------------------------------------------------

    #[test]
    #[cfg(feature = "new-geometry")]
    fn use_one_feature_takes_the_largest_features_attributes() {
        // The large face comes first, so "the group's last feature" would pick
        // the small one: the choice really is by area.
        let features = vec![
            with_attributes(square([1.0, 0.0], 4.0), &[("name", "large")]),
            with_attributes(square([0.0, 0.0], 1.0), &[("name", "small")]),
        ];
        let (area, _) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            features,
        );
        assert_eq!(area.len(), 1);
        assert_eq!(
            attribute(&area[0], "name"),
            Some(AttributeValue::String("large".to_string()))
        );
    }

    #[cfg(feature = "new-geometry")]
    fn rejects(feature: Feature) -> bool {
        let (area, rejected) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![feature],
        );
        area.is_empty() && rejected.len() == 1
    }

    #[test]
    #[cfg(feature = "new-geometry")]
    fn geometries_that_enclose_no_area_are_rejected_rather_than_dropped() {
        use reearth_flow_geometry::line_string::LineString2D;
        use reearth_flow_geometry::point::Point2D;
        use reearth_flow_geometry::polygon::Polygon3D;
        use reearth_flow_geometry::Euclidean3DGeometry;

        assert!(rejects(Feature::from(Geometry::None)));

        assert!(rejects(Feature::from(Geometry::Euclidean2D(
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [1.0, 1.0]))
        ))));

        // A closed line is still a line, not an area.
        assert!(rejects(Feature::from(Geometry::Euclidean2D(
            Euclidean2DGeometry::LineString(LineString2D::from_coords(
                CoordinateFrame::Euclidean,
                square_ring([0.0, 0.0], 2.0),
            ))
        ))));

        // 3D has no boolean overlay to dissolve with.
        assert!(rejects(Feature::from(Geometry::Euclidean3D(
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                [
                    [0.0, 0.0, 0.0],
                    [2.0, 0.0, 0.0],
                    [2.0, 2.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            )))
        ))));
    }

    #[test]
    #[cfg(feature = "new-geometry")]
    fn an_elevated_face_is_rejected() {
        // The dissolve reasons about the plane alone, so a face lifted off it is
        // refused rather than silently flattened onto its neighbours.
        assert!(rejects(Feature::from(Geometry::Euclidean2D(
            Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings_at_elevation(
                CoordinateFrame::Euclidean,
                square_ring([0.0, 0.0], 2.0),
                Vec::<Vec<[f64; 2]>>::new(),
                5.0,
            )))
        ))));
    }

    /// A square face with the corner at `min`, in `frame`.
    #[cfg(feature = "new-geometry")]
    fn square_in(frame: CoordinateFrame, min: [f64; 2], side: f64) -> Feature {
        Feature::from(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
            Box::new(Polygon2D::from_rings(
                frame,
                square_ring(min, side),
                Vec::<Vec<[f64; 2]>>::new(),
            )),
        )))
    }

    #[test]
    #[cfg(feature = "new-geometry")]
    fn a_group_rejects_members_outside_the_frame_its_first_feature_fixed() {
        use reearth_flow_geometry::coordinate::EpsgCode;

        let (area, rejected) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![
                square([0.0, 0.0], 2.0),
                square_in(CoordinateFrame::Crs(EpsgCode::new(6677)), [2.0, 0.0], 3.0),
            ],
        );
        assert_eq!(area.len(), 1);
        assert_eq!(output_area(&area[0]), 4.0);
        assert_eq!(rejected.len(), 1);
        assert_eq!(output_area(&rejected[0]), 9.0);
    }

    #[test]
    #[cfg(feature = "new-geometry")]
    fn the_output_keeps_the_groups_coordinate_frame() {
        use reearth_flow_geometry::coordinate::EpsgCode;

        let frame = CoordinateFrame::Crs(EpsgCode::new(6677));
        let (area, _) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![
                square_in(frame.clone(), [0.0, 0.0], 2.0),
                square_in(frame.clone(), [2.0, 0.0], 2.0),
            ],
        );
        assert_eq!(area.len(), 1);
        let Geometry::Euclidean2D(geom_2d) = area[0].geometry.as_ref() else {
            panic!("expected a 2D geometry");
        };
        let mut leaves = Vec::new();
        flatten_2d(geom_2d, &mut leaves);
        assert!(!leaves.is_empty());
        assert!(leaves.iter().all(|leaf| leaf.frame() == &frame));
    }

    #[test]
    #[cfg(feature = "new-geometry")]
    fn a_mesh_dissolves_with_the_polygon_it_adjoins() {
        use reearth_flow_geometry::triangular_mesh::TriangularMesh2D;

        // Two triangles covering [0,2] x [0,2], adjoining the square at x = 2.
        let mesh = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let mesh = Feature::from(Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(
            Box::new(mesh),
        )));

        let (area, rejected) = run(
            dissolver(None, 0.0, AttributeAccumulationStrategy::UseOneFeature),
            vec![mesh, square([2.0, 0.0], 2.0)],
        );
        assert_eq!(rejected.len(), 0);
        assert_eq!(area.len(), 1);
        assert_eq!(output_faces(&area[0]), 1);
        assert_eq!(output_area(&area[0]), 8.0);
    }
}

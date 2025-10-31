use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reearth_flow_geometry::{
    algorithm::{
        bool_ops::BooleanOps, bounding_rect::BoundingRect, tolerance::glue_vertices_closer_than,
        utils::normalize_vertices_2d,
    },
    types::{
        coordinate::Coordinate2D, line_string::LineString2D, multi_polygon::MultiPolygon2D,
        polygon::Polygon2D, rect::Rect2D,
    },
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use rstar::{RTree, RTreeObject, AABB};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));
static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

#[derive(Debug, Clone, Default)]
pub(super) struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }

    fn description(&self) -> &str {
        "Perform Area Overlay Analysis"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaOnAreaOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            AREA_PORT.clone(),
            REMNANTS_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: AreaOnAreaOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::AreaOnAreaOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = AreaOnAreaOverlayer {
            group_by: param.group_by,
            output_attribute: param.output_attribute,
            generate_list: param.generate_list,
            accumulation_mode: param.accumulation_mode,
            // Default tolerance to 0.0 if not specified.
            // TODO: This default value is to not break existing behavior, but should be changed in the future once we have more unit tests.
            tolerance: param.tolerance.unwrap_or(0.0),
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

/// # AreaOnAreaOverlayer Parameters
/// Configure how area overlay analysis is performed
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaOnAreaOverlayerParam {
    /// # Group By Attributes
    /// Optional attributes to group features by during overlay analysis
    group_by: Option<Vec<Attribute>>,

    /// # Accumulation Mode
    /// Controls how attributes from input features are handled in output features
    #[serde(default)]
    accumulation_mode: AccumulationMode,

    /// # Generate List
    /// Name of the list attribute to store source feature attributes
    generate_list: Option<String>,

    /// # Output Attribute
    /// Name of the attribute to store overlap count
    output_attribute: Option<String>,

    /// # Tolerance
    /// Geometric tolerance. Vertices closer than this distance will be considered identical during the overlay operation.
    tolerance: Option<f64>,
}

#[derive(Debug, Clone)]
struct AreaOnAreaOverlayer {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Option<String>,
    generate_list: Option<String>,
    accumulation_mode: AccumulationMode,
    tolerance: f64,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AccumulationMode {
    #[default]
    UseAttributesFromOneFeature,
    DropIncomingAttributes,
}

impl Processor for AreaOnAreaOverlayer {
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
            GeometryValue::FlowGeometry2D(_) => {
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

                if !self.buffer.contains_key(&key) {
                    let overlaid = self.overlay();
                    for feature in &overlaid.area {
                        fw.send(ctx.new_with_feature_and_port(feature.clone(), AREA_PORT.clone()));
                    }
                    for feature in &overlaid.remnant {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REMNANTS_PORT.clone()),
                        );
                    }
                    self.buffer.clear();
                }

                self.buffer
                    .entry(key.clone())
                    .or_default()
                    .push(feature.clone());
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let overlaid = self.overlay();
        for feature in &overlaid.area {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                AREA_PORT.clone(),
            ));
        }
        for feature in &overlaid.remnant {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REMNANTS_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }
}

/// Polygon that is created in the middle of the overlay process.
#[derive(Debug, Clone)]
struct MiddlePolygon {
    polygon: MultiPolygon2D<f64>,
    parents: Vec<usize>,
}

/// Type of the subpolygon and its parents.
enum MiddlePolygonType {
    None,
    Area(Vec<usize>),
    Remnant(usize),
}

impl MiddlePolygon {
    fn get_type(&self) -> MiddlePolygonType {
        match self.parents.len() {
            0 => MiddlePolygonType::None,
            1 => MiddlePolygonType::Remnant(self.parents[0]),
            _ => MiddlePolygonType::Area(self.parents.clone()),
        }
    }
}

impl AreaOnAreaOverlayer {
    fn overlay(&self) -> OverlaidFeatures {
        let mut overlaid = OverlaidFeatures::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();
            let mut polygons = buffered_features_2d
                .iter()
                .filter_map(|f| f.geometry.value.as_flow_geometry_2d())
                .filter_map(|g| {
                    // Try to get MultiPolygon directly
                    if let Some(multi_polygon) = g.as_multi_polygon() {
                        return Some(multi_polygon);
                    }

                    // Try to get polygon directly
                    if let Some(polygon) = g.as_polygon() {
                        return Some(MultiPolygon2D::new(vec![polygon]));
                    }

                    // If it's a closed LineString, convert to Polygon
                    if let Some(linestring) = g.as_line_string() {
                        let coords = linestring.coords().collect::<Vec<_>>();
                        if coords.len() >= 4 && coords.first() == coords.last() {
                            // Create polygon from closed linestring
                            use reearth_flow_geometry::types::polygon::Polygon2D;
                            let polygon = Polygon2D::new(linestring.clone(), vec![]);
                            return Some(MultiPolygon2D::new(vec![polygon]));
                        }
                    }

                    None
                })
                .collect::<Vec<_>>();

            // glue vertices that are closer than the tolerance
            let mut vertices = Vec::new();
            for polygon in &mut polygons {
                vertices.extend(polygon.get_vertices_mut());
            }
            glue_vertices_closer_than(self.tolerance, vertices);

            let midpolygons = overlay_2d(polygons);

            let overlaid_features = OverlaidFeatures::from_midpolygons(
                midpolygons,
                buffered_features_2d
                    .iter()
                    .map(|f| f.attributes.clone())
                    .collect(),
                &self.group_by,
                &self.output_attribute,
                &self.generate_list,
                &self.accumulation_mode,
            );
            overlaid.extend(overlaid_features);
        }

        overlaid
    }
}

fn overlay_2d(mut polygons: Vec<MultiPolygon2D<f64>>) -> Vec<MiddlePolygon> {
    // normalize vertices
    // TODO: This can be removed to improve performance when we choose the right coordinate system.
    let (avg, norm_avg) = normalize_vertices_2d_for_multipolygons(&mut polygons);
    let overlay_graph = OverlayGraph::bulk_load(&polygons);

    // all (devided) polygons to output
    (0..polygons.len())
        .into_par_iter()
        .map(|i| {
            let mut polygon_target = polygons[i].clone();

            // cut off the target polygon by upper polygons
            for j in overlay_graph.overlaid_iter(i).copied() {
                if i < j {
                    polygon_target = polygon_target.difference(&polygons[j]);
                }
            }

            let mut queue = vec![MiddlePolygon {
                polygon: polygon_target,
                parents: vec![i],
            }];

            // divide the target polygon by lower polygons
            for j in overlay_graph.overlaid_iter(i).copied() {
                if i > j {
                    let mut new_queue = Vec::new();
                    for subpolygon in queue {
                        let intersection = subpolygon.polygon.intersection(&polygons[j]);

                        if !intersection.is_empty() {
                            new_queue.push(MiddlePolygon {
                                polygon: intersection,
                                parents: subpolygon
                                    .parents
                                    .clone()
                                    .into_iter()
                                    .chain(vec![j])
                                    .collect(),
                            });
                        }

                        let difference = subpolygon.polygon.difference(&polygons[j]);
                        if !difference.is_empty() {
                            new_queue.push(MiddlePolygon {
                                polygon: difference,
                                parents: subpolygon.parents.clone(),
                            });
                        }
                    }
                    queue = new_queue;
                }
            }

            queue
        })
        .flatten()
        .map(|mut p| {
            p.polygon.denormalize_vertices_2d(avg, norm_avg);
            p
        })
        .collect::<Vec<_>>()
}

struct PolygonWithMbr2D {
    index: usize,
    mbr: Rect2D<f64>,
}

impl PolygonWithMbr2D {
    fn new(mbr: Rect2D<f64>, index: usize) -> Option<Self> {
        Some(Self { index, mbr })
    }
}

impl rstar::RTreeObject for PolygonWithMbr2D {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.mbr.envelope()
    }
}

/// Overlay graph that stores which polygons overlay which other polygons.
struct OverlayGraph {
    /// Adjacency list representation of the overlay graph.
    /// Each index corresponds to a polygon, and the set contains indices of polygons whose AABB intersects with the polygon at that index.
    graph: Vec<HashSet<usize>>,
}

impl OverlayGraph {
    fn bulk_load(polygons: &[MultiPolygon2D<f64>]) -> Self {
        let polygon_mbrs = polygons
            .iter()
            .map(|p| p.bounding_box())
            .collect::<Vec<_>>();

        let polygon_tree = RTree::bulk_load(
            polygon_mbrs
                .iter()
                .enumerate()
                .filter_map(|(i, mbr)| mbr.as_ref().and_then(|mbr| PolygonWithMbr2D::new(*mbr, i)))
                .collect::<Vec<_>>(),
        );

        let mut graph = vec![HashSet::new(); polygons.len()];

        for i in 0..polygons.len() {
            let mbr_i = if let Some(polygon_mbr) = &polygon_mbrs[i] {
                polygon_mbr
            } else {
                continue;
            };

            let overlaids =
                polygon_tree.locate_in_envelope_intersecting(&mbr_i.bounding_rect().envelope());

            for overlaid in overlaids {
                graph[i].insert(overlaid.index);
            }
        }

        Self { graph }
    }

    fn overlaid_iter(&self, i: usize) -> impl Iterator<Item = &usize> {
        self.graph[i].iter()
    }
}

/// Features that are created as the result of the overlay process.
#[derive(Debug)]
struct OverlaidFeatures {
    area: Vec<Feature>,
    remnant: Vec<Feature>,
}

impl OverlaidFeatures {
    fn new() -> Self {
        Self {
            area: Vec::new(),
            remnant: Vec::new(),
        }
    }

    fn from_midpolygons(
        midpolygons: Vec<MiddlePolygon>,
        base_attributes: Vec<IndexMap<Attribute, AttributeValue>>,
        _group_by: &Option<Vec<Attribute>>,
        output_attribute: &Option<String>,
        generate_list: &Option<String>,
        accumulation_mode: &AccumulationMode,
    ) -> Self {
        let mut area = Vec::new();
        let mut remnant = Vec::new();
        for subpolygon in midpolygons {
            match subpolygon.get_type() {
                MiddlePolygonType::None => {}
                MiddlePolygonType::Area(parents) => {
                    let mut feature = Feature::new();

                    // Handle attributes based on accumulation mode
                    match accumulation_mode {
                        AccumulationMode::DropIncomingAttributes => {
                            feature.attributes = IndexMap::new();
                        }
                        AccumulationMode::UseAttributesFromOneFeature => {
                            let first_feature = &base_attributes[parents[0]];
                            feature.attributes = first_feature.clone();
                        }
                    }

                    // Add overlap count attribute if specified
                    if let Some(attr_name) = output_attribute {
                        let overlap_count = parents.len();
                        feature.attributes.insert(
                            Attribute::new(attr_name.clone()),
                            AttributeValue::Number(overlap_count.into()),
                        );
                    }

                    // Add generate list attribute if specified
                    if let Some(list_name) = generate_list {
                        let list_items: Vec<AttributeValue> = parents
                            .iter()
                            .map(|&parent_index| {
                                let mut map = HashMap::new();
                                for (attr, value) in &base_attributes[parent_index] {
                                    map.insert(attr.as_ref().to_string(), value.clone());
                                }
                                AttributeValue::Map(map)
                            })
                            .collect();

                        feature.attributes.insert(
                            Attribute::new(list_name.clone()),
                            AttributeValue::Array(list_items.clone()),
                        );
                    }

                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                    area.push(feature);
                }
                MiddlePolygonType::Remnant(parent) => {
                    let mut feature = Feature::new();

                    // Handle attributes based on accumulation mode
                    match accumulation_mode {
                        AccumulationMode::DropIncomingAttributes => {
                            feature.attributes = IndexMap::new();
                        }
                        AccumulationMode::UseAttributesFromOneFeature => {
                            feature.attributes = base_attributes[parent].clone();
                        }
                    }

                    // Add overlap count attribute if specified (remnants have overlap count of 1)
                    if let Some(attr_name) = output_attribute {
                        feature.attributes.insert(
                            Attribute::new(attr_name.clone()),
                            AttributeValue::Number(1.into()),
                        );
                    }

                    // Add generate list attribute if specified (single item for remnants)
                    if let Some(list_name) = generate_list {
                        let mut map = HashMap::new();
                        for (attr, value) in &base_attributes[parent] {
                            map.insert(attr.as_ref().to_string(), value.clone());
                        }
                        let list_items = vec![AttributeValue::Map(map)];

                        feature.attributes.insert(
                            Attribute::new(list_name.clone()),
                            AttributeValue::Array(list_items),
                        );
                    }

                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                    remnant.push(feature);
                }
            }
        }
        Self { area, remnant }
    }

    fn extend(&mut self, other: Self) {
        self.area.extend(other.area);
        self.remnant.extend(other.remnant);
    }
}

fn normalize_vertices_2d_for_multipolygons(
    polygons: &mut [MultiPolygon2D<f64>],
) -> (Coordinate2D<f64>, Coordinate2D<f64>) {
    let mut all_vertices = Vec::new();

    for multi_polygon in polygons.iter() {
        for polygon in &multi_polygon.0 {
            for coord in polygon.exterior().coords() {
                all_vertices.push(*coord);
            }
            for interior in polygon.interiors() {
                for coord in interior.coords() {
                    all_vertices.push(*coord);
                }
            }
        }
    }

    let (avg, norm_avg) = normalize_vertices_2d(&mut all_vertices);

    let mut index = 0;

    for multi_polygon in polygons.iter_mut() {
        multi_polygon.0 = multi_polygon
            .0
            .iter()
            .map(|p| {
                let mut exterior = Vec::new();
                for _ in p.exterior().coords() {
                    exterior.push(all_vertices[index]);
                    index += 1;
                }
                let exterior = LineString2D::new(exterior);

                let mut interiors = Vec::new();
                for interior in p.interiors() {
                    let mut coords = Vec::new();
                    for _ in interior.coords() {
                        coords.push(all_vertices[index]);
                        index += 1;
                    }
                    interiors.push(LineString2D::new(coords));
                }

                Polygon2D::new(exterior, interiors)
            })
            .collect::<Vec<_>>();
    }

    (avg, norm_avg)
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::{
        coordinate::Coordinate2D, line_string::LineString2D, polygon::Polygon2D,
    };

    use super::*;

    #[test]
    fn test_overlay_two_squares() {
        let polygons = vec![
            MultiPolygon2D::new(vec![Polygon2D::new(
                LineString2D::new(vec![
                    Coordinate2D::new_(0.0, 0.0),
                    Coordinate2D::new_(2.0, 0.0),
                    Coordinate2D::new_(2.0, 2.0),
                    Coordinate2D::new_(0.0, 2.0),
                    Coordinate2D::new_(0.0, 0.0),
                ]),
                vec![],
            )]),
            MultiPolygon2D::new(vec![Polygon2D::new(
                LineString2D::new(vec![
                    Coordinate2D::new_(1.0, 1.0),
                    Coordinate2D::new_(3.0, 1.0),
                    Coordinate2D::new_(3.0, 3.0),
                    Coordinate2D::new_(1.0, 3.0),
                    Coordinate2D::new_(1.0, 1.0),
                ]),
                vec![],
            )]),
        ];

        let midpolygons = overlay_2d(polygons);
        assert_eq!(midpolygons.len(), 3);
    }

    #[test]
    fn test_overlay_triangles_sharing_an_edge() {
        let polygons = vec![
            MultiPolygon2D::new(vec![Polygon2D::new(
                LineString2D::new(vec![
                    Coordinate2D::new_(0.0, 0.0),
                    Coordinate2D::new_(2.0, 0.0),
                    Coordinate2D::new_(1.0, 2.0),
                    Coordinate2D::new_(0.0, 0.0),
                ]),
                vec![],
            )]),
            MultiPolygon2D::new(vec![Polygon2D::new(
                LineString2D::new(vec![
                    Coordinate2D::new_(0.0, 0.0),
                    Coordinate2D::new_(2.0, 0.0),
                    Coordinate2D::new_(1.0, 1.0),
                    Coordinate2D::new_(0.0, 0.0),
                ]),
                vec![],
            )]),
        ];

        let midpolygons = overlay_2d(polygons);
        assert_eq!(midpolygons.len(), 2);
    }
}

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reearth_flow_geometry::{
    algorithm::{bool_ops::BooleanOps, bounding_rect::BoundingRect},
    types::{multi_polygon::MultiPolygon2D, rect::Rect2D},
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
        "Overlays an area on another area"
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
                    "Failed to serialize 'with' parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize 'with' parameter: {}",
                    e
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
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaOnAreaOverlayerParam {
    /// # Group by
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
struct AreaOnAreaOverlayer {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
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
                    let overlayed = self.overlay();
                    for feature in &overlayed.area {
                        fw.send(ctx.new_with_feature_and_port(feature.clone(), AREA_PORT.clone()));
                    }
                    for feature in &overlayed.remnant {
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
        let overlayed = self.overlay();
        for feature in &overlayed.area {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                AREA_PORT.clone(),
            ));
        }
        for feature in &overlayed.remnant {
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
    fn overlay(&self) -> OverlayedFeatures {
        let mut overlayed = OverlayedFeatures::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();
            overlayed.extend(self.overlay_2d(buffered_features_2d));
        }

        overlayed
    }

    fn overlay_2d(&self, buffered_features_2d: Vec<&Feature>) -> OverlayedFeatures {
        let polygons_incoming = buffered_features_2d
            .iter()
            .filter_map(|f| f.geometry.value.as_flow_geometry_2d())
            .filter_map(|g| {
                g.as_polygon()
                    .map(|polygon| MultiPolygon2D::new(vec![polygon]))
            })
            .collect::<Vec<_>>();

        let overlay_graph = OverlayGraph::bulk_load(&polygons_incoming);

        // all (devided) polygons to output
        let midpolygons = (0..polygons_incoming.len())
            .into_par_iter()
            .map(|i| {
                let mut polygon_target = polygons_incoming[i].clone();

                // cut off the target polygon by upper polygons
                for j in overlay_graph.overlayed_iter(i).copied() {
                    if i < j {
                        polygon_target = polygon_target.difference(&polygons_incoming[j]);
                    }
                }

                let mut queue = vec![MiddlePolygon {
                    polygon: polygon_target,
                    parents: vec![i],
                }];

                // divide the target polygon by lower polygons
                for j in overlay_graph.overlayed_iter(i).copied() {
                    if i > j {
                        let mut new_queue = Vec::new();
                        for subpolygon in queue {
                            let intersected =
                                subpolygon.polygon.intersection(&polygons_incoming[j]);
                            new_queue.push(MiddlePolygon {
                                polygon: intersected,
                                parents: subpolygon
                                    .parents
                                    .clone()
                                    .into_iter()
                                    .chain(vec![j])
                                    .collect(),
                            });

                            let difference = subpolygon.polygon.difference(&polygons_incoming[j]);
                            new_queue.push(MiddlePolygon {
                                polygon: difference,
                                parents: subpolygon.parents.clone(),
                            });
                        }
                        queue = new_queue;
                    }
                }

                queue
            })
            .flatten()
            .collect::<Vec<_>>();

        OverlayedFeatures::from_midpolygons(
            midpolygons,
            buffered_features_2d
                .iter()
                .map(|f| f.attributes.clone())
                .collect(),
            &self.group_by,
        )
    }
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

struct OverlayGraph {
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

            let overlayeds =
                polygon_tree.locate_in_envelope_intersecting(&mbr_i.bounding_rect().envelope());

            for overlayed in overlayeds {
                graph[i].insert(overlayed.index);
            }
        }

        Self { graph }
    }

    fn overlayed_iter(&self, i: usize) -> impl Iterator<Item = &usize> {
        self.graph[i].iter()
    }
}

/// Features that are created as the result of the overlay process.
struct OverlayedFeatures {
    area: Vec<Feature>,
    remnant: Vec<Feature>,
}

impl OverlayedFeatures {
    fn new() -> Self {
        Self {
            area: Vec::new(),
            remnant: Vec::new(),
        }
    }

    fn from_midpolygons(
        midpolygons: Vec<MiddlePolygon>,
        base_attributes: Vec<IndexMap<Attribute, AttributeValue>>,
        group_by: &Option<Vec<Attribute>>,
    ) -> Self {
        let mut area = Vec::new();
        let mut remnant = Vec::new();
        for subpolygon in midpolygons {
            match subpolygon.get_type() {
                MiddlePolygonType::None => {}
                MiddlePolygonType::Area(parents) => {
                    let mut feature = Feature::new();
                    let last_feature = &base_attributes[*parents.last().unwrap()];
                    if let Some(group_by) = group_by {
                        feature.attributes = group_by
                            .iter()
                            .filter_map(|attr| {
                                let value = last_feature.get(attr).cloned()?;
                                Some((attr.clone(), value))
                            })
                            .collect::<IndexMap<_, _>>();
                    } else {
                        feature.attributes = IndexMap::new();
                    }

                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                    area.push(feature);
                }
                MiddlePolygonType::Remnant(parent) => {
                    let mut feature = Feature::new();
                    feature.attributes = base_attributes[parent].clone();
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

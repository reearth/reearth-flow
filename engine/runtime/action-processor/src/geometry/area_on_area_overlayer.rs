use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::bufferable::buffer_polygon;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon, MultiPolygon2D};
use reearth_flow_geometry::types::point::Point2D;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon2DFloat};
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry};
use reearth_flow_types::{Feature, GeometryValue};
use rstar::{RTree, RTreeObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

// const EPSILON: f64 = 0.001;
const TOLERANCE: f64 = 0.2;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));
pub static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

#[derive(Debug, Clone, Default)]
pub struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }

    fn description(&self) -> &str {
        "Overlays an area on another area"
    }

    fn parameter_schema(&self) -> Option<schemars::Schema> {
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
        let params: AreaOnAreaOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::AreaOnAreaOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(AreaOnAreaOverlayer {
            params,
            buffer: HashMap::new(),
            previous_group_key: None,
        }))
    }
}

#[derive(Debug, Clone)]
struct PolygonFeature {
    feature_id: uuid::Uuid,
    geometry: Polygon2D<f64>,
}

impl rstar::RTreeObject for PolygonFeature {
    type Envelope = rstar::AABB<Point2D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.geometry.envelope()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AreaOnAreaOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Attribute,
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct AreaOnAreaOverlayer {
    params: AreaOnAreaOverlayerParam,
    buffer: HashMap<String, (bool, Vec<Feature>, RTree<PolygonFeature>)>, // (complete_grouped, features)
    previous_group_key: Option<String>,
}

impl Processor for AreaOnAreaOverlayer {
    fn num_threads(&self) -> usize {
        10
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::FlowGeometry2D(geometry) => {
                let key = if let Some(group_by) = &self.params.group_by {
                    group_by
                        .iter()
                        .map(|k| feature.get(&k).map(|v| v.to_string()).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    "_all".to_string()
                };

                match self.buffer.entry(key.clone()) {
                    Entry::Occupied(mut entry) => {
                        self.previous_group_key = Some(key.clone());
                        {
                            let (_, buffer, rtree) = entry.get_mut();
                            let feature = feature.clone();
                            match geometry {
                                Geometry2D::Polygon(poly) => {
                                    let Some(poly) = buffer_polygon(poly, -TOLERANCE) else {
                                        return Ok(());
                                    };
                                    rtree.insert(PolygonFeature {
                                        feature_id: feature.id,
                                        geometry: poly,
                                    });
                                }
                                Geometry2D::MultiPolygon(mpoly) => {
                                    for poly in mpoly.iter() {
                                        let Some(poly) = buffer_polygon(poly, -TOLERANCE) else {
                                            return Ok(());
                                        };
                                        rtree.insert(PolygonFeature {
                                            feature_id: feature.id,
                                            geometry: poly,
                                        });
                                    }
                                }
                                _ => {
                                    return Ok(());
                                }
                            }
                            buffer.push(feature);
                        }
                    }
                    Entry::Vacant(entry) => {
                        let mut rtree = RTree::new();
                        match geometry {
                            Geometry2D::Polygon(poly) => {
                                let Some(poly) = buffer_polygon(poly, -TOLERANCE) else {
                                    return Ok(());
                                };
                                rtree.insert(PolygonFeature {
                                    feature_id: feature.id,
                                    geometry: poly,
                                });
                            }
                            Geometry2D::MultiPolygon(mpoly) => {
                                for poly in mpoly.iter() {
                                    let Some(poly) = buffer_polygon(poly, -TOLERANCE) else {
                                        return Ok(());
                                    };
                                    rtree.insert(PolygonFeature {
                                        feature_id: feature.id,
                                        geometry: poly,
                                    });
                                }
                            }
                            _ => {
                                return Ok(());
                            }
                        }
                        entry.insert((false, vec![feature.clone()], rtree));
                        if let Some(previous_group_key) = &self.previous_group_key {
                            if let Entry::Occupied(mut entry) =
                                self.buffer.entry(previous_group_key.clone())
                            {
                                let (complete_grouped_change, _, _) = entry.get_mut();
                                *complete_grouped_change = true;
                            }
                            self.change_group(ctx, fw);
                        }
                        self.previous_group_key = Some(key.clone());
                    }
                }
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (_, (_, features, rtree)) in self.buffer.iter() {
            self.handle_2d_geometry(ctx.as_context(), features, rtree, fw);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }
}

impl AreaOnAreaOverlayer {
    fn handle_2d_geometry(
        &self,
        ctx: Context,
        targets: &[Feature],
        rtree: &RTree<PolygonFeature>,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        let mut polygon_features = HashMap::<Polygon2DFloat, (HashSet<uuid::Uuid>, u64)>::new();
        let target_features = targets
            .iter()
            .map(|feature| (feature.id, feature.clone()))
            .collect::<HashMap<_, _>>();
        for target_feature in target_features.values() {
            let Some(MultiPolygon(target)) =
                self.handle_2d_polygon_and_multi_polygon(target_feature)
            else {
                continue;
            };
            for target in target.iter() {
                let candidates = rtree.locate_in_envelope_intersecting(&target.envelope());
                for other in candidates {
                    if other.feature_id == target_feature.id {
                        continue;
                    }
                    let polygon_float = Polygon2DFloat(other.geometry.clone());
                    match polygon_features.entry(polygon_float) {
                        Entry::Occupied(mut entry) => {
                            let (feature_ids, polygon_feature) = entry.get_mut();
                            feature_ids.insert(target_feature.id);
                            feature_ids.insert(other.feature_id);
                            *polygon_feature += 1;
                        }
                        Entry::Vacant(entry) => {
                            let mut feature_ids = HashSet::new();
                            feature_ids.insert(target_feature.id);
                            feature_ids.insert(other.feature_id);
                            entry.insert((feature_ids, 2));
                        }
                    }
                }
            }
            let mut feature = target_feature.clone();
            feature.attributes.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(1.into()),
            );
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                AREA_PORT.clone(),
            ));
        }
        for (polygon, (feature_ids, polygon_feature)) in polygon_features {
            let features = feature_ids
                .iter()
                .filter_map(|feature_id| target_features.get(feature_id))
                .collect::<Vec<_>>();
            let mut feature = Feature::new();
            feature.attributes.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(polygon_feature.into()),
            );
            for other_feature in features.iter() {
                feature.attributes.extend(other_feature.attributes.clone());
            }
            feature.geometry = Geometry {
                epsg: features.first().unwrap().geometry.epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon.0)),
            };
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                AREA_PORT.clone(),
            ));
        }
    }

    fn handle_2d_polygon_and_multi_polygon(
        &self,
        feature: &Feature,
    ) -> Option<MultiPolygon2D<f64>> {
        match &feature.geometry.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)) => {
                Some(MultiPolygon2D::new(vec![poly.clone()]))
            }
            GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(mpoly)) => Some(mpoly.clone()),
            _ => None,
        }
    }

    fn change_group(&mut self, ctx: ExecutorContext, fw: &mut dyn ProcessorChannelForwarder) {
        let mut remove_keys = Vec::new();
        for (key, (complete_grouped, features, rtree)) in self.buffer.iter() {
            if !*complete_grouped {
                continue;
            }
            remove_keys.push(key.clone());
            self.handle_2d_geometry(ctx.as_context(), features, rtree, fw);
        }
        for key in remove_keys.iter() {
            self.buffer.remove(key);
        }
    }
}

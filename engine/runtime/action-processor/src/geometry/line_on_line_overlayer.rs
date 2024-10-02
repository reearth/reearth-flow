use std::collections::hash_map::Entry;
use std::collections::HashMap;

use itertools::Itertools;
use nusamai_projection::crs::EpsgCode;
use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::line_intersection::line_intersection;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line::Line2DFloat;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};
use rstar::{RTree, RTreeObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));
pub static LINE_PORT: Lazy<Port> = Lazy::new(|| Port::new("line"));
const EPSILON: f64 = 0.001;

#[derive(Debug, Clone, Default)]
pub struct LineOnLineOverlayerFactory;

impl ProcessorFactory for LineOnLineOverlayerFactory {
    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }

    fn description(&self) -> &str {
        "Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(LineOnLineOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![POINT_PORT.clone(), LINE_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: LineOnLineOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::LineOnLineOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(LineOnLineOverlayer {
            params,
            buffer2d: HashMap::new(),
            previous_group_key: None,
        }))
    }
}

#[derive(Debug, Clone)]
struct LineFeature {
    attributes: HashMap<uuid::Uuid, HashMap<Attribute, AttributeValue>>,
    overlap: u64,
    epsg: Option<EpsgCode>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Attribute,
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct LineOnLineOverlayer {
    params: LineOnLineOverlayerParam,
    buffer2d: HashMap<String, (bool, Vec<Feature>, RTree<LineString2D<f64>>)>, // (complete_grouped, features)
    previous_group_key: Option<String>,
}

impl Processor for LineOnLineOverlayer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(geometry) = &feature.geometry else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::FlowGeometry2D(geos) => {
                let line_string = geos.as_line_string();
                let multi_line_string = geos.as_multi_line_string();
                if line_string.is_none() && multi_line_string.is_none() {
                    fw.send(
                        ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()),
                    );
                    return Ok(());
                }
                let key = if let Some(group_by) = &self.params.group_by {
                    group_by
                        .iter()
                        .map(|k| feature.get(&k).map(|v| v.to_string()).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join("\t")
                } else {
                    "_all".to_string()
                };

                match self.buffer2d.entry(key.clone()) {
                    Entry::Occupied(mut entry) => {
                        self.previous_group_key = Some(key.clone());
                        {
                            let (_, buffer, rtree) = entry.get_mut();
                            buffer.push(feature.clone());
                            if let Some(line_string) = line_string {
                                rtree.insert(line_string.clone());
                            }
                            if let Some(multi_line_string) = multi_line_string {
                                for line_string in multi_line_string.iter() {
                                    rtree.insert(line_string.clone());
                                }
                            }
                        }
                    }
                    Entry::Vacant(entry) => {
                        let mut rtree = RTree::new();
                        if let Some(line_string) = line_string {
                            rtree.insert(line_string.clone());
                        }
                        if let Some(multi_line_string) = multi_line_string {
                            for line_string in multi_line_string.iter() {
                                rtree.insert(line_string.clone());
                            }
                        }
                        entry.insert((false, vec![feature.clone()], rtree));
                        if let Some(previous_group_key) = &self.previous_group_key {
                            if let Entry::Occupied(mut entry) =
                                self.buffer2d.entry(previous_group_key.clone())
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
        for (_, (_, buffer, rtree)) in self.buffer2d.iter() {
            self.handle_2d_line_strings(ctx.clone().into(), fw, buffer, rtree);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }
}

impl LineOnLineOverlayer {
    fn change_group(&mut self, ctx: ExecutorContext, fw: &mut dyn ProcessorChannelForwarder) {
        let mut remove_2d_keys = Vec::new();
        for (key, (complete_grouped, buffer, rtree)) in self.buffer2d.iter() {
            if !*complete_grouped {
                continue;
            }
            remove_2d_keys.push(key.clone());
            self.handle_2d_line_strings(ctx.clone().into(), fw, buffer, rtree);
        }
        for key in remove_2d_keys {
            self.buffer2d.remove(&key);
        }
    }

    fn handle_2d_line_strings(
        &self,
        ctx: Context,
        fw: &mut dyn ProcessorChannelForwarder,
        features: &[Feature],
        rtree: &RTree<LineString2D<f64>>,
    ) {
        let mut line_features = HashMap::<Line2DFloat, LineFeature>::new();
        for feature in features.iter() {
            let line_string = feature.geometry.as_ref().and_then(|g| {
                g.value
                    .as_flow_geometry_2d()
                    .and_then(|g| g.as_line_string())
            });
            let multi_line_string = feature.geometry.as_ref().and_then(|g| {
                g.value
                    .as_flow_geometry_2d()
                    .and_then(|g| g.as_multi_line_string())
            });
            if line_string.is_none() && multi_line_string.is_none() {
                continue;
            }
            let line_strings = if let Some(line_string) = line_string {
                vec![line_string]
            } else if let Some(multi_line_string) = multi_line_string {
                multi_line_string.iter().cloned().collect::<Vec<_>>()
            } else {
                continue;
            };
            let mut out_line_strings = Vec::new();
            for line_string in line_strings {
                let output_envelope = line_string.envelope();
                let candidates = rtree.locate_in_envelope_intersecting(&output_envelope);
                let mut intersect = false;
                for candidate in candidates {
                    if line_string.approx_eq(candidate, EPSILON) {
                        continue;
                    }
                    line_string.lines().zip(candidate.lines()).for_each(
                        |(line1, line2)| {
                            if let Some(reearth_flow_geometry::algorithm::line_intersection::LineIntersection::Collinear { intersection }) =  line_intersection(line1, line2) {
                                intersect = true;
                                let line_float = Line2DFloat(intersection);
                                match line_features.entry(line_float.clone()) {
                                    Entry::Occupied(mut entry) => {
                                        let line_feature = entry.get_mut();
                                        line_feature.overlap += 1;
                                        line_feature
                                            .attributes
                                            .insert(feature.id, feature.attributes.clone());
                                    }
                                    Entry::Vacant(entry) => {
                                        let mut attributes = HashMap::new();
                                        for (k, v) in feature.iter() {
                                            attributes.insert(k.clone(), v.clone());
                                        }
                                        let line_feature = LineFeature {
                                            epsg: feature.geometry.as_ref().and_then(|g| g.epsg),
                                            attributes: HashMap::from([(feature.id, attributes)]),
                                            overlap: 1,
                                        };
                                        entry.insert(line_feature);
                                    }
                                }
                            }
                        },
                    );
                }
                if !intersect {
                    out_line_strings.push(line_string.clone());
                }
            }
            for line_string in out_line_strings.iter() {
                let mut line_string_feature = feature.clone();
                line_string_feature.attributes.insert(
                    self.params.output_attribute.clone(),
                    AttributeValue::Number(serde_json::Number::from(1)),
                );
                line_string_feature.refresh_id();
                line_string_feature.geometry = Some(Geometry {
                    epsg: feature.geometry.as_ref().and_then(|g| g.epsg),
                    value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(
                        line_string.clone(),
                    )),
                });
                fw.send(ExecutorContext::new_with_context_feature_and_port(
                    &ctx,
                    line_string_feature.clone(),
                    LINE_PORT.clone(),
                ));
            }
        }
        for (line, line_feature) in line_features.iter() {
            let mut feature = Feature::new();
            feature.attributes.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(serde_json::Number::from(line_feature.overlap)),
            );
            let feature_attributes = line_feature
                .attributes
                .values()
                .map(|kv| {
                    kv.iter()
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect::<HashMap<_, _>>()
                })
                .collect_vec();
            feature.attributes.insert(
                Attribute::new("features"),
                AttributeValue::Array(
                    feature_attributes
                        .into_iter()
                        .map(AttributeValue::Map)
                        .collect(),
                ),
            );
            if line_feature.overlap > 1 {
                let mut start_feature = feature.clone();
                start_feature.refresh_id();
                let start_point = line.0.start_point();
                start_feature.geometry = Some(Geometry {
                    epsg: line_feature.epsg,
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Point(start_point)),
                });
                fw.send(ExecutorContext::new_with_context_feature_and_port(
                    &ctx,
                    start_feature,
                    POINT_PORT.clone(),
                ));
                let mut end_feature = feature.clone();
                end_feature.refresh_id();
                let end_point = line.0.end_point();
                end_feature.geometry = Some(Geometry {
                    epsg: line_feature.epsg,
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Point(end_point)),
                });
                fw.send(ExecutorContext::new_with_context_feature_and_port(
                    &ctx,
                    end_feature,
                    POINT_PORT.clone(),
                ));
            }
            let mut line_feat = feature.clone();
            line_feat.refresh_id();
            line_feat.geometry = Some(Geometry {
                epsg: line_feature.epsg,
                value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(line.0.into())),
            });
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                line_feat,
                LINE_PORT.clone(),
            ));
        }
    }
}

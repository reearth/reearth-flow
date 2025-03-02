use std::collections::HashMap;

use once_cell::sync::Lazy;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use reearth_flow_geometry::algorithm::line_intersection::LineIntersection;
use reearth_flow_geometry::algorithm::line_string_ops::{LineStringOps, LineStringWithTree2D};
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::point::Point;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use super::errors::GeometryProcessorError;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));
pub static LINE_PORT: Lazy<Port> = Lazy::new(|| Port::new("line"));

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
            group_by: params.group_by,
            tolerance: params.tolerance,
            buffer: HashMap::new(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct LineOnLineOverlayer {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for LineOnLineOverlayer {
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
                    for feature in &overlayed.line {
                        fw.send(ctx.new_with_feature_and_port(feature.clone(), LINE_PORT.clone()));
                    }
                    for feature in &overlayed.point {
                        fw.send(ctx.new_with_feature_and_port(feature.clone(), POINT_PORT.clone()));
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
        for feature in &overlayed.line {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                LINE_PORT.clone(),
            ));
        }
        for feature in &overlayed.point {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                POINT_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }
}

struct OverlayResultLineString {
    line_strings: Vec<LineString2D<f64>>,
    overlay_count: usize,
}

struct OverlayResult {
    result_line_strings: Vec<OverlayResultLineString>,
    split_points: Vec<Coordinate2D<f64>>,
}

fn line_string_intersection_2d(
    line_strings: &[LineString2D<f64>],
    tolerance: f64,
) -> OverlayResult {
    let results = line_strings.par_iter().enumerate().map(|(i, line_string)| {
        let packed_line_string = LineStringWithTree2D::new(line_string.clone());
        let intersections_by_others = line_strings
            .iter()
            .enumerate()
            .filter_map(|(j, other_line_string)| {
                if i == j {
                    return None;
                }
                let intersections = packed_line_string.intersection(other_line_string);
                if intersections.is_empty() {
                    return None;
                }

                Some(intersections)
            })
            .collect::<Vec<_>>();

        let overlay_count = intersections_by_others.len();

        let intersections = intersections_by_others.iter().flatten().collect::<Vec<_>>();

        let split_points = intersections
            .iter()
            .flat_map(|intersection| match intersection {
                LineIntersection::SinglePoint { intersection, .. } => vec![*intersection],
                LineIntersection::Collinear { intersection } => {
                    vec![intersection.start, intersection.end]
                }
            })
            .collect::<Vec<_>>();

        let splitted = packed_line_string.split(&split_points, tolerance);

        (
            OverlayResultLineString {
                line_strings: splitted,
                overlay_count,
            },
            split_points,
        )
    });

    let (result_line_strings, split_points): (Vec<_>, Vec<_>) = results.unzip();

    OverlayResult {
        result_line_strings,
        split_points: split_points.into_iter().flatten().collect(),
    }
}

struct OverlayedFeatures {
    point: Vec<Feature>,
    line: Vec<Feature>,
}

impl OverlayedFeatures {
    fn new() -> Self {
        Self {
            point: Vec::new(),
            line: Vec::new(),
        }
    }

    fn extend(&mut self, other: Self) {
        self.point.extend(other.point);
        self.line.extend(other.line);
    }
}

impl LineOnLineOverlayer {
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

    fn overlay_2d(&self, features_2d: Vec<&Feature>) -> OverlayedFeatures {
        let line_strings = features_2d
            .iter()
            .filter_map(|f| f.geometry.value.as_flow_geometry_2d())
            .filter_map(|g| {
                if let Geometry2D::LineString(line) = g {
                    Some(vec![line.clone()])
                } else if let Geometry2D::MultiLineString(multi_line) = g {
                    Some(multi_line.0.clone())
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        let line_string_intersection_result =
            line_string_intersection_2d(&line_strings, self.tolerance);

        let mut overlayed = OverlayedFeatures::new();

        for (i, result_lss) in line_string_intersection_result
            .result_line_strings
            .iter()
            .enumerate()
        {
            let attributes = features_2d[i].attributes.clone();
            let overlay_count = result_lss.overlay_count;
            for result_ls in result_lss.line_strings.iter() {
                let mut feature = Feature::new();
                feature.attributes = attributes.clone();
                feature.attributes.insert(
                    Attribute::new("overlayCount"),
                    AttributeValue::Number(Number::from(overlay_count)),
                );
                feature.geometry.value =
                    GeometryValue::FlowGeometry2D(Geometry2D::LineString(result_ls.clone()));
                overlayed.line.push(feature);
            }
        }

        let last_feature = features_2d.last().unwrap();

        for intersection in line_string_intersection_result.split_points {
            let mut feature = Feature::new();

            if let Some(group_by) = &self.group_by {
                feature.attributes = group_by
                    .iter()
                    .filter_map(|attr| {
                        let value = last_feature.get(attr).cloned()?;
                        Some((attr.clone(), value))
                    })
                    .collect::<HashMap<_, _>>();
            } else {
                feature.attributes = HashMap::new();
            }

            feature.geometry.value =
                GeometryValue::FlowGeometry2D(Geometry2D::Point(Point(intersection)));
            overlayed.point.push(feature);
        }

        overlayed
    }
}

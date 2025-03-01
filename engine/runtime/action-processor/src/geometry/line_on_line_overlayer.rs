use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::intersects::Intersects;
use reearth_flow_geometry::algorithm::line_intersection::{self, line_intersection, LineIntersection};
use reearth_flow_geometry::algorithm::GeoFloat;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line::{Line, Line2D};
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_line_string::MultiLineString2D;
use reearth_flow_geometry::types::no_value::NoValue;
use reearth_flow_geometry::types::point::Point2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
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
            group_by: params.group_by,
            buffer: HashMap::new(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    group_by: Option<Vec<Attribute>>,
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
pub struct LineOnLineOverlayer {
    group_by: Option<Vec<Attribute>>,
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
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), POINT_PORT.clone()),
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

// struct LineRepository2D {
//     multi_line_strings: Vec<(MultiLineString2D<f64>, bool)>, // (MultiLineString, is_from_MultiLineString)
//     wrapped_lines: Vec<WrappedLine2D>, // lines with their index in multi_line_strings
//     overlay_graph: Vec<HashSet<usize>>, // graph of intersected lines
// }

// impl LineRepository2D {
//     fn new(features: &[Feature]) -> Self {
//         let multi_line_strings = features
//             .iter()
//             .filter_map(|f| f.geometry.value.as_flow_geometry_2d())
//             .filter_map(|g| {
//                 if let Geometry2D::LineString(line) = g {
//                     Some((MultiLineString2D::new(vec![line.clone()]), false))
//                 } else if let Geometry2D::MultiLineString(multi_line) = g {
//                     Some((multi_line.clone(), true))
//                 } else {
//                     None
//                 }
//             })
//             .collect::<Vec<_>>();

//         let mut wrapped_lines = Vec::new();

//         for (i, multi_line_string) in multi_line_strings.iter().enumerate() {
//             for (j, line_string) in multi_line_string.0.iter().enumerate() {
//                 for line in line_string.lines() {
//                     wrapped_lines.push(WrappedLine2D {
//                         line: line.clone(),
//                         l_index: wrapped_lines.len(),
//                         ls_index: j,
//                         mls_index: i,
//                     });
//                 }
//             }
//         }

//         let line_rtree = RTree::bulk_load(wrapped_lines.clone());

//         let mut overlay_graph = vec![HashSet::new(); multi_line_strings.len()];
//         for i in 0..wrapped_lines.len() {
//             let wrapped_line = &wrapped_lines[i];
//             let envelope = wrapped_line.line.envelope();
//             let candidates = line_rtree.locate_in_envelope_intersecting(&envelope);   
//         }

//         Self {
//             multi_line_strings,
//             lines,
//             overlay_graph,
//         }
//     }
// }


// struct WrappedLine2D {
//     line: Line2D<f64>,
//     index: usize,
// }

fn line_length_2d(line: Line2D<f64>) -> f64 {
    let delta = line.delta();
    (delta.x * delta.x + delta.y * delta.y).sqrt()
}

// split line with intersection
// if the intersection is not on the line, return None
fn line_split_with_intersection_2d(line: Line2D<f64>, intersecton: LineIntersection<f64, NoValue>) -> Option<(Line2D<f64>, Line2D<f64>)> {
    match intersecton {
        LineIntersection::SinglePoint { intersection, .. } => {
            if !line.intersects(&intersection) { // TODO: consider is_proper
                return None;
            }

            let first = Line::new(line.start, intersection);
            let second = Line::new(intersection, line.end);
            Some((first, second))
        }
        LineIntersection::Collinear { intersection } => {
            if !line.intersects(&intersection) {
                return None;
            }

            let length_line = line_length_2d(line);

            let length_1 = line_length_2d(Line::new(line.start, intersection.start));
            let length_2 = line_length_2d(Line::new(intersection.start, intersection.end));
            let length_3 = line_length_2d(Line::new(intersection.end, line.end));

            let length_123 = length_1 + length_2 + length_3;

            if (length_line-length_123).abs() < f64::EPSILON {
                Some((Line::new(line.start, intersection.start), Line::new(intersection.end, line.end)))
            } else {
                Some((Line::new(line.start, intersection.end), Line::new(intersection.start, line.end)))
            }
        }
    }
}

fn line_split_with_multiple_intersections_2d(line: Line2D<f64>, intersections: Vec<LineIntersection<f64, NoValue>>) -> Vec<Line2D<f64>> {
    let mut current_lines = vec![line];
    for intersection in intersections {
        let mut lines = Vec::new();
        for current_line in current_lines {
            if let Some((first, second)) = line_split_with_intersection_2d(current_line, intersection) {
                lines.push(first);
                lines.push(second);
            } else {
                lines.push(current_line);
            }
        }
        current_lines = lines;
    }

    current_lines
}

fn line_string_from_connected_lines_2d(lines: Vec<Line2D<f64>>) -> LineString2D<f64> {
    let mut points = Vec::new();
    for i in 0..lines.len() {
        if i == 0 {
            points.push(lines[i].start);
        }
        points.push(lines[i].end);
    }

    LineString2D::new(points)
}

struct ToSplit {
    index: usize,
    intersection: LineIntersection<f64, NoValue>,
}

fn split_line_string(ls: &LineString2D<f64>, tosplits: &Vec<ToSplit>) -> Vec<LineString2D<f64>> {
    let mut new_ls = Vec::new();
    let mut lines_buffer = Vec::new();

    for (i, line) in ls.lines().enumerate() {
        let intersections =
        tosplits.iter()
        .filter(|tosplit| tosplit.index == i)
        .map(|tosplit| tosplit.intersection).collect::<Vec<_>>();
        if intersections.is_empty() {
            lines_buffer.push(line.clone());
        } else {
            let intersected = line_split_with_multiple_intersections_2d(line.clone(), intersections);
            match intersected.len() {
                0 => (),
                1 => {
                    lines_buffer.push(intersected[0].clone());
                }
                _ => {
                    for i in 0..intersected.len()-1 {
                        lines_buffer.push(intersected[i].clone());
                        new_ls.push(line_string_from_connected_lines_2d(lines_buffer.clone()));
                        lines_buffer.clear();
                    }
                    lines_buffer.push(intersected[intersected.len()-1].clone());
                }
            }
        }
    }

    new_ls.push(line_string_from_connected_lines_2d(lines_buffer.clone()));

    new_ls
}

struct LineStringIntersectionResult {
    ls1: Vec<LineString2D<f64>>,
    ls2: Vec<LineString2D<f64>>,
    intersections: Vec<LineIntersection<f64, NoValue>>,
}

// intersection 
fn line_string_intersection_2d(ls1: &LineString2D<f64>, ls2: &LineString2D<f64>) -> LineStringIntersectionResult {
    let mut tosplits_ls1 = Vec::new();
    let mut tosplits_ls2 = Vec::new();

    for (i1, line1) in ls1.lines().enumerate() {
        for (i2, line2) in ls2.lines().enumerate() {
            if let Some(intersection) = line_intersection(line1, line2) {
                tosplits_ls1.push(ToSplit { index: i1, intersection });
                tosplits_ls2.push(ToSplit { index: i2, intersection });
            }
        }
    }

    let new_ls1 = split_line_string(ls1, &tosplits_ls1);
    let new_ls2 = split_line_string(ls2, &tosplits_ls2);

    LineStringIntersectionResult {
        ls1: new_ls1,
        ls2: new_ls2,
        intersections: tosplits_ls1.iter().map(|tosplit| tosplit.intersection.clone()).collect(),
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

    fn extend(&mut self, other: OverlayedFeatures) {
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
        let mut overlayed = OverlayedFeatures::new();

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
            .collect::<Vec<_>>();

        // let graph = OverlayGraph::bulk_load(&multi_line_strings);

        // line_intersection

        // for i in 0..multi_line_strings.len() {
        //     graph.intersected_iter(i).for_each(|j| {
        //         let line1 = &multi_line_strings[i];
        //         let line2 = &multi_line_strings_incoming[j];
        //         let intersection = line1.intersection(line2);
        //         if let Some(intersection) = intersection {
        //             let mut feature = Feature::new();
        //             feature.attributes.insert(
        //                 self.params.output_attribute.clone(),
        //                 AttributeValue::Number(serde_json::Number::from(1)),
        //             );
        //             feature.geometry = Geometry {
        //                 epsg: feature.geometry.epsg,
        //                 value: GeometryValue::FlowGeometry2D(Geometry2D::Point(intersection)),
        //             };
        //             overlayed.point.push(feature);
        //         }
        //     });
        // }

        OverlayedFeatures::new()
    }
}
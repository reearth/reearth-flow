use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::intersects::Intersects;
use reearth_flow_geometry::algorithm::line_intersection::{line_intersection, LineIntersection};
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line::{Line, Line2D};
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::no_value::NoValue;
use reearth_flow_geometry::types::point::{Point, Point2D};
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
static EPSILON: f64 = 1e-6;

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

fn line_length_2d(line: Line2D<f64>) -> f64 {
    let delta = line.delta();
    (delta.x * delta.x + delta.y * delta.y).sqrt()
}

// split line with intersection
// if the intersection is not on the line, return None
fn line_split_with_intersection_2d(
    line: Line2D<f64>,
    intersecton: LineIntersection<f64, NoValue>,
) -> Option<(Line2D<f64>, Line2D<f64>)> {
    match intersecton {
        LineIntersection::SinglePoint {
            intersection,
            is_proper,
        } => {
            if !is_proper {
                return None;
            }
            // if intersection is not on line, return None
            let length_line = line_length_2d(line);

            let length_1 = line_length_2d(Line::new(line.start, intersection));
            let length_2 = line_length_2d(Line::new(intersection, line.end));

            let length_12 = length_1 + length_2;

            if (length_line - length_12).abs() >= EPSILON {
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

            if (length_line - length_123).abs() < EPSILON {
                Some((
                    Line::new(line.start, intersection.start),
                    Line::new(intersection.end, line.end),
                ))
            } else {
                Some((
                    Line::new(line.start, intersection.end),
                    Line::new(intersection.start, line.end),
                ))
            }
        }
    }
}

fn line_split_with_multiple_intersections_2d(
    line: Line2D<f64>,
    intersections: Vec<LineIntersection<f64, NoValue>>,
) -> Vec<Line2D<f64>> {
    let mut current_lines = vec![line];
    for intersection in intersections {
        let mut lines = Vec::new();
        for current_line in current_lines {
            if let Some((first, second)) =
                line_split_with_intersection_2d(current_line, intersection)
            {
                //println!("first: {:?}, second: {:?}", first, second);
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
    line_index: usize,
    intersection: LineIntersection<f64, NoValue>,
}

fn split_line_string(ls: &LineString2D<f64>, tosplits: &Vec<ToSplit>) -> Vec<LineString2D<f64>> {
    let mut new_ls = Vec::new();
    let mut lines_buffer = Vec::new();

    for (i, line) in ls.lines().enumerate() {
        let intersections = tosplits
            .iter()
            .filter(|tosplit| tosplit.line_index == i)
            .map(|tosplit| tosplit.intersection)
            .collect::<Vec<_>>();
        if intersections.is_empty() {
            lines_buffer.push(line.clone());
        } else {
            let intersected =
                line_split_with_multiple_intersections_2d(line.clone(), intersections);
            match intersected.len() {
                0 => (),
                1 => {
                    lines_buffer.push(intersected[0].clone());
                }
                _ => {
                    for i in 0..intersected.len() - 1 {
                        lines_buffer.push(intersected[i].clone());
                        new_ls.push(line_string_from_connected_lines_2d(lines_buffer.clone()));
                        lines_buffer.clear();
                    }
                    lines_buffer.push(intersected[intersected.len() - 1].clone());
                }
            }
        }
    }

    new_ls.push(line_string_from_connected_lines_2d(lines_buffer.clone()));

    new_ls
}

struct IntersectionGraph {
    // index of line string -> index of line string -> intersection
    graph: Vec<HashMap<usize, LineIntersection<f64, NoValue>>>,
}

impl IntersectionGraph {
    fn new(line_strings: &Vec<LineString2D<f64>>) -> Self {
        #[derive(Debug, Clone)]
        struct WrappedLine2D {
            line: Line2D<f64>,
            line_string_index: usize,
        }

        impl RTreeObject for WrappedLine2D {
            type Envelope = rstar::AABB<Point2D<f64>>;

            fn envelope(&self) -> Self::Envelope {
                self.line.envelope()
            }
        }

        let mut wrapped_lines = Vec::new();

        for (line_string_index, line_string) in line_strings.iter().enumerate() {
            for line in line_string.lines() {
                wrapped_lines.push(WrappedLine2D {
                    line: line.clone(),
                    line_string_index,
                });
            }
        }

        let line_rtree = RTree::bulk_load(wrapped_lines.clone());

        let mut graph = vec![HashMap::new(); line_strings.len()];

        for wrapped_line in wrapped_lines {
            let envelope = wrapped_line.line.envelope();
            let candidates = line_rtree.locate_in_envelope_intersecting(&envelope);
            for candidate in candidates {
                if wrapped_line.line_string_index >= candidate.line_string_index {
                    continue;
                }

                if graph[wrapped_line.line_string_index].contains_key(&candidate.line_string_index)
                {
                    continue;
                }

                if wrapped_line.line.intersects(&candidate.line) {
                    if let Some(intersection) = line_intersection(wrapped_line.line, candidate.line)
                    {
                        graph[wrapped_line.line_string_index]
                            .insert(candidate.line_string_index, intersection.clone());
                        graph[candidate.line_string_index]
                            .insert(wrapped_line.line_string_index, intersection);
                    }
                }
            }
        }

        Self { graph }
    }

    fn intersected_iter(
        &self,
        i: usize,
    ) -> impl Iterator<Item = (usize, &LineIntersection<f64, NoValue>)> {
        self.graph[i]
            .iter()
            .map(|(j, intersection)| (*j, intersection))
    }

    fn intersections(&self) -> Vec<LineIntersection<f64, NoValue>> {
        let mut intersections = Vec::new();
        for i in 0..self.graph.len() {
            for (j, intersection) in self.intersected_iter(i) {
                if i < j {
                    intersections.push(intersection.clone());
                }
            }
        }

        intersections.into_iter().collect()
    }
}

// struct LineStringIntersectionResult {
//     ls1: Vec<LineString2D<f64>>,
//     ls2: Vec<LineString2D<f64>>,
//     intersections: Vec<LineIntersection<f64, NoValue>>,
// }

// intersection
// fn line_string_intersection_2d(ls1: &LineString2D<f64>, ls2: &LineString2D<f64>) -> LineStringIntersectionResult {
//     let mut tosplits_ls1 = Vec::new();
//     let mut tosplits_ls2 = Vec::new();

//     for (i1, line1) in ls1.lines().enumerate() {
//         for (i2, line2) in ls2.lines().enumerate() {
//             if let Some(intersection) = line_intersection(line1, line2) {
//                 tosplits_ls1.push(ToSplit { index: i1, intersection });
//                 tosplits_ls2.push(ToSplit { index: i2, intersection });
//             }
//         }
//     }

//     let new_ls1 = split_line_string(ls1, &tosplits_ls1);
//     let new_ls2 = split_line_string(ls2, &tosplits_ls2);

//     LineStringIntersectionResult {
//         ls1: new_ls1,
//         ls2: new_ls2,
//         intersections: tosplits_ls1.iter().map(|tosplit| tosplit.intersection.clone()).collect(),
//     }
// }

struct LineStringIntersectionResult {
    // [[final line string; len of lines in line string]; the number of line strings]
    lss: Vec<Vec<LineString2D<f64>>>,
    intersections: Vec<LineIntersection<f64, NoValue>>,
}

fn line_string_intersection_2d(lss: &Vec<LineString2D<f64>>) -> LineStringIntersectionResult {
    let overlay_graph = IntersectionGraph::new(lss);

    let mut new_lss = Vec::new();

    for (i, line_string) in lss.iter().enumerate() {
        let mut tosplits = Vec::new();
        for (j, intersection) in overlay_graph.intersected_iter(i) {
            tosplits.push(ToSplit {
                line_index: j,
                intersection: intersection.clone(),
            });
        }

        let new_ls = split_line_string(line_string, &tosplits);
        new_lss.push(new_ls);
    }

    LineStringIntersectionResult {
        lss: new_lss,
        intersections: overlay_graph.intersections(),
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

        let line_string_intersection_result = line_string_intersection_2d(&line_strings);

        let mut overlayed = OverlayedFeatures::new();

        for (i, new_lss) in line_string_intersection_result.lss.iter().enumerate() {
            let attributes = features_2d[i].attributes.clone();
            for new_ls in new_lss {
                let mut feature = Feature::new();
                feature.attributes = attributes.clone();
                feature.geometry.value =
                    GeometryValue::FlowGeometry2D(Geometry2D::LineString(new_ls.clone()));
                overlayed.line.push(feature);
            }
        }

        let last_feature = features_2d.last().unwrap();

        for intersection in line_string_intersection_result.intersections {
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

            match intersection {
                LineIntersection::SinglePoint { intersection, .. } => {
                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(Geometry2D::Point(Point(intersection)));
                }
                LineIntersection::Collinear { intersection } => {
                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(Geometry2D::LineString(LineString2D::new(
                            vec![intersection.start.clone(), intersection.end.clone()],
                        )));
                }
            }

            overlayed.point.push(feature);
        }

        overlayed
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::coordinate::Coordinate2D;

    use super::*;

    #[test]
    fn test_line_length_2d() {
        let line = Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(3.0, 4.0));
        assert_eq!(line_length_2d(line), 5.0);
    }

    #[test]
    fn test_line_split_with_intersection_2d_single_point() {
        let line = Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(4.0, 4.0));
        let intersection = LineIntersection::SinglePoint {
            intersection: Coordinate2D::new_(1.0, 1.0),
            is_proper: true,
        };
        let result = line_split_with_intersection_2d(line, intersection).unwrap();
        assert_eq!(
            result.0,
            Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(1.0, 1.0))
        );
        assert_eq!(
            result.1,
            Line2D::new(Coordinate2D::new_(1.0, 1.0), Coordinate2D::new_(4.0, 4.0))
        );

        let intersection = LineIntersection::SinglePoint {
            intersection: Coordinate2D::new_(1.0, 3.0),
            is_proper: true,
        };

        let result = line_split_with_intersection_2d(line, intersection);
        assert!(result.is_none());
    }

    #[test]
    fn test_line_split_with_intersection_2d_collinear() {
        let line = Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(4.0, 4.0));
        let intersection = LineIntersection::Collinear {
            intersection: Line2D::new(Coordinate2D::new_(1.0, 1.0), Coordinate2D::new_(3.0, 3.0)),
        };
        let result = line_split_with_intersection_2d(line, intersection).unwrap();
        assert_eq!(
            result.0,
            Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(1.0, 1.0))
        );
        assert_eq!(
            result.1,
            Line2D::new(Coordinate2D::new_(3.0, 3.0), Coordinate2D::new_(4.0, 4.0))
        );

        let intersection = LineIntersection::Collinear {
            intersection: Line2D::new(Coordinate2D::new_(3.0, 3.0), Coordinate2D::new_(1.0, 1.0)),
        };

        let result = line_split_with_intersection_2d(line, intersection).unwrap();
        assert_eq!(
            result.0,
            Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(1.0, 1.0))
        );
        assert_eq!(
            result.1,
            Line2D::new(Coordinate2D::new_(3.0, 3.0), Coordinate2D::new_(4.0, 4.0))
        );
    }

    #[test]
    fn test_line_split_with_multiple_intersections_2d() {
        let line = Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(4.0, 4.0));
        let intersections = vec![
            LineIntersection::SinglePoint {
                intersection: Coordinate2D::new_(1.0, 1.0),
                is_proper: true,
            },
            LineIntersection::SinglePoint {
                intersection: Coordinate2D::new_(3.0, 3.0),
                is_proper: true,
            },
        ];
        let result = line_split_with_multiple_intersections_2d(line, intersections);
        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0],
            Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(1.0, 1.0))
        );
        assert_eq!(
            result[1],
            Line2D::new(Coordinate2D::new_(1.0, 1.0), Coordinate2D::new_(3.0, 3.0))
        );
        assert_eq!(
            result[2],
            Line2D::new(Coordinate2D::new_(3.0, 3.0), Coordinate2D::new_(4.0, 4.0))
        );
    }

    #[test]
    fn test_line_string_from_connected_lines_2d() {
        let lines = vec![
            Line2D::new(Coordinate2D::new_(0.0, 0.0), Coordinate2D::new_(1.0, 1.0)),
            Line2D::new(Coordinate2D::new_(1.0, 1.0), Coordinate2D::new_(2.0, 2.0)),
        ];
        let result = line_string_from_connected_lines_2d(lines);
        let expected = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(2.0, 2.0),
        ])
        .points()
        .collect::<Vec<_>>();
        for (i, point) in result.points().enumerate() {
            assert_eq!(point, expected[i]);
        }
    }

    #[test]
    fn test_split_line_string() {
        let line_string = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(3.0, 3.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);
        let tosplits = vec![ToSplit {
            line_index: 0,
            intersection: LineIntersection::SinglePoint {
                intersection: Coordinate2D::new_(2.0, 2.0),
                is_proper: true,
            },
        }];
        let result = split_line_string(&line_string, &tosplits);
        assert_eq!(result.len(), 2);
    }
}

use std::collections::HashSet;

use rstar::RTreeObject;

use crate::{
    algorithm::line_ops::LineOps,
    types::{
        coordinate::Coordinate,
        line::{Line, Line2D},
        line_string::{LineString, LineString2D},
        no_value::NoValue,
        point::Point2D,
    },
};

use super::{
    intersects::Intersects,
    line_intersection::{line_intersection, LineIntersection},
    GeoFloat,
};

pub trait LineStringOps<T: GeoFloat, Z: GeoFloat> {
    fn intersection(&self, other: LineString<T, Z>) -> Vec<LineIntersection<T, Z>>;
    fn split(&self, points: Vec<Coordinate<T, Z>>, torelance: T) -> Vec<LineString<T, Z>>;
}

#[derive(Debug, Clone)]
struct LineWithIndex2D {
    line: Line<f64, NoValue>,
    index: usize,
}

impl RTreeObject for LineWithIndex2D {
    type Envelope = rstar::AABB<Point2D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.line.envelope()
    }
}

pub struct LineStringWithTree2D {
    rtree: rstar::RTree<LineWithIndex2D>,
    line_string: LineString2D<f64>,
}

impl LineStringWithTree2D {
    fn new(line_string: LineString2D<f64>) -> Self {
        let mut packed_lines = Vec::new();

        for (index, line) in line_string.lines().enumerate() {
            packed_lines.push(LineWithIndex2D { line, index });
        }

        let rtree = rstar::RTree::bulk_load(packed_lines.clone());

        Self { rtree, line_string }
    }
}

impl LineStringOps<f64, NoValue> for LineStringWithTree2D {
    fn intersection(&self, other: LineString2D<f64>) -> Vec<LineIntersection<f64, NoValue>> {
        let mut result = Vec::new();

        for other_line in other.lines() {
            let envelope = other_line.envelope();
            let packed_lines = self.rtree.locate_in_envelope_intersecting(&envelope);

            for packed_line in packed_lines {
                if packed_line.line.intersects(&other_line) {
                    let intersection =
                        line_intersection(packed_line.line.clone(), other_line.clone());
                    if let Some(intersection) = intersection {
                        result.push(intersection);
                    }
                }
            }
        }

        result
    }

    fn split(
        &self,
        coordinates: Vec<Coordinate<f64, NoValue>>,
        torelance: f64,
    ) -> Vec<LineString<f64, NoValue>> {
        fn split_line_by_multiple_coords(
            line: Line<f64, NoValue>,
            coords: Vec<Coordinate<f64, NoValue>>,
            torelance: f64,
        ) -> Vec<Line<f64, NoValue>> {
            let mut lines = vec![line];
            for coord in coords {
                let mut new_lines = Vec::new();
                for line in lines {
                    new_lines.extend(line.split(&coord, torelance));
                }
                lines = new_lines;
            }
            lines
        }

        fn connected_lines_into_line_string(lines: Vec<Line2D<f64>>) -> LineString2D<f64> {
            let mut points = Vec::new();
            for i in 0..lines.len() {
                if i == 0 {
                    points.push(lines[i].start);
                }
                points.push(lines[i].end);
            }

            LineString2D::new(points)
        }

        let mut coords_around_line = vec![HashSet::new(); self.line_string.lines().len()];
        for (coords_index, coords) in coordinates.iter().enumerate() {
            let point = Point2D::new(coords.x, coords.y, NoValue);
            let packed_lines = self
                .rtree
                .locate_in_envelope_intersecting(&point.envelope());
            for packed_line in packed_lines {
                coords_around_line[packed_line.index].insert(coords_index);
            }
        }

        let mut new_lss = Vec::new();
        let mut lines_buffer = Vec::new();

        for (line_index, line) in self.line_string.lines().enumerate() {
            let coords_indexes = coords_around_line[line_index]
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            if coords_indexes.is_empty() {
                lines_buffer.push(line.clone());
            } else {
                let split_points = coords_indexes
                    .iter()
                    .map(|index| coordinates[*index].clone())
                    .collect::<Vec<_>>();
                let splits = split_line_by_multiple_coords(line.clone(), split_points, torelance);

                if splits.is_empty() {
                    continue;
                }

                for i in 0..splits.len() - 1 {
                    lines_buffer.push(splits[i].clone());
                    new_lss.push(connected_lines_into_line_string(lines_buffer.clone()));
                    lines_buffer.clear();
                }
                lines_buffer.push(splits[splits.len() - 1].clone());
            }
        }

        new_lss.push(connected_lines_into_line_string(lines_buffer.clone()));

        new_lss
    }
}

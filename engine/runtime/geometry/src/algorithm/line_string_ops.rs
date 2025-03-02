use rstar::RTreeObject;

use crate::types::{
    coordinate::Coordinate, line::Line, line_string::{LineString, LineString2D}, no_value::NoValue
};

use super::{
    intersects::Intersects,
    line_intersection::{line_intersection, LineIntersection},
    GeoFloat,
};

pub trait LineStringOps<T: GeoFloat, Z: GeoFloat> {
    fn intersection(&self, other: LineString<T, Z>) -> Vec<LineIntersection<T, Z>>;
    //fn split(&self, points: Vec<Coordinate<T, Z>>) -> Vec<LineString<T, Z>>;
}

pub struct LineStringWithTree2D {
    rtree: rstar::RTree<Line<f64, NoValue>>,
    line_string: LineString2D<f64>,
}

impl LineStringWithTree2D {
    fn new(line_string: LineString2D<f64>) -> Self {
        let mut packed_lines = Vec::new();

        for line in line_string.lines() {
            packed_lines.push(line.into());
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
                if packed_line.intersects(&other_line) {
                    let intersection = line_intersection(*packed_line, other_line.clone());
                    if let Some(intersection) = intersection {
                        result.push(intersection);
                    }
                }
            }
        }

        result
    }
}

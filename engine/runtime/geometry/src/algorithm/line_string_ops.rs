use std::collections::HashSet;

use rstar::RTreeObject;

use crate::{
    algorithm::line_ops::{LineOps, SplitResult},
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

pub struct LineStringSplitResult<T: GeoFloat, Z: GeoFloat> {
    /// The resulting line strings after the split.
    pub split_line_strings: Vec<LineString<T, Z>>,
    /// The coordinates used to split the line string. Not all input coordinates may be used, as some split points are invalid.
    pub split_coords: Vec<Coordinate<T, Z>>,
}

/// Trait that defines additional operations on a line string.
pub trait LineStringOps<T: GeoFloat, Z: GeoFloat> {
    /// Returns a vector of intersections between `self` and another line string.
    fn intersection(&self, other: &LineString<T, Z>) -> Vec<LineIntersection<T, Z>>;

    /// Splits the line string using the provided coordinates as split points with a given tolerance.
    fn split(&self, coordinates: &[Coordinate<T, Z>], tolerance: T) -> LineStringSplitResult<T, Z>;
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

/// A wrapper around a line string that contains an RTree for fast LineStringOps operations.
pub struct LineStringWithTree2D {
    rtree: rstar::RTree<LineWithIndex2D>,
    line_string: LineString2D<f64>,
}

impl LineStringWithTree2D {
    pub fn new(line_string: LineString2D<f64>) -> Self {
        let mut packed_lines = Vec::new();

        for (index, line) in line_string.lines().enumerate() {
            packed_lines.push(LineWithIndex2D { line, index });
        }

        let rtree = rstar::RTree::bulk_load(packed_lines.clone());

        Self { rtree, line_string }
    }

    pub fn line_string(&self) -> &LineString2D<f64> {
        &self.line_string
    }
}

impl LineStringOps<f64, NoValue> for LineStringWithTree2D {
    fn intersection(&self, other: &LineString2D<f64>) -> Vec<LineIntersection<f64, NoValue>> {
        let mut result = Vec::new();

        for other_line in other.lines() {
            let envelope = other_line.envelope();
            let packed_lines = self.rtree.locate_in_envelope_intersecting(&envelope);

            for packed_line in packed_lines {
                if !packed_line.line.intersects(&other_line) {
                    continue;
                }

                let intersection =
                    if let Some(intersection) = line_intersection(packed_line.line, other_line) {
                        intersection
                    } else {
                        continue;
                    };

                result.push(intersection);
            }
        }

        result
    }

    fn split(
        &self,
        coordinates: &[Coordinate<f64, NoValue>],
        tolerance: f64,
    ) -> LineStringSplitResult<f64, NoValue> {
        // Helper function to split a single line by multiple coordinates.
        #[allow(clippy::type_complexity)]
        fn split_line_by_multiple_coords(
            line: Line<f64, NoValue>,
            coords: Vec<Coordinate<f64, NoValue>>,
            tolerance: f64,
        ) -> (
            Vec<Line<f64, NoValue>>,
            Vec<Coordinate<f64, NoValue>>,
            Vec<Coordinate<f64, NoValue>>,
        ) {
            let mut lines = vec![line];
            let mut split_coords = Vec::new();
            let mut ignored_coords = Vec::new();
            for coord in coords {
                let mut new_lines = Vec::new();
                // Split each current segment by the coordinate.
                for line in lines {
                    match line.split(&coord, tolerance) {
                        SplitResult::Success([first, second]) => {
                            new_lines.push(first);
                            new_lines.push(second);
                            split_coords.push(coord);
                        }
                        SplitResult::FailureNotOnLine(_) => {
                            new_lines.push(line);
                            ignored_coords.push(coord);
                        }
                    }
                }
                lines = new_lines;
            }
            (lines, split_coords, ignored_coords)
        }

        // Helper function to connect a vector of line segments into a single linestring.
        fn connected_lines_into_line_string(lines: Vec<Line2D<f64>>) -> LineString2D<f64> {
            let mut points = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                if i == 0 {
                    points.push(line.start);
                }
                points.push(line.end);
            }

            LineString2D::new(points)
        }

        // Create a vector of HashSets to collect coordinate indexes for each line segment.
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

        let mut split_coords = Vec::new();
        let mut ignored_coords = Vec::new();

        // Iterate through each line segment and apply splitting if needed.
        for (line_index, line) in self.line_string.lines().enumerate() {
            let coords_indexes = coords_around_line[line_index]
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            if coords_indexes.is_empty() {
                lines_buffer.push(line);
            } else {
                let split_points = coords_indexes
                    .iter()
                    .map(|index| coordinates[*index])
                    .collect::<Vec<_>>();
                let (splits, split_points, ignored_points) =
                    split_line_by_multiple_coords(line, split_points, tolerance);
                split_coords.extend(split_points);
                ignored_coords.extend(ignored_points);
                for line in splits.iter().take(splits.len() - 1) {
                    lines_buffer.push(*line);
                    new_lss.push(connected_lines_into_line_string(lines_buffer.clone()));
                    lines_buffer.clear();
                }
                lines_buffer.push(*splits.last().unwrap());
            }
        }

        // split the line strings further at vertices of degree higher than 2
        new_lss.push(connected_lines_into_line_string(lines_buffer.clone()));
        let mut final_lss = Vec::new();
        for ls in new_lss {
            let mut splits = Vec::new();
            for (i, &v) in ls.coords().enumerate().skip(1).take(ls.len() - 2) {
                if ignored_coords.iter().any(|&c| (c - v).norm() < tolerance) {
                    split_coords.push(v);
                    splits.push(i);
                }
            }
            let mut curr_lss = ls;
            for split in splits.into_iter().rev() {
                let (first, second) = curr_lss.split_at(split);
                curr_lss = first;
                final_lss.push(second);
            }
            final_lss.push(curr_lss);
        }

        LineStringSplitResult {
            split_line_strings: final_lss,
            split_coords,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::coordinate::Coordinate2D;

    use super::*;

    #[test]
    fn test_intersection() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 5.0),
            Coordinate2D::new_(5.0, 0.0),
        ]);

        let tree1 = LineStringWithTree2D::new(line_string1);
        let intersections = tree1.intersection(&line_string2);

        assert_eq!(intersections.len(), 1);

        if let LineIntersection::SinglePoint { intersection, .. } = &intersections[0] {
            assert_eq!(intersection.x, 2.5);
            assert_eq!(intersection.y, 2.5);
        } else {
            panic!("Expected a point intersection");
        }
    }

    #[test]
    fn test_no_intersection() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(3.0, 3.0),
        ]);

        let tree1 = LineStringWithTree2D::new(line_string1);
        let intersections = tree1.intersection(&line_string2);

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_split() {
        let line_string = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
        ]);

        let tree = LineStringWithTree2D::new(line_string);
        let split_points = vec![
            Coordinate2D::new_(2.0, 0.0),
            Coordinate2D::new_(5.0, 0.0),
            Coordinate2D::new_(8.0, 0.0),
        ];
        let tolerance = 1e-6;
        let split_result = tree.split(&split_points, tolerance);

        assert_eq!(split_result.split_line_strings.len(), 4);
    }

    #[test]
    fn test_split_no_points() {
        let line_string = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
        ]);

        let tree = LineStringWithTree2D::new(line_string);
        let split_points = vec![Coordinate2D::new_(2.0, 2.0)];
        let tolerance = 1e-6;
        let split_result = tree.split(&split_points, tolerance);

        assert_eq!(split_result.split_line_strings.len(), 1);
        assert_eq!(split_result.split_line_strings[0].points().len(), 2);
    }
}

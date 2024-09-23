use crate::{
    algorithm::{
        line_intersection::{line_intersection, LineIntersection},
        GeoFloat,
    },
    types::{coordinate::Coordinate, line::Line},
};

use num_traits::Zero;

use super::line_intersector::LineIntersector;

/// A robust version of [LineIntersector](traits.LineIntersector).
#[derive(Clone)]
pub(crate) struct RobustLineIntersector;

impl RobustLineIntersector {
    pub fn new() -> RobustLineIntersector {
        RobustLineIntersector
    }
}

impl<T: GeoFloat, Z: GeoFloat> LineIntersector<T, Z> for RobustLineIntersector {
    fn compute_intersection(
        &mut self,
        p: Line<T, Z>,
        q: Line<T, Z>,
    ) -> Option<LineIntersection<T, Z>> {
        line_intersection(p, q)
    }
}

impl RobustLineIntersector {
    pub fn compute_edge_distance<T: GeoFloat, Z: GeoFloat>(
        intersection: Coordinate<T, Z>,
        line: Line<T, Z>,
    ) -> T {
        let dx = (line.end.x - line.start.x).abs();
        let dy = (line.end.y - line.start.y).abs();

        let mut dist: T;
        if intersection == line.start {
            dist = T::zero();
        } else if intersection == line.end {
            if dx > dy {
                dist = dx;
            } else {
                dist = dy;
            }
        } else {
            let intersection_dx = (intersection.x - line.start.x).abs();
            let intersection_dy = (intersection.y - line.start.y).abs();
            if dx > dy {
                dist = intersection_dx;
            } else {
                dist = intersection_dy;
            }
            // hack to ensure that non-endpoints always have a non-zero distance
            if dist == T::zero() && intersection != line.start {
                dist = intersection_dx.max(intersection_dy);
            }
        }
        dist
    }
}

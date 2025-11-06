use num_traits::{Bounded, Float};

use rstar::primitives::CachedEnvelope;
use rstar::RTree;

use crate::types::coordinate::Coordinate2D;
use crate::types::coordinate::Coordinate3D;
use crate::types::line::Line2D;
use crate::types::line::Line3D;
use crate::types::line_string::LineString2D;
use crate::types::line_string::LineString3D;
use crate::types::point::Point;
use crate::types::point::Point2D;
use crate::types::point::Point3D;
use crate::types::polygon::Polygon;
use crate::types::polygon::Polygon2D;
use crate::types::polygon::Polygon3D;
use crate::utils::line_segment_distance;
use crate::utils::point_line_euclidean_distance;
use crate::utils::point_line_string_euclidean_distance;

use super::coordinate_position::coord_pos_relative_to_ring;
use super::coordinate_position::CoordPos;
use super::euclidean_length::EuclideanLength;
use super::intersects::Intersects;
use super::GeoNum;

/// Returns the distance between two geometries.
pub trait EuclideanDistance<Rhs = Self> {
    fn euclidean_distance(&self, rhs: &Rhs) -> f64;
}

// ┌───────────────────────────┐
// │ Implementations for Coord │
// └───────────────────────────┘

impl EuclideanDistance<Coordinate2D<f64>> for Coordinate2D<f64> {
    /// Minimum distance between two `Coord`s
    fn euclidean_distance(&self, c: &Coordinate2D<f64>) -> f64 {
        Line2D::new_(*self, *c).euclidean_length()
    }
}

impl EuclideanDistance<Coordinate3D<f64>> for Coordinate3D<f64> {
    /// Minimum distance between two `Coord`s
    fn euclidean_distance(&self, c: &Coordinate3D<f64>) -> f64 {
        Line3D::new_(*self, *c).euclidean_length()
    }
}

impl EuclideanDistance<Line2D<f64>> for Coordinate2D<f64> {
    /// Minimum distance from a `Coord` to a `Line`
    fn euclidean_distance(&self, line: &Line2D<f64>) -> f64 {
        line.euclidean_distance(self)
    }
}

impl EuclideanDistance<Line3D<f64>> for Coordinate3D<f64> {
    /// Minimum distance from a `Coord` to a `Line`
    fn euclidean_distance(&self, line: &Line3D<f64>) -> f64 {
        line.euclidean_distance(self)
    }
}

// ┌───────────────────────────┐
// │ Implementations for Point │
// └───────────────────────────┘

impl EuclideanDistance<Point2D<f64>> for Point2D<f64> {
    /// Minimum distance between two Points
    fn euclidean_distance(&self, p: &Point2D<f64>) -> f64 {
        self.0.euclidean_distance(&p.0)
    }
}

impl EuclideanDistance<Line2D<f64>> for Point2D<f64> {
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, line: &Line2D<f64>) -> f64 {
        self.0.euclidean_distance(line)
    }
}

impl EuclideanDistance<Point3D<f64>> for Point3D<f64> {
    /// Minimum distance between two Points
    fn euclidean_distance(&self, p: &Point3D<f64>) -> f64 {
        self.0.euclidean_distance(&p.0)
    }
}

impl EuclideanDistance<Line3D<f64>> for Point3D<f64> {
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, line: &Line3D<f64>) -> f64 {
        self.0.euclidean_distance(line)
    }
}

impl EuclideanDistance<LineString2D<f64>> for Point2D<f64> {
    /// Minimum distance from a Point to a LineString
    fn euclidean_distance(&self, linestring: &LineString2D<f64>) -> f64 {
        point_line_string_euclidean_distance(*self, linestring)
    }
}

impl EuclideanDistance<LineString3D<f64>> for Point3D<f64> {
    /// Minimum distance from a Point to a LineString
    fn euclidean_distance(&self, linestring: &LineString3D<f64>) -> f64 {
        point_line_string_euclidean_distance(*self, linestring)
    }
}

impl EuclideanDistance<Polygon2D<f64>> for Point2D<f64> {
    /// Minimum distance from a Point to a Polygon
    fn euclidean_distance(&self, polygon: &Polygon2D<f64>) -> f64 {
        // No need to continue if the polygon intersects the point, or is zero-length
        if polygon.exterior().0.is_empty() || polygon.intersects(self) {
            return 0.0;
        }
        // fold the minimum interior ring distance if any, followed by the exterior
        // shell distance, returning the minimum of the two distances
        polygon
            .interiors()
            .iter()
            .map(|ring| self.euclidean_distance(ring))
            .fold(<f64 as Bounded>::max_value(), |accum, val| accum.min(val))
            .min(
                polygon
                    .exterior()
                    .lines()
                    .map(|line| line_segment_distance(self.0, line.start, line.end))
                    .fold(<f64 as Bounded>::max_value(), |accum, val| accum.min(val)),
            )
    }
}

impl EuclideanDistance<Polygon3D<f64>> for Point3D<f64> {
    fn euclidean_distance(&self, polygon: &Polygon3D<f64>) -> f64 {
        // No need to continue if the polygon intersects the point, or is zero-length
        if polygon.exterior().0.is_empty() || polygon.intersects(self) {
            return 0.0;
        }
        polygon
            .interiors()
            .iter()
            .map(|ring| self.euclidean_distance(ring))
            .fold(<f64 as Bounded>::max_value(), |accum, val| accum.min(val))
            .min(
                polygon
                    .exterior()
                    .lines()
                    .map(|line| line_segment_distance(self.0, line.start, line.end))
                    .fold(<f64 as Bounded>::max_value(), |accum, val| accum.min(val)),
            )
    }
}

// ┌──────────────────────────┐
// │ Implementations for Line │
// └──────────────────────────┘

impl EuclideanDistance<Coordinate2D<f64>> for Line2D<f64> {
    /// Minimum distance from a `Line` to a `Coord`
    fn euclidean_distance(&self, coord: &Coordinate2D<f64>) -> f64 {
        point_line_euclidean_distance(Point2D::from(*coord), *self)
    }
}

impl EuclideanDistance<Coordinate3D<f64>> for Line3D<f64> {
    /// Minimum distance from a `Line` to a `Coord`
    fn euclidean_distance(&self, coord: &Coordinate3D<f64>) -> f64 {
        point_line_euclidean_distance(Point3D::from(*coord), *self)
    }
}

impl EuclideanDistance<Point2D<f64>> for Line2D<f64> {
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, point: &Point2D<f64>) -> f64 {
        self.euclidean_distance(&point.0)
    }
}

impl EuclideanDistance<Point3D<f64>> for Line3D<f64> {
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, point: &Point3D<f64>) -> f64 {
        self.euclidean_distance(&point.0)
    }
}

/// Line to Line distance
impl EuclideanDistance<Line2D<f64>> for Line2D<f64> {
    fn euclidean_distance(&self, other: &Line2D<f64>) -> f64 {
        if self.intersects(other) {
            return 0.0;
        }
        // minimum of all Point-Line distances
        self.start_point()
            .euclidean_distance(other)
            .min(self.end_point().euclidean_distance(other))
            .min(other.start_point().euclidean_distance(self))
            .min(other.end_point().euclidean_distance(self))
    }
}

impl EuclideanDistance<Line3D<f64>> for Line3D<f64> {
    fn euclidean_distance(&self, other: &Line3D<f64>) -> f64 {
        if self.intersects(other) {
            return 0.0;
        }
        // minimum of all Point-Line distances
        self.start_point()
            .euclidean_distance(other)
            .min(self.end_point().euclidean_distance(other))
            .min(other.start_point().euclidean_distance(self))
            .min(other.end_point().euclidean_distance(self))
    }
}

/// Line to LineString
impl EuclideanDistance<LineString2D<f64>> for Line2D<f64> {
    fn euclidean_distance(&self, other: &LineString2D<f64>) -> f64 {
        other.euclidean_distance(self)
    }
}

impl EuclideanDistance<LineString3D<f64>> for Line3D<f64> {
    fn euclidean_distance(&self, other: &LineString3D<f64>) -> f64 {
        other.euclidean_distance(self)
    }
}

// Line to Polygon distance
impl EuclideanDistance<Polygon2D<f64>> for Line2D<f64> {
    fn euclidean_distance(&self, other: &Polygon2D<f64>) -> f64 {
        if self.intersects(other) {
            return 0.0;
        }
        // line-line distance between each exterior polygon segment and the line
        let exterior_min = other
            .exterior()
            .lines()
            .fold(<f64 as Bounded>::max_value(), |acc, point| {
                acc.min(self.euclidean_distance(&point))
            });
        // line-line distance between each interior ring segment and the line
        // if there are no rings this just evaluates to max_float
        let interior_min = other
            .interiors()
            .iter()
            .map(|ring| {
                ring.lines()
                    .fold(<f64 as Bounded>::max_value(), |acc, line| {
                        acc.min(self.euclidean_distance(&line))
                    })
            })
            .fold(<f64 as Bounded>::max_value(), |acc, ring_min| {
                acc.min(ring_min)
            });
        // return smaller of the two values
        exterior_min.min(interior_min)
    }
}

impl EuclideanDistance<Polygon3D<f64>> for Line3D<f64> {
    fn euclidean_distance(&self, other: &Polygon3D<f64>) -> f64 {
        if self.intersects(other) {
            return 0.0;
        }
        // line-line distance between each exterior polygon segment and the line
        let exterior_min = other
            .exterior()
            .lines()
            .fold(<f64 as Bounded>::max_value(), |acc, point| {
                acc.min(self.euclidean_distance(&point))
            });
        // line-line distance between each interior ring segment and the line
        // if there are no rings this just evaluates to max_float
        let interior_min = other
            .interiors()
            .iter()
            .map(|ring| {
                ring.lines()
                    .fold(<f64 as Bounded>::max_value(), |acc, line| {
                        acc.min(self.euclidean_distance(&line))
                    })
            })
            .fold(<f64 as Bounded>::max_value(), |acc, ring_min| {
                acc.min(ring_min)
            });
        // return smaller of the two values
        exterior_min.min(interior_min)
    }
}

// ┌────────────────────────────────┐
// │ Implementations for LineString │
// └────────────────────────────────┘

impl EuclideanDistance<Point2D<f64>> for LineString2D<f64> {
    /// Minimum distance from a LineString to a Point
    fn euclidean_distance(&self, point: &Point2D<f64>) -> f64 {
        point.euclidean_distance(self)
    }
}

impl EuclideanDistance<Point3D<f64>> for LineString3D<f64> {
    /// Minimum distance from a LineString to a Point
    fn euclidean_distance(&self, point: &Point3D<f64>) -> f64 {
        point.euclidean_distance(self)
    }
}

/// LineString to Line
impl EuclideanDistance<Line2D<f64>> for LineString2D<f64> {
    fn euclidean_distance(&self, other: &Line2D<f64>) -> f64 {
        self.lines().fold(Bounded::max_value(), |acc, line| {
            acc.min(line.euclidean_distance(other))
        })
    }
}

impl EuclideanDistance<Line3D<f64>> for LineString3D<f64> {
    fn euclidean_distance(&self, other: &Line3D<f64>) -> f64 {
        self.lines().fold(Bounded::max_value(), |acc, line| {
            acc.min(line.euclidean_distance(other))
        })
    }
}

impl EuclideanDistance<LineString2D<f64>> for LineString2D<f64> {
    fn euclidean_distance(&self, other: &LineString2D<f64>) -> f64 {
        if self.intersects(other) {
            0.0
        } else {
            nearest_neighbour_distance2d(self, other)
        }
    }
}

impl EuclideanDistance<LineString3D<f64>> for LineString3D<f64> {
    fn euclidean_distance(&self, other: &LineString3D<f64>) -> f64 {
        if self.intersects(other) {
            0.0
        } else {
            nearest_neighbour_distance3d(self, other)
        }
    }
}

/// LineString to Polygon
impl EuclideanDistance<Polygon2D<f64>> for LineString2D<f64> {
    fn euclidean_distance(&self, other: &Polygon2D<f64>) -> f64 {
        if self.intersects(other) {
            0.0
        } else if !other.interiors().is_empty()
            && ring_contains_point(other, Point2D::from(self.0[0]))
        {
            // check each ring distance, returning the minimum
            let mut mindist: f64 = Float::max_value();
            for ring in other.interiors() {
                mindist = mindist.min(nearest_neighbour_distance2d(self, ring))
            }
            mindist
        } else {
            nearest_neighbour_distance2d(self, other.exterior())
        }
    }
}

impl EuclideanDistance<Polygon3D<f64>> for LineString3D<f64> {
    fn euclidean_distance(&self, other: &Polygon3D<f64>) -> f64 {
        if self.intersects(other) {
            0.0
        } else if !other.interiors().is_empty()
            && ring_contains_point(other, Point3D::from(self.0[0]))
        {
            // check each ring distance, returning the minimum
            let mut mindist: f64 = Float::max_value();
            for ring in other.interiors() {
                mindist = mindist.min(nearest_neighbour_distance3d(self, ring))
            }
            mindist
        } else {
            nearest_neighbour_distance3d(self, other.exterior())
        }
    }
}

// ┌─────────────────────────────┐
// │ Implementations for Polygon │
// └─────────────────────────────┘

impl EuclideanDistance<Point2D<f64>> for Polygon2D<f64> {
    /// Minimum distance from a Polygon to a Point
    fn euclidean_distance(&self, point: &Point2D<f64>) -> f64 {
        point.euclidean_distance(self)
    }
}

impl EuclideanDistance<Point3D<f64>> for Polygon3D<f64> {
    /// Minimum distance from a Polygon to a Point
    fn euclidean_distance(&self, point: &Point3D<f64>) -> f64 {
        point.euclidean_distance(self)
    }
}

// Polygon to Line distance
impl EuclideanDistance<Line2D<f64>> for Polygon2D<f64> {
    fn euclidean_distance(&self, other: &Line2D<f64>) -> f64 {
        other.euclidean_distance(self)
    }
}

impl EuclideanDistance<Line3D<f64>> for Polygon3D<f64> {
    fn euclidean_distance(&self, other: &Line3D<f64>) -> f64 {
        other.euclidean_distance(self)
    }
}

/// Polygon to LineString distance
impl EuclideanDistance<LineString2D<f64>> for Polygon2D<f64> {
    fn euclidean_distance(&self, other: &LineString2D<f64>) -> f64 {
        other.euclidean_distance(self)
    }
}

impl EuclideanDistance<LineString3D<f64>> for Polygon3D<f64> {
    fn euclidean_distance(&self, other: &LineString3D<f64>) -> f64 {
        other.euclidean_distance(self)
    }
}

// Polygon to Polygon distance
impl EuclideanDistance<Polygon2D<f64>> for Polygon2D<f64> {
    fn euclidean_distance(&self, poly2: &Polygon2D<f64>) -> f64 {
        if self.intersects(poly2) {
            return 0.0;
        }
        // Containment check
        if !self.interiors().is_empty()
            && ring_contains_point(self, Point::from(poly2.exterior().0[0]))
        {
            // check each ring distance, returning the minimum
            let mut mindist: f64 = Float::max_value();
            for ring in self.interiors() {
                mindist = mindist.min(nearest_neighbour_distance2d(poly2.exterior(), ring))
            }
            return mindist;
        } else if !poly2.interiors().is_empty()
            && ring_contains_point(poly2, Point::from(self.exterior().0[0]))
        {
            let mut mindist: f64 = Float::max_value();
            for ring in poly2.interiors() {
                mindist = mindist.min(nearest_neighbour_distance2d(self.exterior(), ring))
            }
            return mindist;
        }
        nearest_neighbour_distance2d(self.exterior(), poly2.exterior())
    }
}

impl EuclideanDistance<Polygon3D<f64>> for Polygon3D<f64> {
    fn euclidean_distance(&self, poly2: &Polygon3D<f64>) -> f64 {
        if self.intersects(poly2) {
            return 0.0;
        }
        // Containment check
        if !self.interiors().is_empty()
            && ring_contains_point(self, Point::from(poly2.exterior().0[0]))
        {
            // check each ring distance, returning the minimum
            let mut mindist: f64 = Float::max_value();
            for ring in self.interiors() {
                mindist = mindist.min(nearest_neighbour_distance3d(poly2.exterior(), ring))
            }
            return mindist;
        } else if !poly2.interiors().is_empty()
            && ring_contains_point(poly2, Point::from(self.exterior().0[0]))
        {
            let mut mindist: f64 = Float::max_value();
            for ring in poly2.interiors() {
                mindist = mindist.min(nearest_neighbour_distance3d(self.exterior(), ring))
            }
            return mindist;
        }
        nearest_neighbour_distance3d(self.exterior(), poly2.exterior())
    }
}

// ┌───────────┐
// │ Utilities │
// └───────────┘
pub fn ring_contains_point<T, Z>(poly: &Polygon<T, Z>, p: Point<T, Z>) -> bool
where
    T: GeoNum,
    Z: GeoNum,
{
    match coord_pos_relative_to_ring(p.0, poly.exterior()) {
        CoordPos::Inside => true,
        CoordPos::OnBoundary | CoordPos::Outside => false,
    }
}

pub fn nearest_neighbour_distance2d(geom1: &LineString2D<f64>, geom2: &LineString2D<f64>) -> f64 {
    let tree_a = RTree::bulk_load(geom1.lines().map(CachedEnvelope::new).collect());
    let tree_b = RTree::bulk_load(geom2.lines().map(CachedEnvelope::new).collect());
    // Return minimum distance between all geom a points and geom b lines, and all geom b points and geom a lines
    geom2
        .points()
        .fold(<f64 as Bounded>::max_value(), |acc, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        })
        .min(geom1.points().fold(Bounded::max_value(), |acc, point| {
            let nearest = tree_b.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        }))
}

pub fn nearest_neighbour_distance3d(geom1: &LineString3D<f64>, geom2: &LineString3D<f64>) -> f64 {
    let tree_a = RTree::bulk_load(geom1.lines().map(CachedEnvelope::new).collect());
    let tree_b = RTree::bulk_load(geom2.lines().map(CachedEnvelope::new).collect());
    // Return minimum distance between all geom a points and geom b lines, and all geom b points and geom a lines
    geom2
        .points()
        .fold(<f64 as Bounded>::max_value(), |acc, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        })
        .min(geom1.points().fold(Bounded::max_value(), |acc, point| {
            let nearest = tree_b.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, point};

    #[test]
    fn test_point_to_point_distance_same() {
        let p1 = point! { x: 0.0, y: 0.0 };
        let p2 = point! { x: 0.0, y: 0.0 };
        assert_eq!(p1.euclidean_distance(&p2), 0.0);
    }

    #[test]
    fn test_point_to_point_distance_pythagorean() {
        let p1 = point! { x: 0.0, y: 0.0 };
        let p2 = point! { x: 3.0, y: 4.0 };
        assert_eq!(p1.euclidean_distance(&p2), 5.0);
    }

    #[test]
    fn test_point_to_line_distance() {
        let point = point! { x: 0.0, y: 5.0 };
        let line = Line2D::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 10.0, y: 0.0 },
        );
        assert_eq!(point.euclidean_distance(&line), 5.0);
    }

    #[test]
    fn test_point_to_polygon_distance_inside() {
        let point = point! { x: 5.0, y: 5.0 };
        let exterior = LineString2D::new(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 10.0, y: 0.0 },
            coord! { x: 10.0, y: 10.0 },
            coord! { x: 0.0, y: 10.0 },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let polygon = Polygon2D::new(exterior, vec![]);
        
        assert_eq!(point.euclidean_distance(&polygon), 0.0);
    }

    #[test]
    fn test_line_to_line_distance_parallel() {
        let line1 = Line2D::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 10.0, y: 0.0 },
        );
        let line2 = Line2D::new(
            coord! { x: 0.0, y: 5.0 },
            coord! { x: 10.0, y: 5.0 },
        );
        
        assert_eq!(line1.euclidean_distance(&line2), 5.0);
    }

    #[test]
    fn test_line_to_line_distance_intersecting() {
        let line1 = Line2D::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 10.0, y: 10.0 },
        );
        let line2 = Line2D::new(
            coord! { x: 0.0, y: 10.0 },
            coord! { x: 10.0, y: 0.0 },
        );
        
        assert_eq!(line1.euclidean_distance(&line2), 0.0);
    }

    #[test]
    fn test_polygon_to_polygon_distance_separate() {
        let exterior1 = LineString2D::new(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 5.0, y: 0.0 },
            coord! { x: 5.0, y: 5.0 },
            coord! { x: 0.0, y: 5.0 },
            coord! { x: 0.0, y: 0.0 },
        ]);
        
        let exterior2 = LineString2D::new(vec![
            coord! { x: 10.0, y: 10.0 },
            coord! { x: 15.0, y: 10.0 },
            coord! { x: 15.0, y: 15.0 },
            coord! { x: 10.0, y: 15.0 },
            coord! { x: 10.0, y: 10.0 },
        ]);
        
        let poly1 = Polygon2D::new(exterior1, vec![]);
        let poly2 = Polygon2D::new(exterior2, vec![]);
        
        let distance = poly1.euclidean_distance(&poly2);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_building_distance_calculation() {
        let building1 = point! { x: 139.7503, y: 35.6851 };
        let building2 = point! { x: 139.7506, y: 35.6854 };
        
        let distance = building1.euclidean_distance(&building2);
        assert!(distance > 0.0);
        assert!(distance < 1.0);
    }
}

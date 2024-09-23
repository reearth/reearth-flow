use num_traits::{float::FloatConst, Bounded, Float, Signed};

use rstar::primitives::CachedEnvelope;
use rstar::RTree;
use rstar::RTreeNum;

use crate::types::coordinate::Coordinate;
use crate::types::coordnum::CoordNum;
use crate::types::line::Line;
use crate::types::line_string::LineString;
use crate::types::point::Point;
use crate::types::polygon::Polygon;
use crate::utils::line_segment_distance;
use crate::utils::point_line_euclidean_distance;
use crate::utils::point_line_string_euclidean_distance;

use super::coordinate_position::coord_pos_relative_to_ring;
use super::coordinate_position::CoordPos;
use super::euclidean_length::EuclideanLength;
use super::intersects::Intersects;
use super::GeoFloat;
use super::GeoNum;

/// Returns the distance between two geometries.

pub trait EuclideanDistance<T, Rhs = Self> {
    fn euclidean_distance(&self, rhs: &Rhs) -> T;
}

// ┌───────────────────────────┐
// │ Implementations for Coord │
// └───────────────────────────┘

impl<T, Z> EuclideanDistance<T, Coordinate<T, Z>> for Coordinate<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance between two `Coord`s
    fn euclidean_distance(&self, c: &Coordinate<T, Z>) -> T {
        Line::new_(*self, *c).euclidean_length()
    }
}

impl<T, Z> EuclideanDistance<T, Line<T, Z>> for Coordinate<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a `Coord` to a `Line`
    fn euclidean_distance(&self, line: &Line<T, Z>) -> T {
        line.euclidean_distance(self)
    }
}

// ┌───────────────────────────┐
// │ Implementations for Point │
// └───────────────────────────┘

impl<T, Z> EuclideanDistance<T, Point<T, Z>> for Point<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance between two Points
    fn euclidean_distance(&self, p: &Point<T, Z>) -> T {
        self.0.euclidean_distance(&p.0)
    }
}

impl<T, Z> EuclideanDistance<T, Line<T, Z>> for Point<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, line: &Line<T, Z>) -> T {
        self.0.euclidean_distance(line)
    }
}

impl<T, Z> EuclideanDistance<T, LineString<T, Z>> for Point<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a Point to a LineString
    fn euclidean_distance(&self, linestring: &LineString<T, Z>) -> T {
        point_line_string_euclidean_distance(*self, linestring)
    }
}

impl<T, Z> EuclideanDistance<T, Polygon<T, Z>> for Point<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a Point to a Polygon
    fn euclidean_distance(&self, polygon: &Polygon<T, Z>) -> T {
        // No need to continue if the polygon intersects the point, or is zero-length
        if polygon.exterior().0.is_empty() || polygon.intersects(self) {
            return T::zero();
        }
        // fold the minimum interior ring distance if any, followed by the exterior
        // shell distance, returning the minimum of the two distances
        polygon
            .interiors()
            .iter()
            .map(|ring| self.euclidean_distance(ring))
            .fold(<T as Bounded>::max_value(), |accum, val| accum.min(val))
            .min(
                polygon
                    .exterior()
                    .lines()
                    .map(|line| line_segment_distance(self.0, line.start, line.end))
                    .fold(<T as Bounded>::max_value(), |accum, val| accum.min(val)),
            )
    }
}

// ┌──────────────────────────┐
// │ Implementations for Line │
// └──────────────────────────┘

impl<T, Z> EuclideanDistance<T, Coordinate<T, Z>> for Line<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a `Line` to a `Coord`
    fn euclidean_distance(&self, coord: &Coordinate<T, Z>) -> T {
        point_line_euclidean_distance(Point::from(*coord), *self)
    }
}

impl<T, Z> EuclideanDistance<T, Point<T, Z>> for Line<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, point: &Point<T, Z>) -> T {
        self.euclidean_distance(&point.0)
    }
}

/// Line to Line distance
impl<T, Z> EuclideanDistance<T, Line<T, Z>> for Line<T, Z>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
    Z: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Line<T, Z>) -> T {
        if self.intersects(other) {
            return T::zero();
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
impl<T, Z> EuclideanDistance<T, LineString<T, Z>> for Line<T, Z>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
    Z: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &LineString<T, Z>) -> T {
        other.euclidean_distance(self)
    }
}

// Line to Polygon distance
impl<T, Z> EuclideanDistance<T, Polygon<T, Z>> for Line<T, Z>
where
    T: GeoFloat + Signed + RTreeNum + FloatConst,
    Z: GeoFloat + Signed + RTreeNum + FloatConst,
{
    fn euclidean_distance(&self, other: &Polygon<T, Z>) -> T {
        if self.intersects(other) {
            return T::zero();
        }
        // line-line distance between each exterior polygon segment and the line
        let exterior_min = other
            .exterior()
            .lines()
            .fold(<T as Bounded>::max_value(), |acc, point| {
                acc.min(self.euclidean_distance(&point))
            });
        // line-line distance between each interior ring segment and the line
        // if there are no rings this just evaluates to max_float
        let interior_min = other
            .interiors()
            .iter()
            .map(|ring| {
                ring.lines().fold(<T as Bounded>::max_value(), |acc, line| {
                    acc.min(self.euclidean_distance(&line))
                })
            })
            .fold(<T as Bounded>::max_value(), |acc, ring_min| {
                acc.min(ring_min)
            });
        // return smaller of the two values
        exterior_min.min(interior_min)
    }
}

// ┌────────────────────────────────┐
// │ Implementations for LineString │
// └────────────────────────────────┘

impl<T, Z> EuclideanDistance<T, Point<T, Z>> for LineString<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a LineString to a Point
    fn euclidean_distance(&self, point: &Point<T, Z>) -> T {
        point.euclidean_distance(self)
    }
}

/// LineString to Line
impl<T, Z> EuclideanDistance<T, Line<T, Z>> for LineString<T, Z>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
    Z: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Line<T, Z>) -> T {
        self.lines().fold(Bounded::max_value(), |acc, line| {
            acc.min(line.euclidean_distance(other))
        })
    }
}

impl<T, Z> EuclideanDistance<T, LineString<T, Z>> for LineString<T, Z>
where
    T: GeoFloat + Signed + RTreeNum + CoordNum + rstar::Point,
    Z: GeoFloat + Signed + RTreeNum + CoordNum + rstar::Point,
{
    fn euclidean_distance(&self, other: &LineString<T, Z>) -> T {
        if self.intersects(other) {
            T::zero()
        } else {
            nearest_neighbour_distance(self, other)
        }
    }
}

/// LineString to Polygon
impl<T, Z> EuclideanDistance<T, Polygon<T, Z>> for LineString<T, Z>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum + CoordNum + rstar::Point,
    Z: GeoFloat + FloatConst + Signed + RTreeNum + CoordNum + rstar::Point,
{
    fn euclidean_distance(&self, other: &Polygon<T, Z>) -> T {
        if self.intersects(other) {
            T::zero()
        } else if !other.interiors().is_empty()
            && ring_contains_point(other, Point::from(self.0[0]))
        {
            // check each ring distance, returning the minimum
            let mut mindist: T = Float::max_value();
            for ring in other.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(self, ring))
            }
            mindist
        } else {
            nearest_neighbour_distance(self, other.exterior())
        }
    }
}

// ┌─────────────────────────────┐
// │ Implementations for Polygon │
// └─────────────────────────────┘

impl<T, Z> EuclideanDistance<T, Point<T, Z>> for Polygon<T, Z>
where
    T: GeoFloat,
    Z: GeoFloat,
{
    /// Minimum distance from a Polygon to a Point
    fn euclidean_distance(&self, point: &Point<T, Z>) -> T {
        point.euclidean_distance(self)
    }
}

// Polygon to Line distance
impl<T, Z> EuclideanDistance<T, Line<T, Z>> for Polygon<T, Z>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
    Z: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Line<T, Z>) -> T {
        other.euclidean_distance(self)
    }
}

/// Polygon to LineString distance
impl<T, Z> EuclideanDistance<T, LineString<T, Z>> for Polygon<T, Z>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum + CoordNum + rstar::Point,
    Z: GeoFloat + FloatConst + Signed + RTreeNum + CoordNum + rstar::Point,
{
    fn euclidean_distance(&self, other: &LineString<T, Z>) -> T {
        other.euclidean_distance(self)
    }
}

// Polygon to Polygon distance
impl<T, Z> EuclideanDistance<T, Polygon<T, Z>> for Polygon<T, Z>
where
    T: GeoFloat + FloatConst + RTreeNum + CoordNum + rstar::Point,
    Z: GeoFloat + FloatConst + RTreeNum + CoordNum + rstar::Point,
{
    fn euclidean_distance(&self, poly2: &Polygon<T, Z>) -> T {
        if self.intersects(poly2) {
            return T::zero();
        }
        // Containment check
        if !self.interiors().is_empty()
            && ring_contains_point(self, Point::from(poly2.exterior().0[0]))
        {
            // check each ring distance, returning the minimum
            let mut mindist: T = Float::max_value();
            for ring in self.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(poly2.exterior(), ring))
            }
            return mindist;
        } else if !poly2.interiors().is_empty()
            && ring_contains_point(poly2, Point::from(self.exterior().0[0]))
        {
            let mut mindist: T = Float::max_value();
            for ring in poly2.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(self.exterior(), ring))
            }
            return mindist;
        }
        nearest_neighbour_distance(self.exterior(), poly2.exterior())
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

pub fn nearest_neighbour_distance<T, Z>(geom1: &LineString<T, Z>, geom2: &LineString<T, Z>) -> T
where
    T: GeoFloat + RTreeNum + rstar::Point,
    Z: GeoFloat + RTreeNum + rstar::Point,
{
    let tree_a = RTree::bulk_load(geom1.lines().map(CachedEnvelope::new).collect());
    let tree_b = RTree::bulk_load(geom2.lines().map(CachedEnvelope::new).collect());
    // Return minimum distance between all geom a points and geom b lines, and all geom b points and geom a lines
    geom2
        .points()
        .fold(<T as Bounded>::max_value(), |acc, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        })
        .min(geom1.points().fold(Bounded::max_value(), |acc, point| {
            let nearest = tree_b.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        }))
}

use std::cmp::Ordering;

use crate::types::multi_line_string::MultiLineString;
use crate::types::multi_point::MultiPoint;
use crate::types::multi_polygon::MultiPolygon;
use crate::types::polygon::Polygon;
use crate::types::rect::Rect;
use crate::types::triangle::Triangle;
use crate::types::{coordinate::Coordinate, line::Line, line_string::LineString, point::Point};

use super::dimensions::HasDimensions;
use super::geometry_cow::GeometryCow;
use super::intersects::*;
use super::kernels::{Orientation, RobustKernel};
use super::GeoNum;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CoordPos {
    OnBoundary,
    Inside,
    Outside,
}

pub trait CoordinatePosition {
    type ScalarXY: GeoNum;
    type ScalarZ: GeoNum;
    fn coordinate_position(&self, coord: &Coordinate<Self::ScalarXY, Self::ScalarZ>) -> CoordPos {
        let mut is_inside = false;
        let mut boundary_count = 0;

        self.calculate_coordinate_position(coord, &mut is_inside, &mut boundary_count);

        if boundary_count % 2 == 1 {
            CoordPos::OnBoundary
        } else if is_inside {
            CoordPos::Inside
        } else {
            CoordPos::Outside
        }
    }

    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<Self::ScalarXY, Self::ScalarZ>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    );
}

impl<'a, T: GeoNum, Z: GeoNum> CoordinatePosition for GeometryCow<'a, T, Z> {
    type ScalarXY = T;
    type ScalarZ = Z;
    crate::geometry_cow_delegate_impl! {
        fn calculate_coordinate_position(
            &self,
            coord: &Coordinate<T, Z>,
            is_inside: &mut bool,
            boundary_count: &mut usize) -> ();
    }
}

impl<T, Z> CoordinatePosition for Coordinate<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self == coord {
            *is_inside = true;
        }
    }
}

impl<T, Z> CoordinatePosition for Point<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if &self.0 == coord {
            *is_inside = true;
        }
    }
}

impl<T, Z> CoordinatePosition for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        // degenerate line is a point
        if self.start == self.end {
            self.start
                .calculate_coordinate_position(coord, is_inside, boundary_count);
            return;
        }

        if coord == &self.start || coord == &self.end {
            *boundary_count += 1;
        } else if self.intersects(coord) {
            *is_inside = true;
        }
    }
}

impl<T, Z> CoordinatePosition for LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.0.len() < 2 {
            debug_assert!(false, "invalid line string with less than 2 coords");
            return;
        }

        if self.0.len() == 2 {
            // line string with two coords is just a line
            Line::new_(self.0[0], self.0[1]).calculate_coordinate_position(
                coord,
                is_inside,
                boundary_count,
            );
            return;
        }

        // optimization: return early if there's no chance of an intersection
        // since self.0 is non-empty, safe to `unwrap`
        // TODO
        // if !self.bounding_rect().unwrap().intersects(coord) {
        //     return;
        // }

        // A closed linestring has no boundary, per SFS
        if !self.is_closed() {
            // since self.0 is non-empty, safe to `unwrap`
            if coord == self.0.first().unwrap() || coord == self.0.last().unwrap() {
                *boundary_count += 1;
                return;
            }
        }

        if self.intersects(coord) {
            // We've already checked for "Boundary" condition, so if there's an intersection at
            // this point, coord must be on the interior
            *is_inside = true
        }
    }
}

impl<T, Z> CoordinatePosition for Triangle<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        *is_inside = self
            .to_lines()
            .map(|l| {
                let orientation = RobustKernel::orient(l.start, l.end, *coord, None);
                if orientation == Orientation::Collinear
                    && point_in_rect(*coord, l.start, l.end)
                    && coord.x != l.end.x
                {
                    *boundary_count += 1;
                }
                orientation
            })
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear);
    }
}

impl<T, Z> CoordinatePosition for Rect<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        let mut boundary = false;

        let min = self.min();

        match coord.x.partial_cmp(&min.x).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        match coord.y.partial_cmp(&min.y).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        if !coord.z.is_nan() {
            match coord.z.partial_cmp(&min.z).unwrap() {
                Ordering::Less => return,
                Ordering::Equal => boundary = true,
                Ordering::Greater => {}
            }
        }

        let max = self.max();

        match max.x.partial_cmp(&coord.x).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        match max.y.partial_cmp(&coord.y).unwrap() {
            Ordering::Less => return,
            Ordering::Equal => boundary = true,
            Ordering::Greater => {}
        }
        if !coord.z.is_nan() {
            match max.z.partial_cmp(&coord.z).unwrap() {
                Ordering::Less => return,
                Ordering::Equal => boundary = true,
                Ordering::Greater => {}
            }
        }

        if boundary {
            *boundary_count += 1;
        } else {
            *is_inside = true;
        }
    }
}

impl<T, Z> CoordinatePosition for MultiPoint<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        _boundary_count: &mut usize,
    ) {
        if self.0.iter().any(|p| &p.0 == coord) {
            *is_inside = true;
        }
    }
}

impl<T, Z> CoordinatePosition for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if self.is_empty() {
            return;
        }

        match coord_pos_relative_to_ring(*coord, self.exterior()) {
            CoordPos::Outside => {}
            CoordPos::OnBoundary => {
                *boundary_count += 1;
            }
            CoordPos::Inside => {
                for hole in self.interiors() {
                    match coord_pos_relative_to_ring(*coord, hole) {
                        CoordPos::Outside => {}
                        CoordPos::OnBoundary => {
                            *boundary_count += 1;
                            return;
                        }
                        CoordPos::Inside => {
                            return;
                        }
                    }
                }
                // the coord is *outside* the interior holes, so it's *inside* the polygon
                *is_inside = true;
            }
        }
    }
}

impl<T, Z> CoordinatePosition for MultiLineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for line_string in &self.0 {
            line_string.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

impl<T, Z> CoordinatePosition for MultiPolygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;
    fn calculate_coordinate_position(
        &self,
        coord: &Coordinate<T, Z>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        for polygon in &self.0 {
            polygon.calculate_coordinate_position(coord, is_inside, boundary_count);
        }
    }
}

pub fn coord_pos_relative_to_ring<T, Z>(
    coord: Coordinate<T, Z>,
    linestring: &LineString<T, Z>,
) -> CoordPos
where
    T: GeoNum,
    Z: GeoNum,
{
    debug_assert!(linestring.is_closed());

    // LineString without points
    if linestring.0.is_empty() {
        return CoordPos::Outside;
    }
    if linestring.0.len() == 1 {
        // If LineString has one point, it will not generate
        // any lines.  So, we handle this edge case separately.
        return if coord == linestring.0[0] {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    // Use winding number algorithm with on boundary short-cicuit
    // See: https://en.wikipedia.org/wiki/Point_in_polygon#Winding_number_algorithm
    let mut winding_number = 0;
    for line in linestring.lines() {
        // Edge Crossing Rules:
        //   1. an upward edge includes its starting endpoint, and excludes its final endpoint;
        //   2. a downward edge excludes its starting endpoint, and includes its final endpoint;
        //   3. horizontal edges are excluded
        //   4. the edge-ray intersection point must be strictly right of the coord.
        if line.start.y <= coord.y {
            if line.end.y >= coord.y {
                let o = RobustKernel::orient(line.start, line.end, coord, None);
                if o == Orientation::CounterClockwise && line.end.y != coord.y {
                    winding_number += 1
                } else if o == Orientation::Collinear
                    && value_in_between(coord.x, line.start.x, line.end.x)
                {
                    return CoordPos::OnBoundary;
                }
            };
        } else if line.end.y <= coord.y {
            let o = RobustKernel::orient(line.start, line.end, coord, None);
            if o == Orientation::Clockwise {
                winding_number -= 1
            } else if o == Orientation::Collinear
                && value_in_between(coord.x, line.start.x, line.end.x)
            {
                return CoordPos::OnBoundary;
            }
        }
    }
    if winding_number == 0 {
        CoordPos::Outside
    } else {
        CoordPos::Inside
    }
}

#[cfg(test)]
mod test {
    use crate::{
        algorithm::coordinate_position::{CoordPos, CoordinatePosition},
        point,
        types::{coordinate::Coordinate2D, line::Line2D, rect::Rect2D},
    };

    #[test]
    fn test_simple_line() {
        let line = Line2D::new(point!(x: 0.0, y: 0.0), point!(x: 10.0, y: 10.0));

        let start = Coordinate2D::new_(0.0, 0.0);
        assert_eq!(line.coordinate_position(&start), CoordPos::OnBoundary);

        let end = Coordinate2D::new_(10.0, 10.0);
        assert_eq!(line.coordinate_position(&end), CoordPos::OnBoundary);

        let interior = Coordinate2D::new_(5.0, 5.0);
        assert_eq!(line.coordinate_position(&interior), CoordPos::Inside);

        let outside = Coordinate2D::new_(6.0, 5.0);
        assert_eq!(line.coordinate_position(&outside), CoordPos::Outside);
    }

    #[test]
    fn test_rect() {
        let rect = Rect2D::new((0.0, 0.0), (10.0, 10.0));
        assert_eq!(
            rect.coordinate_position(&Coordinate2D::new_(5.0, 5.0)),
            CoordPos::Inside
        );
        assert_eq!(
            rect.coordinate_position(&Coordinate2D::new_(0.0, 5.0)),
            CoordPos::OnBoundary
        );
        assert_eq!(
            rect.coordinate_position(&Coordinate2D::new_(15.0, 15.0)),
            CoordPos::Outside
        );
    }
}

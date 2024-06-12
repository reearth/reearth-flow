use crate::{
    algorithm::{
        kernels::{Orientation, RobustKernel},
        GeoNum,
    },
    types::{coordinate::Coordinate, line::Line, point::Point, triangle::Triangle},
};

use super::{point_in_rect, Intersects};

impl<T, Z> Intersects<Coordinate<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, rhs: &Coordinate<T, Z>) -> bool {
        RobustKernel::orient(self.start, self.end, *rhs, None) == Orientation::Collinear
            && point_in_rect(*rhs, self.start, self.end)
    }
}
symmetric_intersects_impl!(Coordinate<T, Z>, Line<T, Z>);
symmetric_intersects_impl!(Line<T, Z>, Point<T, Z>);

impl<T, Z> Intersects<Line<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, line: &Line<T, Z>) -> bool {
        // Special case: self is equiv. to a point.
        if self.start == self.end {
            return line.intersects(&self.start);
        }
        let check_1_1 = RobustKernel::orient(self.start, self.end, line.start, None);
        let check_1_2 = RobustKernel::orient(self.start, self.end, line.end, None);

        if check_1_1 != check_1_2 {
            let check_2_1 = RobustKernel::orient(line.start, line.end, self.start, None);
            let check_2_2 = RobustKernel::orient(line.start, line.end, self.end, None);
            check_2_1 != check_2_2
        } else if check_1_1 == Orientation::Collinear {
            point_in_rect(line.start, self.start, self.end)
                || point_in_rect(line.end, self.start, self.end)
                || point_in_rect(self.end, line.start, line.end)
                || point_in_rect(self.end, line.start, line.end)
        } else {
            false
        }
    }
}

impl<T, Z> Intersects<Triangle<T, Z>> for Line<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T, Z>) -> bool {
        self.intersects(&rhs.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T, Z>, Line<T, Z>);

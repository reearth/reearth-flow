use crate::{
    algorithm::{
        kernels::{Orientation, RobustKernel},
        GeoNum,
    },
    types::{coordinate::Coordinate, point::Point, triangle::Triangle},
};

use super::Contains;

impl<T, Z> Contains<Coordinate<T, Z>> for Triangle<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        // leverageing robust predicates
        self.to_lines()
            .map(|l| RobustKernel::orient(l.start, l.end, *coord, None))
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear)
    }
}

impl<T, Z> Contains<Point<T, Z>> for Triangle<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, point: &Point<T, Z>) -> bool {
        self.contains(&point.0)
    }
}

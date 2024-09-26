use std::cmp::Ordering;

use crate::types::{coordinate::Coordinate, coordnum::CoordNum};
use num_traits::{Float, NumCast};
use robust::orient3d;
use robust::Coord3D;
use robust::{orient2d, Coord};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Orientation {
    CounterClockwise,
    Clockwise,
    Collinear,
}

impl Orientation {
    #[inline]
    pub fn as_ordering(&self) -> Ordering {
        match self {
            Orientation::CounterClockwise => Ordering::Less,
            Orientation::Clockwise => Ordering::Greater,
            Orientation::Collinear => Ordering::Equal,
        }
    }
}

#[derive(Default, Debug)]
pub struct RobustKernel;

impl RobustKernel {
    pub fn orient<T: CoordNum + Float, Z: CoordNum + Float>(
        p: Coordinate<T, Z>,
        q: Coordinate<T, Z>,
        r: Coordinate<T, Z>,
        d: Option<Coordinate<T, Z>>,
    ) -> Orientation {
        if let Some(pd) = d {
            Self::orient3d(p, q, r, pd)
        } else {
            Self::orient2d(p, q, r)
        }
    }
    fn orient2d<T: CoordNum + Float, Z: CoordNum + Float>(
        p: Coordinate<T, Z>,
        q: Coordinate<T, Z>,
        r: Coordinate<T, Z>,
    ) -> Orientation {
        let orientation = orient2d(
            Coord {
                x: <f64 as NumCast>::from(p.x).unwrap(),
                y: <f64 as NumCast>::from(p.y).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(q.x).unwrap(),
                y: <f64 as NumCast>::from(q.y).unwrap(),
            },
            Coord {
                x: <f64 as NumCast>::from(r.x).unwrap(),
                y: <f64 as NumCast>::from(r.y).unwrap(),
            },
        );

        if orientation < 0. {
            Orientation::Clockwise
        } else if orientation > 0. {
            Orientation::CounterClockwise
        } else {
            Orientation::Collinear
        }
    }
    fn orient3d<T: CoordNum + Float, Z: CoordNum + Float>(
        p: Coordinate<T, Z>,
        q: Coordinate<T, Z>,
        r: Coordinate<T, Z>,
        d: Coordinate<T, Z>,
    ) -> Orientation {
        let orientation = orient3d(
            Coord3D {
                x: <f64 as NumCast>::from(p.x).unwrap(),
                y: <f64 as NumCast>::from(p.y).unwrap(),
                z: <f64 as NumCast>::from(p.z).unwrap(),
            },
            Coord3D {
                x: <f64 as NumCast>::from(q.x).unwrap(),
                y: <f64 as NumCast>::from(q.y).unwrap(),
                z: <f64 as NumCast>::from(q.z).unwrap(),
            },
            Coord3D {
                x: <f64 as NumCast>::from(r.x).unwrap(),
                y: <f64 as NumCast>::from(r.y).unwrap(),
                z: <f64 as NumCast>::from(r.z).unwrap(),
            },
            Coord3D {
                x: <f64 as NumCast>::from(d.x).unwrap(),
                y: <f64 as NumCast>::from(d.y).unwrap(),
                z: <f64 as NumCast>::from(d.z).unwrap(),
            },
        );

        if orientation < 0. {
            Orientation::Clockwise
        } else if orientation > 0. {
            Orientation::CounterClockwise
        } else {
            Orientation::Collinear
        }
    }

    pub fn square_euclidean_distance<T: CoordNum + Float, Z: CoordNum + Float>(
        p: Coordinate<T, Z>,
        q: Coordinate<T, Z>,
    ) -> T {
        (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)
    }
}

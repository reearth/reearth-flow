use serde::{Deserialize, Serialize};

use super::coordinate::Coordinate;
use super::coordnum::CoordNum;
use super::no_value::NoValue;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash, Default)]
pub struct Face<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<Coordinate<T, Z>>);

pub type Face2D<T> = Face<T, NoValue>;
pub type Face3D<T> = Face<T, T>;

impl<T: CoordNum, Z: CoordNum> Face<T, Z> {
    pub fn new(points: Vec<Coordinate<T, Z>>) -> Self {
        Self(points)
    }
}

impl From<Face3D<f64>> for Face2D<f64> {
    fn from(p: Face3D<f64>) -> Face2D<f64> {
        Face2D::new(p.0.into_iter().map(|c| c.into()).collect())
    }
}

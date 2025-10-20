use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::types::line_string::LineString;

use super::coordinate::Coordinate;
use super::coordnum::CoordNum;
use super::no_value::NoValue;
use super::traits::Elevation;

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

impl<T: CoordNum, Z: CoordNum> From<LineString<T, Z>> for Face<T, Z> {
    fn from(ls: LineString<T, Z>) -> Self {
        Face::new(ls.0)
    }
}

impl<T, Z> Elevation for Face<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.0.iter().all(|c| c.is_elevation_zero())
    }
}

impl Face3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.0.iter_mut().for_each(|c| c.transform_inplace(jgd2wgs));
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.0.iter_mut().for_each(|c| c.transform_offset(x, y, z));
    }
}

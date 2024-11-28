use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use super::coordnum::CoordNum;
use super::face::Face;
use super::no_value::NoValue;
use super::traits::Elevation;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash, Default)]
pub struct Solid<T: CoordNum = f64, Z: CoordNum = f64> {
    pub bottom: Vec<Face<T, Z>>,
    pub top: Vec<Face<T, Z>>,
    pub sides: Vec<Face<T, Z>>,
}

pub type Solid2D<T> = Solid<T, NoValue>;
pub type Solid3D<T> = Solid<T, T>;

impl<T: CoordNum, Z: CoordNum> Solid<T, Z> {
    pub fn new(bottom: Vec<Face<T, Z>>, top: Vec<Face<T, Z>>, sides: Vec<Face<T, Z>>) -> Self {
        Self { bottom, top, sides }
    }

    pub fn all_faces(&self) -> Vec<&Face<T, Z>> {
        self.bottom
            .iter()
            .chain(self.top.iter())
            .chain(self.sides.iter())
            .collect()
    }
}

impl From<Solid3D<f64>> for Solid2D<f64> {
    fn from(p: Solid3D<f64>) -> Solid2D<f64> {
        Solid2D::new(
            p.bottom.into_iter().map(|c| c.into()).collect(),
            p.top.into_iter().map(|c| c.into()).collect(),
            p.sides.into_iter().map(|c| c.into()).collect(),
        )
    }
}

impl<T, Z> Elevation for Solid<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.bottom.iter().all(|f| f.is_elevation_zero())
            && self.top.iter().all(|f| f.is_elevation_zero())
            && self.sides.iter().all(|f| f.is_elevation_zero())
    }
}

impl Solid3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.bottom
            .iter_mut()
            .for_each(|f| f.transform_inplace(jgd2wgs));
        self.top
            .iter_mut()
            .for_each(|f| f.transform_inplace(jgd2wgs));
        self.sides
            .iter_mut()
            .for_each(|f| f.transform_inplace(jgd2wgs));
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.bottom
            .iter_mut()
            .for_each(|f| f.transform_offset(x, y, z));
        self.top
            .iter_mut()
            .for_each(|f| f.transform_offset(x, y, z));
        self.sides
            .iter_mut()
            .for_each(|f| f.transform_offset(x, y, z));
    }
}

use serde::{Deserialize, Serialize};

use super::coordnum::CoordNum;
use super::face::Face;
use super::no_value::NoValue;

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
}

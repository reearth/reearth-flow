use super::coordnum::CoordNum;
use super::no_value::NoValue;

pub trait Surface<T: CoordNum = f64, Z: CoordNum = NoValue> {}

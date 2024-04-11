use num_traits::{Float, Num, NumCast};
use std::fmt::Debug;

pub trait CoordNum: Num + Copy + NumCast + PartialOrd + Debug {}
impl<T: Num + Copy + NumCast + PartialOrd + Debug> CoordNum for T {}

pub trait CoordFloat: CoordNum + Float {}
impl<T: CoordNum + Float> CoordFloat for T {}

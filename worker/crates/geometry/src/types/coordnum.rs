use num_traits::{Float, Num, NumCast};
use std::fmt::Debug;

pub trait CoordNum:
    Num
    + Copy
    + NumCast
    + PartialOrd
    + Debug
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + Default
{
}
impl<
        T: Num
            + Copy
            + NumCast
            + PartialOrd
            + Debug
            + std::ops::Sub<Output = Self>
            + std::ops::Mul<Output = Self>
            + Default,
    > CoordNum for T
{
}

pub trait CoordFloat: CoordNum + Float {}
impl<T: CoordNum + Float> CoordFloat for T {}

pub trait CoordNumT: CoordNum {}

impl CoordNumT for i32 {}
impl CoordNumT for i64 {}
impl CoordNumT for f32 {}
impl CoordNumT for f64 {}

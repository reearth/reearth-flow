use std::iter::Sum;

use crate::{
    types::{
        coordnum::CoordFloat, line::Line, line_string::LineString,
        multi_line_string::MultiLineString,
    },
    utils::line_euclidean_length,
};

/// Calculation of the length
pub trait EuclideanLength<T, RHS = Self> {
    fn euclidean_length(&self) -> T;
}

impl<T, Z> EuclideanLength<T> for Line<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    fn euclidean_length(&self) -> T {
        line_euclidean_length(*self)
    }
}

impl<T, Z> EuclideanLength<T> for LineString<T, Z>
where
    T: CoordFloat + Sum,
    Z: CoordFloat + Sum,
{
    fn euclidean_length(&self) -> T {
        self.lines().map(|line| line.euclidean_length()).sum()
    }
}

impl<T, Z> EuclideanLength<T, Z> for MultiLineString<T, Z>
where
    T: CoordFloat + Sum,
    Z: CoordFloat + Sum,
{
    fn euclidean_length(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, line| total + line.euclidean_length())
    }
}

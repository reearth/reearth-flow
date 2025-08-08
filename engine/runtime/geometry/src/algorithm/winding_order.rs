use std::iter::Rev;

use crate::types::{
    coordnum::CoordNum,
    line_string::{LineString, PointsIter},
    point::Point,
};

use super::{
    coords_iter::CoordsIter,
    kernels::{Orientation, RobustKernel},
    utils::{least_index, EitherIter},
    GeoNum,
};

#[allow(missing_debug_implementations)]
pub struct Points<'a, T: CoordNum + 'a, Z: CoordNum + 'a>(
    pub(crate) EitherIter<PointsIter<'a, T, Z>, Rev<PointsIter<'a, T, Z>>>,
);

impl<T, Z> Iterator for Points<'_, T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    type Item = Point<T, Z>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T, Z> ExactSizeIterator for Points<'_, T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// How a linestring is wound, clockwise or counter-clockwise
#[derive(PartialEq, Clone, Debug, Eq, Copy, Hash)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
    None,
}

pub trait Winding {
    type ScalarXY: CoordNum;
    type ScalarZ: CoordNum;

    fn winding_order(&self) -> Option<WindingOrder>;

    /// True iff this is wound clockwise
    fn is_cw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::Clockwise)
    }

    /// True iff this is wound counterclockwise
    fn is_ccw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::CounterClockwise)
    }

    fn points_cw(&self) -> Points<'_, Self::ScalarXY, Self::ScalarZ>;

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The object isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    fn points_ccw(&self) -> Points<'_, Self::ScalarXY, Self::ScalarZ>;

    /// Change this object's points so they are in clockwise winding order
    fn make_cw_winding(&mut self);

    /// Change this line's points so they are in counterclockwise winding order
    fn make_ccw_winding(&mut self);

    /// Return a clone of this object, but in the specified winding order
    fn clone_to_winding_order(&self, winding_order: WindingOrder) -> Self
    where
        Self: Sized + Clone,
    {
        let mut new: Self = self.clone();
        new.make_winding_order(winding_order);
        new
    }

    /// Change the winding order so that it is in this winding order
    fn make_winding_order(&mut self, winding_order: WindingOrder) {
        match winding_order {
            WindingOrder::Clockwise => self.make_cw_winding(),
            WindingOrder::CounterClockwise => self.make_ccw_winding(),
            WindingOrder::None => {}
        }
    }
}

impl<T, Z> Winding for LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    type ScalarXY = T;
    type ScalarZ = Z;

    fn winding_order(&self) -> Option<WindingOrder> {
        // If linestring has at most 3 coords, it is either
        // not closed, or is at most two distinct points.
        // Either way, the WindingOrder is unspecified.
        if self.coords_count() < 4 || !self.is_closed() {
            return None;
        }

        let increment = |x: &mut usize| {
            *x += 1;
            if *x >= self.coords_count() {
                *x = 0;
            }
        };

        let decrement = |x: &mut usize| {
            if *x == 0 {
                *x = self.coords_count() - 1;
            } else {
                *x -= 1;
            }
        };

        let i = least_index(&self.0);

        let mut next = i;
        increment(&mut next);
        while self.0[next] == self.0[i] {
            if next == i {
                // We've looped too much. There aren't
                // enough unique coords to compute orientation.
                return None;
            }
            increment(&mut next);
        }

        let mut prev = i;
        decrement(&mut prev);
        while self.0[prev] == self.0[i] {
            // Note: we don't need to check if prev == i as
            // the previous loop succeeded, and so we have
            // at least two distinct elements in the list
            decrement(&mut prev);
        }

        match RobustKernel::orient(self.0[prev], self.0[i], self.0[next], None) {
            Orientation::CounterClockwise => Some(WindingOrder::CounterClockwise),
            Orientation::Clockwise => Some(WindingOrder::Clockwise),
            _ => None,
        }
    }

    fn points_cw(&self) -> Points<'_, Self::ScalarXY, Self::ScalarZ> {
        match self.winding_order() {
            Some(WindingOrder::CounterClockwise) => Points(EitherIter::B(self.points().rev())),
            _ => Points(EitherIter::A(self.points())),
        }
    }

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    fn points_ccw(&self) -> Points<'_, Self::ScalarXY, Self::ScalarZ> {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => Points(EitherIter::B(self.points().rev())),
            _ => Points(EitherIter::A(self.points())),
        }
    }

    /// Change this line's points so they are in clockwise winding order
    fn make_cw_winding(&mut self) {
        if let Some(WindingOrder::CounterClockwise) = self.winding_order() {
            self.0.reverse();
        }
    }

    /// Change this line's points so they are in counterclockwise winding order
    fn make_ccw_winding(&mut self) {
        if let Some(WindingOrder::Clockwise) = self.winding_order() {
            self.0.reverse();
        }
    }
}

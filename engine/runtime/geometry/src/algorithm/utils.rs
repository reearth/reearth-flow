use num_traits::FromPrimitive;

use std::cmp::Ordering;

use crate::types::{
    coordinate::Coordinate,
    coordnum::{CoordFloat, CoordNum},
};

use super::bounding_rect::*;
use super::intersects::*;

pub fn partition_slice<T, P>(data: &mut [T], predicate: P) -> (&mut [T], &mut [T])
where
    P: Fn(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return (&mut [], &mut []);
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return data.split_at_mut(l);
        }
        data.swap(l, r);
    }
}

pub enum EitherIter<I1, I2> {
    A(I1),
    B(I2),
}

impl<I1, I2> ExactSizeIterator for EitherIter<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        match self {
            EitherIter::A(i1) => i1.len(),
            EitherIter::B(i2) => i2.len(),
        }
    }
}

impl<T, I1, I2> Iterator for EitherIter<I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::A(iter) => iter.next(),
            EitherIter::B(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            EitherIter::A(iter) => iter.size_hint(),
            EitherIter::B(iter) => iter.size_hint(),
        }
    }
}

// The Rust standard library has `max` for `Ord`, but not for `PartialOrd`
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

// The Rust standard library has `min` for `Ord`, but not for `PartialOrd`
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn lex_cmp<T: CoordNum, Z: CoordNum>(p: &Coordinate<T, Z>, q: &Coordinate<T, Z>) -> Ordering {
    p.x.partial_cmp(&q.x)
        .unwrap()
        .then(p.y.partial_cmp(&q.y).unwrap())
}

/// Compute index of the least point in slice. Comparison is
/// done using [`lex_cmp`].
///
/// Should only be called on a non-empty slice with no `nan`
/// coordinates.
pub fn least_index<T: CoordNum, Z: CoordNum>(pts: &[Coordinate<T, Z>]) -> usize {
    pts.iter()
        .enumerate()
        .min_by(|(_, p), (_, q)| lex_cmp(p, q))
        .unwrap()
        .0
}

/// Compute index of the lexicographically least _and_ the
/// greatest coordinate in one pass.
///
/// Should only be called on a non-empty slice with no `nan`
/// coordinates.
pub fn least_and_greatest_index<T: CoordNum, Z: CoordNum>(
    pts: &[Coordinate<T, Z>],
) -> (usize, usize) {
    assert_ne!(pts.len(), 0);
    let (min, max) = pts
        .iter()
        .enumerate()
        .fold((None, None), |(min, max), (idx, p)| {
            (
                if let Some((midx, min)) = min {
                    if lex_cmp(p, min) == Ordering::Less {
                        Some((idx, p))
                    } else {
                        Some((midx, min))
                    }
                } else {
                    Some((idx, p))
                },
                if let Some((midx, max)) = max {
                    if lex_cmp(p, max) == Ordering::Greater {
                        Some((idx, p))
                    } else {
                        Some((midx, max))
                    }
                } else {
                    Some((idx, p))
                },
            )
        });
    (min.unwrap().0, max.unwrap().0)
}

/// Normalize a longitude to coordinate to ensure it's within [-180,180]
pub fn normalize_longitude<T: CoordFloat + FromPrimitive>(coord: T) -> T {
    let one_eighty = T::from(180.0f64).unwrap();
    let three_sixty = T::from(360.0f64).unwrap();
    let five_forty = T::from(540.0f64).unwrap();

    ((coord + five_forty) % three_sixty) - one_eighty
}

pub fn has_disjoint_bboxes<T, Z, A, B>(a: &A, b: &B) -> bool
where
    T: CoordNum,
    Z: CoordNum,
    A: BoundingRect<T, Z>,
    B: BoundingRect<T, Z>,
{
    let mut disjoint_bbox = false;
    if let Some(a_bbox) = a.bounding_rect().into() {
        if let Some(b_bbox) = b.bounding_rect().into() {
            if !a_bbox.intersects(&b_bbox) {
                disjoint_bbox = true;
            }
        }
    }
    disjoint_bbox
}

#[cfg(test)]
mod test {
    use super::{partial_max, partial_min};

    #[test]
    fn test_partial_max() {
        assert_eq!(5, partial_max(5, 4));
        assert_eq!(5, partial_max(5, 5));
    }

    #[test]
    fn test_partial_min() {
        assert_eq!(4, partial_min(5, 4));
        assert_eq!(4, partial_min(4, 4));
    }
}

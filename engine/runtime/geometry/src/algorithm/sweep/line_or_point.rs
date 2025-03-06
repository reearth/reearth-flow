use std::{cmp::Ordering, ops::Deref};

use crate::{
    algorithm::{
        intersects::Intersects,
        kernels::{Orientation, RobustKernel},
        line_intersection::{line_intersection, LineIntersection},
        GeoFloat, GeoNum,
    },
    types::{coordinate::Coordinate, line::Line},
};

use super::SweepPoint;
/// Either a line segment or a point.
///
/// The coordinates are ordered (see [`SweepPoint`]) and a line
/// segment must have distinct points (use the `Point` variant if the
/// coordinates are the equal).
#[derive(Clone, Copy)]
pub enum LineOrPoint<T: GeoNum, Z: GeoNum> {
    Point(SweepPoint<T, Z>),
    Line {
        left: SweepPoint<T, Z>,
        right: SweepPoint<T, Z>,
    },
}

impl<T: GeoNum, Z: GeoNum> std::fmt::Debug for LineOrPoint<T, Z> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineOrPoint::Point(p) => f.debug_tuple("Pt").field(&p.x_y()).finish(),
            LineOrPoint::Line { left, right } => f
                .debug_tuple("LPt")
                .field(&left.x_y())
                .field(&right.x_y())
                .finish(),
        }
    }
}

impl<T: GeoNum, Z: GeoNum> From<SweepPoint<T, Z>> for LineOrPoint<T, Z> {
    fn from(pt: SweepPoint<T, Z>) -> Self {
        Self::Point(pt)
    }
}

impl<T: GeoNum, Z: GeoNum> From<(SweepPoint<T, Z>, SweepPoint<T, Z>)> for LineOrPoint<T, Z> {
    fn from((start, end): (SweepPoint<T, Z>, SweepPoint<T, Z>)) -> Self {
        match start.cmp(&end) {
            Ordering::Less => Self::Line {
                left: start,
                right: end,
            },
            Ordering::Equal => Self::Point(start),
            Ordering::Greater => Self::Line {
                left: end,
                right: start,
            },
        }
    }
}

/// Convert from a [`Line`] ensuring end point ordering.
impl<T: GeoNum, Z: GeoNum> From<Line<T, Z>> for LineOrPoint<T, Z> {
    fn from(l: Line<T, Z>) -> Self {
        let start: SweepPoint<T, Z> = l.start.into();
        let end = l.end.into();
        (start, end).into()
    }
}

/// Convert from a [`Coord`]
impl<T: GeoNum, Z: GeoNum> From<Coordinate<T, Z>> for LineOrPoint<T, Z> {
    fn from(c: Coordinate<T, Z>) -> Self {
        Self::Point(c.into())
    }
}

impl<T: GeoNum, Z: GeoNum> LineOrPoint<T, Z> {
    /// Checks if the variant is a line.
    #[inline]
    pub fn is_line(&self) -> bool {
        matches!(self, Self::Line { .. })
    }

    /// Return a [`Line`] representation of self.
    #[inline]
    pub fn line(&self) -> Line<T, Z> {
        match self {
            LineOrPoint::Point(p) => Line::new_(**p, **p),
            LineOrPoint::Line { left, right } => Line::new_(**left, **right),
        }
    }

    #[inline]
    pub fn left(&self) -> SweepPoint<T, Z> {
        match self {
            LineOrPoint::Point(p) => *p,
            LineOrPoint::Line { left, .. } => *left,
        }
    }

    #[inline]
    pub fn right(&self) -> SweepPoint<T, Z> {
        match self {
            LineOrPoint::Point(p) => *p,
            LineOrPoint::Line { right, .. } => *right,
        }
    }

    #[cfg(test)]
    pub fn coords_equal(&self, other: &LineOrPoint<T, Z>) -> bool {
        self.is_line() == other.is_line() && self.end_points() == other.end_points()
    }

    #[inline]
    pub fn end_points(&self) -> (SweepPoint<T, Z>, SweepPoint<T, Z>) {
        match self {
            LineOrPoint::Point(p) => (*p, *p),
            LineOrPoint::Line { left, right } => (*left, *right),
        }
    }

    pub fn new(left: SweepPoint<T, Z>, right: SweepPoint<T, Z>) -> Self {
        if left == right {
            Self::Point(left)
        } else {
            Self::Line { left, right }
        }
    }

    pub fn orient2d(&self, other: Coordinate<T, Z>) -> Orientation {
        let (left, right) = match self {
            LineOrPoint::Point(p) => (**p, **p),
            LineOrPoint::Line { left, right } => (**left, **right),
        };
        RobustKernel::orient(left, right, other, None)
    }
}

/// Equality based on ordering defined for segments as per algorithm.
impl<T: GeoNum, Z: GeoNum> PartialEq for LineOrPoint<T, Z> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl<T: GeoNum, Z: GeoNum> Eq for LineOrPoint<T, Z> {}

impl<T: GeoNum, Z: GeoNum> PartialOrd for LineOrPoint<T, Z> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoNum, Z: GeoNum> Ord for LineOrPoint<T, Z> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (LineOrPoint::Point(p), LineOrPoint::Point(o)) => {
                if p == o {
                    Ordering::Equal
                } else {
                    p.cmp(o)
                }
            }
            (LineOrPoint::Point(_), LineOrPoint::Line { .. }) => other.cmp(self).reverse(),
            (LineOrPoint::Line { left, right }, LineOrPoint::Point(p)) => {
                if p > right || left > p {
                    //return None;
                    return p.cmp(left).then_with(|| p.cmp(right));
                }
                RobustKernel::orient(**left, **right, **p, None)
                    .as_ordering()
                    .then(Ordering::Greater)
            }
            (
                LineOrPoint::Line {
                    left: left_a,
                    right: right_a,
                },
                LineOrPoint::Line {
                    left: left_b,
                    right: right_b,
                },
            ) => {
                if left_a > left_b {
                    //return other.partial_cmp(self).map(Ordering::reverse);
                    return other.cmp(self).reverse();
                }
                if left_a >= right_b || left_b >= right_a {
                    //return None;
                    //unreachable!()
                    return left_a.cmp(left_b).then_with(|| right_a.cmp(right_b));
                }

                // Assertion: p1 <= p2
                // Assertion: pi < q_j
                RobustKernel::orient(**left_a, **right_a, **left_b, None)
                    .as_ordering()
                    .then_with(|| {
                        RobustKernel::orient(**left_a, **right_a, **right_b, None).as_ordering()
                    })
            }
        }
    }
}

impl<T: GeoFloat, Z: GeoFloat> LineOrPoint<T, Z> {
    /// Intersect a line with self and return a point, a overlapping segment or `None`.
    ///
    /// The `other` argument must be a line variant (debug builds will panic otherwise).
    pub fn intersect_line(&self, other: &Self) -> Option<Self> {
        debug_assert!(other.is_line(), "tried to intersect with a point variant!");

        let line = other.line();
        match self {
            LineOrPoint::Point(p) => {
                if line.intersects(&**p) {
                    Some(*self)
                } else {
                    None
                }
            }
            LineOrPoint::Line { left, right } => {
                line_intersection(self.line(), line).map(|l| match l {
                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    } => {
                        let mut pt = intersection;
                        if is_proper && (&pt == left.deref()) {
                            if left.x == right.x {
                                pt.y = pt.y.next_after(T::infinity());
                            } else {
                                pt.x = pt.x.next_after(T::infinity());
                            }
                        }
                        pt.into()
                    }
                    LineIntersection::Collinear { intersection } => intersection.into(),
                })
            }
        }
    }

    pub fn intersect_line_ordered(&self, other: &Self) -> Option<Self> {
        let ord = self.partial_cmp(other);
        match self.intersect_line(other) {
            Some(Self::Point(p)) => {
                // NOTE: A key issue with using non-exact numbers (f64, etc.) in
                // this algo. is that line-intersection may return
                // counter-intuitive points.
                //
                // Specifically, this causes two issues:
                //
                // 1. The point of intersection r lies between the end-points in
                // the lexicographic ordering. However, with finite repr., the
                // line (1, 1) - (1 + eps, -1), where eps is ulp(1), does not
                // admit r that lies between the end-points. Further, the
                // end-points may be a very bad approximation to the actual
                // intersection points (eg. intersect with x-axis).
                //
                // We detect and force r to be greater than both end-points; the
                // other case is not easy to handle as the sweep has already
                // progressed to a p strictly > r already.
                //
                // 2. The more severe issue is that in general r may not lie
                // exactly on the line. Thus, with the segment stored on the
                // active-segments tree (B-Tree / Splay), this may in adverse
                // cases, cause the ordering between the segments to be
                // incorrect, hence invalidating the segments. This is not easy
                // to correct without a intrusive data-structure built
                // specifically for this algo., that can track the neighbors of
                // tree-nodes, and fix / report this issue. The crate
                // `btree-slab` seems like a great starting point.
                let (mut x, y, z) = p.x_y_z();

                let c = self.left();
                if x == c.x && y < c.y {
                    x = x.next_after(T::infinity());
                }

                let p = Coordinate::new__(x, y, z).into();
                if let Some(ord) = ord {
                    let l1 = LineOrPoint::from((self.left(), p));
                    let l2 = LineOrPoint::from((other.left(), p));
                    let cmp = l1.partial_cmp(&l2).unwrap();
                    if l1.is_line() && l2.is_line() && cmp.then(ord) != ord {
                        // RM: This is a complicated intersection that is changing the ordering.
                        // Heuristic: approximate with a trivial intersection point that preserves the topology.
                        return Some(if self.left() > other.left() {
                            self.left().into()
                        } else {
                            other.left().into()
                        });
                    }
                }
                Some(Self::Point(p))
            }
            e => e,
        }
    }
}

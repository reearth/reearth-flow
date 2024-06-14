use std::{cmp::Ordering, fmt::Debug};

use crate::algorithm::GeoFloat;

use super::{im_segment::IMSegment, Cross, LineOrPoint};

/// A segment of input [`LineOrPoint`] generated during the sweep.
#[derive(Clone)]
pub(super) struct Segment<C: Cross> {
    pub(super) geom: LineOrPoint<C::ScalarXY, C::ScalarZ>,
    pub(super) cross: C,
    pub(super) first_segment: bool,
    pub(super) left_event_done: bool,
    pub(super) overlapping: Option<IMSegment<C>>,
    pub(super) is_overlapping: bool,
}

impl<C: Cross> Segment<C> {
    pub fn new(cross: C, geom: Option<LineOrPoint<C::ScalarXY, C::ScalarZ>>) -> Self {
        let first_segment = geom.is_none();
        let geom = geom.unwrap_or_else(|| cross.line());
        Self {
            geom,
            cross,
            first_segment,
            left_event_done: false,
            overlapping: None,
            is_overlapping: false,
        }
    }

    /// Split a line segment into pieces at points of intersection.
    ///
    /// The initial segment is mutated in place, and extra-segment(s) are
    /// returned if any. Assume exact arithmetic, the ordering of self should
    /// remain the same among active segments. However, with finite-precision,
    /// this may not be the case.
    pub fn adjust_for_intersection(
        &mut self,
        intersection: LineOrPoint<C::ScalarXY, C::ScalarZ>,
    ) -> SplitSegments<C::ScalarXY, C::ScalarZ> {
        let (p, q) = self.geom.end_points();

        if !intersection.is_line() {
            // Handle point intersection
            let r = intersection.left();
            debug_assert!(
                p <= r,
                "intersection point was not ordered within the line: {p:?} <= {r:?} <=> {q:?}",
            );
            if p == r || q == r {
                // If the intersection is at the end point, the
                // segment doesn't need to be split.
                SplitSegments::Unchanged { overlap: false }
            } else {
                // Otherwise, split it. Mutate `self` to be the
                // first part, and return the second part.
                self.geom = (p, r).into();
                // self.first_segment = false;
                SplitSegments::SplitOnce {
                    overlap: None,
                    right: (r, q).into(),
                }
            }
        } else {
            let (r1, r2) = intersection.end_points();
            if p == r1 {
                if r2 == q {
                    // The whole segment overlaps.
                    SplitSegments::Unchanged { overlap: true }
                } else {
                    self.geom = (p, r2).into();
                    // self.first_segment = false;
                    SplitSegments::SplitOnce {
                        overlap: Some(false),
                        right: (r2, q).into(),
                    }
                }
            } else if r2 == q {
                self.geom = (p, r1).into();
                // self.first_segment = false;
                SplitSegments::SplitOnce {
                    overlap: Some(true),
                    right: (r1, q).into(),
                }
            } else {
                self.geom = (p, r1).into();
                // self.first_segment = false;
                SplitSegments::SplitTwice {
                    right: (r2, q).into(),
                }
            }
        }
    }
}

/// A more concise debug impl.
impl<C: Cross> Debug for Segment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment{{ {geom:?}\n\tof {c:?}\n\t{first} [{has}/{ovl}] }}",
            c = self.cross,
            geom = self.geom,
            first = if self.first_segment { "[1st]" } else { "" },
            has = if self.overlapping.is_some() {
                "HAS"
            } else {
                "NON"
            },
            ovl = if self.is_overlapping { "OVL" } else { "NON" },
        )
    }
}

/// Partial equality based on key.
///
/// This is consistent with the `PartialOrd` impl.
impl<C: Cross> PartialEq for Segment<C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// Partial ordering defined as per algorithm.
///
/// This is requires the same pre-conditions as for [`LineOrPoint`].
impl<C: Cross> PartialOrd for Segment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.geom.partial_cmp(&other.geom)
    }
}

/// Stores the type of split and extra geometries from adjusting a
/// segment for intersection.
#[derive(Debug)]
pub(super) enum SplitSegments<T: GeoFloat, Z: GeoFloat> {
    Unchanged {
        overlap: bool,
    },
    SplitOnce {
        overlap: Option<bool>,
        right: LineOrPoint<T, Z>,
    },
    SplitTwice {
        right: LineOrPoint<T, Z>,
    },
}

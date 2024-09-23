use std::cmp::Ordering;

use crate::{
    algorithm::line_intersection::{line_intersection, LineIntersection},
    types::coordinate::Coordinate,
};

use super::{im_segment::IMSegment, proc::Sweep, Cross, EventType, LineOrPoint};

/// A segment of a input [`Cross`] type.
///
/// This type is used to convey the part of the input geometry that is
/// intersecting at a given intersection. This is returned by the
/// [`CrossingsIter::intersections`] method.
#[derive(Debug, Clone)]
pub(crate) struct Crossing<C: Cross> {
    /// The input associated with this segment.
    pub cross: C,

    /// The geometry of this segment.
    ///
    /// This is a part of the input `crossable` geometry and either
    /// starts or ends at the intersection point last yielded by
    /// [`CrossingsIter`]. If it ends at the point (`at_left` is
    /// `false`), then it is guaranteed to not contain any other
    /// intersection point in its interior.
    #[allow(unused)]
    pub line: LineOrPoint<C::ScalarXY, C::ScalarZ>,

    /// Whether this is the first segment of the input line.
    pub first_segment: bool,

    /// Flag that is `true` if the next geom in the sequence overlaps
    /// (i.e. intersects at more than one point) with this. Not
    /// relevant and `false` if this is a point.
    ///
    /// Note that the overlapping segments may not always
    /// _all_ get batched together. They may be reported as
    /// one or more set of overlapping segments in an
    /// arbitrary order.
    pub has_overlap: bool,

    /// Flag that is `true` if the `geom` starts at the intersection
    /// point. Otherwise, it ends at the intersection point.
    pub at_left: bool,

    pub(super) segment: IMSegment<C>,
}

#[allow(unused)]
pub(crate) fn compare_crossings<X: Cross>(a: &Crossing<X>, b: &Crossing<X>) -> Ordering {
    a.at_left.cmp(&b.at_left).then_with(|| {
        let ord = a.segment.partial_cmp(&b.segment).unwrap();
        if a.at_left {
            ord
        } else {
            ord.reverse()
        }
    })
}

impl<C: Cross + Clone> Crossing<C> {
    /// Convert `self` into a `Crossing` to return to user.
    pub(super) fn from_segment(segment: &IMSegment<C>, event_ty: EventType) -> Crossing<C> {
        Crossing {
            cross: segment.cross_cloned(),
            line: segment.geom(),
            first_segment: segment.is_first_segment(),
            has_overlap: segment.is_overlapping(),
            at_left: event_ty == EventType::LineLeft,
            segment: segment.clone(),
        }
    }
}

pub(crate) struct CrossingsIter<C>
where
    C: Cross + Clone,
{
    sweep: Sweep<C>,
    segments: Vec<Crossing<C>>,
}

impl<C> CrossingsIter<C>
where
    C: Cross + Clone,
{
    #[allow(unused)]
    /// Faster sweep when input geometries are known to not intersect except at
    /// end-points.
    pub fn new_simple<I: IntoIterator<Item = C>>(iter: I) -> Self {
        Self::new_ex(iter, true)
    }

    /// Returns the segments that intersect the last point yielded by
    /// the iterator.
    pub fn intersections_mut(&mut self) -> &mut [Crossing<C>] {
        &mut self.segments
    }

    pub fn intersections(&self) -> &[Crossing<C>] {
        &self.segments
    }

    #[allow(unused)]
    #[allow(clippy::type_complexity)]
    pub(crate) fn prev_active(
        &self,
        c: &Crossing<C>,
    ) -> Option<(LineOrPoint<C::ScalarXY, C::ScalarZ>, C)> {
        self.sweep
            .with_prev_active(c, |s| (s.geom, s.cross.clone()))
    }

    fn new_ex<T: IntoIterator<Item = C>>(iter: T, is_simple: bool) -> Self {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };
        let sweep = Sweep::new(iter, is_simple);
        let segments = Vec::with_capacity(4 * size);
        Self { sweep, segments }
    }
}

impl<C> FromIterator<C> for CrossingsIter<C>
where
    C: Cross + Clone,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self::new_ex(iter, false)
    }
}

impl<C> Iterator for CrossingsIter<C>
where
    C: Cross + Clone,
{
    type Item = Coordinate<C::ScalarXY, C::ScalarZ>;

    fn next(&mut self) -> Option<Self::Item> {
        let segments = &mut self.segments;
        segments.clear();
        let mut last_point = self.sweep.peek_point();
        while last_point == self.sweep.peek_point() && self.sweep.peek_point().is_some() {
            last_point = self
                .sweep
                .next_event(|seg, ty| segments.push(Crossing::from_segment(seg, ty)));
        }

        if segments.is_empty() {
            None
        } else {
            last_point.map(|p| *p)
        }
    }
}

pub struct Intersections<C: Cross + Clone> {
    inner: CrossingsIter<C>,
    idx: usize,
    jdx: usize,
    is_overlap: bool,
    pt: Option<Coordinate<C::ScalarXY, C::ScalarZ>>,
}

impl<C> FromIterator<C> for Intersections<C>
where
    C: Cross + Clone,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self {
            inner: FromIterator::from_iter(iter),
            idx: 0,
            jdx: 0,
            is_overlap: false,
            pt: None,
        }
    }
}

impl<C> Intersections<C>
where
    C: Cross + Clone,
{
    #[allow(clippy::type_complexity)]
    fn intersection(&mut self) -> Option<(C, C, LineIntersection<C::ScalarXY, C::ScalarZ>)> {
        let (si, sj) = {
            let segments = self.inner.intersections();
            (&segments[self.idx], &segments[self.jdx])
        };
        // Ignore intersections that have already been processed
        let should_compute = if self.is_overlap {
            si.at_left && (si.first_segment && sj.first_segment)
        } else {
            (!si.at_left || si.first_segment) && (!sj.at_left || sj.first_segment)
        };

        if should_compute {
            let si = si.cross.clone();
            let sj = sj.cross.clone();

            let int = line_intersection(si.line().line(), sj.line().line())
                .expect("line_intersection returned `None` disagreeing with `CrossingsIter`");

            Some((si, sj, int))
        } else {
            None
        }
    }

    fn step(&mut self) -> bool {
        let seg_len = self.inner.intersections_mut().len();
        if 1 + self.jdx < seg_len {
            self.is_overlap =
                self.is_overlap && self.inner.intersections_mut()[self.jdx].has_overlap;
            self.jdx += 1;
        } else {
            self.idx += 1;
            if 1 + self.idx >= seg_len {
                loop {
                    self.pt = self.inner.next();
                    if self.pt.is_none() {
                        return false;
                    }
                    if self.inner.intersections_mut().len() > 1 {
                        break;
                    }
                }
                self.idx = 0;
            }
            self.is_overlap = self.inner.intersections_mut()[self.idx].has_overlap;
            self.jdx = self.idx + 1;
        }
        true
    }
}

impl<C> Iterator for Intersections<C>
where
    C: Cross + Clone,
{
    type Item = (C, C, LineIntersection<C::ScalarXY, C::ScalarZ>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.step() {
                return None;
            }
            let it = self.intersection();
            if let Some(result) = it {
                return Some(result);
            }
        }
    }
}

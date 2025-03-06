use std::{borrow::Cow, cmp::Ordering, fmt::Debug, sync::Arc};

use parking_lot::RwLock;

use super::{
    segment::{Segment, SplitSegments},
    Cross, Event, EventType, LineOrPoint,
};

/// A wrapped segment that allows interior mutability.
pub(super) struct IMSegment<C: Cross> {
    inner: Arc<RwLock<Segment<C>>>,
}

impl<C: Cross> Clone for IMSegment<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<C: Cross> From<Segment<C>> for IMSegment<C> {
    fn from(segment: Segment<C>) -> Self {
        Self {
            inner: Arc::new(segment.into()),
        }
    }
}

impl<C: Cross> Debug for IMSegment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.read().fmt(f)
    }
}

impl<C: Cross> IMSegment<C> {
    pub fn is_overlapping(&self) -> bool {
        self.inner.read().overlapping.is_some()
    }
    pub fn overlap(&self) -> Option<Self> {
        self.inner.read().overlapping.as_ref().cloned()
    }
    pub fn is_first_segment(&self) -> bool {
        self.inner.read().first_segment
    }

    pub fn set_left_event_done(&self) {
        self.inner.write().left_event_done = true;
    }
    pub fn is_left_event_done(&self) -> bool {
        self.inner.read().left_event_done
    }

    pub fn geom(&self) -> LineOrPoint<<C as Cross>::ScalarXY, <C as Cross>::ScalarZ> {
        self.inner.read().geom
    }

    pub fn left_event(&self) -> Event<C::ScalarXY, C::ScalarZ, Self> {
        let geom = self.geom();
        let left = geom.left();
        Event {
            point: left,
            ty: if geom.is_line() {
                EventType::LineLeft
            } else {
                EventType::PointLeft
            },
            payload: self.clone(),
        }
    }

    pub fn right_event(&self) -> Event<C::ScalarXY, C::ScalarZ, Self> {
        let geom = self.geom();
        let right = geom.right();
        Event {
            point: right,
            ty: if geom.is_line() {
                EventType::LineRight
            } else {
                EventType::PointRight
            },
            payload: self.clone(),
        }
    }

    pub fn chain_overlap(&self, child: Self) {
        let mut this = self.clone();
        while let Some(ovl) = this.overlap() {
            this = ovl;
        }
        {
            child.inner.write().is_overlapping = true;
        }
        {
            let mut this_mut = this.inner.write();
            this_mut.overlapping = Some(child);
        }
    }

    pub fn adjust_for_intersection(
        &self,
        adj_intersection: LineOrPoint<C::ScalarXY, C::ScalarZ>,
    ) -> SplitSegments<C::ScalarXY, C::ScalarZ> {
        let (adjust_output, new_geom) = {
            let mut segment = self.inner.write();
            (
                segment.adjust_for_intersection(adj_intersection),
                segment.geom,
            )
        };
        let mut this = self.clone();
        while let Some(ovl) = this.overlap() {
            this = ovl;
            {
                let mut this_mut = this.inner.write();
                this_mut.geom = new_geom;
            }
        }
        adjust_output
    }

    #[allow(unused)]
    pub fn with_segment<F: FnOnce(&Segment<C>) -> R, R>(&self, f: F) -> R {
        f(&self.inner.read())
    }
}

impl<C: Cross + Clone> IMSegment<C> {
    pub(super) fn create_segment<F: FnMut(Event<C::ScalarXY, C::ScalarZ, Self>)>(
        crossable: C,
        geom: Option<LineOrPoint<C::ScalarXY, C::ScalarZ>>,
        parent: Option<&Self>,
        mut cb: F,
    ) -> Self {
        let segment: Self = Segment::new(crossable, geom).into();

        // Push events to process the created segment.
        for e in [segment.left_event(), segment.right_event()] {
            cb(e)
        }

        if let Some(parent) = parent {
            let segment_geom = segment.inner.read().geom;

            let mut child = parent.inner.read().overlapping.as_ref().cloned();
            let mut tgt = Cow::Borrowed(&segment);

            while let Some(child_seg) = child {
                let child_inner_seg = child_seg.inner.read();

                let child_overlapping = &child_inner_seg.overlapping;
                let child_crossable = child_inner_seg.cross.clone();

                let new_segment: Self = Segment::new(child_crossable, Some(segment_geom)).into();

                {
                    tgt.inner.write().overlapping = Some(new_segment.clone());
                }
                {
                    new_segment.inner.write().is_overlapping = true;
                }

                tgt = Cow::Owned(new_segment);
                child = child_overlapping.as_ref().cloned();
            }
        }
        segment
    }

    pub fn adjust_one_segment<F: FnMut(Event<C::ScalarXY, C::ScalarZ, Self>)>(
        &self,
        adj_intersection: LineOrPoint<C::ScalarXY, C::ScalarZ>,
        mut cb: F,
    ) -> Option<Self> {
        let adj_cross = self.cross_cloned();
        use SplitSegments::*;
        match self.adjust_for_intersection(adj_intersection) {
            Unchanged { overlap } => overlap.then(|| self.clone()),
            SplitOnce { overlap, right } => {
                cb(self.right_event());
                let new_key = Self::create_segment(adj_cross, Some(right), Some(self), &mut cb);
                match overlap {
                    Some(false) => Some(self.clone()),
                    Some(true) => Some(new_key),
                    None => None,
                }
            }
            SplitTwice { right } => {
                cb(self.right_event());
                Self::create_segment(adj_cross.clone(), Some(right), Some(self), &mut cb);
                let middle =
                    Self::create_segment(adj_cross, Some(adj_intersection), Some(self), &mut cb);
                Some(middle)
            }
        }
    }

    pub fn is_correct(event: &Event<C::ScalarXY, C::ScalarZ, IMSegment<C>>) -> bool {
        use EventType::*;
        let segment = event.payload.inner.read();
        if let LineRight = event.ty {
            debug_assert!(segment.geom.is_line());
            !segment.is_overlapping && segment.geom.right() == event.point
        } else {
            match event.ty {
                LineLeft => {
                    debug_assert!(segment.geom.is_line());
                    debug_assert_eq!(segment.geom.left(), event.point);
                }
                PointLeft | PointRight => {
                    debug_assert!(!segment.geom.is_line());
                    debug_assert_eq!(segment.geom.left(), event.point);
                }
                _ => unreachable!(),
            }
            true
        }
    }

    pub fn cross_cloned(&self) -> C {
        let inner = self.inner.read();
        inner.cross.clone()
    }
}

impl<C: Cross> PartialEq for IMSegment<C> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<C: Cross> Eq for IMSegment<C> {}

impl<C: Cross> PartialOrd for IMSegment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Cross> Ord for IMSegment<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        // self.inner.read().cmp(&other.inner.read()).then_with(|| {
        //     let addr_self = Arc::as_ptr(&self.inner) as usize;
        //     let addr_other = Arc::as_ptr(&other.inner) as usize;
        //     addr_self.cmp(&addr_other)
        // })

        let self_geom = self.inner.read().geom;
        let other_geom = other.inner.read().geom;

        self_geom.cmp(&other_geom)
    }
}

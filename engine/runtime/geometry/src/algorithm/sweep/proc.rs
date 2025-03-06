use std::{cmp::Ordering, collections::BinaryHeap};

use crate::algorithm::GeoFloat;

use super::*;

pub(crate) struct Sweep<C: Cross> {
    is_simple: bool,
    events: BinaryHeap<Event<C::ScalarXY, C::ScalarZ, IMSegment<C>>>,
    active_segments: VecSet<Active<IMSegment<C>>>,
}

impl<C: Cross + Clone> Sweep<C> {
    pub(crate) fn new<I>(iter: I, is_simple: bool) -> Self
    where
        I: IntoIterator<Item = C>,
    {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };

        let mut sweep = Sweep {
            events: BinaryHeap::with_capacity(size),
            active_segments: Default::default(),
            is_simple,
        };
        for cr in iter {
            IMSegment::create_segment(cr, None, None, |ev| sweep.events.push(ev));
        }

        sweep
    }

    /// Process the next event in heap.
    ///
    /// Calls the callback unless the event is spurious.
    #[inline]
    pub(super) fn next_event<F>(&mut self, mut cb: F) -> Option<SweepPoint<C::ScalarXY, C::ScalarZ>>
    where
        F: for<'a> FnMut(&'a IMSegment<C>, EventType),
    {
        self.events.pop().map(|event| {
            let pt = event.point;
            self.handle_event(event, &mut cb);

            pt
        })
    }

    /// Process two adjacent segments.
    ///
    /// The first argument must be an active segment, and the other may or may not be.
    /// Overlaps are chained from active -> other.
    fn process_adjacent_segments(
        &mut self,
        active: Active<IMSegment<C>>,
        other: &IMSegment<C>,
    ) -> AdjProcOutput<C::ScalarXY, C::ScalarZ> {
        // We handle this by intersecting twice if the segments overlap after adjustment.
        let mut out = AdjProcOutput {
            isec: None,
            should_continue: true,
            should_callback: false,
        };
        while let Some(isec) = other.geom().intersect_line_ordered(&active.geom()) {
            out.isec = Some(isec);

            // 1. Split adj_segment, and extra splits to storage
            let adj_overlap = active.adjust_one_segment(isec, |e| self.events.push(e));

            // 2. Split segment, adding extra segments as needed.
            let seg_overlap = other.adjust_one_segment(isec, |e| self.events.push(e));

            assert_eq!(
                adj_overlap.is_some(),
                seg_overlap.is_some(),
                "one of the intersecting segments had an overlap, but not the other!"
            );
            if let Some(adj_ovl) = adj_overlap {
                let tgt = seg_overlap.unwrap();
                adj_ovl.chain_overlap(tgt.clone());

                if &tgt == other {
                    // The whole event segment is now overlapping
                    // some other active segment.
                    //
                    // We do not need to continue iteration, but
                    // should callback if the left event of the
                    // now-parent has already been processed.
                    out.should_callback = adj_ovl.is_left_event_done();
                    out.should_continue = false;
                }

                // Overlaps are exact compute, so we do not need
                // to re-run the loop.
                return out;
            }

            if active.geom().partial_cmp(&other.geom()) == Some(Ordering::Equal) {
                continue;
            } else {
                break;
            }
        }
        out
    }

    fn handle_event<F>(
        &mut self,
        event: Event<C::ScalarXY, C::ScalarZ, IMSegment<C>>,
        cb: &mut F,
    ) -> bool
    where
        F: for<'a> FnMut(&'a IMSegment<C>, EventType),
    {
        use EventType::*;
        let segment = match IMSegment::is_correct(&event) {
            false => return false,
            _ => event.payload,
        };
        // let prev = self.active_segments.previous(&segment).cloned();
        // let next = self.active_segments.next(&segment).cloned();

        match &event.ty {
            LineLeft => {
                let mut should_add = true;
                //let mut insert_idx = self.active_segments.index_not_of(&segment);
                if !self.is_simple {
                    for is_next in [true, false].into_iter() {
                        // let active = if is_next {
                        //     if insert_idx < self.active_segments.len() {
                        //         self.active_segments[insert_idx].clone()
                        //     } else {
                        //         continue;
                        //     }
                        // } else if insert_idx > 0 {
                        //     self.active_segments[insert_idx - 1].clone()
                        // } else {
                        //     continue;
                        // };
                        let active_opt = if is_next {
                            self.active_segments
                                .get(Active::active_ref(&segment))
                                .cloned()
                        } else {
                            self.active_segments
                                .get_prev(Active::active_ref(&segment))
                                .cloned()
                        };

                        let active = match active_opt {
                            Some(active) => active,
                            None => continue,
                        };

                        let AdjProcOutput {
                            isec,
                            should_continue,
                            should_callback,
                        } = self.process_adjacent_segments(active.clone(), &segment);
                        let isec = match isec {
                            Some(isec) => isec,
                            None => continue,
                        };
                        // A special case is if adj_segment was split, and the
                        // intersection is at the start of this segment. In this
                        // case, there is an right-end event in the heap, that
                        // needs to be handled before finishing up this event.
                        let handle_end_event = {
                            // Get first point of intersection
                            let int_pt = isec.left();
                            // Check its not first point of the adjusted, but is
                            // first point of current segment
                            int_pt != active.geom().left() && int_pt == segment.geom().left()
                        };
                        if handle_end_event {
                            let event = self.events.pop().unwrap();
                            let done = self.handle_event(event, cb);
                            //debug_assert!(done, "special right-end event handling failed");
                            if !is_next {
                                // The prev-segment is now removed
                                //insert_idx -= 1;
                                let prev = self
                                    .active_segments
                                    .get_prev(Active::active_ref(&segment))
                                    .cloned();
                                if let Some(prev) = prev.clone() {
                                    self.active_segments.remove(&prev);
                                }
                            }
                        }

                        if !should_continue {
                            should_add = false;
                            if !should_callback {
                                return true;
                            }
                            break;
                        }

                        // let n = self.active_segments.len();
                        // if is_next && 1 + insert_idx < n {
                        //     (insert_idx..n).find(|&idx| !self.active_segments.check_swap(idx));
                        // } else if !is_next && insert_idx > 1 {
                        //     (0..insert_idx - 2)
                        //         .rev()
                        //         .find(|&idx| !self.active_segments.check_swap(idx));
                        // }
                    }
                }

                if should_add {
                    // NOTE: we bravely track insert_idx as the active-list is adjusted
                    // self.active_segments.insert_active(segment.clone());
                    //self.active_segments.insert_at(insert_idx, segment.clone());
                    self.active_segments.insert(Active(segment.clone()));
                }

                let mut cb_seg = Some(segment);
                while let Some(seg) = cb_seg {
                    cb(&seg, event.ty);
                    seg.set_left_event_done();
                    cb_seg = seg.overlap();
                }
            }
            LineRight => {
                // Safety: `self.segments` is a `Box` that is not
                // de-allocated until `self` is dropped.
                // let el_idx = self.active_segments.index_of(&segment);
                // let prev = (el_idx > 0).then(|| self.active_segments[el_idx - 1].clone());
                // let next = (1 + el_idx < self.active_segments.len())
                //     .then(|| self.active_segments[el_idx + 1].clone());
                // assert_eq!(self.active_segments.remove_at(el_idx), segment);

                let prev = self
                    .active_segments
                    .get_prev(Active::active_ref(&segment))
                    .cloned();
                let next = self
                    .active_segments
                    .get_next(Active::active_ref(&segment))
                    .cloned();
                if let Some(prev) = prev.clone() {
                    self.active_segments.remove(&prev);
                }

                let mut cb_seg = Some(segment);
                while let Some(seg) = cb_seg {
                    cb(&seg, event.ty);
                    cb_seg = seg.overlap();
                }

                if !self.is_simple {
                    if let (Some(prev), Some(next)) = (prev, next) {
                        let prev_geom = prev.geom();
                        let next_geom = next.geom();
                        if let Some(adj_intersection) = prev_geom.intersect_line_ordered(&next_geom)
                        {
                            // 1. Split prev_segment, and extra splits to storage
                            let first = prev
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e))
                                .is_none();
                            let second = next
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e))
                                .is_none();
                            debug_assert!(
                                first && second,
                                "adjacent segments @ removal can't overlap!"
                            );
                        }
                    }
                }
            }
            PointLeft => {
                if !self.is_simple {
                    //let insert_idx = self.active_segments.index_not_of(&segment);
                    // let prev =
                    //     (insert_idx > 0).then(|| self.active_segments[insert_idx - 1].clone());
                    // let next = (insert_idx < self.active_segments.len())
                    //     .then(|| self.active_segments[insert_idx].clone());
                    let prev = self.active_segments.get_prev(Active::active_ref(&segment));
                    let next = self.active_segments.get(Active::active_ref(&segment));

                    for adj_segment in prev.into_iter().chain(next.into_iter()) {
                        let geom = adj_segment.geom();
                        if let Some(adj_intersection) = segment.geom().intersect_line_ordered(&geom)
                        {
                            // 1. Split adj_segment, and extra splits to storage
                            let adj_overlap = adj_segment
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e));

                            // Can't have overlap with a point
                            debug_assert!(adj_overlap.is_none());
                        }
                    }
                }

                // Points need not be active segments.
                // Send the point-segment to callback.
                cb(&segment, event.ty);
            }
            PointRight => {
                // Nothing to do. We could remove this variant once we
                // are confident about the logic.
            }
        }
        true
    }

    #[inline]
    pub(super) fn with_prev_active<F: FnOnce(&Segment<C>) -> R, R>(
        &self,
        c: &Crossing<C>,
        f: F,
    ) -> Option<R> {
        debug_assert!(c.at_left);
        self.active_segments
            .previous(&c.segment)
            .map(|aseg| aseg.with_segment(f))
    }

    #[inline]
    pub fn peek_point(&self) -> Option<SweepPoint<C::ScalarXY, C::ScalarZ>> {
        self.events.peek().map(|e| e.point)
    }
}

/// Internal enum to communicate result from `process_adjacent_segments`
struct AdjProcOutput<T: GeoFloat, Z: GeoFloat> {
    isec: Option<LineOrPoint<T, Z>>,
    should_continue: bool,
    should_callback: bool,
}

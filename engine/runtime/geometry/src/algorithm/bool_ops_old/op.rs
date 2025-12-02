use std::{cell::Cell, cmp::Ordering, fmt::Debug};

use crate::{
    algorithm::{
        coords_iter::CoordsIter,
        sweep::{compare_crossings, Cross, CrossingsIter, LineOrPoint},
        GeoFloat,
    },
    types::{
        line_string::LineString2D, multi_polygon::MultiPolygon2D, no_value::NoValue,
        polygon::Polygon2D,
    },
};

use super::Spec;

#[derive(Debug, Clone)]
pub struct Proc<T: GeoFloat, S: Spec<T>> {
    spec: S,
    edges: Vec<Edge<T, S>>,
}

impl<T: GeoFloat, S: Spec<T>> Proc<T, S> {
    pub fn new(spec: S, capacity: usize) -> Self {
        Proc {
            spec,
            edges: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn add_multi_polygon(&mut self, mp: &MultiPolygon2D<T>, idx: usize) {
        mp.0.iter().for_each(|p| self.add_polygon(p, idx));
    }

    pub(crate) fn add_polygon(&mut self, poly: &Polygon2D<T>, idx: usize) {
        self.add_closed_ring(poly.exterior(), idx, false);
        for hole in poly.interiors() {
            self.add_closed_ring(hole, idx, true);
        }
    }

    /// Adds `ls` to the procedure, with the shape index as `idx`. `idx` should
    /// be either 0 or 1 depending on whether it is the first or second input.
    pub(crate) fn add_line_string(&mut self, ls: &LineString2D<T>, idx: usize) {
        for line in ls.lines() {
            let lp: LineOrPoint<_, NoValue> = line.into();
            if !lp.is_line() {
                continue;
            }

            let region = self.spec.infinity();
            self.edges.push(Edge {
                geom: lp,
                idx,
                _region: region.into(),
                _region_2: region.into(),
            });
        }
    }

    /// Adds `ring` to the procedure, with the shape index as `idx`, and
    /// `_is_hole` if this ring belongs to a hole in the original polygon. `idx`
    /// should be either 0 or 1 depending on whether it is the first or
    /// second input. `_is_hole` is not used right now; remove it once we fully
    /// handle floating-point issues.
    fn add_closed_ring(&mut self, ring: &LineString2D<T>, idx: usize, _is_hole: bool) {
        assert!(ring.is_closed());
        if ring.coords_count() <= 3 {
            return;
        }

        self.add_line_string(ring, idx);
    }

    /// Sweeps across the edges, splits them, then passes the resulting regions
    /// and edges to the spec, to compute the result shape.
    pub fn sweep(mut self) -> S::Output {
        let mut iter = CrossingsIter::from_iter(self.edges.iter());

        // Iterate through the intersection points (including end points).
        while let Some(_) = iter.next() {
            // Sort crossings at `pt`. This means the end of segments will be ordered before
            // the start of any segments, and start/end segments will be ordered from bottom
            // to top.
            iter.intersections_mut().sort_unstable_by(compare_crossings);

            // Process all end-segments.
            let mut idx = 0;
            let mut next_region = None;
            while idx < iter.intersections().len() {
                let c = &iter.intersections()[idx];
                // If we hit a start-segment, we are done with all the end segments.
                if c.at_left {
                    break;
                }
                let cross = c.cross;
                if next_region.is_none() {
                    // This is the first segment in a group of overlapping segments (including when
                    // the group only includes this one segment).
                    next_region = Some(cross.get_region(c.line));
                }
                // Update the region with the new edge.
                next_region = Some(self.spec.cross(next_region.unwrap(), cross.idx));
                // We only want to output one edge per group of overlapping segments (since the
                // output shape should not have overlapping segments).
                let has_overlap = (idx + 1) < iter.intersections().len()
                    && c.line.partial_cmp(&iter.intersections()[idx + 1].line)
                        == Some(Ordering::Equal);
                if !has_overlap {
                    let prev_region = cross.get_region(c.line);
                    // Output the segment once we've reached the end of the segment. We must wait
                    // until the end of the segment (here), since overlapping segments can affect if
                    // an edge is in the output or not.
                    self.spec
                        .output([prev_region, next_region.unwrap()], c.line, c.cross.idx);
                    next_region = None;
                }
                idx += 1;
            }

            // We've processed the last crossing. Just skip as an optimization.
            if idx >= iter.intersections_mut().len() {
                continue;
            }
            let botmost_start_segment = iter.intersections_mut()[idx].clone();
            debug_assert!(botmost_start_segment.at_left);

            // Get the region of the previous edge in the output, or use the "infinity
            // point".
            let prev = iter.prev_active(&botmost_start_segment);
            let mut region = prev
                .as_ref()
                .map(|(g, c)| c.get_region(*g))
                .unwrap_or_else(|| self.spec.infinity());

            // Process all start-segments.
            while idx < iter.intersections().len() {
                let mut c = &iter.intersections()[idx];
                let mut jdx = idx;
                // Loop over all the segments in the intersection that are overlapping with `c`.
                loop {
                    // Update the region with the new edge.
                    region = self.spec.cross(region, c.cross.idx);
                    let has_overlap = (idx + 1) < iter.intersections().len()
                        && c.line.partial_cmp(&iter.intersections()[idx + 1].line)
                            == Some(Ordering::Equal);
                    if !has_overlap {
                        // The next segment is not intersecting, so leave that to the next iteration
                        // of the outer loop.
                        break;
                    }
                    // The next edge is overlapping, so keep looping.
                    idx += 1;
                    c = &iter.intersections()[idx];
                }
                // We have "skipped over" all the overlapping segments of `c`.
                // Set the region of all overlapping segments of `c` to `region`, since the
                // "result" of all of these segments is `region`.
                while jdx <= idx {
                    let gpiece = iter.intersections()[jdx].line;
                    iter.intersections()[jdx].cross.set_region(region, gpiece);
                    jdx += 1;
                }
                idx += 1;
            }
        }
        self.spec.finish()
    }
}

/// An edge of a shape.
#[derive(Clone)]
struct Edge<T: GeoFloat, S: Spec<T>> {
    /// The geometry of the edge.
    geom: LineOrPoint<T, NoValue>,
    /// The index of the shape this edge belongs to.
    idx: usize,
    /// The region of this edge when the piece is going in the same direction as
    /// `geom`.
    _region: Cell<S::Region>,
    _region_2: Cell<S::Region>,
}

impl<T: GeoFloat, S: Spec<T>> Edge<T, S> {
    /// Gets the region for this edge based on the provided `piece`. `piece`
    /// must be some part (i.e., subspan) of `self.geom`.
    fn get_region(&self, piece: LineOrPoint<T, NoValue>) -> S::Region {
        // Note: This is related to the ordering of intersection
        // with respect to the complete geometry. Due to
        // finite-precision errors, intersection points might lie
        // outside the end-points in lexicographic ordering.
        //
        // Thus, while processing, the segment, we might be looking at it from
        // end-to-start as opposed to the typical start-to-end (with respect to
        // the complete geom. the segment is a part of).
        //
        // In this case, the region set/get procedure queries different sides of
        // the segment. Thus, we detect this and store both sides of the region.
        // Finally, note that we need to store both sides of the segment, as
        // this cannot be computed from the current edge alone (it may depend on
        // more overlapping edges).
        if piece.left() < self.geom.right() {
            self._region.get()
        } else {
            self._region_2.get()
        }
    }
    /// Sets the region for this edge to `region` based onm the provided
    /// `piece`. `piece` must be some part (i.e., subspan) of `self.geom`.
    fn set_region(&self, region: S::Region, piece: LineOrPoint<T, NoValue>) {
        if piece.left() < self.geom.right() {
            self._region.set(region);
        } else {
            self._region_2.set(region);
        }
    }
}

impl<T: GeoFloat, S: Spec<T>> std::fmt::Debug for Edge<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.geom.line();
        f.debug_struct("Edge")
            .field(
                "geom",
                &format!(
                    "({:?}, {:?}) <-> ({:?}, {:?})",
                    line.start.x, line.start.y, line.end.x, line.end.y
                ),
            )
            .field("idx", &self.idx)
            .field("region", &self._region)
            .finish()
    }
}

impl<T: GeoFloat, S: Spec<T>> Cross for Edge<T, S> {
    type ScalarXY = T;
    type ScalarZ = NoValue;

    fn line(&self) -> LineOrPoint<Self::ScalarXY, NoValue> {
        self.geom
    }
}

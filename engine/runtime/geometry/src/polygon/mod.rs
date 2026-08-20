//! Polygon leaves.
//!
//! A `Polygon` is a single planar face: one exterior boundary ring with optional
//! interior rings (holes), each ring closed (first vertex == last). It is not a
//! mesh; for connected, vertex-sharing multi-face surfaces use `PolygonMesh`.
//!
//! Flat CSR-style layout: the exterior ring and all interior rings are
//! concatenated into a single `coords` allocation, with `interior_offsets`
//! recording where each interior ring starts (the exterior is the prefix up to
//! the first hole, so it carries no offset of its own).

#[cfg(feature = "debug-geom-feature-write")]
use serde::{Deserialize, Serialize};

use crate::appearance::Appearance;
use crate::coordinate::CoordinateFrame;

mod constructor;
#[cfg(not(feature = "debug-geom-feature-write"))]
mod feature_write;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

pub use constructor::{state, PolygonBuilder2D, PolygonBuilder3D, PolygonFace};

/// A planar polygon face in 2D space, lying at a single optional elevation.
#[cfg_attr(feature = "debug-geom-feature-write", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon2D {
    /// Coordinate frame these coords are expressed in.
    frame: CoordinateFrame,
    /// Exterior ring, then all interior rings (holes), concatenated. A valid polygon
    /// has each ring closed (first == last), with the exterior wound counter-clockwise
    /// and interiors clockwise in canonical orientation (see [`crate::coordinate`]:
    /// winding is judged after applying the frame's orientation sign, not in stored
    /// coordinate order).
    coords: Box<[[f64; 2]]>,
    /// Start index in `coords` of each interior ring; empty when there are no
    /// holes. exterior = `coords[0 .. first interior start (or end)]`;
    /// interior j = `coords[interior_offsets[j] .. interior_offsets[j+1] (or end)]`.
    interior_offsets: Box<[u32]>,
    /// The elevation the whole face lies at. `None` = pure 2D.
    z: Option<f64>,
    /// Materials / themes / single-face binding, incl. per-theme UV parallel to
    /// `coords`; `None` = bare geometry.
    appearance: Option<Appearance>,
}

/// A planar polygon face in 3D space.
#[cfg_attr(feature = "debug-geom-feature-write", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon3D {
    /// Coordinate frame these coords are expressed in.
    frame: CoordinateFrame,
    /// Exterior ring, then all interior rings (holes), concatenated. Its canonical
    /// outward normal is the exterior's right-hand-rule normal times the frame's
    /// orientation sign (see [`crate::coordinate`]). A valid polygon has each ring
    /// closed (first == last), with exterior and interior rings wound opposite to
    /// each other.
    coords: Box<[[f64; 3]]>,
    /// Start index in `coords` of each interior ring; empty when there are no holes.
    interior_offsets: Box<[u32]>,
    /// Materials / themes / single-face binding, incl. per-theme UV parallel to
    /// `coords`; `None` = bare geometry.
    appearance: Option<Appearance>,
}

impl Polygon2D {
    /// The coordinate frame these coords are expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The exterior ring, as stored verbatim — a well-formed ring is closed
    /// (first == last), but an open ring is preserved as-is for later validation.
    pub fn exterior(&self) -> &[[f64; 2]] {
        let end = self
            .interior_offsets
            .first()
            .map_or(self.coords.len(), |&o| o as usize);
        &self.coords[..end]
    }

    /// The interior (hole) rings, each as stored verbatim (not guaranteed closed),
    /// in order.
    pub fn interiors(&self) -> impl Iterator<Item = &[[f64; 2]]> + '_ {
        let coords = &self.coords;
        let offsets = &self.interior_offsets;
        (0..offsets.len()).map(move |j| {
            let start = offsets[j] as usize;
            let end = offsets.get(j + 1).map_or(coords.len(), |&o| o as usize);
            &coords[start..end]
        })
    }

    /// The elevation the face lies at, or `None` when it is pure 2D.
    #[inline]
    pub fn elevation(&self) -> Option<f64> {
        self.z
    }

    /// The unsigned planar area of the face: the exterior ring's area minus the
    /// area of the holes. Ring winding does not affect the result; elevation
    /// does not contribute. Rings stored open are measured as if closed.
    pub fn area(&self) -> f64 {
        let exterior = ring_area(self.exterior());
        let holes: f64 = self.interiors().map(ring_area).sum();
        (exterior - holes).max(0.0)
    }

    /// Borrow the appearance, if any.
    #[inline]
    pub fn appearance(&self) -> &Option<Appearance> {
        &self.appearance
    }

    /// Mutably borrow the appearance, to set, clear, or edit it in place.
    #[inline]
    pub fn appearance_mut(&mut self) -> &mut Option<Appearance> {
        &mut self.appearance
    }
}

/// The unsigned shoelace area of one ring. Rings are stored as the builder
/// received them, so a ring left open is measured with its closing edge
/// restored; fewer than three vertices enclose nothing.
fn ring_area(ring: &[[f64; 2]]) -> f64 {
    if ring.len() < 3 {
        return 0.0;
    }
    let mut sum: f64 = ring
        .windows(2)
        .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
        .sum();
    let (first, last) = (ring[0], ring[ring.len() - 1]);
    if first != last {
        sum += last[0] * first[1] - first[0] * last[1];
    }
    sum.abs() / 2.0
}

impl Polygon3D {
    /// The coordinate frame these coords are expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The exterior ring, as stored verbatim — a well-formed ring is closed
    /// (first == last), but an open ring is preserved as-is for later validation.
    pub fn exterior(&self) -> &[[f64; 3]] {
        let end = self
            .interior_offsets
            .first()
            .map_or(self.coords.len(), |&o| o as usize);
        &self.coords[..end]
    }

    /// The interior (hole) rings, each as stored verbatim (not guaranteed closed),
    /// in order.
    pub fn interiors(&self) -> impl Iterator<Item = &[[f64; 3]]> + '_ {
        let coords = &self.coords;
        let offsets = &self.interior_offsets;
        (0..offsets.len()).map(move |j| {
            let start = offsets[j] as usize;
            let end = offsets.get(j + 1).map_or(coords.len(), |&o| o as usize);
            &coords[start..end]
        })
    }

    /// Borrow the appearance, if any.
    #[inline]
    pub fn appearance(&self) -> &Option<Appearance> {
        &self.appearance
    }

    /// Mutably borrow the appearance, to set, clear, or edit it in place.
    #[inline]
    pub fn appearance_mut(&mut self) -> &mut Option<Appearance> {
        &mut self.appearance
    }
}

// A polygon is a single face, not a multi-part container.
crate::unsupported!(Polygon2D: Split);
crate::unsupported!(Polygon3D: Split);

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2x2 square held away from the origin, so the closing edge carries a
    /// non-zero shoelace term and an omitted one shows up in the result.
    fn square(closed: bool) -> Polygon2D {
        let mut ring = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]];
        if closed {
            ring.push([1.0, 1.0]);
        }
        Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            Vec::<Vec<[f64; 2]>>::new(),
        )
    }

    #[test]
    fn a_ring_left_open_measures_the_same_as_a_closed_one() {
        assert_eq!(square(true).area(), 4.0);
        assert_eq!(square(false).area(), 4.0);
    }

    #[test]
    fn winding_does_not_affect_the_area() {
        let clockwise = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [0.0, 2.0], [2.0, 2.0], [2.0, 0.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(clockwise.area(), 4.0);
    }

    #[test]
    fn holes_are_subtracted() {
        let with_hole = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]],
            vec![vec![
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
                [1.0, 1.0],
            ]],
        );
        assert_eq!(with_hole.area(), 15.0);
    }

    #[test]
    fn a_ring_too_short_to_enclose_anything_has_no_area() {
        let degenerate = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(degenerate.area(), 0.0);
    }
}

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

use serde::{Deserialize, Serialize};

use crate::appearance::{Appearance, UvSet};
use crate::coordinate::Coordinate;

mod constructor;

pub use constructor::{state, PolygonBuilder2D, PolygonBuilder3D};

/// A planar polygon face in 2D space, with optional per-vertex elevation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Polygon2D {
    /// Coordinate frame these coords are expressed in.
    coordinate: Coordinate,
    /// Exterior ring, then all interior rings (holes), concatenated; each ring
    /// closed (first == last).
    coords: Box<[[f64; 2]]>,
    /// Start index in `coords` of each interior ring; empty when there are no
    /// holes. exterior = `coords[0 .. first interior start (or end)]`;
    /// interior j = `coords[interior_offsets[j] .. interior_offsets[j+1] (or end)]`.
    interior_offsets: Box<[u32]>,
    /// Optional per-vertex elevation, parallel to `coords` (same ring
    /// concatenation). INVARIANT: when `Some`, `z.len() == coords.len()`.
    /// `None` = pure 2D (no allocation).
    z: Option<Box<[f64]>>,
    /// UV parallel to `coords` (same ring concatenation); one set per
    /// (theme, side, channel).
    uv_sets: Vec<UvSet>,
    /// Materials / themes / single-face binding; `None` = bare geometry.
    appearance: Option<Appearance>,
}

/// A planar polygon face in 3D space.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Polygon3D {
    /// Coordinate frame these coords are expressed in.
    coordinate: Coordinate,
    /// Exterior ring, then all interior rings (holes), concatenated; each ring
    /// closed (first == last).
    coords: Box<[[f64; 3]]>,
    /// Start index in `coords` of each interior ring; empty when there are no holes.
    interior_offsets: Box<[u32]>,
    /// UV parallel to `coords`; one set per (theme, side, channel).
    uv_sets: Vec<UvSet>,
    /// Materials / themes / single-face binding; `None` = bare geometry.
    appearance: Option<Appearance>,
}

impl Polygon2D {
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

impl Polygon3D {
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

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

use nusamai_projection::crs::EpsgCode;
use serde::{Deserialize, Serialize};

use crate::appearance::{Appearance, UvSet};
use crate::coordinate::Coordinate;
use crate::error::Result;
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d, ReprojectionCache};

mod constructor;
mod ops;

pub use constructor::{state, PolygonBuilder2D, PolygonBuilder3D, PolygonFace};

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
    /// The coordinate frame these coords are expressed in.
    #[inline]
    pub fn coordinate(&self) -> &Coordinate {
        &self.coordinate
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

    /// The UV sets, one per (theme, side, channel); each `Explicit` array is
    /// parallel to `coords` (exterior then interiors, closed).
    #[inline]
    pub fn uv_sets(&self) -> &[UvSet] {
        &self.uv_sets
    }

    pub(crate) fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_2d(cache, from, target, &mut self.coords, self.z.as_deref_mut())?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

impl Polygon3D {
    /// The coordinate frame these coords are expressed in.
    #[inline]
    pub fn coordinate(&self) -> &Coordinate {
        &self.coordinate
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

    /// The UV sets, one per (theme, side, channel); each `Explicit` array is
    /// parallel to `coords` (exterior then interiors, closed).
    #[inline]
    pub fn uv_sets(&self) -> &[UvSet] {
        &self.uv_sets
    }

    pub(crate) fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, &mut self.coords)?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

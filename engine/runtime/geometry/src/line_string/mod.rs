//! LineString leaves.
//!
//! A `LineString` is a polyline: an ordered chain of coordinates, a variant in
//! both embeddings. It follows the `Polygon` flat-buffer convention: a single
//! closed/open chain of coordinates in one `Box<[_]>` allocation, with the 2D
//! form carrying optional per-vertex elevation parallel to `coords`. Lines carry
//! no appearance.

use serde::{Deserialize, Serialize};

use crate::coordinate::CoordinateFrame;

mod constructor;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// A polyline in 2D space, with optional per-vertex elevation (2.5D).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LineString2D {
    /// Coordinate frame these coords are expressed in.
    frame: CoordinateFrame,
    coords: Box<[[f64; 2]]>,
    /// Optional per-vertex elevation, parallel to `coords`.
    /// INVARIANT: when `Some`, `z.len() == coords.len()`. `None` = pure 2D.
    z: Option<Box<[f64]>>,
}

/// A polyline in 3D space.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LineString3D {
    /// Coordinate frame these coords are expressed in.
    frame: CoordinateFrame,
    coords: Box<[[f64; 3]]>,
}

impl LineString3D {
    /// The chain's vertices in order.
    #[inline]
    pub fn coords(&self) -> &[[f64; 3]] {
        &self.coords
    }
}

crate::unsupported!(LineString2D: Triangulate);
crate::unsupported!(LineString3D: Triangulate);

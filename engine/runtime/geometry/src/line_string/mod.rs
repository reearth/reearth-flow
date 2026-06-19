//! LineString leaves.
//!
//! A `LineString` is a polyline: an ordered chain of coordinates, a variant in
//! both embeddings. It follows the `Polygon` flat-buffer convention: a single
//! closed/open chain of coordinates in one `Box<[_]>` allocation, with the 2D
//! form carrying optional per-vertex elevation parallel to `coords`. Lines carry
//! no appearance.

use serde::{Deserialize, Serialize};

use crate::coordinate::Coordinate;

/// A polyline in 2D space, with optional per-vertex elevation (2.5D).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LineString2D {
    /// Coordinate frame these coords are expressed in.
    coordinate: Coordinate,
    coords: Box<[[f64; 2]]>,
    /// Optional per-vertex elevation, parallel to `coords`.
    /// INVARIANT: when `Some`, `z.len() == coords.len()`. `None` = pure 2D.
    z: Option<Box<[f64]>>,
}

/// A polyline in 3D space.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LineString3D {
    /// Coordinate frame these coords are expressed in.
    coordinate: Coordinate,
    coords: Box<[[f64; 3]]>,
}

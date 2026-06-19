//! Per-leaf coordinate frame.

use nusamai_projection::crs::EpsgCode;
use serde::{Deserialize, Serialize};

/// The coordinate frame a geometry leaf is expressed in.
///
/// Every coordinate-bearing leaf carries its own `coordinate: Coordinate`, so an
/// operation reads its source frame from `self` and a collection may hold
/// members in different frames.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum Coordinate {
    /// A geographic / projected CRS identified by its EPSG code.
    Crs(EpsgCode),
    /// Bare Euclidean space with no geo-referencing.
    #[default]
    Euclidean,
    /// A 2D plane embedded in 3D, anchored in a base frame.
    Tangent(TangentPlane),
}

/// The absolute frame a [`TangentPlane`] is anchored in: exactly the non-tangent
/// [`Coordinate`] frames, so a tangent plane cannot be anchored in another
/// tangent plane.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseFrame {
    /// A geographic / projected CRS identified by its EPSG code.
    Crs(EpsgCode),
    /// Bare Euclidean space with no geo-referencing.
    Euclidean,
}

/// A 2D Euclidean plane embedded in 3D space.
///
/// A [`Coordinate::Tangent`] geometry stores in-plane `(x, y)` whose 3D position
/// is `origin + x * u + y * v`. When `base` is a geographic CRS this is the
/// local tangent (ENU) frame at `origin`, with in-plane coordinates in metres.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TangentPlane {
    /// Frame that `origin`, `u` and `v` are expressed in.
    pub base: BaseFrame,
    /// Plane origin, in `base`.
    pub origin: [f64; 3],
    /// Orthonormal in-plane axis; the plane normal is the cross product of `u`
    /// and `v`.
    pub u: [f64; 3],
    /// Orthonormal in-plane axis.
    pub v: [f64; 3],
}

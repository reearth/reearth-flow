//! Per-leaf coordinate frame.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// An EPSG code identifying a coordinate reference system.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct EpsgCode(u16);

impl EpsgCode {
    /// Wrap a raw EPSG code.
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// The raw EPSG code.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for EpsgCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u16> for EpsgCode {
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<EpsgCode> for u16 {
    fn from(code: EpsgCode) -> Self {
        code.0
    }
}

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
    /// A 2D plane embedded in 3D, anchored in a base frame. Boxed: a
    /// `TangentPlane` is ~72 bytes against the 2-byte `EpsgCode`, and `Tangent`
    /// is the rare frame, so boxing it keeps `Coordinate` — embedded in every
    /// geometry leaf — pointer-sized for the common `Crs` / `Euclidean` cases.
    Tangent(Box<TangentPlane>),
}

impl Coordinate {
    /// The EPSG code of this frame, or an error if it is not a CRS frame.
    pub(crate) fn require_crs(&self) -> Result<EpsgCode> {
        match self {
            Coordinate::Crs(epsg) => Ok(*epsg),
            Coordinate::Euclidean => Err(Error::projection(
                "cannot reproject a Euclidean (non-georeferenced) geometry",
            )),
            Coordinate::Tangent(_) => Err(Error::projection(
                "cannot reproject a Tangent-plane geometry",
            )),
        }
    }
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

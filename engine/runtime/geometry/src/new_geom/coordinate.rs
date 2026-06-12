//! The coordinate frame a leaf's coordinates are expressed in.
//!
//! Stored once per leaf node (see the module docs in [`crate::new_geom`]):
//! flat leaves hold it directly; composite leaves (`Solid`, `Csg`) hold exactly
//! one over coordless raw buffers.

/// EPSG code placeholder. The production type would be the engine's CRS code.
pub type EpsgCode = u32;

/// Reference frame of a leaf's coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Coordinate {
    /// A geographic / projected CRS identified by EPSG code.
    Crs(EpsgCode),
    /// Bare Euclidean space, no georeference.
    Euclidean,
    // Tangent(TangentPlane) — deferred; TangentPlane is unspecified in the design doc.
}

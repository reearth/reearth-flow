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
    /// A local tangent-plane frame.
    ///
    /// TODO(new-geometry): `TangentPlane` is unspecified; consolidate
    /// its definition before relying on this variant.
    Tangent(TangentPlane),
}

/// Placeholder for a local tangent-plane frame.
///
/// TODO(new-geometry): the frame is unspecified pending consolidation.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct TangentPlane {}

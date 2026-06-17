//! Per-embedding collections.
//!
//! Each embedding's `Collection` holds primitives of the same intrinsic
//! dimension with no shared vertex topology (equivalent to `Multi*` in
//! GeoJSON/GML). Members are not required to share a coordinate frame: every
//! leaf carries its own `coordinate`. Both collections carry per-child
//! attributes (`attrs`, parallel to `members`), used to preserve a child's
//! attributes; they are not exposed as the feature's own attributes.

use reearth_flow_common::attribute::Attributes;
use serde::{Deserialize, Serialize};

use crate::{Euclidean2DGeometry, Euclidean3DGeometry};

/// A `Multi*` collection of 2D geometries; members may differ in CRS.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Collection2D {
    pub members: Vec<Euclidean2DGeometry>,
    /// Per-member attributes, parallel to `members`; empty = no member carries
    /// any. Child-scoped.
    pub attrs: Vec<Attributes>,
}

/// A `Multi*` collection of 3D geometries; members may differ in CRS.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Collection3D {
    pub members: Vec<Euclidean3DGeometry>,
    /// Per-member attributes, parallel to `members`; empty = no member carries
    /// any. Child-scoped.
    pub attrs: Vec<Attributes>,
}

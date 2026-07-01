//! CSG boolean tree.
//!
//! A recursive binary tree over the volumetric, closed 3D geometries that
//! boolean operations are defined over. Point clouds, open meshes, and
//! lower-dimensional types are intentionally excluded. `Csg` holds no frame of
//! its own; its frame(s) come from its operand `Solid`s.

use serde::{Deserialize, Serialize};

use super::solid::Solid;

mod constructor;
mod ops;

/// Volumetric, closed 3D geometries that `Csg` boolean operations are defined
/// over.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ThreeDimensional {
    /// Boxed: a `Solid` (with its shells' appearance) is far larger than the
    /// boxed `Csg`, so the leaf is boxed to keep the enum small.
    Solid(Box<Solid>),
    Csg(Box<Csg>),
}

/// A boolean combination of two volumetric operands.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Csg {
    Union(Box<ThreeDimensional>, Box<ThreeDimensional>),
    Intersection(Box<ThreeDimensional>, Box<ThreeDimensional>),
    Difference(Box<ThreeDimensional>, Box<ThreeDimensional>),
}

// Tessellation is defined only for `Polygon` / `PolygonMesh`.
crate::unsupported!(Csg: Triangulate);
crate::unsupported!(Csg: Reproject);

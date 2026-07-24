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
#[cfg(feature = "new-geometry")]
mod validation;

/// Volumetric, closed 3D geometries that `Csg` boolean operations are defined
/// over.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ThreeDimensional {
    /// Boxed: a `Solid` (with its shells' appearance) is far larger than the
    /// boxed `Csg`, so the leaf is boxed to keep the enum small.
    Solid(Box<Solid>),
    Csg(Box<Csg>),
}

/// A boolean combination of two volumetric operands.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Csg {
    Union(Box<ThreeDimensional>, Box<ThreeDimensional>),
    Intersection(Box<ThreeDimensional>, Box<ThreeDimensional>),
    Difference(Box<ThreeDimensional>, Box<ThreeDimensional>),
}

// Tessellation is defined only for `Polygon` / `PolygonMesh`.
crate::unsupported!(Csg: Triangulate, Reproject, ConvertFrame, ForceTwoDimension);

// An unevaluated boolean tree has no faces of its own; counting the rings of its
// operands would describe a surface the tree does not yet have.
crate::unsupported!(Csg: CountHoles);

// The boolean tree is unevaluated, so its operands' faces are not this geometry's
// boundary and taking them apart would not describe it.
crate::unsupported!(Csg: ExtractHoles);

// A boolean tree is one logical solid, not a multi-part container.
crate::unsupported!(Csg: Split);

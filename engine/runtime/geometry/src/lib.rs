#![recursion_limit = "2048"]
extern crate alloc;

pub mod algorithm;
pub mod error;
pub mod types;
pub mod utils;
pub mod validation;

#[macro_use]
pub mod macros;

pub mod _alloc {
    pub use ::alloc::vec;
}

// Geometry type hierarchy.
//
// These definitions are compiled unconditionally: they are additive and public,
// so they neither warn nor collide with the current geometry world (`types`).
// The migration switch is the `new-geometry` feature on `reearth-flow-types`,
// which selects this `Geometry` for `Feature.geometry`; the types here are not
// themselves feature-gated.
//
// The denormalized intermediate-data serialization is still future work: the
// types derive default `serde` so the enclosing `Feature` can serialize, which
// is not yet the byte-for-byte round-tripping intermediate form intended for
// that view.
pub mod appearance;
pub mod collection;
pub mod coordinate;
pub mod csg;
pub mod index;
pub mod line_string;
pub mod point;
pub mod point_cloud;
pub mod polygon;
pub mod polygon_mesh;
pub mod solid;
pub mod triangular_mesh;

use reearth_flow_common::attribute::Attributes;
use serde::{Deserialize, Serialize};

use collection::{Collection2D, Collection3D};
use csg::Csg;
use line_string::{LineString2D, LineString3D};
use point::{Point2D, Point3D};
use point_cloud::PointCloud;
use polygon::{Polygon2D, Polygon3D};
use polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
use solid::Solid;
use triangular_mesh::{TriangularMesh2D, TriangularMesh3D};

/// The top-level geometry type: an absent `None`, a geometry in one of the two
/// embedding dimensions, or a heterogeneous, cross-dimensional collection.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum Geometry {
    /// No geometry: a feature carrying attributes but no spatial payload. This
    /// is the default — an absent geometry, distinct from an empty collection.
    #[default]
    None,
    Euclidean2D(Euclidean2DGeometry),
    Euclidean3D(Euclidean3DGeometry),
    /// Heterogeneous, cross-dimensional, cross-frame.
    GeometryCollection(GeometryCollection),
}

/// Ordered members, each optionally carrying its own attributes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct GeometryCollection {
    members: Vec<Geometry>,
    /// Per-member attributes parallel to `members`; empty = no member carries
    /// any. Child-scoped: not exposed as the feature's own attributes.
    attrs: Vec<Attributes>,
}

/// 2D-embedded geometry. All coordinates are 2D `(x, y)`; some leaves carry an
/// optional per-vertex elevation (2.5D).
///
/// The heavy aggregate leaves (`Polygon`, the meshes) are boxed so the small,
/// common variants don't inflate the enum — and `Geometry` with them — to the
/// size of the largest leaf. The small tier (`Point`, `LineString`,
/// `Collection`) stays inline.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Euclidean2DGeometry {
    Point(Point2D),
    LineString(LineString2D),
    /// Exterior ring + optional holes.
    Polygon(Box<Polygon2D>),
    /// Indexed, variable face valence.
    PolygonMesh(Box<PolygonMesh2D>),
    /// Indexed, fixed 3-index stride (variable width).
    TriangularMesh(Box<TriangularMesh2D>),
    /// `Multi*` collection of 2D geometries; members may differ in coordinate frame.
    Collection(Collection2D),
}

/// 3D-embedded geometry. All coordinates are 3D `(x, y, z)`.
///
/// The heavy aggregate leaves (`PointCloud`, `Polygon`, the meshes, `Solid`) are
/// boxed so the small, common variants don't inflate the enum — and `Geometry`
/// with them — to the size of the largest leaf. The small tier (`Point`,
/// `LineString`, `Csg`, `Collection`) stays inline; `Csg` already boxes its own
/// operands.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Euclidean3DGeometry {
    Point(Point3D),
    PointCloud(Box<PointCloud>),
    LineString(LineString3D),
    /// Face in 3D space.
    Polygon(Box<Polygon3D>),
    /// Indexed, variable face valence.
    PolygonMesh(Box<PolygonMesh3D>),
    /// Indexed, fixed 3-index stride (variable width).
    TriangularMesh(Box<TriangularMesh3D>),
    /// Exterior + interior shells as coordless raw meshes; one frame on the Solid.
    Solid(Box<Solid>),
    /// Coordless boolean tree; frames come from its operand Solids.
    Csg(Csg),
    /// `Multi*` collection of 3D geometries; members may differ in coordinate frame.
    Collection(Collection3D),
}

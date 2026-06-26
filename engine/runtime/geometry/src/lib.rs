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
pub mod ops;
pub mod point;
pub mod point_cloud;
pub mod polygon;
pub mod polygon_mesh;
pub mod solid;
pub mod triangular_mesh;

#[cfg(test)]
mod test_support;

use enum_dispatch::enum_dispatch;
use reearth_flow_common::attribute::Attributes;
use serde::{Deserialize, Serialize};

use ops::triangulation::Cache;
use ops::{Aabb, BoundingBox, Triangulate, UnsupportedOperation};

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

impl GeometryCollection {
    /// Collect members, with no per-child attributes.
    pub fn new(members: impl IntoIterator<Item = Geometry>) -> Self {
        Self {
            members: members.into_iter().collect(),
            attrs: Vec::new(),
        }
    }

    /// Build with per-child attributes parallel to `members`. `attrs` must be empty
    /// or exactly one entry per member.
    pub fn with_attributes(
        members: Vec<Geometry>,
        attrs: Vec<Attributes>,
    ) -> Result<Self, error::Error> {
        if !attrs.is_empty() && attrs.len() != members.len() {
            return Err(error::Error::invalid_geometry(format!(
                "attribute count {} does not match member count {}",
                attrs.len(),
                members.len()
            )));
        }
        Ok(Self { members, attrs })
    }
}

/// 2D-embedded geometry. All coordinates are 2D `(x, y)`; some leaves carry an
/// optional per-vertex elevation (2.5D).
///
/// The heavy aggregate leaves (`Polygon`, the meshes) are boxed so the small,
/// common variants don't inflate the enum — and `Geometry` with them — to the
/// size of the largest leaf. The small tier (`Point`, `LineString`,
/// `Collection`) stays inline.
#[enum_dispatch(BoundingBox, Triangulate)]
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
#[enum_dispatch(BoundingBox, Triangulate)]
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

// `Geometry` and `GeometryCollection` are dispatched by hand rather than by
// `enum_dispatch`: `Geometry::None` is a unit variant (nothing to dispatch to)
// and a `GeometryCollection` holds a `Vec<Geometry>` (not a leaf). Both recurse
// over their children, mirroring the design's "GeometryCollection is the
// exception" note (§4.1).

impl BoundingBox for Geometry {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        match self {
            // An absent geometry has no extent, so no box.
            Geometry::None => Err(UnsupportedOperation {
                geometry: "Geometry::None",
                operation: "bounding_box",
            }),
            Geometry::Euclidean2D(g) => g.bounding_box(),
            Geometry::Euclidean3D(g) => g.bounding_box(),
            Geometry::GeometryCollection(c) => c.bounding_box(),
        }
    }
}

impl BoundingBox for GeometryCollection {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        ops::union_results(self.members.iter().map(Geometry::bounding_box)).ok_or(
            UnsupportedOperation {
                geometry: "GeometryCollection",
                operation: "bounding_box",
            },
        )
    }
}

impl Triangulate for Geometry {
    fn triangulate(&self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        match self {
            Geometry::None => Err(UnsupportedOperation {
                geometry: "Geometry::None",
                operation: "triangulate",
            }),
            Geometry::Euclidean2D(g) => g.triangulate(cache),
            Geometry::Euclidean3D(g) => g.triangulate(cache),
            Geometry::GeometryCollection(c) => c.triangulate(cache),
        }
    }
}

impl Triangulate for GeometryCollection {
    fn triangulate(&self, _cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        // Tessellation is defined per-primitive (Polygon / PolygonMesh, §4.2),
        // not over a collection; a caller triangulates members individually.
        Err(UnsupportedOperation {
            geometry: "GeometryCollection",
            operation: "triangulate",
        })
    }
}

#[cfg(test)]
mod bounding_box_tests {
    use super::*;
    use coordinate::Coordinate;
    use point::{Point2D, Point3D};
    use polygon::Polygon2D;

    #[test]
    fn dispatch_reaches_inline_leaf_through_dimension_enum() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            Coordinate::Euclidean,
            [1.0, 2.0, 3.0],
        )));
        assert_eq!(
            g.bounding_box().unwrap(),
            Aabb::D3 {
                min: [1.0, 2.0, 3.0],
                max: [1.0, 2.0, 3.0]
            }
        );
    }

    #[test]
    fn dispatch_reaches_boxed_leaf_through_dimension_enum() {
        // The `Box<Polygon2D>` variant exercises the `Box<T>` blanket impl.
        let p = Polygon2D::from_rings(
            Coordinate::Euclidean,
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(p)));
        assert_eq!(
            g.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [4.0, 4.0]
            }
        );
    }

    #[test]
    fn none_geometry_has_no_box() {
        assert!(Geometry::None.bounding_box().is_err());
    }

    #[test]
    fn geometry_collection_mixing_2d_and_3d_promotes_to_3d() {
        let p2 = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            Coordinate::Euclidean,
            [0.0, 0.0],
        )));
        let p3 = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            Coordinate::Euclidean,
            [4.0, 4.0, 9.0],
        )));
        let gc = Geometry::GeometryCollection(GeometryCollection::new([p2, p3]));
        // The 2D member is placed in z = 0, so the merged z-range is [0, 9].
        assert_eq!(
            gc.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [4.0, 4.0, 9.0]
            }
        );
    }

    #[test]
    fn nested_collection_with_only_empty_members_has_no_box() {
        let empty = Geometry::GeometryCollection(GeometryCollection::new([]));
        let outer = Geometry::GeometryCollection(GeometryCollection::new([Geometry::None, empty]));
        assert!(outer.bounding_box().is_err());
    }
}

#[cfg(test)]
mod triangulate_tests {
    use super::*;
    use coordinate::Coordinate;
    use point::Point2D;
    use polygon::Polygon2D;

    #[test]
    fn triangulate_dispatches_through_geometry_to_polygon() {
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let p = Polygon2D::from_rings(Coordinate::Euclidean, square, Vec::<Vec<[f64; 2]>>::new());
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(p)));
        let out = g.triangulate(&mut Cache::new()).unwrap();
        match out {
            Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(m)) => {
                assert_eq!(m.num_triangles(), 2);
            }
            other => panic!("expected a 2D triangular mesh, got {other:?}"),
        }
    }

    #[test]
    fn triangulate_is_unsupported_for_non_polygonal_types() {
        let point = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            Coordinate::Euclidean,
            [0.0, 0.0],
        )));
        assert!(point.triangulate(&mut Cache::new()).is_err());
        assert!(Geometry::None.triangulate(&mut Cache::new()).is_err());

        let collection = Geometry::GeometryCollection(GeometryCollection::new([]));
        assert!(collection.triangulate(&mut Cache::new()).is_err());
    }
}

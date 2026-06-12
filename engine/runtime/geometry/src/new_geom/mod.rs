//! Reference prototype for the new geometry type (design doc "GEOM: Design of
//! the New Geometry Type", §3.3 and §4). This is NOT wired into the production
//! geometry types; it exists so the design's dispatch model can be compiled and
//! pointed at.
//!
//! Shape under test:
//!
//! - The type hierarchy is two-level and nested:
//!   `Geometry` -> `Euclidean{2,3}DGeometry` -> concrete leaf.
//! - `enum_dispatch` is applied to all three enums. Because each variant is a
//!   single-field newtype wrapping a type that itself implements the operation
//!   traits, the dispatch composes (nested): a call on `Geometry` chains through
//!   the dimension enum to the leaf. The two inner enums need no hand-written
//!   bodies, only the `#[enum_dispatch(...)]` attribute.
//! - `coord` lives INSIDE each leaf, not in a wrapper. That is what lets a trait
//!   method read the frame from `self` without taking a `coord` argument (see
//!   `ops::reproject`). Composite leaves (`Solid`, `Csg`) hold coordless raw
//!   buffers and exactly one `coord`, so a composite can never carry several
//!   disagreeing frames.
//! - `GeometryCollection(Vec<Geometry>)` cannot be auto-dispatched (a `Vec` is
//!   not a leaf), so it is a concrete struct with hand-written recursive impls
//!   that slots into `Geometry` as a newtype variant.
//!
//! Boilerplate note: `enum_dispatch` requires every variant to implement every
//! listed trait. A trait default body does not opt a type in; an `impl ... {}`
//! block must still exist. The `unsupported!` macro below collapses those empty
//! blocks to one line per leaf.

use enum_dispatch::enum_dispatch;

pub mod coordinate;
pub mod ops;
pub mod raw;

pub mod collection;
pub mod csg;
pub mod linestring;
pub mod point;
pub mod solid;

pub use collection::GeometryCollection;
pub use coordinate::{Coordinate, EpsgCode};
pub use csg::Csg;
pub use linestring::LineString2D;
pub use ops::{Aabb, BoundingBox, GltfBuffer, Reproject, UnsupportedOperation, WriteGltf};
pub use point::{Point2D, Point3D};
pub use solid::Solid;

/// Stamp the mandatory empty `impl Trait for Type {}` blocks for operations a
/// leaf does not support, so the default `UnsupportedOperation` body fires.
///
/// `unsupported!(Csg: Reproject, WriteGltf);`
#[macro_export]
macro_rules! unsupported {
    ($ty:ty : $($tr:ident),+ $(,)?) => {
        $( impl $crate::new_geom::ops::$tr for $ty {} )+
    };
}

// ---- intermediate level: leaf enums, attribute only ----------------------

/// Leaves embedded in 2D `(x, y)` (with optional per-vertex elevation).
#[enum_dispatch(BoundingBox, Reproject, WriteGltf)]
#[derive(Clone, Debug)]
pub enum Euclidean2DGeometry {
    Point(Point2D),
    LineString(LineString2D),
}

/// Leaves embedded in 3D `(x, y, z)`. The genuinely 3D-only variants (`Solid`,
/// `Csg`) live only here.
#[enum_dispatch(BoundingBox, Reproject, WriteGltf)]
#[derive(Clone, Debug)]
pub enum Euclidean3DGeometry {
    Point(Point3D),
    Solid(Solid),
    Csg(Csg),
}

// ---- top level: the enum is the type; no coord wrapper -------------------

#[enum_dispatch(BoundingBox, Reproject, WriteGltf)]
#[derive(Clone, Debug)]
pub enum Geometry {
    Euclidean2D(Euclidean2DGeometry),
    Euclidean3D(Euclidean3DGeometry),
    /// heterogeneous, cross-dimensional
    GeometryCollection(GeometryCollection),
}

#[cfg(test)]
mod tests {
    use super::raw::{CsgNode, RawMesh};
    use super::*;

    fn mesh() -> RawMesh {
        RawMesh {
            verts: vec![[0.0, 0.0, 0.0], [2.0, 3.0, 4.0]],
            faces: vec![0, 1, 2],
        }
    }

    fn point2d() -> Geometry {
        Euclidean2DGeometry::from(Point2D {
            x: 1.0,
            y: 2.0,
            z: None,
            coord: Coordinate::Crs(4326),
        })
        .into()
    }

    fn solid() -> Geometry {
        Euclidean3DGeometry::from(Solid {
            exterior: mesh(),
            interiors: vec![],
            coord: Coordinate::Crs(4326),
        })
        .into()
    }

    fn csg() -> Geometry {
        Euclidean3DGeometry::from(Csg {
            root: CsgNode::Union(
                Box::new(CsgNode::Leaf(mesh())),
                Box::new(CsgNode::Leaf(mesh())),
            ),
            coord: Coordinate::Euclidean,
        })
        .into()
    }

    #[test]
    fn nested_dispatch_reaches_leaves() {
        assert_eq!(
            point2d().bounding_box().unwrap(),
            Aabb::point(1.0, 2.0, 0.0)
        );
        assert_eq!(
            solid().bounding_box().unwrap(),
            Aabb {
                min: [0.0, 0.0, 0.0],
                max: [2.0, 3.0, 4.0]
            }
        );
        // composite recursion over the coordless CSG tree
        assert_eq!(
            csg().bounding_box().unwrap(),
            Aabb {
                min: [0.0, 0.0, 0.0],
                max: [2.0, 3.0, 4.0]
            }
        );
    }

    #[test]
    fn reproject_reads_coord_from_leaf_and_mutates_in_place() {
        let mut g = point2d();
        g.reproject(3857).unwrap();
        match g {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                assert_eq!(p.coord, Coordinate::Crs(3857));
                assert_eq!((p.x, p.y), (1001.0, 1002.0));
            }
            _ => panic!("variant changed"),
        }
    }

    #[test]
    fn unsupported_default_fires_through_dispatch() {
        // Point3D / Csg left reproject unsupported via `unsupported!`.
        let mut g = csg();
        assert!(g.reproject(3857).is_err());
    }

    #[test]
    fn collection_recurses_with_per_child_coord() {
        let coll: Geometry = GeometryCollection(vec![point2d(), solid()]).into();
        assert_eq!(coll.num_children_bbox(), 2);
    }

    impl Geometry {
        fn num_children_bbox(&self) -> usize {
            match self {
                Geometry::GeometryCollection(c) => {
                    c.0.iter().filter(|g| g.bounding_box().is_ok()).count()
                }
                _ => usize::from(self.bounding_box().is_ok()),
            }
        }
    }

    #[test]
    fn write_gltf_folds_into_shared_sink() {
        let mut out = GltfBuffer::default();
        solid().write_gltf(&mut out).unwrap();
        point2d().write_gltf(&mut out).unwrap();
        assert_eq!(out.prim_count, 2);
    }
}

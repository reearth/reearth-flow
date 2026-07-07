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
#[cfg(feature = "new-geometry")]
pub mod validation_next;

#[cfg(test)]
mod test_support;

use enum_dispatch::enum_dispatch;
use reearth_flow_common::attribute::Attributes;
use serde::{Deserialize, Serialize};

use ops::triangulation::Cache;
use ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, Triangulate, UnsupportedOperation};
#[cfg(feature = "new-geometry")]
use validation_next::{Validate, ValidationReport, ValidationType};

use coordinate::EpsgCode;

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

    /// The members, mutable.
    pub(crate) fn members_mut(&mut self) -> &mut [Geometry] {
        &mut self.members
    }

    /// The number of members.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether the collection has no members.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// The members, in order.
    pub fn members(&self) -> &[Geometry] {
        &self.members
    }
}

/// 2D-embedded geometry. All coordinates are 2D `(x, y)`; some leaves carry an
/// optional per-vertex elevation (2.5D).
///
/// The heavy aggregate leaves (`Polygon`, the meshes) are boxed so the small,
/// common variants don't inflate the enum — and `Geometry` with them — to the
/// size of the largest leaf. The small tier (`Point`, `LineString`,
/// `Collection`) stays inline.
#[cfg_attr(
    not(feature = "new-geometry"),
    enum_dispatch(BoundingBox, Triangulate, Reproject)
)]
#[cfg_attr(
    feature = "new-geometry",
    enum_dispatch(BoundingBox, Triangulate, Reproject, Validate)
)]
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
#[cfg_attr(
    not(feature = "new-geometry"),
    enum_dispatch(BoundingBox, Triangulate, Reproject)
)]
#[cfg_attr(
    feature = "new-geometry",
    enum_dispatch(BoundingBox, Triangulate, Reproject, Validate)
)]
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
    fn triangulate(&mut self, cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
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
    fn triangulate(&mut self, _cache: &mut Cache) -> Result<Geometry, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: "GeometryCollection",
            operation: "triangulate",
        })
    }
}

impl Reproject for Geometry {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        match self {
            Geometry::None => Ok(()),
            Geometry::Euclidean2D(g) => g.reproject(target, cache),
            Geometry::Euclidean3D(g) => g.reproject(target, cache),
            Geometry::GeometryCollection(c) => c.reproject(target, cache),
        }
    }
}

impl Reproject for GeometryCollection {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        for member in self.members_mut() {
            member.reproject(target, cache)?;
        }
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl Validate for Geometry {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        match self {
            // An absent geometry has nothing to validate.
            Geometry::None => None,
            Geometry::Euclidean2D(g) => g.validate(valid_type),
            Geometry::Euclidean3D(g) => g.validate(valid_type),
            Geometry::GeometryCollection(c) => c.validate(valid_type),
        }
    }
}

#[cfg(feature = "new-geometry")]
impl Validate for GeometryCollection {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        for member in &self.members {
            if let Some(r) = member.validate(valid_type.clone()) {
                report.extend(r);
            }
        }
        report.into_option()
    }
}

#[cfg(test)]
mod bounding_box_tests {
    use super::*;
    use coordinate::CoordinateFrame;
    use point::{Point2D, Point3D};
    use polygon::Polygon2D;

    #[test]
    fn dispatch_reaches_inline_leaf_through_dimension_enum() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
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
            CoordinateFrame::Euclidean,
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
            CoordinateFrame::Euclidean,
            [0.0, 0.0],
        )));
        let p3 = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
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
    use coordinate::CoordinateFrame;
    use point::Point2D;
    use polygon::{Polygon2D, Polygon3D};
    use polygon_mesh::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
    use solid::Solid;
    use triangular_mesh::TriangularMesh3DData;

    /// A spread of supported inputs covering both embeddings, holes, elevation,
    /// multi-face meshes, and a degenerate face.
    fn sample_geometries() -> Vec<Geometry> {
        let e = CoordinateFrame::Euclidean;
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];

        let poly2d = Polygon2D::from_rings(e.clone(), square, Vec::<Vec<[f64; 2]>>::new());
        let poly2d_hole = Polygon2D::from_rings(e.clone(), square, vec![hole]);
        let poly2d_elev = Polygon2D::from_rings_with_elevation(
            e.clone(),
            [
                [0.0, 0.0, 1.0],
                [4.0, 0.0, 2.0],
                [4.0, 4.0, 3.0],
                [0.0, 0.0, 1.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let poly3d = Polygon3D::from_rings(
            e.clone(),
            [
                [0.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [0.0, 4.0, 4.0],
                [0.0, 0.0, 4.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let poly3d_degenerate = Polygon3D::from_rings(
            e.clone(),
            [
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                [2.0, 2.0, 2.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let mesh2d = PolygonMesh2D::from_parts(
            e.clone(),
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [4.0, 0.0],
                [4.0, 2.0],
            ],
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let mesh3d = PolygonMesh3D::from_parts(
            e.clone(),
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0u32, 1, 2, 3]],
        )
        .unwrap();
        // A solid: a quad polygon-mesh exterior shell + a triangle-mesh void.
        let solid = Solid::new(
            e.clone(),
            PolygonMesh3DData::from_parts(
                vec![
                    [0.0, 0.0, 0.0],
                    [2.0, 0.0, 0.0],
                    [2.0, 2.0, 0.0],
                    [0.0, 2.0, 0.0],
                ],
                vec![vec![0u32, 1, 2, 3]],
            )
            .unwrap(),
            vec![TriangularMesh3DData::from_parts(
                vec![[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.0, 6.0, 5.0]],
                [0u32, 1, 2],
            )
            .unwrap()
            .into()],
        );

        vec![
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(poly2d))),
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(poly2d_hole))),
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(poly2d_elev))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(poly3d))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(poly3d_degenerate))),
            Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(mesh2d))),
            Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(mesh3d))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid))),
        ]
    }

    #[test]
    fn cache_state_does_not_affect_output() {
        let geoms = sample_geometries();
        // Tessellation consumes its input, so every call works on a fresh clone.
        for target in &geoms {
            // The reference result, from a pristine cache.
            let expected = target.clone().triangulate(&mut Cache::new());

            // (a) A cache dirtied by every other input in turn.
            for dirty in &geoms {
                let mut cache = Cache::new();
                let _ = dirty.clone().triangulate(&mut cache);
                assert!(
                    target.clone().triangulate(&mut cache) == expected,
                    "result changed after dirtying the cache with {dirty:?}",
                );
            }

            // (b) A cache dirtied by the whole sequence (buffers grown + filled).
            let mut cache = Cache::new();
            for g in &geoms {
                let _ = g.clone().triangulate(&mut cache);
            }
            assert!(
                target.clone().triangulate(&mut cache) == expected,
                "result changed after running the full sequence through the cache",
            );

            // (c) The same target twice through one cache is idempotent.
            let mut cache = Cache::new();
            let first = target.clone().triangulate(&mut cache);
            let second = target.clone().triangulate(&mut cache);
            assert!(first == expected && second == expected);
        }
    }

    #[test]
    fn triangulate_dispatches_through_geometry_to_polygon() {
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let mut g = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(p)));
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
        let mut point = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Euclidean,
            [0.0, 0.0],
        )));
        assert!(point.triangulate(&mut Cache::new()).is_err());
        assert!(Geometry::None.triangulate(&mut Cache::new()).is_err());

        let mut collection = Geometry::GeometryCollection(GeometryCollection::new([]));
        assert!(collection.triangulate(&mut Cache::new()).is_err());
    }
}

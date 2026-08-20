#![recursion_limit = "2048"]
extern crate alloc;

#[cfg(all(feature = "schema", feature = "debug-geom-feature-write"))]
compile_error!(
    "the `schema` feature describes the production intermediate-data form; \
     disable `debug-geom-feature-write` to generate it"
);

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
pub mod overlay;
pub mod point;
pub mod point_cloud;
pub mod polygon;
pub mod polygon_mesh;
pub mod predicates;
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
use ops::{
    Aabb, BoundingBox, Coerce, CoercionTarget, ConvertFrame, CountHoles, ExtractHoles,
    ExtractedPart, ForceTwoDimension, ForceTwoDimensionError, RemoveAppearance, Reproject,
    ReprojectionCache, Translate, Triangulate, UnsupportedOperation,
};
// `ValidationParams` / `ValidationType` / `ValidationReport` are named by the
// `enum_dispatch`-generated `Validate` impls on the geometry enums, so they must
// be in scope here.
use ops::Split;
#[cfg(feature = "new-geometry")]
use ops::{Area, Footprint, FootprintError, FootprintPlane, FootprintSink};
#[cfg(feature = "new-geometry")]
use validation_next::{Validate, ValidationParams, ValidationReport, ValidationType};

use coordinate::{CoordinateFrame, EpsgCode};

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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Geometry"))]
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "schema", schemars(title = "Geometry collection"))]
pub struct GeometryCollection {
    #[cfg_attr(feature = "schema", schemars(title = "Members"))]
    members: Vec<Geometry>,
    /// Per-member attributes parallel to `members`; empty = no member carries
    /// any. Child-scoped: not exposed as the feature's own attributes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Vec<std::collections::HashMap<String, serde_json::Value>>")
    )]
    #[cfg_attr(feature = "schema", schemars(title = "Per-member attributes"))]
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

    /// Per-member attributes, parallel to [`members`](Self::members), or empty
    /// if no member carries any.
    pub fn member_attributes(&self) -> &[Attributes] {
        &self.attrs
    }
}

/// 2D-embedded geometry. All coordinates are 2D `(x, y)`; some leaves carry a
/// single optional elevation the whole leaf lies at (2.5D).
///
/// The heavy aggregate leaves (`Polygon`, the meshes) are boxed so the small,
/// common variants don't inflate the enum — and `Geometry` with them — to the
/// size of the largest leaf. The small tier (`Point`, `LineString`,
/// `Collection`) stays inline.
#[cfg_attr(
    not(feature = "new-geometry"),
    enum_dispatch(
        BoundingBox,
        Triangulate,
        Reproject,
        Coerce,
        ConvertFrame,
        Translate,
        Split,
        ForceTwoDimension,
        RemoveAppearance,
        CountHoles,
        ExtractHoles
    )
)]
#[cfg_attr(
    feature = "new-geometry",
    enum_dispatch(
        BoundingBox,
        Triangulate,
        Reproject,
        Coerce,
        Validate,
        ConvertFrame,
        Translate,
        Split,
        ForceTwoDimension,
        RemoveAppearance,
        CountHoles,
        ExtractHoles,
        Footprint,
        Area
    )
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "2D geometry"))]
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
    enum_dispatch(
        BoundingBox,
        Triangulate,
        Reproject,
        Coerce,
        ConvertFrame,
        Translate,
        Split,
        ForceTwoDimension,
        RemoveAppearance,
        CountHoles,
        ExtractHoles
    )
)]
#[cfg_attr(
    feature = "new-geometry",
    enum_dispatch(
        BoundingBox,
        Triangulate,
        Reproject,
        Coerce,
        Validate,
        ConvertFrame,
        Translate,
        Split,
        ForceTwoDimension,
        RemoveAppearance,
        CountHoles,
        ExtractHoles,
        Footprint,
        Area
    )
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "3D geometry"))]
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

impl Euclidean2DGeometry {
    /// Whether any part of this geometry lies at an elevation (2.5D).
    pub(crate) fn carries_elevation(&self) -> bool {
        match self {
            Self::Point(_) => false,
            Self::LineString(g) => g.elevation().is_some(),
            Self::Polygon(g) => g.elevation().is_some(),
            Self::PolygonMesh(g) => g.elevation().is_some(),
            Self::TriangularMesh(g) => g.elevation().is_some(),
            Self::Collection(c) => c.members().iter().any(Self::carries_elevation),
        }
    }

    /// Whether converting to `target` reprojects this geometry across CRSs.
    pub(crate) fn reprojects_to(
        &self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
    ) -> crate::error::Result<bool> {
        let frame = match self {
            Self::Point(g) => g.frame(),
            Self::LineString(g) => g.frame(),
            Self::Polygon(g) => g.frame(),
            Self::PolygonMesh(g) => g.frame(),
            Self::TriangularMesh(g) => g.frame(),
            Self::Collection(c) => {
                for member in c.members() {
                    if member.reprojects_to(target, base_point)? {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }
        };
        Ok(matches!(
            ops::plan_frame_step(frame, target, base_point)?,
            ops::FrameStep::Reproject(_)
        ))
    }

    /// The 3D counterpart of this geometry, with every coordinate placed at the
    /// elevation its leaf lies at, or at `0.0` where there is none.
    pub(crate) fn into_3d(self) -> Euclidean3DGeometry {
        match self {
            Self::Point(g) => Euclidean3DGeometry::Point(g.into_3d()),
            Self::LineString(g) => Euclidean3DGeometry::LineString(g.into_3d()),
            Self::Polygon(g) => Euclidean3DGeometry::Polygon(Box::new(g.into_3d())),
            Self::PolygonMesh(g) => Euclidean3DGeometry::PolygonMesh(Box::new(g.into_3d())),
            Self::TriangularMesh(g) => Euclidean3DGeometry::TriangularMesh(Box::new(g.into_3d())),
            Self::Collection(c) => Euclidean3DGeometry::Collection(c.into_3d()),
        }
    }
}

impl Reproject for Geometry {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match self {
            Geometry::None => Ok(Geometry::None),
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
    ) -> crate::error::Result<Geometry> {
        let mut out = std::mem::take(self);
        for member in out.members.iter_mut() {
            *member = member.reproject(target, cache)?;
        }
        Ok(Geometry::GeometryCollection(out))
    }
}

impl ConvertFrame for Geometry {
    fn convert_frame(
        &mut self,
        target: &coordinate::CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match self {
            Geometry::None => Ok(Geometry::None),
            Geometry::Euclidean2D(g) => g.convert_frame(target, base_point, cache),
            Geometry::Euclidean3D(g) => g.convert_frame(target, base_point, cache),
            Geometry::GeometryCollection(c) => c.convert_frame(target, base_point, cache),
        }
    }
}

impl ConvertFrame for GeometryCollection {
    fn convert_frame(
        &mut self,
        target: &coordinate::CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let mut out = std::mem::take(self);
        for member in out.members.iter_mut() {
            *member = member.convert_frame(target, base_point, cache)?;
        }
        Ok(Geometry::GeometryCollection(out))
    }
}

impl Translate for Geometry {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        match self {
            Geometry::None => Ok(()),
            Geometry::Euclidean2D(g) => g.translate(delta),
            Geometry::Euclidean3D(g) => g.translate(delta),
            Geometry::GeometryCollection(c) => c.translate(delta),
        }
    }
}

impl Translate for GeometryCollection {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        for member in self.members_mut() {
            member.translate(delta)?;
        }
        Ok(())
    }
}

impl RemoveAppearance for Geometry {
    fn remove_appearance(&mut self) {
        match self {
            Geometry::None => {}
            Geometry::Euclidean2D(g) => g.remove_appearance(),
            Geometry::Euclidean3D(g) => g.remove_appearance(),
            Geometry::GeometryCollection(c) => c.remove_appearance(),
        }
    }
}

impl RemoveAppearance for GeometryCollection {
    fn remove_appearance(&mut self) {
        for member in self.members_mut() {
            member.remove_appearance();
        }
    }
}

impl CountHoles for Geometry {
    fn count_holes(&self) -> usize {
        match self {
            // An absent geometry has no faces, so no holes.
            Geometry::None => 0,
            Geometry::Euclidean2D(g) => g.count_holes(),
            Geometry::Euclidean3D(g) => g.count_holes(),
            Geometry::GeometryCollection(c) => c.count_holes(),
        }
    }
}

impl CountHoles for GeometryCollection {
    fn count_holes(&self) -> usize {
        self.members.iter().map(Geometry::count_holes).sum()
    }
}

impl ExtractHoles for Geometry {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        match self {
            // An absent geometry has no faces to take apart.
            Geometry::None => Err(UnsupportedOperation {
                geometry: "Geometry::None",
                operation: "extract_holes",
            }),
            Geometry::Euclidean2D(g) => g.extract_holes(emit),
            Geometry::Euclidean3D(g) => g.extract_holes(emit),
            Geometry::GeometryCollection(c) => c.extract_holes(emit),
        }
    }
}

impl ExtractHoles for GeometryCollection {
    /// Deaggregate: each member is taken apart on its own, and one that is not
    /// area geometry is emitted as [`ExtractedPart::Rejected`] rather than failing
    /// the whole collection.
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        for member in &self.members {
            if member.extract_holes(emit).is_err() {
                emit(member.clone(), ExtractedPart::Rejected);
            }
        }
        Ok(())
    }
}

impl Split for Geometry {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        match self {
            Geometry::None => Err(UnsupportedOperation {
                geometry: "Geometry::None",
                operation: "split",
            }),
            Geometry::Euclidean2D(g) => g.split(emit),
            Geometry::Euclidean3D(g) => g.split(emit),
            Geometry::GeometryCollection(c) => c.split(emit),
        }
    }
}

impl Split for GeometryCollection {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        ops::split::emit_members(
            std::mem::take(&mut self.members),
            std::mem::take(&mut self.attrs),
            emit,
        );
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for Geometry {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        match self {
            Geometry::None => Ok(()),
            Geometry::Euclidean2D(g) => g.footprint(sink),
            Geometry::Euclidean3D(g) => g.footprint(sink),
            Geometry::GeometryCollection(c) => c.footprint(sink),
        }
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for GeometryCollection {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        self.members.iter().try_for_each(|m| m.footprint(sink))
    }
}

#[cfg(feature = "new-geometry")]
impl Geometry {
    /// The footprint of this geometry on `plane`: every face projected and
    /// dissolved into its union, curves and points projected as they are, as 2D
    /// geometry in the plane's frame. See [`Footprint`] and
    /// [`FootprintSink::finish`] for the contract, and [`FootprintPlane`] for
    /// the frame each plane needs.
    pub fn footprint_on(&self, plane: &FootprintPlane) -> Result<Geometry, FootprintError> {
        let mut sink = FootprintSink::new(plane);
        self.footprint(&mut sink)?;
        sink.finish()
    }
}

impl Geometry {
    /// Force this geometry into a 2D embedding by dropping the Z coordinate,
    /// recursing into collection members. All-or-nothing: one member that cannot
    /// be flattened fails the whole geometry rather than being dropped.
    ///
    /// See [`ForceTwoDimension`] for the frame and elevation contract. Like the
    /// trait, this consumes coordinate buffers, leaving `self` moved-from on
    /// success.
    pub fn force_2d(&mut self) -> Result<Geometry, ForceTwoDimensionError> {
        match self {
            Geometry::None => Ok(Geometry::None),
            Geometry::Euclidean2D(g) => Ok(Geometry::Euclidean2D(g.force_2d()?)),
            Geometry::Euclidean3D(g) => Ok(Geometry::Euclidean2D(g.force_2d()?)),
            Geometry::GeometryCollection(c) => Ok(Geometry::GeometryCollection(c.force_2d()?)),
        }
    }
}

impl GeometryCollection {
    /// Force every member to 2D. Members may differ in coordinate frame, so each
    /// is demoted on its own.
    fn force_2d(&mut self) -> Result<GeometryCollection, ForceTwoDimensionError> {
        let mut members = Vec::with_capacity(self.members.len());
        for member in &mut self.members {
            members.push(member.force_2d()?);
        }
        Ok(GeometryCollection {
            members,
            attrs: std::mem::take(&mut self.attrs),
        })
    }
}

impl Coerce for Geometry {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match self {
            // An absent geometry has no vertices to re-represent.
            Geometry::None => Err(UnsupportedOperation {
                geometry: "Geometry::None",
                operation: "coerce",
            }),
            Geometry::Euclidean2D(g) => g.coerce(target, cache),
            Geometry::Euclidean3D(g) => g.coerce(target, cache),
            Geometry::GeometryCollection(c) => c.coerce(target, cache),
        }
    }
}

impl Coerce for GeometryCollection {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        let mut changed = false;
        for member in self.members_mut() {
            if let Ok(coerced) = member.coerce(target, cache) {
                *member = coerced;
                changed = true;
            }
        }
        if !changed {
            return Err(UnsupportedOperation {
                geometry: "GeometryCollection",
                operation: "coerce",
            });
        }
        Ok(Geometry::GeometryCollection(std::mem::take(self)))
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
        let poly2d_elev = Polygon2D::from_rings_at_elevation(
            e.clone(),
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
            1.0,
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

#[cfg(test)]
mod force_2d_tests {
    use super::*;
    use appearance::{Material, PhongMaterial, ThemeId};
    use collection::Collection3D;
    use coordinate::{CoordinateFrame, EpsgCode};
    use line_string::{LineString2D, LineString3D};
    use point::Point3D;
    use polygon::Polygon3D;
    use polygon_mesh::PolygonMesh3DData;
    use reearth_flow_common::attribute::{Attribute, AttributeValue};
    use solid::Solid;
    use std::sync::Arc;
    use triangular_mesh::TriangularMesh3DData;

    /// EPSG:6697 (JGD2011 + height) — the compound CRS PLATEAU CityGML uses.
    fn crs() -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(6697))
    }

    /// EPSG:6668 (JGD2011) — the horizontal component of [`crs`], so the frame
    /// every geometry below must carry once flattened.
    fn crs_2d() -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(6668))
    }

    /// A colour-only Phong material (no textures, so no UV is required).
    fn plain_material() -> Material {
        Material::Phong(PhongMaterial {
            diffuse: [1.0, 0.0, 0.0],
            specular: [0.0; 3],
            emissive: [0.0; 3],
            ambient_intensity: 0.0,
            shininess: 0.0,
            transparency: 0.0,
            diffuse_map: None,
            emissive_map: None,
            normal_map: None,
        })
    }

    #[test]
    fn point3d_drops_z_and_demotes_the_frame() {
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            crs(),
            [1.0, 2.0, 3.0],
        )));
        match g.force_2d().unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                // x and y survive verbatim; only z and the frame's vertical axis go.
                assert_eq!(p.position(), [1.0, 2.0]);
                assert_eq!(p.frame(), &crs_2d());
            }
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    #[test]
    fn geocentric_frame_is_rejected() {
        // Dropping a geocentric CRS's third axis projects onto the equatorial
        // plane rather than removing a height, so the geometry must be rejected
        // instead of silently retagged.
        for code in [4978u16, 6666] {
            let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Crs(EpsgCode::new(code)),
                [-3_957_314.0, 3_310_254.0, 3_737_540.0],
            )));
            let err = g.force_2d().unwrap_err();
            assert!(
                matches!(&err, ForceTwoDimensionError::UnsupportedFrame(e) if e.epsg.get() == code),
                "EPSG:{code} should be rejected for its frame, got {err:?}"
            );
        }
    }

    #[test]
    fn linestring3d_drops_z() {
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(crs(), [[0.0, 0.0, 5.0], [2.0, 1.0, 9.0]]),
        ));
        match g.force_2d().unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(ls)) => {
                assert_eq!(ls.coords(), &[[0.0, 0.0], [2.0, 1.0]]);
                assert_eq!(ls.frame(), &crs_2d());
            }
            other => panic!("expected a 2D line string, got {other:?}"),
        }
    }

    #[test]
    fn two_and_a_half_d_elevation_is_cleared_and_idempotent() {
        // A 2.5D input loses its elevation and its frame's vertical axis, giving
        // exactly what a natively 2D line string in the demoted frame would.
        let mut g = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords_at_elevation(crs(), [[0.0, 0.0], [2.0, 1.0]], 5.0),
        ));
        let forced = g.force_2d().unwrap();
        let expected = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(crs_2d(), [[0.0, 0.0], [2.0, 1.0]]),
        ));
        assert_eq!(forced, expected);
        // The demoted frame demotes to itself, so a second pass is a no-op.
        let mut forced2 = forced.clone();
        assert_eq!(forced2.force_2d().unwrap(), expected);
    }

    #[test]
    fn polygon3d_preserves_rings_holes_and_appearance() {
        let exterior = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 1.0],
            [4.0, 4.0, 2.0],
            [0.0, 4.0, 3.0],
            [0.0, 0.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let mut poly = Polygon3D::from_rings(crs(), exterior, vec![hole]);
        poly.set_appearance(ThemeId(Arc::from("t")), plain_material(), None)
            .unwrap();
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(poly)));
        match g.force_2d().unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => {
                assert_eq!(p.frame(), &crs_2d());
                assert_eq!(p.exterior().len(), 5);
                assert_eq!(p.interiors().count(), 1);
                assert!(p.appearance().is_some(), "appearance must survive");
            }
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    #[test]
    fn vertical_polygon_projects_to_a_degenerate_footprint() {
        // A wall in the x = 0 plane collapses onto the y axis; the conversion
        // still succeeds (degeneracy is allowed, matching FME).
        let wall = [
            [0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 4.0, 4.0],
            [0.0, 0.0, 4.0],
            [0.0, 0.0, 0.0],
        ];
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                wall,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        match g.force_2d().unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => {
                // All x are 0: the footprint has zero area but is still returned.
                assert!(p.exterior().iter().all(|&[x, _]| x == 0.0));
            }
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    #[test]
    fn collection_forces_members_and_demotes_each_frame() {
        // Members may differ in frame, so each is demoted on its own; a
        // Euclidean member has no vertical axis to shed and stays as it is.
        let a = Euclidean3DGeometry::Point(Point3D::new(crs(), [1.0, 2.0, 3.0]));
        let b =
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [4.0, 5.0, 6.0]));
        let mut g =
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([a, b])));
        match g.force_2d().unwrap() {
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)) => {
                assert_eq!(c.members().len(), 2);
                let frames: Vec<_> = c
                    .members()
                    .iter()
                    .map(|m| match m {
                        Euclidean2DGeometry::Point(p) => p.frame().clone(),
                        other => panic!("expected a 2D point member, got {other:?}"),
                    })
                    .collect();
                assert_eq!(frames[0], crs_2d());
                assert_eq!(frames[1], CoordinateFrame::Euclidean);
            }
            other => panic!("expected a 2D collection, got {other:?}"),
        }
    }

    fn sample_solid() -> Solid {
        Solid::new(
            CoordinateFrame::Euclidean,
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
        )
    }

    #[test]
    fn solid_has_no_2d_counterpart() {
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(sample_solid())));
        assert!(g.force_2d().is_err());
    }

    #[test]
    fn collection_is_all_or_nothing_on_an_unsupported_member() {
        let point = Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0; 3]));
        let solid = Euclidean3DGeometry::Solid(Box::new(sample_solid()));
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
            point, solid,
        ])));
        assert!(g.force_2d().is_err());
    }

    #[test]
    fn none_passes_through() {
        let mut g = Geometry::None;
        assert_eq!(g.force_2d().unwrap(), Geometry::None);
    }

    #[test]
    fn geometry_collection_forces_members_and_keeps_attributes() {
        let members = vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                crs(),
                [1.0, 2.0, 3.0],
            ))),
            Geometry::None,
        ];
        let attrs = vec![
            Attributes::from([(Attribute::new("a"), AttributeValue::Number(1.into()))]),
            Attributes::from([(Attribute::new("b"), AttributeValue::Number(2.into()))]),
        ];
        let mut g = Geometry::GeometryCollection(
            GeometryCollection::with_attributes(members, attrs.clone()).unwrap(),
        );
        match g.force_2d().unwrap() {
            Geometry::GeometryCollection(c) => {
                assert_eq!(c.member_attributes(), attrs.as_slice());
                match c.members() {
                    [Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)), Geometry::None] => {
                        assert_eq!(p.position(), [1.0, 2.0]);
                        assert_eq!(p.frame(), &crs_2d());
                    }
                    other => panic!("expected a 2D point then None, got {other:?}"),
                }
            }
            other => panic!("expected a geometry collection, got {other:?}"),
        }
    }

    #[test]
    fn geometry_collection_is_all_or_nothing() {
        let members = vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                crs(),
                [1.0, 2.0, 3.0],
            ))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(sample_solid()))),
        ];
        let mut g = Geometry::GeometryCollection(GeometryCollection::new(members));
        assert!(g.force_2d().is_err());
    }
}

#[cfg(test)]
mod remove_appearance_tests {
    use super::*;
    use coordinate::CoordinateFrame;
    use point::Point3D;
    use polygon_mesh::PolygonMesh3DData;
    use solid::{Shell, Solid};
    use test_support::{textured, theme, uv};
    use triangular_mesh::TriangularMesh3D;

    /// A one-triangle mesh carrying a textured appearance.
    fn textured_mesh() -> TriangularMesh3D {
        let mut mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        mesh.set_appearance(theme("rgb"), textured(), Some(uv(3)))
            .unwrap();
        mesh
    }

    /// A one-quad polygon-mesh shell carrying a textured appearance.
    fn textured_shell() -> PolygonMesh3DData {
        let mut face = polygon::Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        face.set_appearance(theme("rgb"), textured(), Some(uv(4)))
            .unwrap();
        PolygonMesh3DData::from_polygons([&face])
    }

    /// Whether the shell's mesh carries an appearance.
    fn shell_appearance(shell: &Shell) -> bool {
        match shell {
            Shell::PolygonMesh(data) => {
                polygon_mesh::PolygonMesh3D::new(CoordinateFrame::Euclidean, data.clone())
                    .appearance()
                    .is_some()
            }
            Shell::TriangularMesh(data) => {
                TriangularMesh3D::new(CoordinateFrame::Euclidean, data.clone())
                    .appearance()
                    .is_some()
            }
        }
    }

    #[test]
    fn a_surface_leaf_loses_its_appearance() {
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            textured_mesh(),
        )));
        g.remove_appearance();
        let Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) = g else {
            panic!("expected a 3D triangular mesh");
        };
        assert!(mesh.appearance().is_none());
        assert_eq!(mesh.num_triangles(), 1);
        assert_eq!(mesh.vertices().len(), 3);
    }

    #[test]
    fn every_shell_of_a_solid_loses_its_appearance() {
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            textured_shell(),
            vec![Shell::from(textured_shell())],
        );
        assert!(shell_appearance(solid.exterior()));
        assert!(shell_appearance(&solid.interiors()[0]));

        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid)));
        g.remove_appearance();
        let Geometry::Euclidean3D(Euclidean3DGeometry::Solid(solid)) = g else {
            panic!("expected a solid");
        };
        assert!(!shell_appearance(solid.exterior()));
        assert!(!shell_appearance(&solid.interiors()[0]));
    }

    #[test]
    fn a_nested_csg_tree_loses_the_appearance_of_every_operand() {
        let solid = || Solid::from_exterior(CoordinateFrame::Euclidean, textured_shell());
        let inner = csg::Csg::union(solid(), solid());
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg::Csg::difference(
            inner,
            solid(),
        )));
        g.remove_appearance();
        let Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg)) = g else {
            panic!("expected a CSG tree");
        };
        let mut shells = Vec::new();
        collect_shells(&csg, &mut shells);
        assert_eq!(shells.len(), 3);
        assert!(shells.iter().all(|has_appearance| !has_appearance));
    }

    /// Collect, for every solid reachable from `csg`, whether it carries appearance.
    fn collect_shells(csg: &csg::Csg, out: &mut Vec<bool>) {
        let (left, right) = match csg {
            csg::Csg::Union(a, b) | csg::Csg::Intersection(a, b) | csg::Csg::Difference(a, b) => {
                (a, b)
            }
        };
        for operand in [left, right] {
            match &**operand {
                csg::ThreeDimensional::Solid(s) => out.push(shell_appearance(s.exterior())),
                csg::ThreeDimensional::Csg(c) => collect_shells(c, out),
            }
        }
    }

    #[test]
    fn removal_reaches_through_nested_collections() {
        let member = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
            textured_mesh(),
        )));
        let inner = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            collection::Collection3D::new([Euclidean3DGeometry::TriangularMesh(Box::new(
                textured_mesh(),
            ))]),
        ));
        let mut g = Geometry::GeometryCollection(GeometryCollection::new([member, inner]));
        g.remove_appearance();

        let Geometry::GeometryCollection(outer) = g else {
            panic!("expected a geometry collection");
        };
        let meshes = outer.members().iter().flat_map(|m| match m {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) => {
                vec![mesh.appearance().is_some()]
            }
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) => c
                .members()
                .iter()
                .map(|m| match m {
                    Euclidean3DGeometry::TriangularMesh(mesh) => mesh.appearance().is_some(),
                    other => panic!("unexpected member {other:?}"),
                })
                .collect(),
            other => panic!("unexpected member {other:?}"),
        });
        assert_eq!(meshes.collect::<Vec<_>>(), vec![false, false]);
    }

    #[test]
    fn a_geometry_that_carries_no_appearance_is_left_alone() {
        let point = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        let mut g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(point.clone()));
        g.remove_appearance();
        assert_eq!(g, Geometry::Euclidean3D(Euclidean3DGeometry::Point(point)));

        let mut none = Geometry::None;
        none.remove_appearance();
        assert_eq!(none, Geometry::None);
    }
}

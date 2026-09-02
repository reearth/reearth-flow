//! Per-embedding collections.
//!
//! Each embedding's `Collection` holds primitives of the same intrinsic
//! dimension with no shared vertex topology (equivalent to `Multi*` in
//! GeoJSON/GML). Members are not required to share a coordinate frame: every
//! leaf carries its own `frame`. Both collections carry per-child
//! attributes (`attrs`, parallel to `members`), used to preserve a child's
//! attributes; they are not exposed as the feature's own attributes.

use reearth_flow_common::attribute::Attributes;
use serde::{Deserialize, Serialize};

use crate::coordinate::EpsgCode;
use crate::error::Error;
use crate::ops::coerce::unchanged;
use crate::ops::triangulation::Cache;
use crate::ops::union_results;
use crate::ops::{
    Aabb, BoundingBox, Coerce, CoercionTarget, ForceTwoDimension, ForceTwoDimensionError,
    Reproject, ReprojectionCache, UnsupportedOperation,
};
#[cfg(feature = "new-geometry")]
use crate::ops::{Elevation, Footprint, FootprintError, FootprintSink};
#[cfg(feature = "new-geometry")]
use crate::validation_next::Validate;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// A `Multi*` collection of 2D geometries; members may differ in coordinate frame.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "schema", schemars(title = "2D collection"))]
pub struct Collection2D {
    #[cfg_attr(feature = "schema", schemars(title = "Members"))]
    members: Vec<Euclidean2DGeometry>,
    /// Per-member attributes, parallel to `members`; empty = no member carries
    /// any. Child-scoped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Vec<std::collections::HashMap<String, serde_json::Value>>")
    )]
    #[cfg_attr(feature = "schema", schemars(title = "Per-member attributes"))]
    attrs: Vec<Attributes>,
}

/// A `Multi*` collection of 3D geometries; members may differ in coordinate frame.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "schema", schemars(title = "3D collection"))]
pub struct Collection3D {
    #[cfg_attr(feature = "schema", schemars(title = "Members"))]
    members: Vec<Euclidean3DGeometry>,
    /// Per-member attributes, parallel to `members`; empty = no member carries
    /// any. Child-scoped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Vec<std::collections::HashMap<String, serde_json::Value>>")
    )]
    #[cfg_attr(feature = "schema", schemars(title = "Per-member attributes"))]
    attrs: Vec<Attributes>,
}

/// Validate that `attrs` is either empty or exactly parallel to `members`.
fn check_attrs<T>(members: &[T], attrs: &[Attributes]) -> Result<(), Error> {
    if !attrs.is_empty() && attrs.len() != members.len() {
        return Err(Error::invalid_geometry(format!(
            "attribute count {} does not match member count {}",
            attrs.len(),
            members.len()
        )));
    }
    Ok(())
}

impl Collection2D {
    /// Collect members, with no per-child attributes.
    pub fn new(members: impl IntoIterator<Item = Euclidean2DGeometry>) -> Self {
        Self {
            members: members.into_iter().collect(),
            attrs: Vec::new(),
        }
    }

    /// Build with per-child attributes parallel to `members`. `attrs` must be empty
    /// or exactly one entry per member.
    pub fn with_attributes(
        members: Vec<Euclidean2DGeometry>,
        attrs: Vec<Attributes>,
    ) -> Result<Self, Error> {
        check_attrs(&members, &attrs)?;
        Ok(Self { members, attrs })
    }

    /// The members, mutable.
    pub(crate) fn members_mut(&mut self) -> &mut [Euclidean2DGeometry] {
        &mut self.members
    }

    /// The members, in order.
    pub fn members(&self) -> &[Euclidean2DGeometry] {
        &self.members
    }

    /// Per-member attributes, parallel to [`members`](Self::members), or empty
    /// if no member carries any.
    pub fn member_attributes(&self) -> &[Attributes] {
        &self.attrs
    }
}

impl Collection3D {
    /// Collect members, with no per-child attributes.
    pub fn new(members: impl IntoIterator<Item = Euclidean3DGeometry>) -> Self {
        Self {
            members: members.into_iter().collect(),
            attrs: Vec::new(),
        }
    }

    /// Build with per-child attributes parallel to `members`. `attrs` must be empty
    /// or exactly one entry per member.
    pub fn with_attributes(
        members: Vec<Euclidean3DGeometry>,
        attrs: Vec<Attributes>,
    ) -> Result<Self, Error> {
        check_attrs(&members, &attrs)?;
        Ok(Self { members, attrs })
    }

    /// The members, mutable.
    pub(crate) fn members_mut(&mut self) -> &mut [Euclidean3DGeometry] {
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
    pub fn members(&self) -> &[Euclidean3DGeometry] {
        &self.members
    }

    /// Per-member attributes, parallel to [`members`](Self::members), or empty
    /// if no member carries any.
    pub fn member_attributes(&self) -> &[Attributes] {
        &self.attrs
    }
}

impl BoundingBox for Collection2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        union_results(self.members.iter().map(|m| m.bounding_box())).ok_or(UnsupportedOperation {
            geometry: "Collection2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for Collection3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        union_results(self.members.iter().map(|m| m.bounding_box())).ok_or(UnsupportedOperation {
            geometry: "Collection3D",
            operation: "bounding_box",
        })
    }
}

impl Collection2D {
    /// The 3D counterpart of this collection: members carrying no elevation are
    /// placed at `0.0`.
    pub(crate) fn into_3d(self) -> Collection3D {
        Collection3D {
            members: self.members.into_iter().map(|m| m.into_3d()).collect(),
            attrs: self.attrs,
        }
    }
}

impl Collection2D {
    /// Whether any member lies at an elevation.
    fn carries_elevation(&self) -> bool {
        self.members
            .iter()
            .any(Euclidean2DGeometry::carries_elevation)
    }
}

/// Unwrap a member's converted result back to a 2D geometry.
fn expect_2d(g: Geometry) -> Result<Euclidean2DGeometry, Error> {
    match g {
        Geometry::Euclidean2D(g) => Ok(g),
        other => Err(Error::projection(format!(
            "a member of a pure 2D collection did not stay 2D: {other:?}"
        ))),
    }
}

/// Unwrap a member's converted result back to a 3D geometry.
fn expect_3d(g: Geometry) -> Result<Euclidean3DGeometry, Error> {
    match g {
        Geometry::Euclidean3D(g) => Ok(g),
        other => Err(Error::projection(format!(
            "a member of a 3D collection did not stay 3D: {other:?}"
        ))),
    }
}

impl Reproject for Collection2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        if self.carries_elevation() {
            return std::mem::take(self).into_3d().reproject(target, cache);
        }
        let mut out = std::mem::take(self);
        for member in out.members.iter_mut() {
            *member = expect_2d(member.reproject(target, cache)?)?;
        }
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(out)))
    }
}

impl Reproject for Collection3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let mut out = std::mem::take(self);
        for member in out.members.iter_mut() {
            *member = expect_3d(member.reproject(target, cache)?)?;
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(out)))
    }
}

// Tessellation is defined per-primitive, not over a collection.
crate::unsupported!(Collection2D: Triangulate);
crate::unsupported!(Collection3D: Triangulate);

impl crate::ops::ConvertFrame for Collection2D {
    fn convert_frame(
        &mut self,
        target: &crate::coordinate::CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut crate::ops::ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let mut reprojects = false;
        for member in self.members.iter() {
            reprojects |= member.reprojects_to(target, base_point)?;
        }
        if reprojects && self.carries_elevation() {
            return std::mem::take(self)
                .into_3d()
                .convert_frame(target, base_point, cache);
        }
        let mut out = std::mem::take(self);
        for member in out.members.iter_mut() {
            *member = expect_2d(member.convert_frame(target, base_point, cache)?)?;
        }
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(out)))
    }
}

impl crate::ops::ConvertFrame for Collection3D {
    fn convert_frame(
        &mut self,
        target: &crate::coordinate::CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut crate::ops::ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let mut out = std::mem::take(self);
        for member in out.members.iter_mut() {
            *member = expect_3d(member.convert_frame(target, base_point, cache)?)?;
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(out)))
    }
}

impl crate::ops::Translate for Collection2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        for member in self.members_mut() {
            member.translate(delta)?;
        }
        Ok(())
    }
}

impl crate::ops::Translate for Collection3D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        for member in self.members_mut() {
            member.translate(delta)?;
        }
        Ok(())
    }
}

impl crate::ops::RemoveAppearance for Collection2D {
    fn remove_appearance(&mut self) {
        for member in self.members_mut() {
            member.remove_appearance();
        }
    }
}

impl crate::ops::RemoveAppearance for Collection3D {
    fn remove_appearance(&mut self) {
        for member in self.members_mut() {
            member.remove_appearance();
        }
    }
}

impl crate::ops::CountHoles for Collection2D {
    fn count_holes(&self) -> usize {
        self.members()
            .iter()
            .map(Euclidean2DGeometry::count_holes)
            .sum()
    }
}

impl crate::ops::CountHoles for Collection3D {
    fn count_holes(&self) -> usize {
        self.members()
            .iter()
            .map(Euclidean3DGeometry::count_holes)
            .sum()
    }
}

#[cfg(feature = "new-geometry")]
impl crate::ops::Area for Collection2D {
    /// The measurable members' areas, summed. An unmeasurable member is skipped
    /// rather than failing its siblings; [`area_report`](crate::ops::area::area_report)
    /// counts the skips so a caller can say how many there were. An empty
    /// collection measures zero.
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        Ok(self
            .members
            .iter()
            .filter_map(|m| m.surface_area().ok())
            .sum())
    }
}

#[cfg(feature = "new-geometry")]
impl crate::ops::Area for Collection3D {
    /// See [`Collection2D`]'s impl: measurable members summed, unmeasurable
    /// ones skipped, empty is zero.
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        Ok(self
            .members
            .iter()
            .filter_map(|m| m.surface_area().ok())
            .sum())
    }
}

// Deaggregate: a member that is not area geometry is handed back as `Rejected`
// rather than failing the whole collection, so one curve among the surfaces does
// not discard the surfaces.
impl crate::ops::ExtractHoles for Collection2D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(crate::Geometry, crate::ops::ExtractedPart),
    ) -> Result<(), crate::ops::UnsupportedOperation> {
        for member in self.members() {
            if member.extract_holes(emit).is_err() {
                emit(
                    crate::Geometry::Euclidean2D(member.clone()),
                    crate::ops::ExtractedPart::Rejected,
                );
            }
        }
        Ok(())
    }
}

impl crate::ops::ExtractHoles for Collection3D {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(crate::Geometry, crate::ops::ExtractedPart),
    ) -> Result<(), crate::ops::UnsupportedOperation> {
        for member in self.members() {
            if member.extract_holes(emit).is_err() {
                emit(
                    crate::Geometry::Euclidean3D(member.clone()),
                    crate::ops::ExtractedPart::Rejected,
                );
            }
        }
        Ok(())
    }
}

impl crate::ops::Split for Collection2D {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(crate::Geometry, Attributes),
    ) -> Result<(), crate::ops::UnsupportedOperation> {
        let members = std::mem::take(&mut self.members)
            .into_iter()
            .map(crate::Geometry::Euclidean2D);
        crate::ops::split::emit_members(members, std::mem::take(&mut self.attrs), emit);
        Ok(())
    }
}

impl crate::ops::Split for Collection3D {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(crate::Geometry, Attributes),
    ) -> Result<(), crate::ops::UnsupportedOperation> {
        let members = std::mem::take(&mut self.members)
            .into_iter()
            .map(crate::Geometry::Euclidean3D);
        crate::ops::split::emit_members(members, std::mem::take(&mut self.attrs), emit);
        Ok(())
    }
}

impl ForceTwoDimension for Collection2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let mut members = Vec::with_capacity(self.members.len());
        for member in &mut self.members {
            members.push(member.force_2d()?);
        }
        Ok(Euclidean2DGeometry::Collection(Collection2D {
            members,
            attrs: std::mem::take(&mut self.attrs),
        }))
    }
}

impl ForceTwoDimension for Collection3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let mut members = Vec::with_capacity(self.members.len());
        for member in &mut self.members {
            members.push(member.force_2d()?);
        }
        Ok(Euclidean2DGeometry::Collection(Collection2D {
            members,
            attrs: std::mem::take(&mut self.attrs),
        }))
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for Collection2D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        self.members.iter().try_for_each(|m| m.footprint(sink))
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for Collection3D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        self.members.iter().try_for_each(|m| m.footprint(sink))
    }
}

#[cfg(feature = "new-geometry")]
mod grid_impl {
    //! `DivideByGrid` for `Collection2D`/`Collection3D`, plus the regrouping
    //! `GeometryCollection` (in `lib.rs`) reuses through
    //! [`grid_divide_members`].
    //!
    //! A collection is a bag: a member that cannot be divided (`Unsupported`,
    //! e.g. a bare `Point`) or has nothing to give (`Empty`) is skipped rather
    //! than failing its siblings, so one undividable member never costs the
    //! caller the rest. Any other error (`MixedFrames`, `InvalidSpec`)
    //! propagates, since those describe the request itself, not one member's
    //! shape.
    //!
    //! Pieces are regrouped by cell into a `BTreeMap` keyed `(row, col)` --
    //! sorting row-major for free, the same idiom the mesh leaves use -- so one
    //! cell yields one `GeometryCollection` holding every survivor that landed
    //! there, not one geometry per member. Coverage is judged once per cell
    //! over that whole group's XY area (`geometry_area_xy`, summed), so
    //! members that only *together* fill a cell still report `Full`. A
    //! container whose members all declined reports `Empty`, not
    //! `Unsupported`: a collection is something this op knows how to divide,
    //! it just had nothing to give.
    //!
    //! `Collection2D`/`Collection3D` members each carry their own frame (see
    //! the module doc above), and this op lays one grid over all of them, so
    //! they must agree or the grid would be silently misapplied to whichever
    //! member does not share it -- checked by [`frames_agree`]. Every leaf
    //! that exposes a frame is considered, divisible or not (a bare `Point`
    //! is `Unsupported` here yet still contributes its frame), so this stays
    //! the same question `Geometry::frame()` answers: `PointCloud` and `Csg`
    //! are the only leaves left out, and only because neither exposes a frame
    //! to read (`Csg`'s lives on its operand `Solid`s) -- exactly the pair
    //! `Geometry::frame` omits, and for the same reason. `Solid` *does*
    //! expose one (`Solid::frame`) and is collected, even though
    //! `DivideByGrid` is `Unsupported` for it: were it skipped here, this
    //! check and `Geometry::frame()` -- which the grid-divider action reads
    //! to warn about angular units -- would disagree about the very same
    //! geometry. `GeometryCollection` members are `Geometry`, which carry no
    //! single frame to compare at all, so [`grid_divide_members`] skips this
    //! check entirely rather than fabricating one.

    use std::collections::BTreeMap;

    use super::{Collection2D, Collection3D};
    use crate::coordinate::CoordinateFrame;
    use crate::ops::grid::{
        geometry_area_xy, CellCoverage, DivideByGrid, GridCell, GridDivideError, GridSpec,
    };
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    /// Collect the frame of every member that exposes one, recursing into a
    /// nested `Collection`. Deliberately the same set of leaves
    /// `lib.rs`'s `collect_leaf_frames_2d`/`collect_leaf_frames_3d` gather,
    /// so this check and `Geometry::frame()` never disagree; see the module
    /// doc for why `PointCloud`/`Csg` (3D-only) are the only omissions.
    fn collect_frames_2d(m: &Euclidean2DGeometry, out: &mut Vec<CoordinateFrame>) {
        match m {
            Euclidean2DGeometry::Point(g) => out.push(g.frame().clone()),
            Euclidean2DGeometry::LineString(g) => out.push(g.frame().clone()),
            Euclidean2DGeometry::Polygon(g) => out.push(g.frame().clone()),
            Euclidean2DGeometry::PolygonMesh(g) => out.push(g.frame().clone()),
            Euclidean2DGeometry::TriangularMesh(g) => out.push(g.frame().clone()),
            Euclidean2DGeometry::Collection(c) => {
                c.members().iter().for_each(|m| collect_frames_2d(m, out));
            }
        }
    }

    /// As [`collect_frames_2d`], for the 3D leaf.
    fn collect_frames_3d(m: &Euclidean3DGeometry, out: &mut Vec<CoordinateFrame>) {
        match m {
            Euclidean3DGeometry::Point(g) => out.push(g.frame().clone()),
            Euclidean3DGeometry::LineString(g) => out.push(g.frame().clone()),
            Euclidean3DGeometry::Polygon(g) => out.push(g.frame().clone()),
            Euclidean3DGeometry::PolygonMesh(g) => out.push(g.frame().clone()),
            Euclidean3DGeometry::TriangularMesh(g) => out.push(g.frame().clone()),
            // Collected even though `DivideByGrid` is `Unsupported` for it:
            // it exposes a frame, and `collect_leaf_frames_3d` counts it, so
            // leaving it out would make this check and `Geometry::frame()`
            // disagree about the same geometry.
            Euclidean3DGeometry::Solid(g) => out.push(g.frame().clone()),
            Euclidean3DGeometry::Collection(c) => {
                c.members().iter().for_each(|m| collect_frames_3d(m, out));
            }
            // The only leaves with no frame to read: `PointCloud` has none,
            // and `Csg`'s lives on its operand `Solid`s.
            Euclidean3DGeometry::PointCloud(_) | Euclidean3DGeometry::Csg(_) => {}
        }
    }

    /// Whether every collected frame agrees (vacuously true when none were
    /// collected at all).
    fn frames_agree<T>(members: &[T], collect: impl Fn(&T, &mut Vec<CoordinateFrame>)) -> bool {
        let mut frames = Vec::new();
        for m in members {
            collect(m, &mut frames);
        }
        match frames.split_first() {
            Some((first, rest)) => rest.iter().all(|f| f == first),
            None => true,
        }
    }

    /// Divide every member, regroup the pieces by cell, and emit one geometry
    /// per cell. See the module doc for the skip/propagate and coverage
    /// rules; this performs no frame check of its own -- callers that need
    /// one (`Collection2D`/`Collection3D`) run it first.
    fn regroup_and_emit(
        members: impl Iterator<Item = Geometry>,
        grid: &GridSpec,
        emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
    ) -> Result<(), GridDivideError> {
        // BTreeMap keyed `(row, col)` sorts row-major for free, which is the
        // emission order the op promises.
        let mut by_cell: BTreeMap<(i64, i64), Vec<Geometry>> = BTreeMap::new();
        let mut any = false;

        for member in members {
            let divided = member.divide_by_grid(grid, &mut |cell, _cov, piece| {
                by_cell.entry((cell.row, cell.col)).or_default().push(piece);
            });
            match divided {
                Ok(()) => any = true,
                // A member with nothing to give is not the container's failure.
                Err(GridDivideError::Unsupported(_)) | Err(GridDivideError::Empty) => {}
                Err(other) => return Err(other),
            }
        }

        if !any || by_cell.is_empty() {
            return Err(GridDivideError::Empty);
        }

        for ((row, col), pieces) in by_cell {
            let cell = GridCell { row, col };
            let area: f64 = pieces.iter().map(geometry_area_xy).sum();
            let geom = crate::GeometryCollection::new(pieces);
            emit(
                cell,
                // The cell's *own* window area, never `cell_size^2`: the clip
                // pins a full piece's area to `window.area()`, which differs
                // from the square of the side by more than
                // `COVERAGE_TOLERANCE` at a large origin. Judging against the
                // nominal square would then call an exactly-filled cell
                // `Partial` and drop it under `completeCellsOnly`.
                CellCoverage::from_area(area, grid.window(cell).area()),
                Geometry::GeometryCollection(geom),
            );
        }
        Ok(())
    }

    /// Entry point [`GeometryCollection`](crate::GeometryCollection) reuses
    /// from `lib.rs`: its members are already `Geometry`, which carry no
    /// single frame to compare, so this skips straight to regrouping with no
    /// frame check.
    pub(crate) fn grid_divide_members(
        members: impl Iterator<Item = Geometry>,
        grid: &GridSpec,
        emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
    ) -> Result<(), GridDivideError> {
        regroup_and_emit(members, grid, emit)
    }

    impl DivideByGrid for Collection2D {
        fn divide_by_grid(
            &self,
            grid: &GridSpec,
            emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
        ) -> Result<(), GridDivideError> {
            if !frames_agree(self.members(), collect_frames_2d) {
                return Err(GridDivideError::MixedFrames);
            }
            regroup_and_emit(
                self.members().iter().cloned().map(Geometry::Euclidean2D),
                grid,
                emit,
            )
        }
    }

    impl DivideByGrid for Collection3D {
        fn divide_by_grid(
            &self,
            grid: &GridSpec,
            emit: &mut dyn FnMut(GridCell, CellCoverage, Geometry),
        ) -> Result<(), GridDivideError> {
            if !frames_agree(self.members(), collect_frames_3d) {
                return Err(GridDivideError::MixedFrames);
            }
            regroup_and_emit(
                self.members().iter().cloned().map(Geometry::Euclidean3D),
                grid,
                emit,
            )
        }
    }
}

#[cfg(feature = "new-geometry")]
pub(crate) use grid_impl::grid_divide_members;

// A collection reports the first member that has an elevation, rather than only
// its head: a member with none (an absent geometry, a 2D point, an empty leaf) is
// ordinary and must not hide the ones behind it.
#[cfg(feature = "new-geometry")]
impl Elevation for Collection2D {
    fn elevation(&self) -> Option<f64> {
        self.members.iter().find_map(Elevation::elevation)
    }
}

#[cfg(feature = "new-geometry")]
impl Elevation for Collection3D {
    fn elevation(&self) -> Option<f64> {
        self.members.iter().find_map(Elevation::elevation)
    }
}

// A collection validates by recursing into its members (see
// `validation_next::validate`), so it declares no direct checks and inherits
// every `Validate` default.
#[cfg(feature = "new-geometry")]
impl Validate for Collection2D {}

#[cfg(feature = "new-geometry")]
impl Validate for Collection3D {}

impl Coerce for Collection2D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        let mut changed = false;
        let members = std::mem::take(&mut self.members)
            .into_iter()
            .map(|mut member| match member.coerce(target, cache) {
                Ok(Geometry::Euclidean2D(coerced)) => {
                    changed = true;
                    coerced
                }
                // A 2D leaf coerces to a 2D geometry, so the other `Ok` shapes
                // do not arise; an `Err` left the member untouched.
                _ => member,
            })
            .collect();
        self.members = members;
        if !changed {
            return Err(unchanged::<Self>());
        }
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Collection(
            std::mem::take(self),
        )))
    }
}

impl Coerce for Collection3D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        let mut changed = false;
        let members = std::mem::take(&mut self.members)
            .into_iter()
            .map(|mut member| match member.coerce(target, cache) {
                Ok(Geometry::Euclidean3D(coerced)) => {
                    changed = true;
                    coerced
                }
                _ => member,
            })
            .collect();
        self.members = members;
        if !changed {
            return Err(unchanged::<Self>());
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            std::mem::take(self),
        )))
    }
}

impl crate::ops::ExtractBoundary for Collection2D {
    fn extract_boundary(&self) -> Result<crate::ops::Boundary, crate::ops::UnsupportedOperation> {
        crate::ops::container_boundary(
            self.members(),
            self.member_attributes(),
            |geometry| match geometry {
                crate::Geometry::Euclidean2D(g) => Some(g),
                _ => None,
            },
            wrap_members_2d,
        )
        .ok_or_else(crate::ops::boundary::unsupported::<Self>)
    }
}

impl crate::ops::ExtractBoundary for Collection3D {
    fn extract_boundary(&self) -> Result<crate::ops::Boundary, crate::ops::UnsupportedOperation> {
        crate::ops::container_boundary(
            self.members(),
            self.member_attributes(),
            |geometry| match geometry {
                crate::Geometry::Euclidean3D(g) => Some(g),
                _ => None,
            },
            wrap_members_3d,
        )
        .ok_or_else(crate::ops::boundary::unsupported::<Self>)
    }
}

/// Gather members into a collection, keeping their attributes when the source
/// carried any. A collection's boundary stays a collection even when one member
/// gave it, so the shape does not turn on how many members contributed.
fn wrap_members_2d(members: Vec<Euclidean2DGeometry>, attrs: Vec<Attributes>) -> crate::Geometry {
    if members.is_empty() {
        return crate::Geometry::None;
    }
    let attrs = if attrs.len() == members.len() {
        attrs
    } else {
        Vec::new()
    };
    crate::Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D {
        members,
        attrs,
    }))
}

/// The 3D counterpart of [`wrap_members_2d`].
fn wrap_members_3d(members: Vec<Euclidean3DGeometry>, attrs: Vec<Attributes>) -> crate::Geometry {
    if members.is_empty() {
        return crate::Geometry::None;
    }
    let attrs = if attrs.len() == members.len() {
        attrs
    } else {
        Vec::new()
    };
    crate::Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D {
        members,
        attrs,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::point::{Point2D, Point3D};

    #[test]
    fn new_2d_collects_members_without_attrs() {
        let c = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [0.0, 0.0])),
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [1.0, 1.0])),
        ]);
        assert_eq!(c.members.len(), 2);
        assert!(c.attrs.is_empty());
    }

    #[test]
    fn with_attributes_rejects_length_mismatch() {
        let members = vec![Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [0.0, 0.0, 0.0],
        ))];
        assert!(Collection3D::with_attributes(members, vec![Attributes::default(); 2]).is_err());
    }

    #[test]
    fn collection2d_box_merges_members() {
        let c = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [0.0, 3.0])),
            Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Euclidean, [4.0, -1.0])),
        ]);
        assert_eq!(
            c.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, -1.0],
                max: [4.0, 3.0]
            }
        );
    }

    #[test]
    fn collection3d_box_merges_members() {
        let c = Collection3D::new([
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0, 3.0, 1.0])),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [4.0, -1.0, 7.0])),
        ]);
        assert_eq!(
            c.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, -1.0, 1.0],
                max: [4.0, 3.0, 7.0]
            }
        );
    }

    #[test]
    fn empty_collection_has_no_box() {
        let c = Collection2D::new(std::iter::empty());
        assert!(c.bounding_box().is_err());
    }
}

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

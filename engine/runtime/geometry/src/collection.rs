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
use crate::ops::union_results;
use crate::ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation};
#[cfg(feature = "new-geometry")]
use crate::validation_next::Validate;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry};

/// A `Multi*` collection of 2D geometries; members may differ in coordinate frame.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Collection2D {
    members: Vec<Euclidean2DGeometry>,
    /// Per-member attributes, parallel to `members`; empty = no member carries
    /// any. Child-scoped.
    attrs: Vec<Attributes>,
}

/// A `Multi*` collection of 3D geometries; members may differ in coordinate frame.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Collection3D {
    members: Vec<Euclidean3DGeometry>,
    /// Per-member attributes, parallel to `members`; empty = no member carries
    /// any. Child-scoped.
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

impl Reproject for Collection2D {
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

impl Reproject for Collection3D {
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

// Tessellation is defined per-primitive, not over a collection.
crate::unsupported!(Collection2D: Triangulate);
crate::unsupported!(Collection3D: Triangulate);

// A collection validates by recursing into its members (see
// `validation_next::validate`), so it declares no direct checks and inherits
// every `Validate` default.
#[cfg(feature = "new-geometry")]
impl Validate for Collection2D {}

#[cfg(feature = "new-geometry")]
impl Validate for Collection3D {}

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

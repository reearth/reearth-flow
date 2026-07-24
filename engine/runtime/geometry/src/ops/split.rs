//! One-level splitting of container geometries.
//!
//! A collection yields its direct members, a mesh yields one polygon per face,
//! and a point cloud yields one point per sample. Splitting descends a single
//! level: a collection of meshes splits into meshes, not into their polygons.
//! Members that carry their own attributes (collection children, point-cloud
//! points) pair each emitted geometry with those attributes so a caller can
//! hoist them; every other member pairs with an empty attribute set.
//!
//! Members are handed to a callback one at a time rather than collected into a
//! vector, so a caller streams each split feature downstream without ever
//! holding the whole decomposition in memory.
//!
//! Following the op discipline, the trait carries a default body reporting
//! [`UnsupportedOperation`]. A container leaf overrides `split` in its
//! `{type}/ops.rs`; a non-container leaf opts out via
//! [`unsupported!`](crate::unsupported), which a caller reads as "pass this
//! geometry through unchanged".

use reearth_flow_common::attribute::Attributes;

use crate::ops::UnsupportedOperation;
use crate::Geometry;

/// One-level decomposition of a container geometry into its members.
#[enum_dispatch::enum_dispatch]
pub trait Split {
    /// Split one level, invoking `emit` once per member with the member geometry
    /// and the per-member attributes to hoist onto the emitting feature (empty
    /// when the member carries none). Moves members out of `self`, leaving it
    /// emptied. Returns [`UnsupportedOperation`] for a geometry that is not a
    /// splittable container, in which case `emit` is never called.
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        let _ = emit;
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "split",
        })
    }
}

// The boxed enum variants (`Box<Polygon3D>`, `Box<PointCloud>`, …) need the
// trait on the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Split + ?Sized> Split for Box<T> {
    fn split(
        &mut self,
        emit: &mut dyn FnMut(Geometry, Attributes),
    ) -> Result<(), UnsupportedOperation> {
        (**self).split(emit)
    }
}

/// Emit each member paired with its per-member attributes, consuming both and
/// defaulting to an empty attribute set when `attrs` is empty (no member carries
/// any) or shorter than `members`.
pub(crate) fn emit_members(
    members: impl IntoIterator<Item = Geometry>,
    attrs: Vec<Attributes>,
    emit: &mut dyn FnMut(Geometry, Attributes),
) {
    let mut attrs = attrs.into_iter();
    for geometry in members {
        emit(geometry, attrs.next().unwrap_or_default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::CoordinateFrame;
    use crate::point::Point3D;
    use crate::point_cloud::PointCloud;
    use crate::triangular_mesh::{TriangularMesh3D, TriangularMesh3DData};
    use crate::Euclidean3DGeometry;
    use reearth_flow_common::attribute::{Attribute, AttributeValue};

    fn attrs(pairs: &[(&str, &str)]) -> Attributes {
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.to_string()),
                    AttributeValue::String(v.to_string()),
                )
            })
            .collect()
    }

    /// Collect a `split` into a vector for assertions.
    fn collect(geometry: &mut Geometry) -> Result<Vec<(Geometry, Attributes)>, UnsupportedOperation> {
        let mut out = Vec::new();
        geometry.split(&mut |g, a| out.push((g, a)))?;
        Ok(out)
    }

    #[test]
    fn none_and_singletons_are_unsupported() {
        assert!(collect(&mut Geometry::None).is_err());
        let mut point = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [1.0, 2.0, 3.0],
        )));
        assert!(collect(&mut point).is_err());
    }

    #[test]
    fn collection_splits_one_level_and_hoists_child_attributes() {
        let members = vec![
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0, 0.0, 0.0])),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [1.0, 1.0, 1.0])),
        ];
        let collection = Collection3D::with_attributes(
            members,
            vec![attrs(&[("name", "a")]), attrs(&[("name", "b")])],
        )
        .unwrap();
        let mut geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(collection));

        let out = collect(&mut geometry).unwrap();
        assert_eq!(out.len(), 2);
        assert!(matches!(
            out[0].0,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(_))
        ));
        assert_eq!(
            out[0].1.get(&Attribute::new("name")),
            Some(&AttributeValue::String("a".to_string()))
        );
        assert_eq!(
            out[1].1.get(&Attribute::new("name")),
            Some(&AttributeValue::String("b".to_string()))
        );
    }

    #[test]
    fn collection_of_collections_splits_only_one_level() {
        let inner = Collection3D::new([Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [0.0, 0.0, 0.0],
        ))]);
        let outer = Collection3D::new([Euclidean3DGeometry::Collection(inner)]);
        let mut geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(outer));

        let out = collect(&mut geometry).unwrap();
        assert_eq!(out.len(), 1);
        // The single member is still a collection, not its inner point.
        assert!(matches!(
            out[0].0,
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(_))
        ));
    }

    #[test]
    fn triangular_mesh_splits_to_one_polygon_per_triangle() {
        let data = TriangularMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        let mesh = TriangularMesh3D::new(CoordinateFrame::Euclidean, data);
        let mut geometry =
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));

        let out = collect(&mut geometry).unwrap();
        assert_eq!(out.len(), 2);
        for (member, member_attrs) in &out {
            assert!(matches!(
                member,
                Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(_))
            ));
            assert!(member_attrs.is_empty());
        }
    }

    #[test]
    fn point_cloud_splits_to_one_point_per_sample() {
        let cloud = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]],
        );
        let mut geometry = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud)));

        let out = collect(&mut geometry).unwrap();
        assert_eq!(out.len(), 2);
        assert!(matches!(
            out[1].0,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(_))
        ));
    }
}

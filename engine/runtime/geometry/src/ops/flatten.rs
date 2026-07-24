//! One-level flattening of container geometries.
//!
//! A collection yields its direct members, a mesh yields one polygon per face,
//! and a point cloud yields one point per sample. Flattening descends a single
//! level: a collection of meshes flattens to meshes, not to their polygons.
//! Members that carry their own attributes (collection children, point-cloud
//! points) pair each emitted geometry with those attributes so a caller can
//! hoist them; every other member pairs with an empty attribute set.

use reearth_flow_common::attribute::Attributes;

use crate::collection::{Collection2D, Collection3D};
use crate::csg::Csg;
use crate::line_string::{LineString2D, LineString3D};
use crate::point::{Point2D, Point3D};
use crate::point_cloud::PointCloud;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
use crate::solid::Solid;
use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

/// One-level decomposition of a container geometry into its members.
#[enum_dispatch::enum_dispatch]
pub trait Flatten {
    /// Flatten one level into the member geometries, each paired with the
    /// per-member attributes to hoist onto the emitting feature (empty when the
    /// member carries none). Returns `None` for a geometry that is not a
    /// flattenable container, signalling the caller to pass it through unchanged.
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        None
    }
}

// The boxed enum variants (`Box<Polygon3D>`, `Box<PointCloud>`, …) need the
// trait on the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Flatten + ?Sized> Flatten for Box<T> {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        (**self).flatten()
    }
}

// Leaves with no members to flatten fall through to the default and pass
// through unchanged.
impl Flatten for Point2D {}
impl Flatten for Point3D {}
impl Flatten for LineString2D {}
impl Flatten for LineString3D {}
impl Flatten for Polygon2D {}
impl Flatten for Polygon3D {}
impl Flatten for Solid {}
impl Flatten for Csg {}

impl Flatten for Collection2D {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        Some(pair_members(
            self.members().iter().cloned().map(Geometry::Euclidean2D),
            self.member_attributes(),
        ))
    }
}

impl Flatten for Collection3D {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        Some(pair_members(
            self.members().iter().cloned().map(Geometry::Euclidean3D),
            self.member_attributes(),
        ))
    }
}

impl Flatten for TriangularMesh2D {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        let vertices = self.vertices();
        let frame = self.frame();
        Some(
            self.triangles()
                .map(|[i, j, k]| {
                    let ring = [
                        vertices[i as usize],
                        vertices[j as usize],
                        vertices[k as usize],
                        vertices[i as usize],
                    ];
                    let polygon =
                        Polygon2D::from_rings(frame.clone(), ring, Vec::<Vec<[f64; 2]>>::new());
                    (
                        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(polygon))),
                        Attributes::new(),
                    )
                })
                .collect(),
        )
    }
}

impl Flatten for TriangularMesh3D {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        let vertices = self.vertices();
        let frame = self.frame();
        Some(
            self.triangles()
                .map(|[i, j, k]| {
                    let ring = [
                        vertices[i as usize],
                        vertices[j as usize],
                        vertices[k as usize],
                        vertices[i as usize],
                    ];
                    let polygon =
                        Polygon3D::from_rings(frame.clone(), ring, Vec::<Vec<[f64; 3]>>::new());
                    (
                        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(polygon))),
                        Attributes::new(),
                    )
                })
                .collect(),
        )
    }
}

impl Flatten for PolygonMesh2D {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        Some(
            self.faces_as_polygons()
                .into_iter()
                .map(|polygon| {
                    (
                        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(polygon))),
                        Attributes::new(),
                    )
                })
                .collect(),
        )
    }
}

impl Flatten for PolygonMesh3D {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        Some(
            self.faces_as_polygons()
                .into_iter()
                .map(|polygon| {
                    (
                        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(polygon))),
                        Attributes::new(),
                    )
                })
                .collect(),
        )
    }
}

impl Flatten for PointCloud {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        Some(
            self.to_points()
                .into_iter()
                .map(|(point, attributes)| {
                    (
                        Geometry::Euclidean3D(Euclidean3DGeometry::Point(point)),
                        attributes,
                    )
                })
                .collect(),
        )
    }
}

impl Flatten for GeometryCollection {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        Some(pair_members(
            self.members().iter().cloned(),
            self.member_attributes(),
        ))
    }
}

impl Flatten for Geometry {
    fn flatten(&self) -> Option<Vec<(Geometry, Attributes)>> {
        match self {
            Geometry::None => None,
            Geometry::Euclidean2D(geometry) => geometry.flatten(),
            Geometry::Euclidean3D(geometry) => geometry.flatten(),
            Geometry::GeometryCollection(collection) => collection.flatten(),
        }
    }
}

/// Pair each member with its per-member attributes, defaulting to an empty set
/// when `attrs` is empty (no member carries any) or shorter than `members`.
fn pair_members(
    members: impl Iterator<Item = Geometry>,
    attrs: &[Attributes],
) -> Vec<(Geometry, Attributes)> {
    members
        .enumerate()
        .map(|(i, geometry)| (geometry, attrs.get(i).cloned().unwrap_or_default()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
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

    #[test]
    fn none_and_singletons_pass_through() {
        assert!(Geometry::None.flatten().is_none());
        let point = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [1.0, 2.0, 3.0],
        )));
        assert!(point.flatten().is_none());
    }

    #[test]
    fn collection_flattens_one_level_and_hoists_child_attributes() {
        let members = vec![
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0, 0.0, 0.0])),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [1.0, 1.0, 1.0])),
        ];
        let collection = Collection3D::with_attributes(
            members,
            vec![attrs(&[("name", "a")]), attrs(&[("name", "b")])],
        )
        .unwrap();
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(collection));

        let out = geometry.flatten().unwrap();
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
    fn collection_of_collections_flattens_only_one_level() {
        let inner = Collection3D::new([Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [0.0, 0.0, 0.0],
        ))]);
        let outer = Collection3D::new([Euclidean3DGeometry::Collection(inner)]);
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(outer));

        let out = geometry.flatten().unwrap();
        assert_eq!(out.len(), 1);
        // The single member is still a collection, not its inner point.
        assert!(matches!(
            out[0].0,
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(_))
        ));
    }

    #[test]
    fn triangular_mesh_flattens_to_one_polygon_per_triangle() {
        use crate::triangular_mesh::TriangularMesh3DData;

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
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));

        let out = geometry.flatten().unwrap();
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
    fn point_cloud_flattens_to_one_point_per_sample() {
        let cloud = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]],
        );
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud)));

        let out = geometry.flatten().unwrap();
        assert_eq!(out.len(), 2);
        assert!(matches!(
            out[1].0,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(_))
        ));
    }
}

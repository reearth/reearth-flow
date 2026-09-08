//! Elevation: the z of a geometry's representative vertex.
//!
//! [`Elevation`] reads the z of the first vertex reached when the geometry is
//! walked in its natural nesting order — part, then ring, then vertex. A 2D leaf
//! has no per-vertex z and reports instead the single elevation the whole leaf
//! lies at, `None` when it is pure 2D.
//!
//! The value only describes the geometry as a whole when the geometry lies at one
//! elevation; on anything else it is one arbitrary vertex's z. That is inherent
//! to the definition, not a shortcut in the implementation.

/// The elevation of a geometry's representative vertex.
#[enum_dispatch::enum_dispatch]
pub trait Elevation {
    /// The z of the first vertex in nesting order, or the elevation a 2D leaf
    /// lies at. `None` when the geometry holds no vertex, carries no elevation,
    /// or has no vertices of its own to read.
    fn elevation(&self) -> Option<f64> {
        None
    }
}

impl<T: Elevation + ?Sized> Elevation for Box<T> {
    fn elevation(&self) -> Option<f64> {
        (**self).elevation()
    }
}

#[cfg(test)]
mod tests {
    use crate::collection::{Collection2D, Collection3D};
    use crate::coordinate::CoordinateFrame;
    use crate::csg::{Csg, ThreeDimensional};
    use crate::line_string::{LineString2D, LineString3D};
    use crate::ops::Elevation;
    use crate::point::{Point2D, Point3D};
    use crate::point_cloud::PointCloud;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
    use crate::predicates::test3d::{box_shell, e, g3, solid_geometry};
    use crate::solid::{Shell, Solid};
    use crate::triangular_mesh::{
        TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData as TriData,
    };
    use crate::{
        Euclidean2DGeometry as G2, Euclidean3DGeometry as G3, Geometry, GeometryCollection,
    };
    use pretty_assertions::assert_eq;

    fn g2(g: G2) -> Geometry {
        Geometry::Euclidean2D(g)
    }

    /// A closed ring at `z`, offset in x by `x0` so callers can build several.
    fn ring(x0: f64, z: f64) -> Vec<[f64; 3]> {
        vec![
            [x0, 0.0, z],
            [x0 + 1.0, 0.0, z],
            [x0 + 1.0, 1.0, z],
            [x0, 0.0, z],
        ]
    }

    /// A two-face mesh whose vertex pool is ordered so that the pool's first
    /// vertex is *not* the first face's first vertex: reading the pool head
    /// instead of the face would give `9.0`.
    fn two_face_mesh() -> PolygonMesh3D {
        PolygonMesh3D::from_parts(
            e(),
            vec![
                [0.0, 0.0, 9.0],
                [1.0, 0.0, 9.0],
                [1.0, 1.0, 9.0],
                [0.0, 0.0, 4.0],
                [1.0, 0.0, 4.0],
                [1.0, 1.0, 4.0],
            ],
            // The first face is the second triple of vertices.
            vec![vec![3u32, 4, 5], vec![0u32, 1, 2]],
        )
        .unwrap()
    }

    #[test]
    fn a_3d_leaf_reports_its_first_vertex_not_its_extreme() {
        // Every ring here descends, so the first vertex is the maximum: a `min z`
        // implementation would return the last coordinate instead.
        let coords = [[0.0, 0.0, 7.0], [1.0, 0.0, -3.0], [1.0, 1.0, 2.0]];
        let cases: Vec<(&str, Geometry)> = vec![
            ("point", g3(G3::Point(Point3D::new(e(), [1.0, 2.0, 7.0])))),
            (
                "line string",
                g3(G3::LineString(LineString3D::from_coords(e(), coords))),
            ),
            (
                "polygon",
                g3(G3::Polygon(Box::new(Polygon3D::from_rings(
                    e(),
                    ring(0.0, 7.0),
                    // A hole lower than the exterior must not win.
                    [ring(0.25, -3.0)],
                )))),
            ),
            (
                "triangular mesh",
                g3(G3::TriangularMesh(Box::new(TriangularMesh3D::from_soup(
                    e(),
                    coords,
                )))),
            ),
            (
                "point cloud",
                g3(G3::PointCloud(Box::new(PointCloud::from_positions(
                    e(),
                    coords,
                )))),
            ),
        ];
        for (name, geometry) in cases {
            assert_eq!(geometry.elevation(), Some(7.0), "{name}");
        }
    }

    #[test]
    fn a_mesh_reports_its_first_face_not_its_vertex_pool_head() {
        let mesh = two_face_mesh();
        assert_eq!(mesh.elevation(), Some(4.0));
        assert_eq!(g3(G3::PolygonMesh(Box::new(mesh))).elevation(), Some(4.0));
    }

    #[test]
    fn a_solid_reports_its_exterior_shell_only() {
        // The void sits below the exterior box, so a solid that folded its
        // interiors in would report the void's z.
        let solid = Solid::new(
            e(),
            Shell::TriangularMesh(box_shell([0.0, 0.0, 5.0], [4.0, 4.0, 4.0])),
            vec![Shell::TriangularMesh(box_shell(
                [1.0, 1.0, -8.0],
                [1.0, 1.0, 1.0],
            ))],
        );
        assert_eq!(g3(solid_geometry(solid)).elevation(), Some(5.0));
    }

    #[test]
    fn a_solid_whose_shell_has_no_face_has_no_elevation() {
        let empty = Solid::from_exterior(e(), TriData::from_parts(Vec::new(), Vec::new()).unwrap());
        assert_eq!(g3(solid_geometry(empty)).elevation(), None);
    }

    #[test]
    fn a_3d_leaf_without_a_vertex_has_no_elevation() {
        let cases: Vec<(&str, Geometry)> = vec![
            (
                "line string",
                g3(G3::LineString(LineString3D::from_coords(
                    e(),
                    Vec::<[f64; 3]>::new(),
                ))),
            ),
            (
                "polygon",
                g3(G3::Polygon(Box::new(Polygon3D::from_rings(
                    e(),
                    Vec::<[f64; 3]>::new(),
                    Vec::<Vec<[f64; 3]>>::new(),
                )))),
            ),
            (
                "triangular mesh",
                g3(G3::TriangularMesh(Box::new(TriangularMesh3D::from_soup(
                    e(),
                    Vec::<[f64; 3]>::new(),
                )))),
            ),
            (
                "polygon mesh",
                g3(G3::PolygonMesh(Box::new(
                    PolygonMesh3D::from_parts(e(), Vec::new(), Vec::<Vec<u32>>::new()).unwrap(),
                ))),
            ),
            (
                "point cloud",
                g3(G3::PointCloud(Box::new(PointCloud::from_positions(
                    e(),
                    Vec::<[f64; 3]>::new(),
                )))),
            ),
        ];
        for (name, geometry) in cases {
            assert_eq!(geometry.elevation(), None, "{name}");
        }
    }

    #[test]
    fn a_2d_leaf_reports_the_elevation_it_lies_at() {
        let square = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]];
        let cases: Vec<(&str, Geometry, Option<f64>)> = vec![
            (
                "line string at an elevation",
                g2(G2::LineString(LineString2D::from_coords_at_elevation(
                    e(),
                    square,
                    3.5,
                ))),
                Some(3.5),
            ),
            (
                "pure 2D line string",
                g2(G2::LineString(LineString2D::from_coords(e(), square))),
                None,
            ),
            (
                "polygon at an elevation",
                g2(G2::Polygon(Box::new(Polygon2D::from_rings_at_elevation(
                    e(),
                    square,
                    Vec::<Vec<[f64; 2]>>::new(),
                    3.5,
                )))),
                Some(3.5),
            ),
            (
                "pure 2D polygon",
                g2(G2::Polygon(Box::new(Polygon2D::from_rings(
                    e(),
                    square,
                    Vec::<Vec<[f64; 2]>>::new(),
                )))),
                None,
            ),
            (
                "polygon mesh at an elevation",
                g2(G2::PolygonMesh(Box::new(
                    PolygonMesh2D::from_parts_at_elevation(
                        e(),
                        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                        vec![vec![0u32, 1, 2]],
                        3.5,
                    )
                    .unwrap(),
                ))),
                Some(3.5),
            ),
            (
                "triangular mesh at an elevation",
                g2(G2::TriangularMesh(Box::new(
                    TriangularMesh2D::from_parts_at_elevation(
                        e(),
                        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                        [0u32, 1, 2],
                        3.5,
                    )
                    .unwrap(),
                ))),
                Some(3.5),
            ),
            (
                // A position cannot lie at a height, so it never reports one.
                "point",
                g2(G2::Point(Point2D::new(e(), [1.0, 2.0]))),
                None,
            ),
        ];
        for (name, geometry, expected) in cases {
            assert_eq!(geometry.elevation(), expected, "{name}");
        }
    }

    #[test]
    fn an_absent_geometry_has_no_elevation() {
        assert_eq!(Geometry::None.elevation(), None);
    }

    #[test]
    fn an_unevaluated_boolean_tree_has_no_elevation() {
        let operand = |z: f64| {
            ThreeDimensional::Solid(Box::new(Solid::from_exterior(
                e(),
                box_shell([0.0, 0.0, z], [1.0; 3]),
            )))
        };
        let csg = Csg::Union(Box::new(operand(5.0)), Box::new(operand(9.0)));
        assert_eq!(g3(G3::Csg(csg)).elevation(), None);
    }

    #[test]
    fn a_collection_reports_the_first_member_that_has_one() {
        // The leading members carry nothing, so a collection that only looked at
        // its head would report nothing at all.
        let members = [
            G3::LineString(LineString3D::from_coords(e(), Vec::<[f64; 3]>::new())),
            G3::Point(Point3D::new(e(), [0.0, 0.0, 6.0])),
            G3::Point(Point3D::new(e(), [0.0, 0.0, 1.0])),
        ];
        assert_eq!(
            g3(G3::Collection(Collection3D::new(members.clone()))).elevation(),
            Some(6.0)
        );
        // Frames may differ across members; the first value still wins.
        let mixed = GeometryCollection::new([
            g2(G2::Point(Point2D::new(e(), [0.0, 0.0]))),
            Geometry::None,
            g2(G2::LineString(LineString2D::from_coords_at_elevation(
                CoordinateFrame::Euclidean,
                [[0.0, 0.0], [1.0, 0.0]],
                2.5,
            ))),
            g3(G3::Collection(Collection3D::new(members))),
        ]);
        assert_eq!(Geometry::GeometryCollection(mixed).elevation(), Some(2.5));
    }

    #[test]
    fn an_empty_collection_has_no_elevation() {
        assert_eq!(
            g2(G2::Collection(Collection2D::new(Vec::new()))).elevation(),
            None
        );
        assert_eq!(
            Geometry::GeometryCollection(GeometryCollection::new(Vec::new())).elevation(),
            None
        );
    }
}

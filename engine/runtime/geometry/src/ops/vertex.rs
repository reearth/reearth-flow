//! Vertex counting, and the first vertex of a geometry.
//!
//! A ring's closing vertex repeats its first one, so it is not counted: a
//! triangle counts three whether or not it is stored closed. Whether a ring is
//! closed is a validation question, not a counting one.

use crate::coordinate::CoordinateFrame;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// The number of vertices a geometry has.
///
/// Total over the hierarchy: a container sums its members. The default body
/// returns `0`, so a leaf that stores no coordinates of its own needs only an
/// (empty) `impl`, stamped by [`unsupported!`](crate::unsupported). Counting
/// never fails; a geometry with nothing to count answers `0`.
///
/// * A ring drops its closing vertex if it has one, so a triangle counts `3`
///   stored closed or open.
/// * A face sums its exterior and every interior ring.
/// * A mesh counts its vertex pool, so a vertex shared by several faces counts
///   once.
/// * A [`Solid`](crate::solid::Solid) sums the pools of its exterior and
///   interior shells.
#[enum_dispatch::enum_dispatch]
pub trait CountVertices {
    /// The number of vertices this geometry has, recursing into
    /// collections.
    fn count_vertices(&self) -> usize {
        0
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: CountVertices + ?Sized> CountVertices for Box<T> {
    fn count_vertices(&self) -> usize {
        (**self).count_vertices()
    }
}

/// The first vertex reached when the geometry is walked in nesting order (part,
/// then ring, then vertex), with the frame it is expressed in, in the
/// geometry's own coordinate order. A mesh starts at its first face, not at the
/// head of its vertex pool. A 2D leaf lifts to the elevation it lies at, `0.0`
/// when it has none.
///
/// `None` when the geometry holds no vertex to read: an absent geometry, a point
/// cloud, or an unevaluated boolean tree.
pub fn first_vertex(geometry: &Geometry) -> Option<([f64; 3], CoordinateFrame)> {
    match geometry {
        Geometry::None => None,
        Geometry::Euclidean3D(g) => first_vertex_3d(g),
        Geometry::Euclidean2D(g) => first_vertex_2d(g),
        Geometry::GeometryCollection(c) => c.members().iter().find_map(first_vertex),
    }
}

fn first_vertex_3d(geometry: &Euclidean3DGeometry) -> Option<([f64; 3], CoordinateFrame)> {
    match geometry {
        Euclidean3DGeometry::Point(p) => Some((p.position(), p.frame().clone())),
        // A point cloud has no single vertex that represents it.
        Euclidean3DGeometry::PointCloud(_) => None,
        Euclidean3DGeometry::LineString(l) => {
            l.coords().first().copied().map(|v| (v, l.frame().clone()))
        }
        Euclidean3DGeometry::Polygon(p) => p
            .exterior()
            .first()
            .copied()
            .map(|v| (v, p.frame().clone())),
        Euclidean3DGeometry::PolygonMesh(m) => {
            m.first_face_vertex().map(|v| (v, m.frame().clone()))
        }
        Euclidean3DGeometry::TriangularMesh(m) => {
            let [i, _, _] = m.triangles().next()?;
            Some((m.vertices()[i as usize], m.frame().clone()))
        }
        Euclidean3DGeometry::Solid(s) => s.first_vertex().map(|v| (v, s.frame().clone())),
        // A boolean tree carries no coordinates of its own until evaluated.
        Euclidean3DGeometry::Csg(_) => None,
        Euclidean3DGeometry::Collection(c) => c.members().iter().find_map(first_vertex_3d),
    }
}

fn first_vertex_2d(geometry: &Euclidean2DGeometry) -> Option<([f64; 3], CoordinateFrame)> {
    match geometry {
        // A 2D point carries no elevation at all.
        Euclidean2DGeometry::Point(p) => {
            let [x, y] = p.position();
            Some(([x, y, 0.0], p.frame().clone()))
        }
        Euclidean2DGeometry::LineString(l) => {
            let [x, y] = *l.coords().first()?;
            Some(([x, y, l.elevation().unwrap_or(0.0)], l.frame().clone()))
        }
        Euclidean2DGeometry::Polygon(p) => {
            let [x, y] = *p.exterior().first()?;
            Some(([x, y, p.elevation().unwrap_or(0.0)], p.frame().clone()))
        }
        Euclidean2DGeometry::PolygonMesh(m) => {
            let [x, y] = m.first_face_vertex()?;
            Some(([x, y, m.elevation().unwrap_or(0.0)], m.frame().clone()))
        }
        Euclidean2DGeometry::TriangularMesh(m) => {
            let [i, _, _] = m.triangles().next()?;
            let [x, y] = m.vertices()[i as usize];
            Some(([x, y, m.elevation().unwrap_or(0.0)], m.frame().clone()))
        }
        Euclidean2DGeometry::Collection(c) => c.members().iter().find_map(first_vertex_2d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::polygon::Polygon3D;
    use crate::{Euclidean3DGeometry, Geometry, GeometryCollection};

    fn triangle(ring: Vec<[f64; 3]>) -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    #[test]
    fn a_closing_vertex_is_not_counted() {
        let closed = triangle(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ]);
        let open = triangle(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(closed.count_vertices(), 3);
        assert_eq!(open.count_vertices(), 3);
    }

    #[test]
    fn a_face_sums_its_exterior_and_every_hole() {
        let with_hole = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 4.0, 0.0],
                [0.0, 4.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            vec![vec![
                [1.0, 1.0, 0.0],
                [1.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
                [3.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ]],
        );
        assert_eq!(with_hole.count_vertices(), 8);
    }

    #[test]
    fn a_point_is_one_and_a_curve_is_its_coordinates() {
        let point = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        let curve = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]],
        );
        assert_eq!(point.count_vertices(), 1);
        assert_eq!(curve.count_vertices(), 3);
    }

    /// A collection sums its members, and an absent geometry contributes
    /// nothing rather than refusing to be counted.
    #[test]
    fn a_collection_sums_its_members_and_none_counts_zero() {
        let collection = GeometryCollection::new(vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(triangle(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ])))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Euclidean,
                [0.0, 0.0, 0.0],
            ))),
            Geometry::None,
        ]);
        assert_eq!(Geometry::GeometryCollection(collection).count_vertices(), 4);
    }

    fn point(x: f64, y: f64, z: f64) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::default(),
            [x, y, z],
        )))
    }

    fn line() -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            CoordinateFrame::default(),
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        )))
    }

    fn square() -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::default(),
                [
                    [7.0, 8.0, 9.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [7.0, 8.0, 9.0],
                ],
                std::iter::empty::<Vec<[f64; 3]>>(),
            ),
        )))
    }

    #[test]
    fn a_point_is_its_own_first_vertex() {
        assert_eq!(
            first_vertex(&point(1.0, 2.0, 3.0)).unwrap().0,
            [1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn a_line_uses_its_first_vertex() {
        assert_eq!(first_vertex(&line()).unwrap().0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn a_face_uses_its_first_exterior_vertex() {
        assert_eq!(first_vertex(&square()).unwrap().0, [7.0, 8.0, 9.0]);
    }

    #[test]
    fn geometry_none_has_no_first_vertex() {
        assert!(first_vertex(&Geometry::None).is_none());
    }

    #[test]
    fn the_frame_travels_with_the_vertex() {
        let (_p, frame) = first_vertex(&point(1.0, 2.0, 3.0)).unwrap();
        assert_eq!(frame, CoordinateFrame::default());
    }

    #[test]
    fn a_2d_leaf_uses_its_elevation_as_the_height() {
        // `Point2D` carries no elevation, so the lift is tested on `LineString2D`.
        use crate::line_string::LineString2D;
        use crate::Euclidean2DGeometry;

        let l = LineString2D::from_coords_at_elevation(
            CoordinateFrame::default(),
            vec![[1.0, 2.0], [3.0, 4.0]],
            42.5,
        );
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(l));
        assert_eq!(first_vertex(&g).unwrap().0, [1.0, 2.0, 42.5]);
    }

    #[test]
    fn a_2d_leaf_without_elevation_uses_zero_height() {
        use crate::point::Point2D;
        use crate::Euclidean2DGeometry;

        let p = Point2D::new(CoordinateFrame::default(), [1.0, 2.0]);
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Point(p));
        assert_eq!(first_vertex(&g).unwrap().0, [1.0, 2.0, 0.0]);
    }

    #[test]
    fn a_polygon_mesh_uses_its_first_faces_first_vertex_not_the_vertex_pool_head() {
        use crate::polygon_mesh::PolygonMesh3D;

        // The vertex pool's own head ([9,9,9]) is not referenced by the first
        // face, so reading it instead of walking the CSR face topology would
        // give the wrong answer.
        let mesh = PolygonMesh3D::from_parts(
            CoordinateFrame::default(),
            vec![
                [9.0, 9.0, 9.0],
                [1.0, 1.0, 1.0],
                [2.0, 2.0, 2.0],
                [3.0, 3.0, 3.0],
            ],
            vec![vec![1u32, 2, 3], vec![0u32, 1, 2]],
        )
        .unwrap();
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));
        assert_eq!(first_vertex(&g).unwrap().0, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn a_triangular_mesh_uses_its_first_triangles_first_vertex_not_the_vertex_pool_head() {
        use crate::triangular_mesh::TriangularMesh3D;

        // Same trap as the polygon-mesh case: the pool is ordered so its head
        // is not the first triangle's first vertex.
        let mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::default(),
            vec![
                [0.0, 0.0, 9.0],
                [1.0, 0.0, 9.0],
                [1.0, 1.0, 9.0],
                [0.0, 0.0, 4.0],
                [1.0, 0.0, 4.0],
                [1.0, 1.0, 4.0],
            ],
            [3u32, 4, 5, 0, 1, 2],
        )
        .unwrap();
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));
        assert_eq!(first_vertex(&g).unwrap().0, [0.0, 0.0, 4.0]);
    }

    #[test]
    fn a_solid_uses_its_exterior_shells_first_vertex_and_the_solids_own_frame() {
        use crate::coordinate::EpsgCode;
        use crate::solid::Solid;
        use crate::triangular_mesh::TriangularMesh3DData;

        // A `Solid`'s shell is coordinate-free (`TriangularMesh3DData` here
        // carries no frame of its own); the frame lives on the `Solid`. Using
        // a distinctive, non-default frame (EPSG:6677) makes it obvious if a
        // future refactor read a shell-level frame instead.
        let shell = TriangularMesh3DData::from_parts(
            vec![[5.0, 6.0, 7.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        let solid = Solid::from_exterior(CoordinateFrame::Crs(EpsgCode::new(6677)), shell);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid)));

        let (pos, frame) = first_vertex(&g).unwrap();
        assert_eq!(pos, [5.0, 6.0, 7.0]);
        assert_eq!(frame, CoordinateFrame::Crs(EpsgCode::new(6677)));
    }

    #[test]
    fn a_collection_returns_its_first_members_vertex_not_its_last() {
        use crate::collection::Collection3D;

        // Both members carry a vertex; a bug that returned the last member
        // (or picked one arbitrarily) would still pass an `is_some()`-only
        // check but fail this one.
        let members = [
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::default(), [1.0, 2.0, 3.0])),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::default(), [9.0, 9.0, 9.0])),
        ];
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(members)));
        assert_eq!(first_vertex(&g).unwrap().0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn a_nested_collection_recurses_past_a_member_with_no_vertex() {
        use crate::collection::Collection3D;
        use crate::point_cloud::PointCloud;
        use crate::GeometryCollection;

        // The outer collection's first member (a `PointCloud`) has no
        // vertex of its own; the real vertex sits one level deeper, in a
        // nested collection. Only actual recursion reaches it.
        let leading_member_with_no_position =
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::PointCloud(Box::new(PointCloud::from_positions(
                    CoordinateFrame::default(),
                    vec![[0.0, 0.0, 0.0]],
                ))),
            ])));
        let nested =
            Geometry::GeometryCollection(GeometryCollection::new([Geometry::Euclidean3D(
                Euclidean3DGeometry::Point(Point3D::new(
                    CoordinateFrame::default(),
                    [42.0, 43.0, 44.0],
                )),
            )]));
        let outer = Geometry::GeometryCollection(GeometryCollection::new([
            leading_member_with_no_position,
            nested,
        ]));
        assert_eq!(first_vertex(&outer).unwrap().0, [42.0, 43.0, 44.0]);
    }

    #[test]
    fn a_point_cloud_has_no_first_vertex() {
        use crate::point_cloud::PointCloud;

        let cloud = PointCloud::from_positions(CoordinateFrame::default(), vec![[1.0, 2.0, 3.0]]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud)));
        assert_eq!(first_vertex(&g), None);
    }

    #[test]
    fn an_unevaluated_csg_tree_has_no_first_vertex() {
        use crate::csg::{Csg, ThreeDimensional};
        use crate::solid::Solid;
        use crate::triangular_mesh::TriangularMesh3DData;

        let operand = || {
            ThreeDimensional::Solid(Box::new(Solid::from_exterior(
                CoordinateFrame::default(),
                TriangularMesh3DData::from_parts(
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    [0u32, 1, 2],
                )
                .unwrap(),
            )))
        };
        let csg = Csg::Union(Box::new(operand()), Box::new(operand()));
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg));
        assert_eq!(first_vertex(&g), None);
    }
}

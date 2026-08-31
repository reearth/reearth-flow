//! Pulling from the new geometry model the two things CZML wants: a
//! representative position, and a set of faces.
//!
//! These are genuinely different questions. Embedded and Grouped modes place an
//! entity at one point; the areal path draws its surface. A line's first vertex
//! is a fine *position* and a terrible *shape*.

use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// One representative point for a feature, with the frame it is expressed in.
///
/// Returned **in the geometry's own frame and its own coordinate order** — no
/// reprojection and no axis swapping happen here. Reprojection happens later
/// (`coords::to_wgs84`), and the lat/lon swap into CZML's `cartographicDegrees`
/// happens later still, in the packet builder.
///
/// `None` when the geometry carries no coordinates at all.
pub(crate) fn position_of(geometry: &Geometry) -> Option<([f64; 3], CoordinateFrame)> {
    match geometry {
        Geometry::None => None,
        Geometry::Euclidean3D(g) => position_3d(g),
        Geometry::Euclidean2D(g) => position_2d(g),
        Geometry::GeometryCollection(c) => c.members().iter().find_map(position_of),
    }
}

/// [`position_of`] for a 3D-embedded geometry.
fn position_3d(geometry: &Euclidean3DGeometry) -> Option<([f64; 3], CoordinateFrame)> {
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
        Euclidean3DGeometry::Collection(c) => c.members().iter().find_map(position_3d),
    }
}

/// [`position_of`] for a 2D-embedded geometry. A 2D leaf's optional elevation
/// becomes the height when present, `0.0` when absent — today's writer
/// hardcodes `0.0` for every 2D feature, which silently flattens genuine 2.5D
/// data.
fn position_2d(geometry: &Euclidean2DGeometry) -> Option<([f64; 3], CoordinateFrame)> {
    // `Point2D` has no elevation field at all: a position cannot lie at a
    // height, so it always lifts to `0.0`.
    match geometry {
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
        Euclidean2DGeometry::Collection(c) => c.members().iter().find_map(position_2d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::line_string::LineString3D;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

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
    fn a_point_is_its_own_position() {
        assert_eq!(
            position_of(&point(1.0, 2.0, 3.0)).unwrap().0,
            [1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn a_line_uses_its_first_vertex() {
        // Correct for a *position*. The line's full shape is the areal path's
        // job (Task 5), not this one's.
        assert_eq!(position_of(&line()).unwrap().0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn a_face_uses_its_first_exterior_vertex() {
        assert_eq!(position_of(&square()).unwrap().0, [7.0, 8.0, 9.0]);
    }

    #[test]
    fn geometry_none_has_no_position() {
        assert!(position_of(&Geometry::None).is_none());
    }

    #[test]
    fn the_frame_travels_with_the_position() {
        let (_p, frame) = position_of(&point(1.0, 2.0, 3.0)).unwrap();
        assert_eq!(frame, CoordinateFrame::default());
    }

    #[test]
    fn a_2d_leaf_uses_its_elevation_as_the_height() {
        // `Point2D` has no elevation field at all (a position cannot lie at a
        // height, by the geometry crate's own design — see
        // `reearth_flow_geometry::point::ops`'s `impl Elevation for Point2D`),
        // so `Point2D::new_at_elevation` from the brief does not exist and this
        // case is tested on `LineString2D` instead: any 2D leaf that carries an
        // elevation exercises the same `position_2d` height-fallback logic.
        use reearth_flow_geometry::line_string::LineString2D;
        use reearth_flow_geometry::Euclidean2DGeometry;

        let l = LineString2D::from_coords_at_elevation(
            CoordinateFrame::default(),
            vec![[1.0, 2.0], [3.0, 4.0]],
            42.5,
        );
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(l));
        assert_eq!(position_of(&g).unwrap().0, [1.0, 2.0, 42.5]);
    }

    #[test]
    fn a_2d_leaf_without_elevation_uses_zero_height() {
        // Today's writer hardcodes 0.0 for every 2D feature; this pins the
        // fallback for the case that has no elevation to fall back from.
        use reearth_flow_geometry::point::Point2D;
        use reearth_flow_geometry::Euclidean2DGeometry;

        let p = Point2D::new(CoordinateFrame::default(), [1.0, 2.0]);
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Point(p));
        assert_eq!(position_of(&g).unwrap().0, [1.0, 2.0, 0.0]);
    }

    #[test]
    fn a_polygon_mesh_uses_its_first_faces_first_vertex_not_the_vertex_pool_head() {
        use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;

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
        assert_eq!(position_of(&g).unwrap().0, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn a_triangular_mesh_uses_its_first_triangles_first_vertex_not_the_vertex_pool_head() {
        use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;

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
        assert_eq!(position_of(&g).unwrap().0, [0.0, 0.0, 4.0]);
    }

    #[test]
    fn a_solid_uses_its_exterior_shells_first_vertex_and_the_solids_own_frame() {
        use reearth_flow_geometry::coordinate::EpsgCode;
        use reearth_flow_geometry::solid::Solid;
        use reearth_flow_geometry::triangular_mesh::TriangularMesh3DData;

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

        let (pos, frame) = position_of(&g).unwrap();
        assert_eq!(pos, [5.0, 6.0, 7.0]);
        assert_eq!(frame, CoordinateFrame::Crs(EpsgCode::new(6677)));
    }

    #[test]
    fn a_collection_returns_its_first_members_position_not_its_last() {
        use reearth_flow_geometry::collection::Collection3D;

        // Both members carry a position; a bug that returned the last member
        // (or picked one arbitrarily) would still pass an `is_some()`-only
        // check but fail this one.
        let members = [
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::default(), [1.0, 2.0, 3.0])),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::default(), [9.0, 9.0, 9.0])),
        ];
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(members)));
        assert_eq!(position_of(&g).unwrap().0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn a_nested_collection_recurses_past_a_member_with_no_position() {
        use reearth_flow_geometry::collection::Collection3D;
        use reearth_flow_geometry::point_cloud::PointCloud;
        use reearth_flow_geometry::GeometryCollection;

        // The outer collection's first member (a `PointCloud`) has no
        // position of its own; the real position sits one level deeper, in a
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
        assert_eq!(position_of(&outer).unwrap().0, [42.0, 43.0, 44.0]);
    }

    #[test]
    fn a_point_cloud_has_no_position() {
        use reearth_flow_geometry::point_cloud::PointCloud;

        let cloud = PointCloud::from_positions(CoordinateFrame::default(), vec![[1.0, 2.0, 3.0]]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(cloud)));
        assert_eq!(position_of(&g), None);
    }

    #[test]
    fn an_unevaluated_csg_tree_has_no_position() {
        use reearth_flow_geometry::csg::{Csg, ThreeDimensional};
        use reearth_flow_geometry::solid::Solid;
        use reearth_flow_geometry::triangular_mesh::TriangularMesh3DData;

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
        assert_eq!(position_of(&g), None);
    }
}

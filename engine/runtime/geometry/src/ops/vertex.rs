//! Vertex counting, the first vertex of a geometry, and vertex selection by
//! position.
//!
//! A ring's closing vertex repeats its first one, so it is not counted: a
//! triangle counts three whether or not it is stored closed. Whether a ring is
//! closed is a validation question, not a counting one.

use std::num::NonZeroUsize;

use super::UnsupportedOperation;
use crate::coordinate::CoordinateFrame;
use crate::line_string::{LineString2D, LineString3D};
use crate::point::{Point2D, Point3D};
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

/// Which vertices [`SelectVertices`] leaves: those in the range, or all others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexSelection {
    Keep,
    Remove,
}

/// A run of vertices picked by position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexRange {
    /// Zero-based position of the first vertex. A negative start counts back
    /// from the end, so `-1` is the last vertex.
    pub start: i64,
    /// How many vertices the run spans toward the end. A run that passes the
    /// last vertex stops there.
    pub count: NonZeroUsize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectVerticesError {
    /// The geometry type has no single chain of vertices to number.
    UnsupportedGeometry(UnsupportedOperation),
    /// The range starts outside the geometry's vertices.
    StartOutOfRange { start: i64, vertex_count: usize },
}

impl core::fmt::Display for SelectVerticesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SelectVerticesError::UnsupportedGeometry(e) => e.fmt(f),
            SelectVerticesError::StartOutOfRange {
                start,
                vertex_count,
            } => write!(
                f,
                "start index {start} is outside the {vertex_count} vertices"
            ),
        }
    }
}

impl std::error::Error for SelectVerticesError {}

impl From<UnsupportedOperation> for SelectVerticesError {
    fn from(e: UnsupportedOperation) -> Self {
        SelectVerticesError::UnsupportedGeometry(e)
    }
}

/// Keep or remove a run of vertices picked by position, and build the geometry
/// the remaining vertices form.
///
/// Vertices are numbered in stored order. Unlike [`CountVertices`], a ring's
/// closing vertex is numbered as a vertex of its own. A face numbers its
/// exterior ring, then each interior ring.
///
/// The number of remaining vertices decides the result, in the input's
/// dimension and frame: none gives [`Geometry::None`], one a point, and two or
/// more a line string. A 2D line string keeps the elevation; a 2D point has
/// none to keep. A face whose vertices are all kept becomes a closed line
/// string, not a face.
///
/// A point is a chain of one vertex. A mesh, a solid, a point cloud, a boolean
/// tree and a collection have no single chain of vertices and are rejected.
#[enum_dispatch::enum_dispatch]
pub trait SelectVertices {
    /// The geometry left by the remaining vertices. The default reports the
    /// type as unsupported; a leaf opts in by overriding it.
    fn select_vertices(
        &self,
        selection: VertexSelection,
        range: VertexRange,
    ) -> Result<Geometry, SelectVerticesError> {
        let _ = (selection, range);
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "select_vertices",
        }
        .into())
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: SelectVertices + ?Sized> SelectVertices for Box<T> {
    fn select_vertices(
        &self,
        selection: VertexSelection,
        range: VertexRange,
    ) -> Result<Geometry, SelectVerticesError> {
        (**self).select_vertices(selection, range)
    }
}

/// The geometry left by selecting from a 2D vertex chain lying at `elevation`.
pub(crate) fn select_vertices_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    elevation: Option<f64>,
    selection: VertexSelection,
    range: VertexRange,
) -> Result<Geometry, SelectVerticesError> {
    let remaining = remaining_vertices(coords, selection, range)?;
    Ok(match remaining.as_slice() {
        [] => Geometry::None,
        [position] => Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            frame.clone(),
            *position,
        ))),
        _ => Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_raw_parts(frame.clone(), remaining.into_boxed_slice(), elevation),
        )),
    })
}

/// The geometry left by selecting from a 3D vertex chain.
pub(crate) fn select_vertices_3d(
    frame: &CoordinateFrame,
    coords: &[[f64; 3]],
    selection: VertexSelection,
    range: VertexRange,
) -> Result<Geometry, SelectVerticesError> {
    let remaining = remaining_vertices(coords, selection, range)?;
    Ok(match remaining.as_slice() {
        [] => Geometry::None,
        [position] => Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            frame.clone(),
            *position,
        ))),
        _ => Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_raw_parts(frame.clone(), remaining.into_boxed_slice()),
        )),
    })
}

fn remaining_vertices<const N: usize>(
    coords: &[[f64; N]],
    selection: VertexSelection,
    range: VertexRange,
) -> Result<Vec<[f64; N]>, SelectVerticesError> {
    let out_of_range = SelectVerticesError::StartOutOfRange {
        start: range.start,
        vertex_count: coords.len(),
    };
    let start = if range.start < 0 {
        coords
            .len()
            .checked_sub(range.start.unsigned_abs() as usize)
    } else {
        usize::try_from(range.start).ok()
    }
    .filter(|&start| start < coords.len())
    .ok_or(out_of_range)?;
    let end = start.saturating_add(range.count.get()).min(coords.len());
    Ok(match selection {
        VertexSelection::Keep => coords[start..end].to_vec(),
        VertexSelection::Remove => coords[..start]
            .iter()
            .chain(&coords[end..])
            .copied()
            .collect(),
    })
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

        // The pool head ([9,9,9]) is not used by the first face.
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

        // The shell carries no frame; the frame lives on the `Solid`.
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

        // The first member, a point cloud, has no vertex.
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

#[cfg(test)]
mod select_vertices_tests {
    use super::*;
    use crate::collection::Collection2D;
    use crate::coordinate::EpsgCode;
    use crate::polygon::{Polygon2D, Polygon3D};
    use crate::solid::Solid;
    use crate::triangular_mesh::{TriangularMesh3D, TriangularMesh3DData};
    use pretty_assertions::assert_eq;

    use VertexSelection::{Keep, Remove};

    fn select(
        geometry: &Geometry,
        selection: VertexSelection,
        start: i64,
        count: usize,
    ) -> Result<Geometry, SelectVerticesError> {
        let count = NonZeroUsize::new(count).unwrap();
        geometry.select_vertices(selection, VertexRange { start, count })
    }

    fn frame() -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(6677))
    }

    fn line_2d(coords: &[[f64; 2]], elevation: Option<f64>) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_raw_parts(frame(), coords.into(), elevation),
        ))
    }

    fn line_3d(coords: &[[f64; 3]]) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_raw_parts(frame(), coords.into()),
        ))
    }

    fn point_2d(position: [f64; 2]) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame(), position)))
    }

    fn point_3d(position: [f64; 3]) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(frame(), position)))
    }

    const OPEN_2D: [[f64; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
    const OPEN_3D: [[f64; 3]; 4] = [
        [0.0, 0.0, 5.0],
        [1.0, 0.0, 6.0],
        [2.0, 0.0, 7.0],
        [3.0, 0.0, 8.0],
    ];
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 1.0],
        [10.0, 0.0, 1.0],
        [10.0, 10.0, 1.0],
        [0.0, 10.0, 1.0],
        [0.0, 0.0, 1.0],
    ];
    const HOLE: [[f64; 3]; 5] = [
        [2.0, 2.0, 1.0],
        [2.0, 4.0, 1.0],
        [4.0, 4.0, 1.0],
        [4.0, 2.0, 1.0],
        [2.0, 2.0, 1.0],
    ];

    #[test]
    fn a_line_string_keeps_or_removes_the_range() {
        let line_2d_at_3 = line_2d(&OPEN_2D, Some(3.0));
        let square = line_3d(&SQUARE);
        let cases = [
            // One remaining vertex is a point of the same dimension.
            ("first 2D", &line_2d_at_3, Keep, 0, 1, point_2d([0.0, 0.0])),
            (
                "first 3D",
                &line_3d(&OPEN_3D),
                Keep,
                0,
                1,
                point_3d(OPEN_3D[0]),
            ),
            (
                "last",
                &line_3d(&OPEN_3D),
                Keep,
                -1,
                1,
                point_3d(OPEN_3D[3]),
            ),
            // The closing vertex of a closed line is a vertex of its own.
            ("closing", &square, Keep, -1, 1, point_3d(SQUARE[4])),
            ("before closing", &square, Keep, -2, 1, point_3d(SQUARE[3])),
            // Two or more form a line string keeping the frame and elevation.
            (
                "keep 2D run",
                &line_2d_at_3,
                Keep,
                1,
                2,
                line_2d(&OPEN_2D[1..3], Some(3.0)),
            ),
            (
                "keep 3D run",
                &line_3d(&OPEN_3D),
                Keep,
                1,
                2,
                line_3d(&OPEN_3D[1..3]),
            ),
            (
                "keep past end",
                &line_3d(&OPEN_3D),
                Keep,
                2,
                10,
                line_3d(&OPEN_3D[2..]),
            ),
            (
                "remove run",
                &line_2d(&OPEN_2D, None),
                Remove,
                1,
                2,
                line_2d(&[OPEN_2D[0], OPEN_2D[3]], None),
            ),
            (
                "remove past end",
                &line_2d(&OPEN_2D, None),
                Remove,
                2,
                10,
                line_2d(&OPEN_2D[..2], None),
            ),
            (
                "remove all",
                &line_3d(&OPEN_3D),
                Remove,
                0,
                4,
                Geometry::None,
            ),
        ];
        for (label, geometry, selection, start, count, expected) in cases {
            assert_eq!(
                select(geometry, selection, start, count),
                Ok(expected),
                "{label}"
            );
        }
    }

    #[test]
    fn a_face_numbers_its_exterior_then_each_interior() {
        let face = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame(), SQUARE, Vec::<Vec<[f64; 3]>>::new()),
        )));
        assert_eq!(select(&face, Keep, 0, 5), Ok(line_3d(&SQUARE)));

        let flat: Vec<[f64; 2]> = SQUARE.iter().map(|&[x, y, _]| [x, y]).collect();
        let face = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings_at_elevation(
                frame(),
                flat.clone(),
                Vec::<Vec<[f64; 2]>>::new(),
                1.0,
            ),
        )));
        assert_eq!(select(&face, Keep, 0, 5), Ok(line_2d(&flat, Some(1.0))));

        let face = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame(), SQUARE, [HOLE]),
        )));
        assert_eq!(select(&face, Keep, -1, 1), Ok(point_3d(HOLE[4])));
        assert_eq!(select(&face, Keep, 5, 1), Ok(point_3d(HOLE[0])));
    }

    #[test]
    fn a_point_is_a_chain_of_one_vertex() {
        for point in [point_2d([1.0, 2.0]), point_3d([1.0, 2.0, 3.0])] {
            assert_eq!(select(&point, Keep, -1, 100), Ok(point.clone()));
            assert_eq!(select(&point, Remove, 0, 100), Ok(Geometry::None));
            for start in [1, -2] {
                assert_eq!(
                    select(&point, Keep, start, 1),
                    Err(SelectVerticesError::StartOutOfRange {
                        start,
                        vertex_count: 1
                    })
                );
            }
        }
    }

    #[test]
    fn a_start_outside_the_vertices_is_rejected() {
        for (start, vertex_count, geometry) in [
            (4, 4, line_3d(&OPEN_3D)),
            (-5, 4, line_3d(&OPEN_3D)),
            (0, 0, Geometry::None),
        ] {
            assert_eq!(
                select(&geometry, Keep, start, 1),
                Err(SelectVerticesError::StartOutOfRange {
                    start,
                    vertex_count
                }),
                "{start}"
            );
        }
    }

    #[test]
    fn a_geometry_without_a_single_vertex_chain_is_rejected() {
        let corners = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let mesh = TriangularMesh3D::from_parts(frame(), corners.clone(), [0u32, 1, 2]).unwrap();
        let shell = TriangularMesh3DData::from_parts(corners, [0u32, 1, 2]).unwrap();
        let collection = Collection2D::new([Euclidean2DGeometry::Point(Point2D::new(
            frame(),
            [0.0, 0.0],
        ))]);
        for (label, geometry) in [
            (
                "mesh",
                Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh))),
            ),
            (
                "solid",
                Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(Solid::from_exterior(
                    frame(),
                    shell,
                )))),
            ),
            (
                "collection",
                Geometry::Euclidean2D(Euclidean2DGeometry::Collection(collection)),
            ),
        ] {
            assert!(
                matches!(
                    select(&geometry, Keep, 0, 1),
                    Err(SelectVerticesError::UnsupportedGeometry(_))
                ),
                "{label}"
            );
        }
    }
}

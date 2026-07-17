//! Point position relative to 2D geometry.
//!
//! [`point_position_2d`] classifies a coordinate against any 2D geometry as
//! [`Inside`](CoordPos::Inside) (the geometry's interior),
//! [`OnBoundary`](CoordPos::OnBoundary), or [`Outside`](CoordPos::Outside),
//! with OGC point-set semantics per leaf:
//!
//! - **Point**: the interior is the point itself; the boundary is empty.
//! - **LineString**: the interior is the chain minus its endpoints; the
//!   endpoints are the boundary, unless the chain is closed (then everything
//!   is interior).
//! - **Polygon / meshes**: faces are treated as a point-set union. A
//!   coordinate on a ring edge shared by two faces (or on a mesh vertex whose
//!   incident faces cover a full disk) is *interior* to the union, not on its
//!   boundary; the refinement is an exact angular-coverage test around the
//!   coordinate. It assumes valid winding (exteriors CCW, holes CW) and
//!   non-overlapping face interiors; on invalid input and on degenerate
//!   (all-equal) rings it degrades to `OnBoundary`, never to a false `Inside`.
//! - **Collections**: the union of the members, the highest member
//!   classification wins (`Inside` > `OnBoundary` > `Outside`), with the
//!   line-endpoint boundary counted mod 2 across all line members.
//!
//! The optional per-vertex elevation of 2D leaves is ignored throughout.

use super::kernel::{coord_pos_relative_to_edges, point_on_segment, same_direction, CoordPos};
use super::view::{AreaView, FaceView, Leaf2D, Operand2D, RingView};
use crate::Euclidean2DGeometry;

/// Position of a coordinate relative to a 2D geometry, treating collections as
/// point-set unions of their members.
pub fn point_position_2d(coord: [f64; 2], geometry: &Euclidean2DGeometry) -> CoordPos {
    union_position(coord, &Operand2D::new(geometry))
}

/// Position of a coordinate relative to an operand's union of leaves.
pub(crate) fn union_position(coord: [f64; 2], operand: &Operand2D<'_>) -> CoordPos {
    // Areal members: exact union with shared-boundary refinement.
    let area_pos = areal_union_position(coord, operand.areas());
    if area_pos == CoordPos::Inside {
        return CoordPos::Inside;
    }

    // Line members: interior on any chain, boundary endpoints counted mod 2
    // across members (two chains meeting at an endpoint join into interior).
    let mut on_any_line = false;
    let mut boundary_endpoints = 0usize;
    // Point members.
    let mut on_point = false;

    for prepared in &operand.leaves {
        match prepared.leaf {
            Leaf2D::Point(p) => on_point |= coord == p.position(),
            Leaf2D::Line(l) => {
                let coords = l.coords();
                if on_chain(coord, coords) {
                    on_any_line = true;
                }
                let closed = coords.len() >= 2 && coords.first() == coords.last();
                if !closed && coords.len() >= 2 {
                    if coord == coords[0] {
                        boundary_endpoints += 1;
                    }
                    if coord == coords[coords.len() - 1] {
                        boundary_endpoints += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let line_pos = if !on_any_line {
        CoordPos::Outside
    } else if boundary_endpoints % 2 == 1 {
        CoordPos::OnBoundary
    } else {
        CoordPos::Inside
    };
    let point_pos = if on_point {
        CoordPos::Inside
    } else {
        CoordPos::Outside
    };

    max_pos(max_pos(area_pos, line_pos), point_pos)
}

/// The higher of two classifications (`Inside` > `OnBoundary` > `Outside`).
fn max_pos(a: CoordPos, b: CoordPos) -> CoordPos {
    fn rank(p: CoordPos) -> u8 {
        match p {
            CoordPos::Outside => 0,
            CoordPos::OnBoundary => 1,
            CoordPos::Inside => 2,
        }
    }
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

/// Whether the coordinate lies anywhere on the chain (interior or endpoint).
fn on_chain(coord: [f64; 2], coords: &[[f64; 2]]) -> bool {
    match coords {
        [] => false,
        [single] => coord == *single,
        _ => coords
            .windows(2)
            .any(|e| point_on_segment(coord, e[0], e[1])),
    }
}

/// Position of a coordinate on a polyline: `Inside` on the chain (including a
/// closed chain's join), `OnBoundary` at an open chain's endpoints, `Outside`
/// off the chain. A single-vertex chain is point-like: its vertex is interior.
pub(crate) fn line_position(coord: [f64; 2], coords: &[[f64; 2]]) -> CoordPos {
    if !on_chain(coord, coords) {
        return CoordPos::Outside;
    }
    let closed = coords.len() >= 2 && coords.first() == coords.last();
    if !closed && coords.len() >= 2 && (coord == coords[0] || coord == coords[coords.len() - 1]) {
        CoordPos::OnBoundary
    } else {
        CoordPos::Inside
    }
}

/// Position of a coordinate relative to a single ring's closed loop.
pub(crate) fn ring_position(coord: [f64; 2], ring: RingView<'_>) -> CoordPos {
    coord_pos_relative_to_edges(coord, ring.edges())
}

/// Position of a coordinate relative to one face (exterior minus holes).
pub(crate) fn face_position(coord: [f64; 2], face: FaceView<'_>) -> CoordPos {
    match ring_position(coord, face.exterior()) {
        CoordPos::Outside => CoordPos::Outside,
        CoordPos::OnBoundary => CoordPos::OnBoundary,
        CoordPos::Inside => {
            for hole in face.interiors() {
                match ring_position(coord, hole) {
                    CoordPos::Inside => return CoordPos::Outside,
                    CoordPos::OnBoundary => return CoordPos::OnBoundary,
                    CoordPos::Outside => {}
                }
            }
            CoordPos::Inside
        }
    }
}

/// Position of a coordinate relative to the union of areal views.
///
/// Strictly inside any face is inside the union. On a face boundary, the
/// classification is refined by angular coverage: the coordinate is interior
/// to the union exactly when the incident faces cover a full disk around it
/// (a shared mesh edge, or a fully surrounded vertex).
pub(crate) fn areal_union_position<'a, 'b>(
    coord: [f64; 2],
    areas: impl Iterator<Item = &'b AreaView<'a>>,
) -> CoordPos
where
    'a: 'b,
{
    let mut wedges: Vec<([f64; 2], [f64; 2])> = Vec::new();
    let mut on_boundary = false;
    let mut degenerate = false;
    for area in areas {
        for face in area.faces() {
            match face_position(coord, face) {
                CoordPos::Inside => return CoordPos::Inside,
                CoordPos::OnBoundary => {
                    on_boundary = true;
                    gather_wedges(coord, face, &mut wedges, &mut degenerate);
                }
                CoordPos::Outside => {}
            }
        }
    }
    if !on_boundary {
        CoordPos::Outside
    } else if !degenerate && wedges_cover(coord, &wedges) {
        CoordPos::Inside
    } else {
        CoordPos::OnBoundary
    }
}

/// Collect the angular wedges of face interior incident to `coord`, one per
/// occurrence of `coord` on the face's rings.
///
/// With valid winding the face interior lies locally left of every directed
/// ring edge (exterior CCW, holes CW), so an edge through `coord` contributes
/// the left half-plane `(toward end, toward start)` and a ring vertex at
/// `coord` contributes `(toward next, toward prev)`. Wedge ends are the actual
/// neighbor coordinates, keeping the later direction comparisons exact. A ring
/// collapsed entirely onto `coord` sets `degenerate` instead.
fn gather_wedges(
    coord: [f64; 2],
    face: FaceView<'_>,
    wedges: &mut Vec<([f64; 2], [f64; 2])>,
    degenerate: &mut bool,
) {
    for ring in face.rings() {
        let n = ring.open_len();
        for i in 0..n {
            let v = ring.coord(i);
            let next = ring.coord(if i + 1 == n { 0 } else { i + 1 });
            if v == coord {
                // Vertex occurrence: walk to the nearest distinct neighbors.
                let mut prev = None;
                for step in 1..n {
                    let c = ring.coord((i + n - step) % n);
                    if c != coord {
                        prev = Some(c);
                        break;
                    }
                }
                let mut nxt = None;
                for step in 1..n {
                    let c = ring.coord((i + step) % n);
                    if c != coord {
                        nxt = Some(c);
                        break;
                    }
                }
                match (nxt, prev) {
                    (Some(nxt), Some(prev)) => wedges.push((nxt, prev)),
                    _ => *degenerate = true,
                }
            } else if next != v && next != coord && point_on_segment(coord, v, next) {
                // Interior of the edge `v -> next`: the left half-plane.
                wedges.push((next, v));
            }
        }
    }
}

/// Whether disjoint wedges around `coord` cover the full disk: every wedge end
/// must continue into another wedge's start (same exact direction), leaving no
/// angular gap. Assumes non-overlapping face interiors.
fn wedges_cover(coord: [f64; 2], wedges: &[([f64; 2], [f64; 2])]) -> bool {
    if wedges.is_empty() {
        return false;
    }
    let mut starts: Vec<[f64; 2]> = wedges.iter().map(|w| w.0).collect();
    for &(_, end) in wedges {
        let Some(pos) = starts
            .iter()
            .position(|&start| same_direction(coord, end, start))
        else {
            return false;
        };
        starts.swap_remove(pos);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection2D;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString2D;
    use crate::point::Point2D;
    use crate::polygon::Polygon2D;
    use crate::polygon_mesh::PolygonMesh2D;
    use crate::triangular_mesh::TriangularMesh2D;
    use pretty_assertions::assert_eq;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn polygon_with_hole() -> Euclidean2DGeometry {
        let square = [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0], [0.0, 0.0]];
        let hole = vec![[2.0, 2.0], [2.0, 6.0], [6.0, 6.0], [6.0, 2.0], [2.0, 2.0]];
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(e(), square, vec![hole])))
    }

    /// Two quads sharing the edge x = 2 (both wound CCW).
    fn two_quads() -> Euclidean2DGeometry {
        let mesh = PolygonMesh2D::from_parts(
            e(),
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
        Euclidean2DGeometry::PolygonMesh(Box::new(mesh))
    }

    #[test]
    fn point_leaf_interior_is_itself() {
        let p = Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 2.0]));
        assert_eq!(point_position_2d([1.0, 2.0], &p), CoordPos::Inside);
        assert_eq!(point_position_2d([1.0, 2.1], &p), CoordPos::Outside);
    }

    #[test]
    fn line_endpoints_are_boundary_interior_is_inside() {
        let l = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]],
        ));
        assert_eq!(point_position_2d([2.0, 0.0], &l), CoordPos::Inside);
        assert_eq!(point_position_2d([4.0, 0.0], &l), CoordPos::Inside); // mid vertex
        assert_eq!(point_position_2d([0.0, 0.0], &l), CoordPos::OnBoundary);
        assert_eq!(point_position_2d([4.0, 4.0], &l), CoordPos::OnBoundary);
        assert_eq!(point_position_2d([1.0, 1.0], &l), CoordPos::Outside);
    }

    #[test]
    fn closed_line_has_no_boundary() {
        let l = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]],
        ));
        assert_eq!(point_position_2d([0.0, 0.0], &l), CoordPos::Inside);
    }

    #[test]
    fn polygon_hole_positions() {
        let p = polygon_with_hole();
        assert_eq!(point_position_2d([1.0, 1.0], &p), CoordPos::Inside);
        assert_eq!(point_position_2d([4.0, 4.0], &p), CoordPos::Outside); // in the hole
        assert_eq!(point_position_2d([2.0, 4.0], &p), CoordPos::OnBoundary); // hole ring
        assert_eq!(point_position_2d([0.0, 4.0], &p), CoordPos::OnBoundary); // outer ring
        assert_eq!(point_position_2d([9.0, 4.0], &p), CoordPos::Outside);
    }

    #[test]
    fn shared_mesh_edge_is_interior() {
        let m = two_quads();
        // Interior of the shared edge x = 2.
        assert_eq!(point_position_2d([2.0, 1.0], &m), CoordPos::Inside);
        // The outer rim stays boundary.
        assert_eq!(point_position_2d([0.0, 1.0], &m), CoordPos::OnBoundary);
        assert_eq!(point_position_2d([3.0, 0.0], &m), CoordPos::OnBoundary);
        // A fully surrounded shared vertex is interior; a rim vertex is not.
        assert_eq!(point_position_2d([2.0, 0.0], &m), CoordPos::OnBoundary);
    }

    #[test]
    fn surrounded_vertex_is_interior() {
        // Four quads around the central vertex (1, 1).
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [2.0, 0.0],
                [0.0, 1.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [0.0, 2.0],
                [1.0, 2.0],
                [2.0, 2.0],
            ],
            vec![
                vec![0u32, 1, 4, 3],
                vec![1, 2, 5, 4],
                vec![4, 5, 8, 7],
                vec![3, 4, 7, 6],
            ],
        )
        .unwrap();
        let m = Euclidean2DGeometry::PolygonMesh(Box::new(mesh));
        assert_eq!(point_position_2d([1.0, 1.0], &m), CoordPos::Inside);
        // An edge midpoint between two quads is interior too.
        assert_eq!(point_position_2d([1.0, 0.5], &m), CoordPos::Inside);
        // A rim vertex is boundary.
        assert_eq!(point_position_2d([1.0, 0.0], &m), CoordPos::OnBoundary);
    }

    #[test]
    fn pinched_vertex_is_boundary() {
        // Two triangles touching only at (1, 1): opposite quadrants.
        let mesh = TriangularMesh2D::from_parts(
            e(),
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [2.0, 2.0], [1.0, 2.0]],
            [0u32, 1, 2, 2, 3, 4],
        )
        .unwrap();
        let m = Euclidean2DGeometry::TriangularMesh(Box::new(mesh));
        assert_eq!(point_position_2d([1.0, 1.0], &m), CoordPos::OnBoundary);
    }

    #[test]
    fn triangular_mesh_shared_edge_is_interior() {
        let mesh = TriangularMesh2D::from_parts(
            e(),
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let m = Euclidean2DGeometry::TriangularMesh(Box::new(mesh));
        // Midpoint of the shared diagonal.
        assert_eq!(point_position_2d([1.0, 1.0], &m), CoordPos::Inside);
        assert_eq!(point_position_2d([1.0, 0.0], &m), CoordPos::OnBoundary);
        assert_eq!(point_position_2d([1.5, 0.5], &m), CoordPos::Inside);
    }

    #[test]
    fn two_chains_joined_at_endpoint_make_it_interior() {
        let a = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[0.0, 0.0], [1.0, 0.0]],
        ));
        let b = Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[1.0, 0.0], [2.0, 0.0]],
        ));
        let c = Euclidean2DGeometry::Collection(Collection2D::new([a, b]));
        assert_eq!(point_position_2d([1.0, 0.0], &c), CoordPos::Inside);
        assert_eq!(point_position_2d([0.0, 0.0], &c), CoordPos::OnBoundary);
    }

    #[test]
    fn empty_geometry_is_outside() {
        let c = Euclidean2DGeometry::Collection(Collection2D::new([]));
        assert_eq!(point_position_2d([0.0, 0.0], &c), CoordPos::Outside);
    }
}

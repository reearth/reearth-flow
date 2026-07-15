//! The `contains` and `covers` predicates, with OGC point-set semantics.
//!
//! `covers(a, b)`: every point of `b` lies in the closure of `a`.
//! `contains(a, b)`: `covers(a, b)` **and** the interiors intersect — so a
//! geometry lying entirely on `a`'s boundary is covered but not contained.
//!
//! The evaluation is split-based rather than heuristic: every boundary segment
//! of `b` is split at each intersection with `a`'s boundary, so each piece lies
//! entirely in one region of `a` and its midpoint classifies the whole piece.
//! Three checks compose the answer:
//!
//! 1. **Boundary coverage** — every piece midpoint of `b`'s segments and ring
//!    edges must not be outside `a`.
//! 2. **Face interiors** — every face of an areal `b` contributes one interior
//!    sample point, which must be strictly inside `a`'s areal union. Since a
//!    face interior that never meets `a`'s boundary lies in a single region,
//!    one sample decides it (check 3 guarantees the premise).
//! 3. **Reverse leak** — no piece of `a`'s areal boundary may run strictly
//!    inside `b`'s areal interior while being true union boundary of `a` (a
//!    hole or gap of `a` swallowed by `b`'s interior). Internal shared mesh
//!    edges of `a` classify as interior and pass.
//!
//! Both operands are treated as point-set unions of their flattened leaves, so
//! collection containers are exact: a `b` spanning several touching members of
//! a collection `a` is contained. Split points and piece midpoints are
//! constructed (f64-rounded) coordinates; classification at them inherits that
//! rounding. Collections mixing 2D and 3D members error, as elsewhere.

use super::intersects::type_name_3d;
use super::kernel::{segment_intersection, CoordPos, SegmentIntersection};
use super::position::{areal_union_position, face_position, union_position};
use super::view::{flatten_2d, require_common_frame, FaceView, Leaf2D, Operand2D};
use super::Result;
use crate::ops::Aabb;
use crate::Geometry;

/// Whether `a` contains `b`: `b` lies in `a`'s closure and their interiors
/// intersect. Nothing contains an absent or empty geometry.
pub fn contains(a: &Geometry, b: &Geometry) -> Result<bool> {
    containment(a, b, true)
}

/// Whether `a` covers `b`: every point of `b` lies in `a`'s closure. Unlike
/// [`contains`], pure boundary contact suffices. Nothing covers an absent or
/// empty geometry.
pub fn covers(a: &Geometry, b: &Geometry) -> Result<bool> {
    containment(a, b, false)
}

fn containment(a: &Geometry, b: &Geometry, need_witness: bool) -> Result<bool> {
    let (a_leaves, b_leaves) = super::flatten_2d_pair(a, b)?;
    let a = Operand2D::from_leaves(a_leaves);
    let b = Operand2D::from_leaves(b_leaves);
    require_common_frame(&a, &b)?;
    Ok(containment_2d(&a, &b, need_witness))
}

/// Flatten a `Geometry` into its 2D leaves; the second value names the first
/// 3D leaf encountered, if any.
pub(crate) fn flatten_geometry(geometry: &Geometry) -> (Vec<Leaf2D<'_>>, Option<&'static str>) {
    fn walk<'a>(
        geometry: &'a Geometry,
        leaves: &mut Vec<Leaf2D<'a>>,
        first_3d: &mut Option<&'static str>,
    ) {
        match geometry {
            Geometry::None => {}
            Geometry::Euclidean2D(g) => flatten_2d(g, leaves),
            Geometry::Euclidean3D(g) => {
                first_3d.get_or_insert(type_name_3d(g));
            }
            Geometry::GeometryCollection(c) => {
                for member in c.members() {
                    walk(member, leaves, first_3d);
                }
            }
        }
    }
    let mut leaves = Vec::new();
    let mut first_3d = None;
    walk(geometry, &mut leaves, &mut first_3d);
    (leaves, first_3d)
}

fn containment_2d(a: &Operand2D<'_>, b: &Operand2D<'_>, need_witness: bool) -> bool {
    // Coverage requires b's extent within a's.
    let Some(bbox_b) = union_bbox(b) else {
        return false;
    };
    let Some(bbox_a) = union_bbox(a) else {
        return false;
    };
    if !bbox_within(&bbox_b, &bbox_a) {
        return false;
    }

    // The splitting edge set of `a`: areal ring edges and line segments.
    let a_edges: Vec<([f64; 2], [f64; 2])> = collect_edges(a);
    let mut witness = false;

    for prepared in &b.leaves {
        match prepared.leaf {
            Leaf2D::Point(p) => match union_position(p.position(), a) {
                CoordPos::Outside => return false,
                CoordPos::Inside => witness = true,
                CoordPos::OnBoundary => {}
            },
            Leaf2D::Line(l) => {
                let coords = l.coords();
                let closed = coords.len() >= 2 && coords.first() == coords.last();
                // Chain vertices: coverage, and interior vertices as witnesses.
                for (i, &v) in coords.iter().enumerate() {
                    match union_position(v, a) {
                        CoordPos::Outside => return false,
                        CoordPos::Inside => {
                            let endpoint = !closed && (i == 0 || i + 1 == coords.len());
                            // A single-vertex chain is point-like: all interior.
                            if !endpoint || coords.len() == 1 {
                                witness = true;
                            }
                        }
                        CoordPos::OnBoundary => {}
                    }
                }
                // Piece midpoints: coverage and witnesses (chain interior).
                for seg in coords.windows(2) {
                    for m in piece_midpoints(seg[0], seg[1], &a_edges) {
                        match union_position(m, a) {
                            CoordPos::Outside => return false,
                            CoordPos::Inside => witness = true,
                            CoordPos::OnBoundary => {}
                        }
                    }
                }
            }
            _ => {
                let view = prepared.area.as_ref().expect("leaf is areal");
                // Boundary coverage: every ring edge piece within a's closure.
                for (u, v) in view.edges() {
                    for m in piece_midpoints(u, v, &a_edges) {
                        if union_position(m, a) == CoordPos::Outside {
                            return false;
                        }
                    }
                }
                // Face interiors: one sample per face, strictly inside a's
                // areal union. A degenerate face has no interior and no sample.
                for face in view.faces() {
                    if let Some(sample) = face_interior_point(face) {
                        if areal_union_position(sample, a.areas()) != CoordPos::Inside {
                            return false;
                        }
                        witness = true;
                    }
                }
            }
        }
    }

    // Reverse leak: a piece of a's areal boundary strictly inside b's areal
    // interior must be interior to a as well (a shared mesh edge), or b's
    // interior extends past a across it.
    if b.areas().next().is_some() {
        let b_area_edges: Vec<([f64; 2], [f64; 2])> =
            b.areas().flat_map(|view| view.edges()).collect();
        for (u, v) in a.areas().flat_map(|view| view.edges()) {
            for m in piece_midpoints(u, v, &b_area_edges) {
                if areal_union_position(m, b.areas()) == CoordPos::Inside
                    && areal_union_position(m, a.areas()) != CoordPos::Inside
                {
                    return false;
                }
            }
        }
    }

    !need_witness || witness
}

/// The areal ring edges and line segments of an operand (points contribute
/// nothing to splitting).
fn collect_edges(operand: &Operand2D<'_>) -> Vec<([f64; 2], [f64; 2])> {
    let mut edges = Vec::new();
    for prepared in &operand.leaves {
        match prepared.leaf {
            Leaf2D::Point(_) => {}
            Leaf2D::Line(l) => {
                edges.extend(l.coords().windows(2).map(|s| (s[0], s[1])));
            }
            _ => {
                edges.extend(prepared.area.as_ref().expect("leaf is areal").edges());
            }
        }
    }
    edges
}

/// Split the segment `u -> v` at every intersection with `edges` and yield the
/// midpoint of each resulting piece. Each piece lies entirely in one region of
/// the geometry the edges came from, so its midpoint classifies it.
fn piece_midpoints(u: [f64; 2], v: [f64; 2], edges: &[([f64; 2], [f64; 2])]) -> Vec<[f64; 2]> {
    let mut cuts: Vec<[f64; 2]> = vec![u, v];
    for &(s, t) in edges {
        match segment_intersection(u, v, s, t) {
            Some(SegmentIntersection::SinglePoint { intersection, .. }) => cuts.push(intersection),
            Some(SegmentIntersection::Collinear { start, end }) => {
                cuts.push(start);
                cuts.push(end);
            }
            None => {}
        }
    }
    // Order along the segment by projection onto its direction.
    let d = [v[0] - u[0], v[1] - u[1]];
    let param = |p: &[f64; 2]| (p[0] - u[0]) * d[0] + (p[1] - u[1]) * d[1];
    cuts.sort_by(|p, q| param(p).total_cmp(&param(q)));
    cuts.dedup();
    cuts.windows(2)
        .map(|w| [(w[0][0] + w[1][0]) / 2.0, (w[0][1] + w[1][1]) / 2.0])
        .collect()
}

/// A point strictly inside the face, or `None` when the face has no interior.
///
/// Horizontal scanline construction: pick a scan height strictly between two
/// adjacent vertex levels (so no edge endpoint lies on the line), collect the
/// edge crossings of all rings, and take the midpoint of an even-odd interior
/// interval. Every candidate is verified with [`face_position`] before being
/// returned, so an exotic face can only fail to a `None`, never to a wrong
/// point.
fn face_interior_point(face: FaceView<'_>) -> Option<[f64; 2]> {
    let mut levels: Vec<f64> = face
        .rings()
        .flat_map(|r| r.coords().map(|c| c[1]))
        .collect();
    levels.sort_by(f64::total_cmp);
    levels.dedup();
    if levels.len() < 2 {
        return None;
    }

    for pair in levels.windows(2) {
        let y = (pair[0] + pair[1]) / 2.0;
        if !(y > pair[0] && y < pair[1]) {
            continue; // adjacent levels too close for a strict midpoint
        }
        // Crossings of the scanline with every ring edge; no endpoint lies on
        // it, so each edge either straddles cleanly or is skipped.
        let mut xs: Vec<f64> = Vec::new();
        for (s, t) in face.edges() {
            if (s[1] < y && t[1] > y) || (t[1] < y && s[1] > y) {
                xs.push(s[0] + (y - s[1]) * (t[0] - s[0]) / (t[1] - s[1]));
            }
        }
        xs.sort_by(f64::total_cmp);
        // Even-odd: [xs[0], xs[1]], [xs[2], xs[3]], … are interior intervals.
        for span in xs.chunks_exact(2) {
            let candidate = [(span[0] + span[1]) / 2.0, y];
            if face_position(candidate, face) == CoordPos::Inside {
                return Some(candidate);
            }
        }
    }
    None
}

/// The union of the operand's leaf boxes, `None` when every leaf is empty.
fn union_bbox(operand: &Operand2D<'_>) -> Option<Aabb> {
    operand
        .leaves
        .iter()
        .filter_map(|l| l.bbox)
        .reduce(Aabb::union)
}

/// Whether `inner` lies within `outer` (inclusive). 2D leaf boxes only.
fn bbox_within(inner: &Aabb, outer: &Aabb) -> bool {
    match (inner, outer) {
        (
            Aabb::D2 {
                min: imin,
                max: imax,
            },
            Aabb::D2 {
                min: omin,
                max: omax,
            },
        ) => (0..2).all(|i| omin[i] <= imin[i] && imax[i] <= omax[i]),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection2D;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::line_string::LineString2D;
    use crate::point::Point2D;
    use crate::polygon::Polygon2D;
    use crate::polygon_mesh::PolygonMesh2D;
    use crate::predicates::PredicateError;
    use crate::{Euclidean2DGeometry, Geometry};
    use pretty_assertions::assert_eq;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn g2(g: Euclidean2DGeometry) -> Geometry {
        Geometry::Euclidean2D(g)
    }

    fn point(x: f64, y: f64) -> Geometry {
        g2(Euclidean2DGeometry::Point(Point2D::new(e(), [x, y])))
    }

    fn line(coords: Vec<[f64; 2]>) -> Geometry {
        g2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            coords,
        )))
    }

    fn square(x: f64, y: f64, s: f64) -> Geometry {
        g2(square_2d(x, y, s))
    }

    fn square_2d(x: f64, y: f64, s: f64) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            [[x, y], [x + s, y], [x + s, y + s], [x, y + s], [x, y]],
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    /// `[0, 8]²` with the hole `[hx, hx+hs] × [hy, hy+hs]`.
    fn holey(hx: f64, hy: f64, hs: f64) -> Geometry {
        let outer = [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0], [0.0, 0.0]];
        let hole = vec![
            [hx, hy],
            [hx, hy + hs],
            [hx + hs, hy + hs],
            [hx + hs, hy],
            [hx, hy],
        ];
        g2(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(e(), outer, vec![hole]),
        )))
    }

    #[test]
    fn point_containment() {
        let sq = square(0.0, 0.0, 4.0);
        assert_eq!(contains(&sq, &point(2.0, 2.0)), Ok(true));
        // A boundary point is covered but not contained.
        assert_eq!(contains(&sq, &point(0.0, 2.0)), Ok(false));
        assert_eq!(covers(&sq, &point(0.0, 2.0)), Ok(true));
        assert_eq!(contains(&sq, &point(9.0, 2.0)), Ok(false));
        // In the hole is outside.
        let a = holey(2.0, 2.0, 4.0);
        assert_eq!(covers(&a, &point(4.0, 4.0)), Ok(false));
        assert_eq!(covers(&a, &point(2.0, 4.0)), Ok(true)); // on hole ring
        assert_eq!(contains(&a, &point(2.0, 4.0)), Ok(false));
    }

    #[test]
    fn point_contains_point() {
        assert_eq!(contains(&point(1.0, 1.0), &point(1.0, 1.0)), Ok(true));
        assert_eq!(contains(&point(1.0, 1.0), &point(1.0, 2.0)), Ok(false));
    }

    #[test]
    fn polygon_contains_line() {
        let sq = square(0.0, 0.0, 4.0);
        assert_eq!(contains(&sq, &line(vec![[1.0, 1.0], [3.0, 3.0]])), Ok(true));
        // Touching the boundary from inside still contains.
        assert_eq!(contains(&sq, &line(vec![[0.0, 0.0], [3.0, 3.0]])), Ok(true));
        // Along the boundary: covered, not contained.
        let rim = line(vec![[0.0, 0.0], [4.0, 0.0]]);
        assert_eq!(contains(&sq, &rim), Ok(false));
        assert_eq!(covers(&sq, &rim), Ok(true));
        // Poking out is neither.
        let out = line(vec![[2.0, 2.0], [6.0, 2.0]]);
        assert_eq!(covers(&sq, &out), Ok(false));
        // Crossing the hole of a holey polygon is not covered, even though
        // both endpoints are in the solid part.
        let a = holey(2.0, 2.0, 4.0);
        assert_eq!(covers(&a, &line(vec![[1.0, 4.0], [7.0, 4.0]])), Ok(false));
    }

    #[test]
    fn line_contains_line_and_point() {
        let long = line(vec![[0.0, 0.0], [8.0, 0.0]]);
        let sub = line(vec![[2.0, 0.0], [5.0, 0.0]]);
        assert_eq!(contains(&long, &sub), Ok(true));
        assert_eq!(contains(&sub, &long), Ok(false));
        // Sharing only a crossing point is not containment.
        let cross = line(vec![[4.0, -1.0], [4.0, 1.0]]);
        assert_eq!(contains(&long, &cross), Ok(false));
        // Chain interior point is contained; an endpoint is only covered.
        assert_eq!(contains(&long, &point(3.0, 0.0)), Ok(true));
        assert_eq!(contains(&long, &point(0.0, 0.0)), Ok(false));
        assert_eq!(covers(&long, &point(0.0, 0.0)), Ok(true));
    }

    #[test]
    fn polygon_contains_polygon() {
        let big = square(0.0, 0.0, 8.0);
        assert_eq!(contains(&big, &square(1.0, 1.0, 2.0)), Ok(true));
        // Touching the boundary from inside still contains (interiors meet).
        assert_eq!(contains(&big, &square(0.0, 0.0, 2.0)), Ok(true));
        assert_eq!(contains(&big, &square(7.0, 7.0, 2.0)), Ok(false));
        assert_eq!(contains(&big, &big.clone()), Ok(true));
        assert_eq!(contains(&square(1.0, 1.0, 2.0), &big), Ok(false));
    }

    #[test]
    fn hole_defeats_containment() {
        // b covers a's (off-center) hole entirely: the reverse-leak check must
        // reject it even though b's boundary and interior samples all pass.
        let a = holey(1.0, 1.0, 1.0);
        let b = square(0.0, 0.0, 8.0);
        assert_eq!(covers(&a, &b), Ok(false));
        // And a genuinely-inside b (avoiding the hole) is contained.
        assert_eq!(contains(&a, &square(4.0, 4.0, 3.0)), Ok(true));
        // b sitting exactly in the hole touches only closure: covered by the
        // outer square minus hole? No: the hole interior is outside a.
        assert_eq!(covers(&a, &square(1.0, 1.0, 1.0)), Ok(false));
    }

    #[test]
    fn hole_filler_mesh_is_not_covered() {
        // b = mesh { the hole-filling square, a small square in a's solid }.
        // Every ring of b lies in a's closure, but b1's interior is a's hole.
        let a = holey(2.0, 2.0, 4.0);
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                // b1: the hole filler [2,6]².
                [2.0, 2.0],
                [6.0, 2.0],
                [6.0, 6.0],
                [2.0, 6.0],
                // b2: solid part [0.5,1.5]².
                [0.5, 0.5],
                [1.5, 0.5],
                [1.5, 1.5],
                [0.5, 1.5],
            ],
            vec![vec![0u32, 1, 2, 3], vec![4, 5, 6, 7]],
        )
        .unwrap();
        let b = g2(Euclidean2DGeometry::PolygonMesh(Box::new(mesh)));
        assert_eq!(covers(&a, &b), Ok(false));
    }

    #[test]
    fn mesh_shared_edge_interior_supports_containment() {
        // a = two quads sharing the edge x = 2; a line along that edge lies in
        // the union's interior.
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
        let a = g2(Euclidean2DGeometry::PolygonMesh(Box::new(mesh)));
        let along = line(vec![[2.0, 0.5], [2.0, 1.5]]);
        assert_eq!(contains(&a, &along), Ok(true));
        // The outer rim is only covered.
        let rim = line(vec![[0.0, 0.5], [0.0, 1.5]]);
        assert_eq!(contains(&a, &rim), Ok(false));
        assert_eq!(covers(&a, &rim), Ok(true));
    }

    #[test]
    fn collection_union_contains_spanning_line() {
        // Two squares sharing the edge x = 2: their union contains a line
        // crossing from one into the other.
        let a = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            square_2d(0.0, 0.0, 2.0),
            square_2d(2.0, 0.0, 2.0),
        ])));
        let spanning = line(vec![[1.0, 1.0], [3.0, 1.0]]);
        assert_eq!(contains(&a, &spanning), Ok(true));
        // A line escaping both is not covered.
        assert_eq!(covers(&a, &line(vec![[1.0, 1.0], [5.0, 1.0]])), Ok(false));
    }

    #[test]
    fn collection_containee_needs_every_member_covered() {
        let a = square(0.0, 0.0, 8.0);
        let b_ok = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 1.0])),
            Euclidean2DGeometry::Point(Point2D::new(e(), [2.0, 2.0])),
        ])));
        let b_escapes = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 1.0])),
            Euclidean2DGeometry::Point(Point2D::new(e(), [9.0, 9.0])),
        ])));
        assert_eq!(contains(&a, &b_ok), Ok(true));
        assert_eq!(contains(&a, &b_escapes), Ok(false));
        // All members on the boundary: covered, not contained; one interior
        // member is witness enough.
        let b_boundary = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 1.0])),
            Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 2.0])),
        ])));
        assert_eq!(covers(&a, &b_boundary), Ok(true));
        assert_eq!(contains(&a, &b_boundary), Ok(false));
        let b_mixed = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 1.0])),
            Euclidean2DGeometry::Point(Point2D::new(e(), [3.0, 3.0])),
        ])));
        assert_eq!(contains(&a, &b_mixed), Ok(true));
    }

    #[test]
    fn nothing_contains_empty_or_none() {
        let sq = square(0.0, 0.0, 4.0);
        assert_eq!(contains(&sq, &Geometry::None), Ok(false));
        assert_eq!(covers(&sq, &Geometry::None), Ok(false));
        let empty = g2(Euclidean2DGeometry::Collection(Collection2D::new([])));
        assert_eq!(covers(&sq, &empty), Ok(false));
        assert_eq!(contains(&Geometry::None, &sq), Ok(false));
    }

    #[test]
    fn frame_and_dimension_errors() {
        let a = square(0.0, 0.0, 4.0);
        let b = g2(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [1.0, 1.0],
        )));
        assert_eq!(contains(&a, &b), Err(PredicateError::MixedFrames));

        let p3 = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Point(
            crate::point::Point3D::new(e(), [0.0, 0.0, 0.0]),
        ));
        assert_eq!(contains(&a, &p3), Err(PredicateError::CrossDimension));
        assert!(matches!(
            contains(&p3, &p3),
            Err(PredicateError::UnsupportedPair { .. })
        ));
    }

    #[test]
    fn face_interior_point_lands_inside() {
        // Sample a holey face: the point must avoid the hole.
        let outer = [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0], [0.0, 0.0]];
        let hole = vec![[2.0, 2.0], [2.0, 6.0], [6.0, 6.0], [6.0, 2.0], [2.0, 2.0]];
        let p = Polygon2D::from_rings(e(), outer, vec![hole]);
        let view = super::super::view::AreaView::from_polygon(&p);
        let sample = face_interior_point(view.face(0)).unwrap();
        assert_eq!(face_position(sample, view.face(0)), CoordPos::Inside);
    }
}

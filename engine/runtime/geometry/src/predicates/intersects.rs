//! The `intersects` predicate: whether two geometries share at least one point.
//!
//! Boundary contact counts: a shared vertex, a T-junction, or a segment lying
//! along a ring all intersect. Collections are point-set unions (any member
//! pair intersecting suffices). Every leaf pair goes through a bounding-box
//! quick reject first.
//!
//! 2D × 2D pairs dispatch here; 3D × 3D pairs dispatch to
//! [`intersects3d`](super::intersects3d). A 2D × 3D pair is a
//! [`CrossDimension`](PredicateError::CrossDimension) error (members of a
//! [`GeometryCollection`](crate::GeometryCollection) pair per member, so
//! same-dimension member pairs still evaluate). `Geometry::None` intersects
//! nothing.

use super::edge_set::{operand_edges, operand_segment_count, should_index, EdgeSet};
use super::kernel::segment_intersection;
use super::kernel::CoordPos;
use super::position::{face_position, line_position};
use super::view::{require_common_frame, AreaView, Leaf2D, Operand2D, PreparedLeaf};
use super::{PredicateError, Result};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Whether `a` and `b` share at least one point.
pub fn intersects(a: &Geometry, b: &Geometry) -> Result<bool> {
    match (a, b) {
        (Geometry::None, _) | (_, Geometry::None) => Ok(false),
        (Geometry::GeometryCollection(c), other) => {
            for member in c.members() {
                if intersects(member, other)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        (other, Geometry::GeometryCollection(c)) => {
            for member in c.members() {
                if intersects(other, member)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        (Geometry::Euclidean2D(a), Geometry::Euclidean2D(b)) => intersects_2d(a, b),
        (Geometry::Euclidean2D(_), Geometry::Euclidean3D(_))
        | (Geometry::Euclidean3D(_), Geometry::Euclidean2D(_)) => {
            Err(PredicateError::CrossDimension)
        }
        (Geometry::Euclidean3D(a), Geometry::Euclidean3D(b)) => {
            super::intersects3d::intersects_3d(a, b)
        }
    }
}

/// `intersects` over two 2D geometries.
pub fn intersects_2d(a: &Euclidean2DGeometry, b: &Euclidean2DGeometry) -> Result<bool> {
    let a = Operand2D::new(a);
    let b = Operand2D::new(b);
    require_common_frame(&a, &b)?;
    Ok(intersects_operands(&a, &b))
}

/// `intersects` over prepared operands, dispatching on input size: a direct
/// leaf-pair scan for small inputs, an rstar-indexed crossing sweep above
/// [`DIRECT_WORK_LIMIT`](super::edge_set::DIRECT_WORK_LIMIT) kernel calls.
fn intersects_operands(a: &Operand2D<'_>, b: &Operand2D<'_>) -> bool {
    let (na, nb) = (operand_segment_count(a), operand_segment_count(b));
    if should_index(na, nb) {
        intersects_indexed(a, b, na <= nb)
    } else {
        intersects_direct(a, b)
    }
}

/// The direct strategy: every leaf pair through [`leaf_intersects`].
fn intersects_direct(a: &Operand2D<'_>, b: &Operand2D<'_>) -> bool {
    for la in &a.leaves {
        for lb in &b.leaves {
            if leaf_intersects(la, lb) {
                return true;
            }
        }
    }
    false
}

/// The indexed strategy: one global early-exit crossing sweep of one operand's
/// segments against an rstar index over the other's, then the point and
/// containment cases no segment crossing can witness. `index_b` picks which
/// operand is indexed (the one with more segments).
fn intersects_indexed(a: &Operand2D<'_>, b: &Operand2D<'_>, index_b: bool) -> bool {
    let (probe, indexed) = if index_b { (a, b) } else { (b, a) };
    let set = EdgeSet::with_strategy(operand_edges(indexed), true);
    for prepared in &probe.leaves {
        let crossing = |u: [f64; 2], v: [f64; 2]| {
            set.probe(u, v, |s, t| segment_intersection(u, v, s, t).is_some())
        };
        let hit = match prepared.leaf {
            Leaf2D::Point(_) => false,
            Leaf2D::Line(l) => l.coords().windows(2).any(|s| crossing(s[0], s[1])),
            _ => prepared
                .area
                .as_ref()
                .expect("leaf is areal")
                .edges()
                .any(|(u, v)| crossing(u, v)),
        };
        if hit {
            return true;
        }
    }
    // No segment of one operand touches any segment of the other: only
    // point contact and strict containment remain, decided per leaf pair.
    for la in &a.leaves {
        for lb in &b.leaves {
            if residual_leaf_intersects(la, lb) {
                return true;
            }
        }
    }
    false
}

/// The concrete 3D leaf name, for `UnsupportedPair` diagnostics.
pub(crate) fn type_name_3d(g: &Euclidean3DGeometry) -> &'static str {
    match g {
        Euclidean3DGeometry::Point(_) => "Point3D",
        Euclidean3DGeometry::PointCloud(_) => "PointCloud",
        Euclidean3DGeometry::LineString(_) => "LineString3D",
        Euclidean3DGeometry::Polygon(_) => "Polygon3D",
        Euclidean3DGeometry::PolygonMesh(_) => "PolygonMesh3D",
        Euclidean3DGeometry::TriangularMesh(_) => "TriangularMesh3D",
        Euclidean3DGeometry::Solid(_) => "Solid",
        Euclidean3DGeometry::Csg(_) => "Csg",
        Euclidean3DGeometry::Collection(_) => "Collection3D",
    }
}

/// Whether two prepared leaves intersect, after a bounding-box quick reject.
/// An empty leaf (no bounding box) intersects nothing.
pub(crate) fn leaf_intersects(a: &PreparedLeaf<'_>, b: &PreparedLeaf<'_>) -> bool {
    let (Some(box_a), Some(box_b)) = (&a.bbox, &b.bbox) else {
        return false;
    };
    if !box_a.intersects(box_b) {
        return false;
    }
    match (&a.leaf, &b.leaf) {
        (Leaf2D::Point(pa), Leaf2D::Point(pb)) => pa.position() == pb.position(),
        (Leaf2D::Point(p), Leaf2D::Line(l)) | (Leaf2D::Line(l), Leaf2D::Point(p)) => {
            line_position(p.position(), l.coords()) != CoordPos::Outside
        }
        (Leaf2D::Point(p), _) => point_vs_area(p.position(), area(b)),
        (_, Leaf2D::Point(p)) => point_vs_area(p.position(), area(a)),
        (Leaf2D::Line(la), Leaf2D::Line(lb)) => line_vs_line(la.coords(), lb.coords()),
        (Leaf2D::Line(l), _) => line_vs_area(l.coords(), area(b)),
        (_, Leaf2D::Line(l)) => line_vs_area(l.coords(), area(a)),
        (_, _) => area_vs_area(area(a), area(b)),
    }
}

/// Whether two prepared leaves intersect, given that no segment of one crosses
/// any segment of the other (established by the indexed sweep): only point
/// contact and strict containment can still hold.
fn residual_leaf_intersects(a: &PreparedLeaf<'_>, b: &PreparedLeaf<'_>) -> bool {
    let (Some(box_a), Some(box_b)) = (&a.bbox, &b.bbox) else {
        return false;
    };
    if !box_a.intersects(box_b) {
        return false;
    }
    match (&a.leaf, &b.leaf) {
        (Leaf2D::Point(pa), Leaf2D::Point(pb)) => pa.position() == pb.position(),
        (Leaf2D::Point(p), Leaf2D::Line(l)) | (Leaf2D::Line(l), Leaf2D::Point(p)) => {
            line_position(p.position(), l.coords()) != CoordPos::Outside
        }
        (Leaf2D::Point(p), _) => point_vs_area(p.position(), area(b)),
        (_, Leaf2D::Point(p)) => point_vs_area(p.position(), area(a)),
        (Leaf2D::Line(la), Leaf2D::Line(lb)) => {
            // Chains with segments were fully swept; only a point-like
            // single-vertex chain can still touch.
            if la.coords().len() == 1 {
                line_position(la.coords()[0], lb.coords()) != CoordPos::Outside
            } else if lb.coords().len() == 1 {
                line_position(lb.coords()[0], la.coords()) != CoordPos::Outside
            } else {
                false
            }
        }
        (Leaf2D::Line(l), _) => {
            // No boundary contact: the chain lies in one region; one vertex
            // decides.
            l.coords()
                .first()
                .is_some_and(|&c| point_vs_area(c, area(b)))
        }
        (_, Leaf2D::Line(l)) => l
            .coords()
            .first()
            .is_some_and(|&c| point_vs_area(c, area(a))),
        (_, _) => areas_nested(area(a), area(b)),
    }
}

/// The areal view of a leaf known to be areal.
fn area<'a, 'b>(leaf: &'b PreparedLeaf<'a>) -> &'b AreaView<'a> {
    leaf.area.as_ref().expect("leaf is areal")
}

fn point_vs_area(coord: [f64; 2], area: &AreaView<'_>) -> bool {
    area.faces()
        .any(|f| face_position(coord, f) != CoordPos::Outside)
}

fn line_vs_line(a: &[[f64; 2]], b: &[[f64; 2]]) -> bool {
    // A single-vertex chain is point-like.
    if a.len() == 1 {
        return line_position(a[0], b) != CoordPos::Outside;
    }
    if b.len() == 1 {
        return line_position(b[0], a) != CoordPos::Outside;
    }
    a.windows(2).any(|sa| {
        b.windows(2)
            .any(|sb| segment_intersection(sa[0], sa[1], sb[0], sb[1]).is_some())
    })
}

fn line_vs_area(coords: &[[f64; 2]], area: &AreaView<'_>) -> bool {
    if coords.len() == 1 {
        return point_vs_area(coords[0], area);
    }
    // Any chain segment meeting the boundary intersects. Otherwise the chain
    // never touches the boundary, so it lies entirely in one region: one
    // vertex decides.
    if coords.windows(2).any(|s| {
        area.edges()
            .any(|(u, v)| segment_intersection(s[0], s[1], u, v).is_some())
    }) {
        return true;
    }
    coords.first().is_some_and(|&c| point_vs_area(c, area))
}

fn area_vs_area(a: &AreaView<'_>, b: &AreaView<'_>) -> bool {
    // Any boundary contact intersects.
    if a.edges().any(|(u, v)| {
        b.edges()
            .any(|(s, t)| segment_intersection(u, v, s, t).is_some())
    }) {
        return true;
    }
    areas_nested(a, b)
}

/// Whether one boundary-disjoint areal view lies inside the other. With no
/// boundary contact every ring lies entirely in one region of the other
/// operand, so one vertex per ring decides full containment either way (a
/// ring inside the other's hole classifies as outside).
fn areas_nested(a: &AreaView<'_>, b: &AreaView<'_>) -> bool {
    let ring_inside = |x: &AreaView<'_>, y: &AreaView<'_>| {
        x.faces().any(|f| {
            f.rings().any(|r| {
                !r.is_empty()
                    && y.faces()
                        .any(|g| face_position(r.coord(0), g) != CoordPos::Outside)
            })
        })
    };
    ring_inside(a, b) || ring_inside(b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection2D;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::line_string::{LineString2D, LineString3D};
    use crate::point::Point2D;
    use crate::polygon::Polygon2D;
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, GeometryCollection};
    use pretty_assertions::assert_eq;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn poly(ring: [[f64; 2]; 5]) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            ring,
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    fn unit_square_at(x: f64, y: f64, s: f64) -> Geometry {
        g2(poly([
            [x, y],
            [x + s, y],
            [x + s, y + s],
            [x, y + s],
            [x, y],
        ]))
    }

    fn g2(g: Euclidean2DGeometry) -> Geometry {
        Geometry::Euclidean2D(g)
    }

    #[test]
    fn point_pairs() {
        let p = |x, y| g2(Euclidean2DGeometry::Point(Point2D::new(e(), [x, y])));
        assert_eq!(intersects(&p(1.0, 1.0), &p(1.0, 1.0)), Ok(true));
        assert_eq!(intersects(&p(1.0, 1.0), &p(1.0, 2.0)), Ok(false));
    }

    #[test]
    fn line_line_crossing_touching_disjoint() {
        let l = |coords: Vec<[f64; 2]>| {
            g2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                e(),
                coords,
            )))
        };
        let cross_a = l(vec![[0.0, 0.0], [2.0, 2.0]]);
        let cross_b = l(vec![[0.0, 2.0], [2.0, 0.0]]);
        assert_eq!(intersects(&cross_a, &cross_b), Ok(true));
        // T-junction (improper) also intersects.
        let touch = l(vec![[1.0, 1.0], [1.0, 5.0]]);
        assert_eq!(intersects(&cross_a, &touch), Ok(true));
        let disjoint = l(vec![[5.0, 5.0], [6.0, 5.0]]);
        assert_eq!(intersects(&cross_a, &disjoint), Ok(false));
    }

    #[test]
    fn polygon_pairs_cover_touch_contain_hole() {
        let big = unit_square_at(0.0, 0.0, 8.0);
        let inside = unit_square_at(1.0, 1.0, 2.0);
        let overlapping = unit_square_at(6.0, 6.0, 4.0);
        let touching = unit_square_at(8.0, 0.0, 2.0);
        let disjoint = unit_square_at(20.0, 0.0, 1.0);
        assert_eq!(intersects(&big, &inside), Ok(true)); // containment, no crossing
        assert_eq!(intersects(&inside, &big), Ok(true));
        assert_eq!(intersects(&big, &overlapping), Ok(true));
        assert_eq!(intersects(&big, &touching), Ok(true)); // shared edge only
        assert_eq!(intersects(&big, &disjoint), Ok(false));

        // A polygon sitting entirely in the other's hole does not intersect.
        let outer = [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0], [0.0, 0.0]];
        let hole = vec![[2.0, 2.0], [2.0, 6.0], [6.0, 6.0], [6.0, 2.0], [2.0, 2.0]];
        let with_hole = g2(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(e(), outer, vec![hole]),
        )));
        let in_hole = unit_square_at(3.0, 3.0, 2.0);
        assert_eq!(intersects(&with_hole, &in_hole), Ok(false));
        // But one crossing the hole ring does.
        let across = unit_square_at(1.0, 3.0, 2.0);
        assert_eq!(intersects(&with_hole, &across), Ok(true));
    }

    #[test]
    fn line_polygon_inside_crossing_disjoint() {
        let square = unit_square_at(0.0, 0.0, 4.0);
        let l = |coords: Vec<[f64; 2]>| {
            g2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                e(),
                coords,
            )))
        };
        assert_eq!(
            intersects(&square, &l(vec![[1.0, 1.0], [2.0, 2.0]])),
            Ok(true)
        );
        assert_eq!(
            intersects(&square, &l(vec![[-1.0, 2.0], [5.0, 2.0]])),
            Ok(true)
        );
        assert_eq!(
            intersects(&square, &l(vec![[5.0, 5.0], [6.0, 6.0]])),
            Ok(false)
        );
    }

    #[test]
    fn collections_intersect_through_any_member() {
        let c = g2(Euclidean2DGeometry::Collection(Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(e(), [10.0, 10.0])),
            Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 1.0])),
        ])));
        let square = unit_square_at(0.0, 0.0, 2.0);
        assert_eq!(intersects(&c, &square), Ok(true));

        let gc = Geometry::GeometryCollection(GeometryCollection::new([square.clone()]));
        assert_eq!(intersects(&gc, &c), Ok(true));
    }

    #[test]
    fn none_intersects_nothing() {
        let square = unit_square_at(0.0, 0.0, 2.0);
        assert_eq!(intersects(&Geometry::None, &square), Ok(false));
        assert_eq!(intersects(&Geometry::None, &Geometry::None), Ok(false));
    }

    #[test]
    fn mixed_frames_error() {
        let a = g2(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [0.0, 0.0],
        )));
        let b = g2(Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0])));
        assert_eq!(intersects(&a, &b), Err(PredicateError::MixedFrames));
    }

    #[test]
    fn cross_dimension_errors_and_3d_dispatches() {
        let a = unit_square_at(0.0, 0.0, 1.0);
        let b3 = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        )));
        assert_eq!(intersects(&a, &b3), Err(PredicateError::CrossDimension));
        // 3D × 3D pairs evaluate through the 3D dispatch.
        assert_eq!(intersects(&b3, &b3), Ok(true));
    }

    #[test]
    fn bbox_reject_short_circuits() {
        // Far-apart geometries with expensive shapes still answer quickly and
        // correctly through the box reject.
        let a = unit_square_at(0.0, 0.0, 1.0);
        let b = unit_square_at(1000.0, 1000.0, 1.0);
        assert_eq!(intersects(&a, &b), Ok(false));
    }

    // --- direct / indexed strategy parity ------------------------------------

    /// Deterministic splitmix-style generator.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        }

        fn range(&mut self, n: u64) -> u64 {
            self.next() % n
        }

        fn coord(&mut self) -> [f64; 2] {
            [self.range(9) as f64, self.range(9) as f64]
        }
    }

    /// One random 2D geometry on a small integer grid, so touching, collinear,
    /// and degenerate configurations occur often.
    fn rng_geometry(rng: &mut Rng) -> Euclidean2DGeometry {
        let rect = |x0: f64, y0: f64, w: f64, h: f64| {
            [
                [x0, y0],
                [x0 + w, y0],
                [x0 + w, y0 + h],
                [x0, y0 + h],
                [x0, y0],
            ]
        };
        match rng.range(9) {
            0 => Euclidean2DGeometry::Point(Point2D::new(e(), rng.coord())),
            // Single-vertex chain: point-like, no segments.
            1 => Euclidean2DGeometry::LineString(LineString2D::from_coords(e(), [rng.coord()])),
            2 => Euclidean2DGeometry::LineString(LineString2D::from_coords(
                e(),
                [rng.coord(), rng.coord()],
            )),
            3 => Euclidean2DGeometry::LineString(LineString2D::from_coords(
                e(),
                [rng.coord(), rng.coord(), rng.coord(), rng.coord()],
            )),
            4 => {
                // Closed chain.
                let (a, b, c) = (rng.coord(), rng.coord(), rng.coord());
                Euclidean2DGeometry::LineString(LineString2D::from_coords(e(), [a, b, c, a]))
            }
            5 => {
                let (x0, y0) = (rng.range(6) as f64, rng.range(6) as f64);
                let (w, h) = (1 + rng.range(3), 1 + rng.range(3));
                poly(rect(x0, y0, w as f64, h as f64))
            }
            6 => {
                // Rect with a hole strictly inside.
                let (x0, y0) = (rng.range(4) as f64, rng.range(4) as f64);
                let (w, h) = (3 + rng.range(2), 3 + rng.range(2));
                let outer = rect(x0, y0, w as f64, h as f64);
                let hole = vec![
                    [x0 + 1.0, y0 + 1.0],
                    [x0 + 1.0, y0 + h as f64 - 1.0],
                    [x0 + w as f64 - 1.0, y0 + h as f64 - 1.0],
                    [x0 + w as f64 - 1.0, y0 + 1.0],
                    [x0 + 1.0, y0 + 1.0],
                ];
                Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                    e(),
                    outer,
                    vec![hole],
                )))
            }
            7 => {
                // Two quads sharing an edge, as a mesh.
                let (x0, y0) = (rng.range(5) as f64, rng.range(5) as f64);
                let mesh = crate::polygon_mesh::PolygonMesh2D::from_parts(
                    e(),
                    vec![
                        [x0, y0],
                        [x0 + 2.0, y0],
                        [x0 + 2.0, y0 + 2.0],
                        [x0, y0 + 2.0],
                        [x0 + 4.0, y0],
                        [x0 + 4.0, y0 + 2.0],
                    ],
                    vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
                )
                .unwrap();
                Euclidean2DGeometry::PolygonMesh(Box::new(mesh))
            }
            _ => {
                // Collection of two rects (possibly overlapping or touching).
                let member = |rng: &mut Rng| {
                    let (x0, y0) = (rng.range(6) as f64, rng.range(6) as f64);
                    poly(rect(
                        x0,
                        y0,
                        (1 + rng.range(3)) as f64,
                        (1 + rng.range(3)) as f64,
                    ))
                };
                Euclidean2DGeometry::Collection(Collection2D::new([member(rng), member(rng)]))
            }
        }
    }

    #[test]
    fn direct_and_indexed_strategies_agree() {
        let mut rng = Rng(20260716);
        for case in 0..400 {
            let ga = rng_geometry(&mut rng);
            let gb = rng_geometry(&mut rng);
            let a = Operand2D::new(&ga);
            let b = Operand2D::new(&gb);
            let direct = intersects_direct(&a, &b);
            assert_eq!(
                direct,
                intersects_indexed(&a, &b, true),
                "case {case}: indexed(b) diverges for {ga:?} / {gb:?}"
            );
            assert_eq!(
                direct,
                intersects_indexed(&a, &b, false),
                "case {case}: indexed(a) diverges for {ga:?} / {gb:?}"
            );
            assert_eq!(
                Ok(direct),
                intersects_2d(&ga, &gb),
                "case {case}: public path diverges for {ga:?} / {gb:?}"
            );
        }
    }

    /// A closed regular `n`-gon ring around `(cx, cy)`.
    fn ngon_ring(cx: f64, cy: f64, r: f64, n: usize) -> Vec<[f64; 2]> {
        (0..=n)
            .map(|i| {
                let t = core::f64::consts::TAU * (i % n) as f64 / n as f64;
                [cx + r * t.cos(), cy + r * t.sin()]
            })
            .collect()
    }

    #[test]
    fn auto_indexed_large_inputs_answer_correctly() {
        // 100-gon pairs: the segment-count product crosses the index gate.
        let ngon = |cx: f64, cy: f64, r: f64| {
            g2(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings(e(), ngon_ring(cx, cy, r, 100), Vec::<Vec<[f64; 2]>>::new()),
            )))
        };
        // Strict containment: no boundary crossing, decided by the residual.
        assert_eq!(
            intersects(&ngon(0.0, 0.0, 10.0), &ngon(0.0, 0.0, 5.0)),
            Ok(true)
        );
        assert_eq!(
            intersects(&ngon(0.0, 0.0, 5.0), &ngon(0.0, 0.0, 10.0)),
            Ok(true)
        );
        // Overlapping boundaries: found by the indexed sweep.
        assert_eq!(
            intersects(&ngon(0.0, 0.0, 10.0), &ngon(12.0, 0.0, 5.0)),
            Ok(true)
        );
        // Disjoint.
        assert_eq!(
            intersects(&ngon(0.0, 0.0, 10.0), &ngon(40.0, 0.0, 5.0)),
            Ok(false)
        );
    }
}

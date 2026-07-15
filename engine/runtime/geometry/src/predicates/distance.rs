//! Euclidean [`distance`] between two geometries, 2D or 3D.
//!
//! The minimum Euclidean distance between the operands' closed point sets:
//! `0` exactly when they [`intersects`](super::intersects()) (including a
//! geometry inside a `Solid` with no shell contact — the upfront check is the
//! exact phase-5 test), otherwise the minimum over primitive **element**
//! pairs:
//!
//! - 2D leaves decompose into points and segments (an areal leaf contributes
//!   its ring edges — for disjoint operands the nearest point of a region
//!   lies on its boundary, and a mesh's internal shared edges lie behind the
//!   rim, so including them never changes the minimum);
//! - 3D leaves decompose into points, segments, and triangles (surfaces and
//!   solid shells through the borrowed triangulation of
//!   [`view3d`](super::view3d); a solid's void shells are included, so a
//!   geometry in a hollow measures its distance to the hollow's wall).
//!
//! The sweep is rstar-pruned: each element queries the other operand's tree
//! within the best distance found so far, so far-apart element pairs are
//! never evaluated. Unlike the boolean predicates, distances are
//! *constructed* values — plain f64 arithmetic, not exact — but the
//! intersecting/disjoint decision itself is exact via the upfront check.
//!
//! `Ok(None)` when either operand is empty (no leaves, or only leaves that
//! re-represent as the empty set, e.g. a fully degenerate polygon). The
//! operands must be of one dimension each and match
//! ([`CrossDimension`](super::PredicateError::CrossDimension) otherwise);
//! `Csg` and `PointCloud` are
//! [`Unsupported`](super::PredicateError::Unsupported). The optional
//! elevation of 2D leaves is ignored, as everywhere in the predicates.

use rstar::{Point, RTree, RTreeObject, AABB};

use crate::ops::triangulation::Cache;
use crate::Geometry;

use super::contains::flatten_geometry;
use super::intersects::intersects;
use super::kernel::segment_intersection;
use super::view::Leaf2D;
use super::view3d::{flatten_geometry_3d, Leaf3D, TriangleSet};
use super::{PredicateError, Result};

/// The minimum Euclidean distance between two geometries' point sets, or
/// `None` when either is empty. See the [module docs](self).
pub fn distance(a: &Geometry, b: &Geometry) -> Result<Option<f64>> {
    if intersects(a, b)? {
        return Ok(Some(0.0));
    }

    let (a2, a3_name) = flatten_geometry(a);
    let (b2, b3_name) = flatten_geometry(b);
    // A truly empty operand has no distance to anything.
    if a2.is_empty() && a3_name.is_none() || b2.is_empty() && b3_name.is_none() {
        return Ok(None);
    }
    match (a3_name, b3_name) {
        (None, None) => Ok(distance_2d(&a2, &b2)),
        (Some(_), Some(_)) if a2.is_empty() && b2.is_empty() => {
            let a3 = leaves_3d(a)?;
            let b3 = leaves_3d(b)?;
            Ok(distance_3d(&a3, &b3))
        }
        _ => Err(PredicateError::CrossDimension),
    }
}

/// The 3D leaves of an operand (2D absence already established).
fn leaves_3d(geometry: &Geometry) -> Result<Vec<Leaf3D<'_>>> {
    let (leaves, _, unsupported) = flatten_geometry_3d(geometry);
    match unsupported {
        Some(name) => Err(PredicateError::Unsupported { geometry: name }),
        None => Ok(leaves),
    }
}

// --- 2D ------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Element2 {
    Point([f64; 2]),
    Segment([[f64; 2]; 2]),
}

struct Obj2 {
    elem: Element2,
    env: AABB<[f64; 2]>,
}

impl RTreeObject for Obj2 {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.env
    }
}

fn distance_2d(a: &[Leaf2D<'_>], b: &[Leaf2D<'_>]) -> Option<f64> {
    let a_objs = elements_2d(a);
    let b_objs = elements_2d(b);
    if a_objs.is_empty() || b_objs.is_empty() {
        return None;
    }
    Some(sweep(a_objs, b_objs, element_distance_sq_2d).sqrt())
}

fn elements_2d(leaves: &[Leaf2D<'_>]) -> Vec<Obj2> {
    let mut out = Vec::new();
    let push_point = |p: [f64; 2], out: &mut Vec<Obj2>| {
        out.push(Obj2 {
            elem: Element2::Point(p),
            env: AABB::from_point(p),
        })
    };
    let push_chain = |coords: &[[f64; 2]], out: &mut Vec<Obj2>| match coords {
        [] => {}
        [p] => push_point(*p, out),
        _ => out.extend(coords.windows(2).map(|w| Obj2 {
            elem: Element2::Segment([w[0], w[1]]),
            env: AABB::from_corners(w[0], w[1]),
        })),
    };
    for leaf in leaves {
        match leaf {
            Leaf2D::Point(p) => push_point(p.position(), &mut out),
            Leaf2D::Line(l) => push_chain(l.coords(), &mut out),
            _ => {
                let area = leaf.area_view().expect("areal leaf");
                out.extend(area.edges().map(|(u, v)| Obj2 {
                    elem: Element2::Segment([u, v]),
                    env: AABB::from_corners(u, v),
                }));
            }
        }
    }
    out
}

fn element_distance_sq_2d(a: &Obj2, b: &Obj2) -> f64 {
    use Element2::*;
    match (a.elem, b.elem) {
        (Point(p), Point(q)) => pp2(p, q),
        (Point(p), Segment(s)) | (Segment(s), Point(p)) => ps2(p, s),
        (Segment(s), Segment(t)) => ss2(s, t),
    }
}

fn pp2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

fn ps2(p: [f64; 2], [a, b]: [[f64; 2]; 2]) -> f64 {
    let (ex, ey) = (b[0] - a[0], b[1] - a[1]);
    let len_sq = ex * ex + ey * ey;
    if len_sq == 0.0 {
        return pp2(p, a);
    }
    let t = (((p[0] - a[0]) * ex + (p[1] - a[1]) * ey) / len_sq).clamp(0.0, 1.0);
    pp2(p, [a[0] + t * ex, a[1] + t * ey])
}

fn ss2(s: [[f64; 2]; 2], t: [[f64; 2]; 2]) -> f64 {
    if segment_intersection(s[0], s[1], t[0], t[1]).is_some() {
        return 0.0;
    }
    ps2(s[0], t)
        .min(ps2(s[1], t))
        .min(ps2(t[0], s))
        .min(ps2(t[1], s))
}

// --- 3D ------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Element3 {
    Point([f64; 3]),
    Segment([[f64; 3]; 2]),
    Triangle([[f64; 3]; 3]),
}

struct Obj3 {
    elem: Element3,
    env: AABB<[f64; 3]>,
}

impl RTreeObject for Obj3 {
    type Envelope = AABB<[f64; 3]>;

    fn envelope(&self) -> Self::Envelope {
        self.env
    }
}

fn distance_3d(a: &[Leaf3D<'_>], b: &[Leaf3D<'_>]) -> Option<f64> {
    let mut cache = Cache::new();
    let a_objs = elements_3d(a, &mut cache);
    let b_objs = elements_3d(b, &mut cache);
    if a_objs.is_empty() || b_objs.is_empty() {
        return None;
    }
    Some(sweep(a_objs, b_objs, element_distance_sq_3d).sqrt())
}

fn elements_3d(leaves: &[Leaf3D<'_>], cache: &mut Cache) -> Vec<Obj3> {
    let mut out = Vec::new();
    let push_triangles = |set: &TriangleSet<'_>, out: &mut Vec<Obj3>| {
        out.extend(set.triangles().map(|t| {
            let mut min = t[0];
            let mut max = t[0];
            for p in &t[1..] {
                for k in 0..3 {
                    min[k] = min[k].min(p[k]);
                    max[k] = max[k].max(p[k]);
                }
            }
            Obj3 {
                elem: Element3::Triangle(t),
                env: AABB::from_corners(min, max),
            }
        }))
    };
    for leaf in leaves {
        match leaf {
            Leaf3D::Point(p) => out.push(Obj3 {
                elem: Element3::Point(p.position()),
                env: AABB::from_point(p.position()),
            }),
            Leaf3D::Line(l) => match l.coords() {
                [] => {}
                [p] => out.push(Obj3 {
                    elem: Element3::Point(*p),
                    env: AABB::from_point(*p),
                }),
                coords => out.extend(coords.windows(2).map(|w| Obj3 {
                    elem: Element3::Segment([w[0], w[1]]),
                    env: AABB::from_corners(w[0], w[1]),
                })),
            },
            Leaf3D::Polygon(p) => push_triangles(&TriangleSet::from_polygon(p, cache), &mut out),
            Leaf3D::PolygonMesh(m) => push_triangles(
                &TriangleSet::from_polygon_mesh_data(m.data(), cache),
                &mut out,
            ),
            Leaf3D::TriangularMesh(m) => {
                push_triangles(&TriangleSet::from_triangular_data(m.data()), &mut out)
            }
            Leaf3D::Solid(s) => {
                for shell in core::iter::once(s.exterior()).chain(s.interiors().iter()) {
                    push_triangles(&TriangleSet::from_shell(shell, cache), &mut out);
                }
            }
        }
    }
    out
}

fn element_distance_sq_3d(a: &Obj3, b: &Obj3) -> f64 {
    use Element3::*;
    match (a.elem, b.elem) {
        (Point(p), Point(q)) => pp3(p, q),
        (Point(p), Segment(s)) | (Segment(s), Point(p)) => ps3(p, s),
        (Point(p), Triangle(t)) | (Triangle(t), Point(p)) => pt3(p, t),
        (Segment(s), Segment(t)) => ss3(s, t),
        (Segment(s), Triangle(t)) | (Triangle(t), Segment(s)) => st3(s, t),
        (Triangle(t), Triangle(u)) => tt3(t, u),
    }
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn at(a: [f64; 3], d: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + t * d[0], a[1] + t * d[1], a[2] + t * d[2]]
}

fn pp3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

fn ps3(p: [f64; 3], [a, b]: [[f64; 3]; 2]) -> f64 {
    let e = sub(b, a);
    let len_sq = dot(e, e);
    if len_sq == 0.0 {
        return pp3(p, a);
    }
    let t = (dot(sub(p, a), e) / len_sq).clamp(0.0, 1.0);
    pp3(p, at(a, e, t))
}

/// Closest distance of two segments: the standard clamped closest-point
/// parameters.
fn ss3([p1, p2]: [[f64; 3]; 2], [q1, q2]: [[f64; 3]; 2]) -> f64 {
    let d1 = sub(p2, p1);
    let d2 = sub(q2, q1);
    let r = sub(p1, q1);
    let a = dot(d1, d1);
    let e = dot(d2, d2);
    let f = dot(d2, r);
    if a == 0.0 {
        return ps3(p1, [q1, q2]);
    }
    if e == 0.0 {
        return ps3(q1, [p1, p2]);
    }
    let c = dot(d1, r);
    let b = dot(d1, d2);
    let denom = a * e - b * b;
    // Parallel segments pick one endpoint's parameter; the clamps below make
    // the pair consistent.
    let mut s = if denom != 0.0 {
        ((b * f - c * e) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = (b * s + f) / e;
    if t < 0.0 {
        t = 0.0;
        s = (-c / a).clamp(0.0, 1.0);
    } else if t > 1.0 {
        t = 1.0;
        s = ((b - c) / a).clamp(0.0, 1.0);
    }
    pp3(at(p1, d1, s), at(q1, d2, t))
}

/// Point × triangle: the plane distance when the foot of the perpendicular
/// lands inside, else the nearest edge.
fn pt3(p: [f64; 3], t: [[f64; 3]; 3]) -> f64 {
    let edges = [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]];
    let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
    let n_sq = dot(n, n);
    let edge_min = |p: [f64; 3]| {
        edges
            .iter()
            .map(|&e| ps3(p, e))
            .fold(f64::INFINITY, f64::min)
    };
    if n_sq == 0.0 {
        // Degenerate triangle: its edges cover its point set.
        return edge_min(p);
    }
    let offset = dot(sub(p, t[0]), n);
    let foot = at(p, n, -offset / n_sq);
    // Foot-inside test via consistent signs of the edge cross products
    // (plain f64: a misclassification near an edge is harmless — the edge
    // distance converges to the plane distance there).
    let inside = edges
        .iter()
        .all(|&[a, b]| dot(cross(sub(b, a), sub(foot, a)), n) >= 0.0)
        || edges
            .iter()
            .all(|&[a, b]| dot(cross(sub(b, a), sub(foot, a)), n) <= 0.0);
    if inside {
        offset * offset / n_sq
    } else {
        edge_min(p)
    }
}

/// Segment × triangle: both endpoints against the face plus the segment
/// against each edge (complete for disjoint operands: an interior–interior
/// closest pair forces the segment parallel to the plane, where an endpoint
/// projects inside or the segment passes over an edge).
fn st3(s: [[f64; 3]; 2], t: [[f64; 3]; 3]) -> f64 {
    let edges = [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]];
    let mut best = pt3(s[0], t).min(pt3(s[1], t));
    for e in edges {
        best = best.min(ss3(s, e));
    }
    best
}

/// Triangle × triangle: each edge against the other triangle (complete for
/// disjoint triangles — the closest pair is edge × edge or vertex × face).
fn tt3(t: [[f64; 3]; 3], u: [[f64; 3]; 3]) -> f64 {
    let mut best = f64::INFINITY;
    for e in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
        best = best.min(st3(e, u));
    }
    for e in [[u[0], u[1]], [u[1], u[2]], [u[2], u[0]]] {
        best = best.min(st3(e, t));
    }
    best
}

// --- the pruned sweep ----------------------------------------------------------

/// The minimum squared element-pair distance: each left element queries the
/// right tree within the best distance found so far.
fn sweep<O: RTreeObject<Envelope = AABB<P>>, P: rstar::Point<Scalar = f64>>(
    a: Vec<O>,
    b: Vec<O>,
    dist_sq: impl Fn(&O, &O) -> f64,
) -> f64 {
    let tree = RTree::bulk_load(b);
    let mut best = f64::INFINITY;
    for ea in &a {
        if best == 0.0 {
            break;
        }
        if best.is_finite() {
            let radius = best.sqrt();
            let query = inflate(&ea.envelope(), radius);
            for eb in tree.locate_in_envelope_intersecting(&query) {
                best = best.min(dist_sq(ea, eb));
            }
        } else {
            for eb in tree.iter() {
                best = best.min(dist_sq(ea, eb));
            }
        }
    }
    best
}

/// The AABB grown by `radius` on every axis.
fn inflate<P: Point<Scalar = f64>>(env: &AABB<P>, radius: f64) -> AABB<P> {
    let lower = P::generate(|i| env.lower().nth(i) - radius);
    let upper = P::generate(|i| env.upper().nth(i) + radius);
    AABB::from_corners(lower, upper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::line_string::{LineString2D, LineString3D};
    use crate::point::{Point2D, Point3D};
    use crate::polygon::Polygon2D;
    use crate::predicates::test3d::{
        box_solid, box_solid_with_void, e, g3, solid_geometry, tetra_solid, Rng,
    };
    use crate::triangular_mesh::TriangularMesh3D;
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry};
    use pretty_assertions::assert_eq;

    fn g2(g: Euclidean2DGeometry) -> Geometry {
        Geometry::Euclidean2D(g)
    }

    fn p3(p: [f64; 3]) -> Geometry {
        g3(Euclidean3DGeometry::Point(Point3D::new(e(), p)))
    }

    fn tri3(t: [[f64; 3]; 3]) -> Geometry {
        g3(Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(e(), t.to_vec(), [0u32, 1, 2]).unwrap(),
        )))
    }

    const TRI: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]];

    #[test]
    fn point_to_triangle_regions() {
        // Above the interior: the plane distance.
        assert_eq!(distance(&p3([1.0, 1.0, 3.0]), &tri3(TRI)), Ok(Some(3.0)));
        // Beyond an edge: distance to the edge.
        assert_eq!(distance(&p3([2.0, -3.0, 4.0]), &tri3(TRI)), Ok(Some(5.0)));
        // Beyond a vertex: distance to the vertex.
        assert_eq!(distance(&p3([-2.0, -1.0, 2.0]), &tri3(TRI)), Ok(Some(3.0)));
        // Touching counts as zero.
        assert_eq!(distance(&p3([1.0, 1.0, 0.0]), &tri3(TRI)), Ok(Some(0.0)));
    }

    #[test]
    fn segment_parallel_over_a_triangle_interior() {
        // The segment's interior hovers over the face; neither endpoint
        // projects inside, so the edge-crossing terms must capture it.
        let seg = g3(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            e(),
            [[-2.0, 1.0, 5.0], [6.0, 1.0, 5.0]],
        )));
        assert_eq!(distance(&seg, &tri3(TRI)), Ok(Some(5.0)));
    }

    #[test]
    fn solids_measure_from_their_shells() {
        let cube = solid_geometry(box_solid([0.0; 3], [2.0; 3]));
        // Face-to-face gap along one axis.
        let other = solid_geometry(box_solid([5.0, 0.0, 0.0], [2.0; 3]));
        assert_eq!(distance(&g3(cube.clone()), &g3(other)), Ok(Some(3.0)));
        // Corner-to-corner gap across all three axes.
        let corner = solid_geometry(box_solid([3.0, 3.0, 3.0], [2.0; 3]));
        assert_eq!(
            distance(&g3(cube.clone()), &g3(corner)),
            Ok(Some(3.0f64.sqrt()))
        );
        // Containment is intersection: distance zero.
        let inner = solid_geometry(box_solid([0.5; 3], [1.0; 3]));
        assert_eq!(distance(&g3(cube), &g3(inner)), Ok(Some(0.0)));

        // A point in a hollow measures its distance to the void's wall.
        let hollow = solid_geometry(box_solid_with_void([0.0; 3], [6.0; 3], [2.0; 3], [2.0; 3]));
        assert_eq!(distance(&p3([3.0, 3.0, 3.0]), &g3(hollow)), Ok(Some(1.0)));
    }

    #[test]
    fn two_d_pairs_and_hole_distances() {
        let square =
            |x: f64, y: f64, s: f64| vec![[x, y], [x + s, y], [x + s, y + s], [x, y + s], [x, y]];
        let with_hole = g2(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(e(), square(0.0, 0.0, 8.0), vec![square(2.0, 2.0, 4.0)]),
        )));
        // A point in the hole measures to the hole ring.
        let p = g2(Euclidean2DGeometry::Point(Point2D::new(e(), [4.0, 5.0])));
        assert_eq!(distance(&p, &with_hole), Ok(Some(1.0)));
        // Inside the material: zero.
        let q = g2(Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 1.0])));
        assert_eq!(distance(&q, &with_hole), Ok(Some(0.0)));
        // Disjoint lines.
        let l1 = g2(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            [[0.0, 10.0], [4.0, 10.0]],
        )));
        assert_eq!(distance(&l1, &with_hole), Ok(Some(2.0)));
    }

    #[test]
    fn empty_and_error_operands() {
        let cube = g3(solid_geometry(box_solid([0.0; 3], [1.0; 3])));
        assert_eq!(distance(&Geometry::None, &cube), Ok(None));
        let empty = g3(Euclidean3DGeometry::Collection(Collection3D::new([])));
        assert_eq!(distance(&empty, &cube), Ok(None));
        let flat = g2(Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0])));
        assert_eq!(distance(&flat, &cube), Err(PredicateError::CrossDimension));
        use crate::csg::{Csg, ThreeDimensional};
        let solid = || Box::new(box_solid([0.0; 3], [1.0; 3]));
        let csg = g3(Euclidean3DGeometry::Csg(Csg::Union(
            Box::new(ThreeDimensional::Solid(solid())),
            Box::new(ThreeDimensional::Solid(solid())),
        )));
        assert_eq!(
            distance(&csg, &cube),
            Err(PredicateError::Unsupported { geometry: "Csg" })
        );
    }

    #[test]
    fn axis_aligned_boxes_match_the_gap_oracle() {
        let mut rng = Rng(20260719);
        for case in 0..120 {
            let random_box = |rng: &mut Rng| {
                let min = [
                    rng.int(-6, 5) as f64,
                    rng.int(-6, 5) as f64,
                    rng.int(-6, 5) as f64,
                ];
                let size = [
                    rng.int(1, 3) as f64,
                    rng.int(1, 3) as f64,
                    rng.int(1, 3) as f64,
                ];
                (min, size)
            };
            let (a_min, a_size) = random_box(&mut rng);
            let (b_min, b_size) = random_box(&mut rng);
            let gap_sq: f64 = (0..3)
                .map(|k| {
                    let gap = (b_min[k] - (a_min[k] + a_size[k]))
                        .max(a_min[k] - (b_min[k] + b_size[k]))
                        .max(0.0);
                    gap * gap
                })
                .sum();
            let a = g3(solid_geometry(box_solid(a_min, a_size)));
            let b = g3(solid_geometry(box_solid(b_min, b_size)));
            let d = distance(&a, &b).unwrap().unwrap();
            assert!(
                (d - gap_sq.sqrt()).abs() < 1e-12,
                "case {case}: got {d}, expected {}",
                gap_sq.sqrt()
            );
            // Zero exactly when the exact intersects test says so.
            assert_eq!(d == 0.0, crate::predicates::intersects(&a, &b).unwrap());
        }
    }

    #[test]
    fn pruned_sweep_matches_brute_force() {
        let mut rng = Rng(20260720);
        for case in 0..60 {
            // Two random tetra solids plus a random polyline each: mixed
            // element kinds, many elements.
            let tetra = |rng: &mut Rng, offset: f64| loop {
                let v = [
                    rng.grid_point(-4, 4),
                    rng.grid_point(-4, 4),
                    rng.grid_point(-4, 4),
                    rng.grid_point(-4, 4),
                ];
                let v: [[f64; 3]; 4] = v.map(|p| [p[0] + offset, p[1], p[2]]);
                use crate::predicates::kernel::{orient3d, Orientation};
                if orient3d(v[0], v[1], v[2], v[3]) != Orientation::Collinear {
                    return v;
                }
            };
            let chain = |rng: &mut Rng, offset: f64| {
                let coords: Vec<[f64; 3]> = (0..4)
                    .map(|_| {
                        let p = rng.grid_point(-4, 4);
                        [p[0] + offset, p[1], p[2]]
                    })
                    .collect();
                Euclidean3DGeometry::LineString(LineString3D::from_coords(e(), coords))
            };
            let a = g3(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::Solid(Box::new(tetra_solid(tetra(&mut rng, 0.0)))),
                chain(&mut rng, 0.0),
            ])));
            let b = g3(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::Solid(Box::new(tetra_solid(tetra(&mut rng, 12.0)))),
                chain(&mut rng, 12.0),
            ])));

            let expected = brute_force(&a, &b);
            let actual = distance(&a, &b).unwrap().unwrap();
            assert_eq!(actual, expected, "case {case}");
        }
    }

    /// The un-pruned all-pairs minimum over the same elements.
    fn brute_force(a: &Geometry, b: &Geometry) -> f64 {
        let (a3, _, _) = flatten_geometry_3d(a);
        let (b3, _, _) = flatten_geometry_3d(b);
        let mut cache = Cache::new();
        let a_objs = elements_3d(&a3, &mut cache);
        let b_objs = elements_3d(&b3, &mut cache);
        let mut best = f64::INFINITY;
        for ea in &a_objs {
            for eb in &b_objs {
                best = best.min(element_distance_sq_3d(ea, eb));
            }
        }
        best.sqrt()
    }
}

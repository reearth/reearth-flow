//! Relate test suites: canonical DE-9IM matrices, mesh union semantics, and
//! consistency with the fast-path predicates.

use pretty_assertions::assert_eq;

use crate::collection::Collection2D;
use crate::coordinate::CoordinateFrame;
use crate::line_string::LineString2D;
use crate::point::Point2D;
use crate::polygon::Polygon2D;
use crate::polygon_mesh::PolygonMesh2D;
use crate::predicates::{contains, covers, intersects, relate};
use crate::triangular_mesh::TriangularMesh2D;
use crate::{Euclidean2DGeometry, Geometry};

fn e() -> CoordinateFrame {
    CoordinateFrame::Euclidean
}

/// The 9-char DE-9IM string of a matrix.
fn m(im: &super::IntersectionMatrix) -> String {
    let debug = format!("{im:?}");
    debug["IntersectionMatrix(".len()..debug.len() - 1].to_string()
}

// --- builders over shared coordinate lists -----------------------------------

fn point(p: [f64; 2]) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(e(), p)))
}

fn line(coords: &[[f64; 2]]) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
        e(),
        coords.to_vec(),
    )))
}

fn polygon(exterior: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
        Polygon2D::from_rings(e(), exterior.to_vec(), holes.to_vec()),
    )))
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
}

/// A clockwise rect ring (hole orientation).
fn rect_cw(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0], [x0, y0]]
}

// --- canonical matrices -------------------------------------------------------

#[test]
fn canonical_polygon_pairs() {
    let a = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);

    let equal = relate(&a, &polygon(&rect(0.0, 0.0, 4.0, 4.0), &[])).unwrap();
    assert_eq!(m(&equal), "2FFF1FFF2");
    assert!(equal.is_equal_topo());

    // The 2D entry point gives the same matrix.
    let b = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);
    let (Geometry::Euclidean2D(a_2d), Geometry::Euclidean2D(b_2d)) = (&a, &b) else {
        unreachable!("builders produce 2D geometry")
    };
    assert_eq!(m(&super::relate_2d(a_2d, b_2d).unwrap()), "2FFF1FFF2");

    let overlapping = relate(&a, &polygon(&rect(2.0, 2.0, 6.0, 6.0), &[])).unwrap();
    assert_eq!(m(&overlapping), "212101212");
    assert!(overlapping.is_overlaps());

    let edge_touching = relate(&a, &polygon(&rect(4.0, 0.0, 8.0, 4.0), &[])).unwrap();
    assert_eq!(m(&edge_touching), "FF2F11212");
    assert!(edge_touching.is_touches() && !edge_touching.is_overlaps());

    let corner_touching = relate(&a, &polygon(&rect(4.0, 4.0, 8.0, 8.0), &[])).unwrap();
    assert_eq!(m(&corner_touching), "FF2F01212");
    assert!(corner_touching.is_touches());

    let containing = relate(&a, &polygon(&rect(1.0, 1.0, 3.0, 3.0), &[])).unwrap();
    assert_eq!(m(&containing), "212FF1FF2");
    assert!(containing.is_contains() && containing.is_covers());

    let disjoint = relate(&a, &polygon(&rect(9.0, 9.0, 12.0, 12.0), &[])).unwrap();
    assert_eq!(m(&disjoint), "FF2FF1212");
    assert!(disjoint.is_disjoint());

    // Contained but sharing part of the boundary: covered, not contained-by
    // the strict interior on all sides — still contains under OGC (interiors
    // intersect, nothing outside).
    let flush = relate(&a, &polygon(&rect(0.0, 0.0, 2.0, 2.0), &[])).unwrap();
    assert_eq!(m(&flush), "212F11FF2");
    assert!(flush.is_contains() && flush.is_covers());

    // A polygon inside the other's hole is disjoint.
    let holey = polygon(&rect(0.0, 0.0, 8.0, 8.0), &[rect_cw(2.0, 2.0, 6.0, 6.0)]);
    let in_hole = relate(&holey, &polygon(&rect(3.0, 3.0, 5.0, 5.0), &[])).unwrap();
    assert_eq!(m(&in_hole), "FF2FF1212");
}

#[test]
fn canonical_line_pairs() {
    let horizontal = line(&[[0.0, 0.0], [4.0, 0.0]]);

    let crossing = relate(&horizontal, &line(&[[2.0, -2.0], [2.0, 2.0]])).unwrap();
    assert_eq!(m(&crossing), "0F1FF0102");
    assert!(crossing.is_crosses());

    let overlapping = relate(&horizontal, &line(&[[2.0, 0.0], [6.0, 0.0]])).unwrap();
    assert_eq!(m(&overlapping), "1010F0102");
    assert!(overlapping.is_overlaps());

    let endpoint_touch = relate(&horizontal, &line(&[[4.0, 0.0], [6.0, 0.0]])).unwrap();
    assert!(endpoint_touch.is_touches());

    let equal = relate(&horizontal, &line(&[[0.0, 0.0], [4.0, 0.0]])).unwrap();
    assert!(equal.is_equal_topo());
}

#[test]
fn canonical_mixed_pairs() {
    let square = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);

    let inside_point = relate(&point([2.0, 2.0]), &square).unwrap();
    assert_eq!(m(&inside_point), "0FFFFF212");
    assert!(inside_point.is_within());

    let boundary_point = relate(&point([0.0, 2.0]), &square).unwrap();
    assert_eq!(m(&boundary_point), "F0FFFF212");
    assert!(boundary_point.is_coveredby() && !boundary_point.is_within());
    assert!(boundary_point.is_touches());

    let crossing_line = relate(&line(&[[-2.0, 2.0], [6.0, 2.0]]), &square).unwrap();
    assert_eq!(m(&crossing_line), "101FF0212");
    assert!(crossing_line.is_crosses());

    let interior_line = relate(&line(&[[1.0, 1.0], [3.0, 3.0]]), &square).unwrap();
    assert_eq!(m(&interior_line), "1FF0FF212");
    assert!(interior_line.is_within());

    // A line running along the square's boundary is covered but not within.
    let boundary_line = relate(&line(&[[1.0, 0.0], [3.0, 0.0]]), &square).unwrap();
    assert_eq!(m(&boundary_line), "F1FF0F212");
    assert!(boundary_line.is_coveredby() && !boundary_line.is_within());
}

#[test]
fn empty_and_error_operands() {
    let square = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);

    let none = relate(&Geometry::None, &square).unwrap();
    assert_eq!(m(&none), "FFFFFF212");
    assert!(none.is_disjoint() && !none.is_intersects());

    let both_empty = relate(&Geometry::None, &Geometry::None).unwrap();
    assert_eq!(m(&both_empty), "FFFFFFFF2");

    use crate::coordinate::EpsgCode;
    let other_frame = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
        CoordinateFrame::Crs(EpsgCode::new(4326)),
        [0.0, 0.0],
    )));
    assert_eq!(
        relate(&square, &other_frame),
        Err(crate::predicates::PredicateError::MixedFrames)
    );

    let solid_3d = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Point(
        crate::point::Point3D::new(e(), [0.0, 0.0, 0.0]),
    ));
    assert_eq!(
        relate(&square, &solid_3d),
        Err(crate::predicates::PredicateError::CrossDimension)
    );
    assert!(matches!(
        relate(&solid_3d, &solid_3d),
        Err(crate::predicates::PredicateError::UnsupportedPair { .. })
    ));
}

// --- mesh union semantics ------------------------------------------------------

/// Two quads sharing the edge x = 2, dissolving to the rect (0,0)-(4,2).
fn two_quad_mesh() -> Geometry {
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
    Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(mesh)))
}

#[test]
fn mesh_relates_as_its_dissolved_union() {
    let mesh = two_quad_mesh();
    let dissolved = polygon(&rect(0.0, 0.0, 4.0, 2.0), &[]);

    let others: Vec<Geometry> = vec![
        // Crossing the internal shared edge: interior contact, not boundary.
        polygon(&rect(1.0, 0.5, 3.0, 1.5), &[]),
        // Equal to the union.
        polygon(&rect(0.0, 0.0, 4.0, 2.0), &[]),
        // Touching the union's outer boundary.
        polygon(&rect(4.0, 0.0, 6.0, 2.0), &[]),
        // A line along the internal shared edge is interior to the union.
        line(&[[2.0, 0.5], [2.0, 1.5]]),
        // A point on the internal shared edge.
        point([2.0, 1.0]),
        // Overlapping one face only.
        polygon(&rect(-1.0, 0.5, 1.0, 1.5), &[]),
        // Disjoint.
        point([9.0, 9.0]),
    ];

    for other in &others {
        let ours = relate(&mesh, other).unwrap();
        let dissolved_union = relate(&dissolved, other).unwrap();
        assert_eq!(
            m(&ours),
            m(&dissolved_union),
            "mesh relate diverges from dissolved-union relate for {other:?}"
        );
        // And in the flipped argument order.
        let ours = relate(other, &mesh).unwrap();
        let dissolved_union = relate(other, &dissolved).unwrap();
        assert_eq!(m(&ours), m(&dissolved_union), "flipped order for {other:?}");
    }
}

#[test]
fn mesh_specific_matrices() {
    let mesh = two_quad_mesh();

    // A line along the internal shared edge lies in the union's interior.
    let internal_line = relate(&mesh, &line(&[[2.0, 0.5], [2.0, 1.5]])).unwrap();
    assert_eq!(m(&internal_line), "102FF1FF2");
    assert!(internal_line.is_contains());

    // The union equals the dissolved rectangle.
    let union_rect = relate(&mesh, &polygon(&rect(0.0, 0.0, 4.0, 2.0), &[])).unwrap();
    assert!(union_rect.is_equal_topo());

    // Triangulated square against the square it triangulates.
    let tri_mesh = TriangularMesh2D::from_parts(
        e(),
        vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        [0u32, 1, 2, 0, 2, 3],
    )
    .unwrap();
    let tri_mesh = Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(Box::new(tri_mesh)));
    let im = relate(&tri_mesh, &polygon(&rect(0.0, 0.0, 2.0, 2.0), &[])).unwrap();
    assert!(im.is_equal_topo());

    // An annulus mesh (3x3 grid minus center) vs geometry in its hole.
    let mut vertices = Vec::new();
    for y in 0..4 {
        for x in 0..4 {
            vertices.push([x as f64, y as f64]);
        }
    }
    let index = |x: u32, y: u32| y * 4 + x;
    let mut faces = Vec::new();
    for y in 0..3u32 {
        for x in 0..3u32 {
            if x == 1 && y == 1 {
                continue;
            }
            faces.push(vec![
                index(x, y),
                index(x + 1, y),
                index(x + 1, y + 1),
                index(x, y + 1),
            ]);
        }
    }
    let annulus = PolygonMesh2D::from_parts(e(), vertices, faces).unwrap();
    let annulus = Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(annulus)));

    let in_hole = relate(&annulus, &point([1.5, 1.5])).unwrap();
    assert!(in_hole.is_disjoint());
    // ... and equals the polygon-with-hole it dissolves to.
    let holey = polygon(&rect(0.0, 0.0, 3.0, 3.0), &[rect_cw(1.0, 1.0, 2.0, 2.0)]);
    assert!(relate(&annulus, &holey).unwrap().is_equal_topo());
}

// --- consistency sweeps -------------------------------------------------------

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

/// One random geometry.
fn random_geometry(rng: &mut Rng) -> Geometry {
    match rng.range(7) {
        0 => {
            let p = rng.coord();
            point(p)
        }
        1 => {
            let coords = [rng.coord(), rng.coord()];
            line(&coords)
        }
        2 => {
            let coords = [rng.coord(), rng.coord(), rng.coord(), rng.coord()];
            line(&coords)
        }
        3 => {
            // Closed chain.
            let (a, b, c) = (rng.coord(), rng.coord(), rng.coord());
            let coords = [a, b, c, a];
            line(&coords)
        }
        4 => {
            let (x0, y0) = (rng.range(6) as f64, rng.range(6) as f64);
            let (w, h) = (1 + rng.range(3), 1 + rng.range(3));
            let ring = rect(x0, y0, x0 + w as f64, y0 + h as f64);
            polygon(&ring, &[])
        }
        5 => {
            // Rect with a hole strictly inside.
            let (x0, y0) = (rng.range(4) as f64, rng.range(4) as f64);
            let (x1, y1) = (
                x0 + 3.0 + rng.range(2) as f64,
                y0 + 3.0 + rng.range(2) as f64,
            );
            let hole = rect_cw(x0 + 1.0, y0 + 1.0, x1 - 1.0, y1 - 1.0);
            let ring = rect(x0, y0, x1, y1);
            polygon(&ring, std::slice::from_ref(&hole))
        }
        _ => {
            // Triangle, arbitrary winding; regenerate until non-collinear.
            loop {
                let (a, b, c) = (rng.coord(), rng.coord(), rng.coord());
                let doubled_area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
                if doubled_area != 0.0 {
                    let ring = [a, b, c, a];
                    return polygon(&ring, &[]);
                }
            }
        }
    }
}

#[test]
fn matrix_agrees_with_fast_path_predicates() {
    let mut rng = Rng(42);
    for case in 0..300 {
        let a = random_geometry(&mut rng);
        let b = random_geometry(&mut rng);
        let im = relate(&a, &b).unwrap();
        assert_eq!(
            im.is_intersects(),
            intersects(&a, &b).unwrap(),
            "case {case}: is_intersects vs intersects() for {a:?} / {b:?}"
        );
        assert_eq!(
            im.is_contains(),
            contains(&a, &b).unwrap(),
            "case {case}: is_contains vs contains() for {a:?} / {b:?}"
        );
        assert_eq!(
            im.is_covers(),
            covers(&a, &b).unwrap(),
            "case {case}: is_covers vs covers() for {a:?} / {b:?}"
        );
    }
}

#[test]
fn mesh_matrix_agrees_with_fast_path_predicates() {
    let mut rng = Rng(7);
    for case in 0..150 {
        let (x0, y0) = (rng.range(4) as f64, rng.range(4) as f64);
        let mesh = PolygonMesh2D::from_parts(
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
        let a = Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(mesh)));
        let b = random_geometry(&mut rng);
        for (left, right) in [(&a, &b), (&b, &a)] {
            let im = relate(left, right).unwrap();
            assert_eq!(
                im.is_intersects(),
                intersects(left, right).unwrap(),
                "case {case}: is_intersects for {left:?} / {right:?}"
            );
            assert_eq!(
                im.is_contains(),
                contains(left, right).unwrap(),
                "case {case}: is_contains for {left:?} / {right:?}"
            );
            assert_eq!(
                im.is_covers(),
                covers(left, right).unwrap(),
                "case {case}: is_covers for {left:?} / {right:?}"
            );
        }
    }
}

fn square_collection(rects: &[[f64; 4]]) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
        rects.iter().map(|&[x0, y0, x1, y1]| {
            Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                e(),
                rect(x0, y0, x1, y1),
                Vec::<Vec<[f64; 2]>>::new(),
            )))
        }),
    )))
}

#[test]
fn collections_of_disjoint_members_relate_as_unions() {
    // A collection flattens into one operand and relates as the union of its
    // members.
    let two_squares = square_collection(&[[0.0, 0.0, 2.0, 2.0], [6.0, 0.0, 8.0, 2.0]]);

    let in_first = relate(&two_squares, &line(&[[0.5, 1.0], [1.5, 1.0]])).unwrap();
    assert!(in_first.is_contains());

    let in_second = relate(&two_squares, &point([7.0, 1.0])).unwrap();
    assert!(in_second.is_covers());

    let across_the_gap = relate(&two_squares, &line(&[[1.0, 1.0], [7.0, 1.0]])).unwrap();
    assert!(across_the_gap.is_intersects() && !across_the_gap.is_contains());
}

#[test]
fn collection_with_edge_adjacent_members_is_jts_limited() {
    // Two squares sharing the edge x = 2. As a *mesh* this dissolves and the
    // shared edge becomes interior; as a *collection of polygons* it is
    // invalid MultiPolygon topology, which the JTS graph does not support:
    // the shared edge stays labeled boundary, so a line crossing it is not
    // `is_contains` even though it lies in the point-set union.
    let adjacent = square_collection(&[[0.0, 0.0, 2.0, 2.0], [2.0, 0.0, 4.0, 2.0]]);
    let spanning = line(&[[1.0, 1.0], [3.0, 1.0]]);

    let im = relate(&adjacent, &spanning).unwrap();
    assert!(im.is_intersects());
    assert!(!im.is_contains()); // JTS limitation, documented in relate/mod.rs

    // The split-based predicate is exact on the same input.
    assert_eq!(contains(&adjacent, &spanning), Ok(true));

    // The equivalent mesh gets the union answer from relate as well.
    let im = relate(&two_quad_mesh(), &spanning).unwrap();
    assert!(im.is_contains());
}

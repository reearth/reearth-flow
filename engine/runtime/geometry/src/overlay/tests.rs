//! Overlay tests: canonical cases, differential sweeps against the legacy
//! `BooleanOps` (same `i_overlay` backend, so results must be bit-identical),
//! and cross-phase consistency with the phase-2/3 predicates.

use pretty_assertions::assert_eq;

use super::*;
use crate::collection::Collection2D;
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::point::Point2D;
use crate::polygon_mesh::PolygonMesh2D;
use crate::predicates::kernel::CoordPos;
use crate::predicates::relate::Dimensions;
use crate::predicates::view::polygon2d_rings;
use crate::predicates::{covers, relate};
use crate::triangular_mesh::TriangularMesh2D;

use crate::algorithm::bool_ops::{BooleanOps, OpType};
use crate::types::coordinate::Coordinate2D;
use crate::types::line_string::LineString2D as LegacyLineString2D;
use crate::types::multi_line_string::MultiLineString2D as LegacyMultiLineString2D;
use crate::types::multi_polygon::MultiPolygon2D as LegacyMultiPolygon2D;
use crate::types::polygon::Polygon2D as LegacyPolygon2D;

fn e() -> CoordinateFrame {
    CoordinateFrame::Euclidean
}

// --- builders ------------------------------------------------------------------

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

/// Wrap an overlay result as a geometry, for feeding back into the predicates.
fn to_geometry(polygons: Vec<Polygon2D>) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
        polygons
            .into_iter()
            .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
    )))
}

/// Twice the signed area of a closed ring (positive = CCW).
fn doubled_signed_area(ring: &[[f64; 2]]) -> f64 {
    ring.windows(2)
        .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
        .sum()
}

/// The area of an overlay result (holes wound CW subtract themselves).
fn area(polygons: &[Polygon2D]) -> f64 {
    polygons
        .iter()
        .flat_map(polygon2d_rings)
        .map(|ring| doubled_signed_area(ring) / 2.0)
        .sum()
}

// --- canonical overlay cases ---------------------------------------------------

#[test]
fn two_overlapping_squares() {
    let a = polygon(&rect(0.0, 0.0, 2.0, 2.0), &[]);
    let b = polygon(&rect(1.0, 1.0, 3.0, 3.0), &[]);

    let union = union(&a, &b).unwrap();
    assert_eq!(union.len(), 1);
    assert_eq!(area(&union), 7.0);
    assert_eq!(union[0].exterior().len(), 9); // 8-corner staircase, closed
    assert!(doubled_signed_area(union[0].exterior()) > 0.0); // CCW

    let intersection = intersection(&a, &b).unwrap();
    assert_eq!(intersection.len(), 1);
    assert_eq!(area(&intersection), 1.0);

    let difference = difference(&a, &b).unwrap();
    assert_eq!(difference.len(), 1);
    assert_eq!(area(&difference), 3.0);

    let xor = xor(&a, &b).unwrap();
    assert_eq!(xor.len(), 2);
    assert_eq!(area(&xor), 6.0);
}

#[test]
fn difference_can_create_a_hole() {
    let outer = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);
    let inner = polygon(&rect(1.0, 1.0, 3.0, 3.0), &[]);

    let result = difference(&outer, &inner).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(area(&result), 12.0);
    let rings: Vec<&[[f64; 2]]> = polygon2d_rings(&result[0]).collect();
    assert_eq!(rings.len(), 2);
    assert!(doubled_signed_area(rings[0]) > 0.0); // exterior CCW
    assert!(doubled_signed_area(rings[1]) < 0.0); // hole CW

    // ... and a hole in the input participates: the donut minus nothing.
    let donut = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[rect_cw(1.0, 1.0, 3.0, 3.0)]);
    let kept = union(&donut, &Geometry::None).unwrap();
    assert_eq!(area(&kept), 12.0);
}

#[test]
fn disjoint_squares() {
    let a = polygon(&rect(0.0, 0.0, 2.0, 2.0), &[]);
    let b = polygon(&rect(5.0, 5.0, 7.0, 7.0), &[]);

    assert_eq!(union(&a, &b).unwrap().len(), 2);
    assert!(intersection(&a, &b).unwrap().is_empty());
    assert_eq!(area(&difference(&a, &b).unwrap()), 4.0);
    assert_eq!(area(&xor(&a, &b).unwrap()), 8.0);
}

#[test]
fn empty_and_none_operands() {
    let a = polygon(&rect(0.0, 0.0, 2.0, 2.0), &[]);
    let empty = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([])));

    assert_eq!(area(&union(&a, &Geometry::None).unwrap()), 4.0);
    assert_eq!(area(&union(&Geometry::None, &a).unwrap()), 4.0);
    assert_eq!(area(&difference(&a, &empty).unwrap()), 4.0);
    assert!(intersection(&a, &Geometry::None).unwrap().is_empty());
    assert!(difference(&Geometry::None, &a).unwrap().is_empty());
    assert!(union(&Geometry::None, &empty).unwrap().is_empty());
}

#[test]
fn collection_operand_is_a_point_set_union() {
    // Edge-adjacent members — the pair `relate` cannot label — dissolve
    // exactly under the non-zero fill rule.
    let adjacent = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(0.0, 0.0, 2.0, 2.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(2.0, 0.0, 4.0, 2.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
    ])));
    let dissolved = union(&adjacent, &Geometry::None).unwrap();
    assert_eq!(dissolved.len(), 1);
    assert_eq!(area(&dissolved), 8.0);

    // Overlapping members dissolve too (counts add, not cancel).
    let overlapping = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(0.0, 0.0, 3.0, 2.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(1.0, 0.0, 4.0, 2.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
    ])));
    let dissolved = union(&overlapping, &Geometry::None).unwrap();
    assert_eq!(dissolved.len(), 1);
    assert_eq!(area(&dissolved), 8.0);
}

#[test]
fn mesh_operands_dissolve_to_their_union() {
    // Two quads sharing the edge x = 2 form the rect [0,4] x [0,2].
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
    let mesh = Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(mesh)));
    let dissolved = polygon(&rect(0.0, 0.0, 4.0, 2.0), &[]);
    let other = polygon(&rect(1.0, 1.0, 3.0, 3.0), &[]);

    for op in [
        OverlayOp::Union,
        OverlayOp::Intersection,
        OverlayOp::Difference,
        OverlayOp::Xor,
    ] {
        let from_mesh = to_geometry(overlay(&mesh, &other, op).unwrap());
        let from_polygon = to_geometry(overlay(&dissolved, &other, op).unwrap());
        assert!(
            relate(&from_mesh, &from_polygon).unwrap().is_equal_topo(),
            "{op:?} over the mesh diverges from its dissolved polygon"
        );
    }

    // A triangulated square unions like the square.
    let tri = TriangularMesh2D::from_parts(
        e(),
        vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        [0u32, 1, 2, 0, 2, 3],
    )
    .unwrap();
    let tri = Geometry::Euclidean2D(Euclidean2DGeometry::TriangularMesh(Box::new(tri)));
    let out = union(&tri, &Geometry::None).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(area(&out), 4.0);
}

#[test]
fn error_operands() {
    let a = polygon(&rect(0.0, 0.0, 2.0, 2.0), &[]);

    // A non-areal leaf in an overlay operand.
    assert_eq!(
        union(&a, &line(&[[0.0, 0.0], [1.0, 1.0]])),
        Err(PredicateError::UnsupportedPair {
            left: "Polygon2D",
            right: "LineString2D",
        })
    );
    assert_eq!(
        union(&point([0.0, 0.0]), &a),
        Err(PredicateError::UnsupportedPair {
            left: "Point2D",
            right: "Polygon2D",
        })
    );

    // Mixed frames.
    let other_frame = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
        Polygon2D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            rect(0.0, 0.0, 2.0, 2.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ),
    )));
    assert_eq!(union(&a, &other_frame), Err(PredicateError::MixedFrames));

    // Dimension policy.
    let p3 = Geometry::Euclidean3D(crate::Euclidean3DGeometry::Point(
        crate::point::Point3D::new(e(), [0.0, 0.0, 0.0]),
    ));
    assert_eq!(union(&a, &p3), Err(PredicateError::CrossDimension));
    assert!(matches!(
        union(&p3, &p3),
        Err(PredicateError::UnsupportedPair { .. })
    ));
}

// --- clip ------------------------------------------------------------------------

fn clip_coords(result: &[LineString2D]) -> Vec<Vec<[f64; 2]>> {
    result.iter().map(|l| l.coords().to_vec()).collect()
}

#[test]
fn clip_line_against_square() {
    let square = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);
    let crossing = line(&[[-1.0, 1.0], [5.0, 1.0]]);

    let inside = clip(&crossing, &square, false).unwrap();
    assert_eq!(clip_coords(&inside), vec![vec![[0.0, 1.0], [4.0, 1.0]]]);

    let mut outside = clip_coords(&clip(&crossing, &square, true).unwrap());
    outside.sort_by(|a, b| a[0][0].total_cmp(&b[0][0]));
    assert_eq!(
        outside,
        vec![vec![[-1.0, 1.0], [0.0, 1.0]], vec![[4.0, 1.0], [5.0, 1.0]],]
    );

    // Fully inside comes back verbatim; on the boundary counts as inside.
    let inner = line(&[[1.0, 1.0], [3.0, 3.0]]);
    assert_eq!(
        clip_coords(&clip(&inner, &square, false).unwrap()),
        vec![vec![[1.0, 1.0], [3.0, 3.0]]]
    );
    assert!(clip(&inner, &square, true).unwrap().is_empty());
    let boundary = line(&[[1.0, 0.0], [3.0, 0.0]]);
    assert_eq!(
        clip_coords(&clip(&boundary, &square, false).unwrap()),
        vec![vec![[1.0, 0.0], [3.0, 0.0]]]
    );
}

#[test]
fn clip_against_mesh_crosses_internal_edges_unbroken() {
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
    let mesh = Geometry::Euclidean2D(Euclidean2DGeometry::PolygonMesh(Box::new(mesh)));
    // Crosses the dissolved internal edge x = 2: must survive as one piece.
    let inner = line(&[[1.0, 1.0], [3.0, 1.0]]);
    assert_eq!(
        clip_coords(&clip(&inner, &mesh, false).unwrap()),
        vec![vec![[1.0, 1.0], [3.0, 1.0]]]
    );
}

#[test]
fn clip_empty_and_error_operands() {
    let square = polygon(&rect(0.0, 0.0, 4.0, 4.0), &[]);
    let l = line(&[[1.0, 1.0], [3.0, 1.0]]);

    // No lines -> nothing; no area -> nothing inside, everything outside.
    assert!(clip(&Geometry::None, &square, false).unwrap().is_empty());
    assert!(clip(&l, &Geometry::None, false).unwrap().is_empty());
    assert_eq!(
        clip_coords(&clip(&l, &Geometry::None, true).unwrap()),
        vec![vec![[1.0, 1.0], [3.0, 1.0]]]
    );

    // Wrong leaf kinds on either side.
    assert_eq!(
        clip(&square, &square, false),
        Err(PredicateError::UnsupportedPair {
            left: "Polygon2D",
            right: "Polygon2D",
        })
    );
    assert_eq!(
        clip(&l, &l, false),
        Err(PredicateError::UnsupportedPair {
            left: "LineString2D",
            right: "LineString2D",
        })
    );
}

#[test]
fn clip_differential_against_legacy() {
    let mut rng = Rng(20260716);
    for case in 0..100 {
        let (ext, holes) = random_polygon_rings(&mut rng);
        let area_new = polygon(&ext, &holes);
        let area_legacy = LegacyPolygon2D::new(
            LegacyLineString2D::new(legacy_coords(&ext)),
            holes
                .iter()
                .map(|h| LegacyLineString2D::new(legacy_coords(h)))
                .collect(),
        );

        let coords: Vec<[f64; 2]> = (0..2 + rng.range(3)).map(|_| rng.coord()).collect();
        let line_new = line(&coords);
        let line_legacy =
            LegacyMultiLineString2D::new(vec![LegacyLineString2D::new(legacy_coords(&coords))]);

        for invert in [false, true] {
            let ours = clip_coords(&clip(&line_new, &area_new, invert).unwrap());
            let oracle: Vec<Vec<[f64; 2]>> =
                <LegacyPolygon2D<f64> as BooleanOps>::clip(&area_legacy, &line_legacy, invert)
                    .0
                    .into_iter()
                    .map(|l| l.0.into_iter().map(|c| [c.x, c.y]).collect())
                    .collect();
            assert_eq!(
                ours, oracle,
                "case {case} invert={invert}: clip diverges from legacy"
            );
        }
    }
}

// --- segment intersections -------------------------------------------------------

#[test]
fn segment_intersections_cases() {
    let horizontal = line(&[[0.0, 0.0], [4.0, 0.0]]);

    // Proper crossing.
    assert_eq!(
        segment_intersections(&horizontal, &line(&[[2.0, -2.0], [2.0, 2.0]])).unwrap(),
        vec![SegmentIntersection::SinglePoint {
            intersection: [2.0, 0.0],
            is_proper: true,
        }]
    );

    // T-junction: an endpoint on the other's interior is improper.
    assert_eq!(
        segment_intersections(&horizontal, &line(&[[2.0, 0.0], [2.0, 2.0]])).unwrap(),
        vec![SegmentIntersection::SinglePoint {
            intersection: [2.0, 0.0],
            is_proper: false,
        }]
    );

    // Collinear overlap, canonicalized endpoints.
    assert_eq!(
        segment_intersections(&horizontal, &line(&[[6.0, 0.0], [2.0, 0.0]])).unwrap(),
        vec![SegmentIntersection::Collinear {
            start: [2.0, 0.0],
            end: [4.0, 0.0],
        }]
    );

    // A crossing at a shared vertex of consecutive segments deduplicates.
    let bent = line(&[[0.0, 0.0], [2.0, 2.0], [4.0, 0.0]]);
    assert_eq!(
        segment_intersections(&bent, &line(&[[2.0, 0.0], [2.0, 4.0]])).unwrap(),
        vec![SegmentIntersection::SinglePoint {
            intersection: [2.0, 2.0],
            is_proper: false,
        }]
    );

    // Disjoint and empty.
    assert!(
        segment_intersections(&horizontal, &line(&[[0.0, 1.0], [4.0, 1.0]]))
            .unwrap()
            .is_empty()
    );
    assert!(segment_intersections(&horizontal, &Geometry::None)
        .unwrap()
        .is_empty());

    // Non-line leaves error.
    assert_eq!(
        segment_intersections(&horizontal, &polygon(&rect(0.0, 0.0, 2.0, 2.0), &[])),
        Err(PredicateError::UnsupportedPair {
            left: "LineString2D",
            right: "Polygon2D",
        })
    );
}

// --- differential + consistency sweeps --------------------------------------------

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

fn legacy_coords(coords: &[[f64; 2]]) -> Vec<Coordinate2D<f64>> {
    coords
        .iter()
        .map(|c| Coordinate2D::new_(c[0], c[1]))
        .collect()
}

/// One random valid polygon as `(exterior, holes)` rings on the integer grid:
/// a rect, a rect with a hole, or a CCW triangle.
fn random_polygon_rings(rng: &mut Rng) -> PolygonRings {
    match rng.range(3) {
        0 => {
            let (x0, y0) = (rng.range(6) as f64, rng.range(6) as f64);
            let (w, h) = (1 + rng.range(3), 1 + rng.range(3));
            (rect(x0, y0, x0 + w as f64, y0 + h as f64), vec![])
        }
        1 => {
            let (x0, y0) = (rng.range(4) as f64, rng.range(4) as f64);
            let (x1, y1) = (
                x0 + 3.0 + rng.range(2) as f64,
                y0 + 3.0 + rng.range(2) as f64,
            );
            (
                rect(x0, y0, x1, y1),
                vec![rect_cw(x0 + 1.0, y0 + 1.0, x1 - 1.0, y1 - 1.0)],
            )
        }
        _ => loop {
            // Triangle, wound CCW: overlay assumes Flow's validated winding
            // (a CW exterior reads as negative space when members overlap).
            let (a, b, c) = (rng.coord(), rng.coord(), rng.coord());
            let doubled_area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
            if doubled_area > 0.0 {
                return (vec![a, b, c, a], vec![]);
            }
            if doubled_area < 0.0 {
                return (vec![a, c, b, a], vec![]);
            }
        },
    }
}

/// The `(exterior, holes)` rings of one polygon.
type PolygonRings = (Vec<[f64; 2]>, Vec<Vec<[f64; 2]>>);

/// One random areal operand (one or two polygons) in both representations.
fn random_areal(rng: &mut Rng) -> (Geometry, LegacyMultiPolygon2D<f64>) {
    let polygons: Vec<PolygonRings> = (0..1 + rng.range(2))
        .map(|_| random_polygon_rings(rng))
        .collect();
    let new = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
        polygons.iter().map(|(ext, holes)| {
            Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                e(),
                ext.clone(),
                holes.clone(),
            )))
        }),
    )));
    let legacy = LegacyMultiPolygon2D::new(
        polygons
            .iter()
            .map(|(ext, holes)| {
                LegacyPolygon2D::new(
                    LegacyLineString2D::new(legacy_coords(ext)),
                    holes
                        .iter()
                        .map(|h| LegacyLineString2D::new(legacy_coords(h)))
                        .collect(),
                )
            })
            .collect(),
    );
    (new, legacy)
}

/// An overlay result's rings, for exact comparison with the legacy output.
fn result_rings(polygons: &[Polygon2D]) -> Vec<Vec<Vec<[f64; 2]>>> {
    polygons
        .iter()
        .map(|p| polygon2d_rings(p).map(<[[f64; 2]]>::to_vec).collect())
        .collect()
}

#[test]
fn differential_against_legacy_boolean_ops() {
    let mut rng = Rng(20260715);
    for case in 0..200 {
        let (a, legacy_a) = random_areal(&mut rng);
        let (b, legacy_b) = random_areal(&mut rng);
        for (op, legacy_op) in [
            (OverlayOp::Union, OpType::Union),
            (OverlayOp::Intersection, OpType::Intersection),
            (OverlayOp::Difference, OpType::Difference),
            (OverlayOp::Xor, OpType::Xor),
        ] {
            let ours = result_rings(&overlay(&a, &b, op).unwrap());
            let oracle: Vec<Vec<Vec<[f64; 2]>>> = legacy_a
                .boolean_op(&legacy_b, legacy_op)
                .0
                .iter()
                .map(|p| {
                    core::iter::once(p.exterior())
                        .chain(p.interiors().iter())
                        .map(|ring| ring.0.iter().map(|c| [c.x, c.y]).collect())
                        .collect()
                })
                .collect();
            assert_eq!(
                ours, oracle,
                "case {case}: {op:?} diverges from legacy BooleanOps"
            );
        }
    }
}

#[test]
fn overlay_satisfies_area_identities() {
    let mut rng = Rng(42);
    for case in 0..200 {
        let (a, _) = random_areal(&mut rng);
        let (b, _) = random_areal(&mut rng);

        let area_a = area(&union(&a, &Geometry::None).unwrap());
        let area_b = area(&union(&b, &Geometry::None).unwrap());
        let union_ab = area(&union(&a, &b).unwrap());
        let intersection_ab = area(&intersection(&a, &b).unwrap());
        let difference_ab = area(&difference(&a, &b).unwrap());
        let xor_ab = area(&xor(&a, &b).unwrap());

        // Not exact: `i_overlay` snaps constructed intersection points (the
        // triangle kinds produce off-grid ones) to its adaptive grid, moving
        // them by a relative ~1e-9 — areas inherit an absolute ~1e-7.
        let close = |x: f64, y: f64| (x - y).abs() < 1e-6;
        assert!(
            close(union_ab + intersection_ab, area_a + area_b),
            "case {case}: |A∪B| + |A∩B| = |A| + |B| violated"
        );
        assert!(
            close(difference_ab, area_a - intersection_ab),
            "case {case}: |A\\B| = |A| - |A∩B| violated"
        );
        assert!(
            close(xor_ab, union_ab - intersection_ab),
            "case {case}: |A⊕B| = |A∪B| - |A∩B| violated"
        );
    }
}

#[test]
fn overlay_agrees_with_predicates() {
    let mut rng = Rng(7);
    for case in 0..150 {
        // Single-polygon operands: `relate` does not support operands whose
        // own members overlap (see its module docs), which `random_areal`
        // produces — overlay itself handles those, tested elsewhere.
        let (ext, holes) = random_polygon_rings(&mut rng);
        let a = polygon(&ext, &holes);
        let (ext, holes) = random_polygon_rings(&mut rng);
        let b = polygon(&ext, &holes);

        // The interiors share area exactly when the intersection is non-empty.
        // (Robust on this generator: two integer-grid polygons either share no
        // area or share far more of it than the backend's snap can erase.)
        let im = relate(&a, &b).unwrap();
        let interiors_share_area =
            im.get(CoordPos::Inside, CoordPos::Inside) == Dimensions::TwoDimensional;
        let intersection = intersection(&a, &b).unwrap();
        assert_eq!(
            interiors_share_area,
            !intersection.is_empty(),
            "case {case}: relate II dimension vs intersection emptiness"
        );
    }
}

#[test]
fn overlay_output_covers_exactly_on_the_grid() {
    // Axis-aligned operands only: every constructed intersection point then
    // lies on the integer grid, the backend snap is the identity, and the
    // set relations hold *exactly* under the phase-2 `covers`. (With oblique
    // edges the snapped union boundary may cut ~1e-9 inside the true union.)
    let mut rng = Rng(11);
    for case in 0..150 {
        let rectal = |rng: &mut Rng| loop {
            let (a, _) = random_areal(rng);
            let oblique = match &a {
                Geometry::Euclidean2D(Euclidean2DGeometry::Collection(c)) => {
                    c.members().iter().any(|m| match m {
                        Euclidean2DGeometry::Polygon(p) => p.exterior().len() == 4, // triangle
                        _ => true,
                    })
                }
                _ => true,
            };
            if !oblique {
                return a;
            }
        };
        let a = rectal(&mut rng);
        let b = rectal(&mut rng);

        // The union covers both operands; both operands cover the intersection.
        let union = to_geometry(union(&a, &b).unwrap());
        assert!(
            covers(&union, &a).unwrap() && covers(&union, &b).unwrap(),
            "case {case}: union does not cover its operands"
        );
        let intersection = intersection(&a, &b).unwrap();
        if !intersection.is_empty() {
            let intersection = to_geometry(intersection);
            assert!(
                covers(&a, &intersection).unwrap() && covers(&b, &intersection).unwrap(),
                "case {case}: operands do not cover their intersection"
            );
        }
    }
}

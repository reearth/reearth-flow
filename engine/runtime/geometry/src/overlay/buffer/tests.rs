//! Buffer tests: canonical shapes and their areas, winding, planes, and the
//! operand policy.

use core::f64::consts::PI;

use pretty_assertions::assert_eq;

use super::*;
use crate::coordinate::EpsgCode;
use crate::line_string::{LineString2D, LineString3D};
use crate::point::{Point2D, Point3D};
use crate::polygon_mesh::PolygonMesh2D;
use crate::predicates::covers;
use crate::predicates::view::polygon2d_rings;

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

fn polygon_in(frame: CoordinateFrame, exterior: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
        Polygon2D::from_rings(frame, exterior.to_vec(), holes.to_vec()),
    )))
}

fn polygon(exterior: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Geometry {
    polygon_in(e(), exterior, holes)
}

fn polygon_3d(exterior: &[[f64; 3]], holes: &[Vec<[f64; 3]>]) -> Geometry {
    Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
        Polygon3D::from_rings(e(), exterior.to_vec(), holes.to_vec()),
    )))
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
}

fn rect_cw(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0], [x0, y0]]
}

fn to_geometry(polygons: Vec<Polygon2D>) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
        polygons
            .into_iter()
            .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
    )))
}

// --- measures ------------------------------------------------------------------

/// Twice the signed area of a closed ring (positive = CCW).
fn doubled_signed_area(ring: &[[f64; 2]]) -> f64 {
    ring.windows(2)
        .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
        .sum()
}

/// The area of a 2D result (holes wound CW subtract themselves).
fn area(polygons: &[Polygon2D]) -> f64 {
    polygons
        .iter()
        .flat_map(polygon2d_rings)
        .map(|ring| doubled_signed_area(ring) / 2.0)
        .sum()
}

/// The unnormalised Newell vector of a closed 3D ring.
fn newell(ring: &[[f64; 3]]) -> [f64; 3] {
    let mut n = [0.0; 3];
    for w in ring.windows(2) {
        let (a, b) = (w[0], w[1]);
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

/// The in-plane area of a 3D polygon: exterior area minus its holes'.
fn area_3d(polygon: &Polygon3D) -> f64 {
    let mag = |n: [f64; 3]| (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0;
    mag(newell(polygon.exterior())) - polygon.interiors().map(|h| mag(newell(h))).sum::<f64>()
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected} ± {tolerance}, got {actual}"
    );
}

/// The area a disc of radius `r` loses to an inscribed polygon with the given
/// number of vertices.
fn inscribed_loss(r: f64, n: usize) -> f64 {
    let n = n as f64;
    r * r * (PI - n / 2.0 * (2.0 * PI / n).sin())
}

fn style(distance: f64) -> BufferStyle {
    BufferStyle::new(distance)
}

// --- points ------------------------------------------------------------------

#[test]
fn point_buffers_to_a_disc() {
    let result = buffer_2d(
        &Euclidean2DGeometry::Point(Point2D::new(e(), [3.0, 4.0])),
        &style(2.0),
    )
    .unwrap();
    assert_eq!(result.len(), 1);
    let ring = result[0].exterior();
    let n = ring.len() - 1;
    assert_eq!(n, 32); // 2π / (π/16)
    for p in ring {
        assert_close(
            ((p[0] - 3.0).powi(2) + (p[1] - 4.0).powi(2)).sqrt(),
            2.0,
            1e-9,
        );
    }
    assert!(doubled_signed_area(ring) > 0.0);
    assert_close(area(&result), 4.0 * PI - inscribed_loss(2.0, 32), 1e-9);
    assert_eq!(result[0].elevation(), None);
}

#[test]
fn arc_step_sets_the_disc_resolution_within_its_range() {
    let p = Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0]));
    let coarse = buffer_2d(&p, &style(1.0).arc_step(PI)).unwrap();
    assert_eq!(coarse[0].exterior().len() - 1, 8); // clamped to π/4
    let fine = buffer_2d(&p, &style(1.0).arc_step(0.0)).unwrap();
    assert_eq!(fine[0].exterior().len() - 1, 200); // clamped to π/100
}

#[test]
fn point_and_line_vanish_under_a_non_positive_distance() {
    assert_eq!(
        buffer(&point([0.0, 0.0]), &style(-1.0)).unwrap(),
        Geometry::None
    );
    assert_eq!(
        buffer(&point([0.0, 0.0]), &style(0.0)).unwrap(),
        Geometry::None
    );
    let l = line(&[[0.0, 0.0], [5.0, 0.0]]);
    assert_eq!(buffer(&l, &style(-1.0)).unwrap(), Geometry::None);
    assert_eq!(buffer(&l, &style(0.0)).unwrap(), Geometry::None);
}

// --- lines -------------------------------------------------------------------

#[test]
fn line_buffers_to_a_rounded_stroke() {
    let l = line(&[[0.0, 0.0], [10.0, 0.0]]);
    let result = buffer(&l, &style(1.0)).unwrap();
    let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) = result else {
        panic!("expected one polygon, got {result:?}");
    };
    // 2rL + πr², minus what the polygonal caps lose.
    assert_close(area(&[*p.clone()]), 20.0 + PI, 0.05);
    assert!(covers(&to_geometry(vec![*p]), &l).unwrap());
}

#[test]
fn polyline_stroke_dissolves_its_own_overlaps() {
    // A tight zigzag whose stroke segments overlap heavily.
    let l = line(&[[0.0, 0.0], [1.0, 0.2], [0.0, 0.4], [1.0, 0.6]]);
    let result = buffer_2d(
        &Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            vec![[0.0, 0.0], [1.0, 0.2], [0.0, 0.4], [1.0, 0.6]],
        )),
        &style(1.0),
    )
    .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].interiors().count(), 0);
    assert!(covers(&to_geometry(result), &l).unwrap());
}

// --- polygons ----------------------------------------------------------------

#[test]
fn polygon_expands_with_round_corners() {
    let square = polygon(&rect(0.0, 0.0, 10.0, 10.0), &[]);
    let result = buffer_2d(
        &Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(0.0, 0.0, 10.0, 10.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
        &style(1.0),
    )
    .unwrap();
    assert_eq!(result.len(), 1);
    // 100 + 4·10·1 + π·1², minus the polygonal-corner loss.
    assert_close(area(&result), 140.0 + PI, 0.05);
    assert!(doubled_signed_area(result[0].exterior()) > 0.0);
    assert!(covers(&to_geometry(result), &square).unwrap());
}

#[test]
fn polygon_contracts_exactly_and_vanishes() {
    let square = Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
        e(),
        rect(0.0, 0.0, 10.0, 10.0),
        Vec::<Vec<[f64; 2]>>::new(),
    )));
    let result = buffer_2d(&square, &style(-1.0)).unwrap();
    assert_eq!(result.len(), 1);
    assert_close(area(&result), 64.0, 1e-6);
    let ring = result[0].exterior();
    assert_eq!(ring.len(), 5);
    for want in rect(1.0, 1.0, 9.0, 9.0) {
        assert!(
            ring.iter()
                .any(|p| (p[0] - want[0]).abs() < 1e-6 && (p[1] - want[1]).abs() < 1e-6),
            "vertex {want:?} missing from {ring:?}"
        );
    }

    assert!(buffer_2d(&square, &style(-5.0)).unwrap().is_empty());
    assert_eq!(
        buffer(&Geometry::Euclidean2D(square), &style(-6.0)).unwrap(),
        Geometry::None
    );
}

#[test]
fn zero_distance_returns_the_dissolved_input() {
    let g = polygon(&rect(0.0, 0.0, 10.0, 10.0), &[]);
    let result = buffer(&g, &style(0.0)).unwrap();
    assert!(covers(&result, &g).unwrap());
    assert!(covers(&g, &result).unwrap());
}

#[test]
fn holes_shrink_on_expansion_and_grow_on_contraction() {
    let with_hole = Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
        e(),
        rect(0.0, 0.0, 10.0, 10.0),
        vec![rect_cw(4.0, 4.0, 6.0, 6.0)],
    )));

    let expanded = buffer_2d(&with_hole, &style(0.5)).unwrap();
    assert_eq!(expanded.len(), 1);
    let holes: Vec<&[[f64; 2]]> = expanded[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    assert_close(doubled_signed_area(holes[0]) / 2.0, -1.0, 1e-6);

    let filled = buffer_2d(&with_hole, &style(2.0)).unwrap();
    assert_eq!(filled.len(), 1);
    assert_eq!(filled[0].interiors().count(), 0);

    let contracted = buffer_2d(&with_hole, &style(-1.0)).unwrap();
    assert_eq!(contracted.len(), 1);
    let holes: Vec<&[[f64; 2]]> = contracted[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    // 4·1² sides + 4·2·1 flanks + π·1² rounded corners.
    assert_close(doubled_signed_area(holes[0]) / 2.0, -(4.0 + 8.0 + PI), 0.05);
    assert_close(
        doubled_signed_area(contracted[0].exterior()) / 2.0,
        64.0,
        1e-6,
    );
}

#[test]
fn contraction_can_split_a_polygon() {
    // Two 4×4 blocks joined by a 1-wide neck.
    let dumbbell = Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
        e(),
        vec![
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 1.5],
            [6.0, 1.5],
            [6.0, 0.0],
            [10.0, 0.0],
            [10.0, 4.0],
            [6.0, 4.0],
            [6.0, 2.5],
            [4.0, 2.5],
            [4.0, 4.0],
            [0.0, 4.0],
            [0.0, 0.0],
        ],
        Vec::<Vec<[f64; 2]>>::new(),
    )));
    let parts = buffer_2d(&dumbbell, &style(-1.0)).unwrap();
    assert_eq!(parts.len(), 2);
    // Two 2×2 cores, each with a little extra area under the arcs the inset
    // sweeps around the neck's reflex corners.
    assert!(
        area(&parts) > 8.0 && area(&parts) < 8.5,
        "area {}",
        area(&parts)
    );
    let g = buffer(&Geometry::Euclidean2D(dumbbell), &style(-1.0)).unwrap();
    assert!(matches!(
        g,
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(_))
    ));
}

#[test]
fn stored_winding_does_not_matter_but_output_follows_the_frame() {
    let cw = Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
        e(),
        rect_cw(0.0, 0.0, 10.0, 10.0),
        vec![rect(4.0, 4.0, 6.0, 6.0)],
    )));
    let result = buffer_2d(&cw, &style(0.5)).unwrap();
    assert_eq!(result.len(), 1);
    // A Euclidean frame is right-handed: exterior CCW, hole CW.
    assert!(doubled_signed_area(result[0].exterior()) > 0.0);
    let holes: Vec<&[[f64; 2]]> = result[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    assert!(doubled_signed_area(holes[0]) < 0.0);
    assert_close(area(&result), 120.0 + PI * 0.25 - 1.0, 0.05);
}

#[test]
fn a_reflected_frame_gets_clockwise_stored_exteriors() {
    // EPSG:4326 is (lat, lon): its stored winding is the mirror of canonical.
    let frame = CoordinateFrame::Crs(EpsgCode::new(4326));
    assert_eq!(frame.orientation_sign().unwrap(), -1);
    let g = polygon_in(frame.clone(), &rect_cw(0.0, 0.0, 1.0, 1.0), &[]);
    let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) = buffer(&g, &style(0.1)).unwrap()
    else {
        panic!("expected one polygon");
    };
    assert_eq!(p.frame(), &frame);
    assert!(doubled_signed_area(p.exterior()) < 0.0);
    // Points buffer to the frame's convention too.
    let g = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame, [0.0, 0.0])));
    let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) = buffer(&g, &style(0.1)).unwrap()
    else {
        panic!("expected one polygon");
    };
    assert!(doubled_signed_area(p.exterior()) < 0.0);
}

#[test]
fn elevation_survives_when_every_leaf_agrees() {
    let at_five = Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings_at_elevation(
        e(),
        rect(0.0, 0.0, 1.0, 1.0),
        Vec::<Vec<[f64; 2]>>::new(),
        5.0,
    )));
    let result = buffer_2d(&at_five, &style(1.0)).unwrap();
    assert_eq!(result[0].elevation(), Some(5.0));

    let mixed = Euclidean2DGeometry::Collection(Collection2D::new([
        at_five,
        Euclidean2DGeometry::Point(Point2D::new(e(), [0.5, 0.5])),
    ]));
    let result = buffer_2d(&mixed, &style(1.0)).unwrap();
    assert_eq!(result[0].elevation(), None);
}

// --- meshes and collections --------------------------------------------------

#[test]
fn mesh_buffers_like_its_dissolved_outline() {
    // Two triangles forming the unit square, sharing the diagonal.
    let mesh = PolygonMesh2D::from_parts(
        e(),
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        vec![vec![0u32, 1, 2], vec![0, 2, 3]],
    );
    let from_mesh = buffer_2d(
        &Euclidean2DGeometry::PolygonMesh(Box::new(mesh.unwrap())),
        &style(0.5),
    )
    .unwrap();
    let from_polygon = buffer_2d(
        &Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(0.0, 0.0, 1.0, 1.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
        &style(0.5),
    )
    .unwrap();
    assert_eq!(from_mesh.len(), 1);
    assert_eq!(from_mesh[0].interiors().count(), 0);
    assert_close(area(&from_mesh), area(&from_polygon), 1e-9);
}

#[test]
fn collection_buffers_the_union_of_its_leaves() {
    let far = Euclidean2DGeometry::Collection(Collection2D::new([
        Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0])),
        Euclidean2DGeometry::Point(Point2D::new(e(), [10.0, 0.0])),
    ]));
    assert_eq!(buffer_2d(&far, &style(1.0)).unwrap().len(), 2);

    let near = Euclidean2DGeometry::Collection(Collection2D::new([
        Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0])),
        Euclidean2DGeometry::Point(Point2D::new(e(), [1.0, 0.0])),
    ]));
    let merged = buffer_2d(&near, &style(1.0)).unwrap();
    assert_eq!(merged.len(), 1);
    assert!(area(&merged) < 2.0 * PI && area(&merged) > PI);

    // Point, line, and polygon together: one dissolved region covering all.
    let mixed = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
        Euclidean2DGeometry::Point(Point2D::new(e(), [-1.0, 0.0])),
        Euclidean2DGeometry::LineString(LineString2D::from_coords(
            e(),
            vec![[0.0, 0.0], [5.0, 0.0]],
        )),
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            e(),
            rect(5.0, -1.0, 8.0, 1.0),
            Vec::<Vec<[f64; 2]>>::new(),
        ))),
    ])));
    let result = buffer(&mixed, &style(1.0)).unwrap();
    assert!(matches!(
        result,
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(_))
    ));
    assert!(covers(&result, &mixed).unwrap());
}

#[test]
fn empty_and_absent_geometry_buffer_to_nothing() {
    assert_eq!(
        buffer(&Geometry::None, &style(1.0)).unwrap(),
        Geometry::None
    );
    let empty = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([])));
    assert_eq!(buffer(&empty, &style(1.0)).unwrap(), Geometry::None);
}

#[test]
fn mixed_frames_are_refused() {
    let g = Euclidean2DGeometry::Collection(Collection2D::new([
        Euclidean2DGeometry::Point(Point2D::new(e(), [0.0, 0.0])),
        Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(EpsgCode::new(3857)),
            [0.0, 0.0],
        )),
    ]));
    assert_eq!(
        buffer_2d(&g, &style(1.0)).unwrap_err(),
        PredicateError::MixedFrames
    );
}

#[test]
fn non_finite_distance_buffers_to_nothing() {
    let g = polygon(&rect(0.0, 0.0, 1.0, 1.0), &[]);
    assert_eq!(buffer(&g, &style(f64::NAN)).unwrap(), Geometry::None);
    assert_eq!(buffer(&g, &style(f64::INFINITY)).unwrap(), Geometry::None);
}

// --- 3D polygons -------------------------------------------------------------

/// The 1 × √2 rectangle in the plane `z = x`.
fn tilted_square() -> Vec<[f64; 3]> {
    vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ]
}

fn assert_on_plane_z_eq_x(polygon: &Polygon3D) {
    for ring in polygon3d_rings(polygon) {
        for p in ring {
            assert_close(p[2], p[0], 1e-9);
        }
    }
}

#[test]
fn polygon_3d_expands_in_its_own_plane() {
    let face = Polygon3D::from_rings(e(), tilted_square(), Vec::<Vec<[f64; 3]>>::new());
    let result = buffer_polygon_3d(&face, &style(0.5)).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].frame(), &e());
    assert_on_plane_z_eq_x(&result[0]);
    // √2 + 2·(1 + √2)·0.5 + π·0.25, minus the corner loss.
    let s = 2.0f64.sqrt();
    assert_close(area_3d(&result[0]), s + (1.0 + s) + PI / 4.0, 0.02);
    // Same winding sense as the input.
    let n_in = newell(face.exterior());
    let n_out = newell(result[0].exterior());
    assert!(n_in[0] * n_out[0] + n_in[1] * n_out[1] + n_in[2] * n_out[2] > 0.0);
}

#[test]
fn polygon_3d_contracts_to_the_expected_vertices() {
    let face = Polygon3D::from_rings(e(), tilted_square(), Vec::<Vec<[f64; 3]>>::new());
    let result = buffer_polygon_3d(&face, &style(-0.25)).unwrap();
    assert_eq!(result.len(), 1);
    assert_on_plane_z_eq_x(&result[0]);
    let s = 2.0f64.sqrt();
    // The in-plane inset of 0.25 moves x (and z) by 0.25 / √2 along the tilt.
    let expected = [
        [0.25 / s, 0.25, 0.25 / s],
        [1.0 - 0.25 / s, 0.25, 1.0 - 0.25 / s],
        [1.0 - 0.25 / s, 0.75, 1.0 - 0.25 / s],
        [0.25 / s, 0.75, 0.25 / s],
    ];
    let ring = result[0].exterior();
    assert_eq!(ring.len(), 5);
    for want in expected {
        assert!(
            ring.iter()
                .any(|p| (0..3).all(|k| (p[k] - want[k]).abs() < 1e-6)),
            "vertex {want:?} missing from {ring:?}"
        );
    }
    assert!(buffer_polygon_3d(&face, &style(-0.6)).unwrap().is_empty());
}

#[test]
fn polygon_3d_keeps_a_downward_normal_and_its_hole() {
    // The square wound clockwise from above (normal -z), with a hole.
    let face = Polygon3D::from_rings(
        e(),
        vec![
            [0.0, 0.0, 2.0],
            [0.0, 10.0, 2.0],
            [10.0, 10.0, 2.0],
            [10.0, 0.0, 2.0],
            [0.0, 0.0, 2.0],
        ],
        vec![vec![
            [4.0, 4.0, 2.0],
            [6.0, 4.0, 2.0],
            [6.0, 6.0, 2.0],
            [4.0, 6.0, 2.0],
            [4.0, 4.0, 2.0],
        ]],
    );
    let result = buffer_polygon_3d(&face, &style(0.5)).unwrap();
    assert_eq!(result.len(), 1);
    assert!(newell(result[0].exterior())[2] < 0.0);
    let holes: Vec<&[[f64; 3]]> = result[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    assert!(newell(holes[0])[2] > 0.0);
    for ring in polygon3d_rings(&result[0]) {
        for p in ring {
            assert_close(p[2], 2.0, 1e-9);
        }
    }
    assert_close(area_3d(&result[0]), 120.0 + PI / 4.0 - 1.0, 0.05);
}

#[test]
fn vertical_polygon_3d_buffers_in_its_wall() {
    let wall = Polygon3D::from_rings(
        e(),
        vec![
            [0.0, 0.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 4.0, 3.0],
            [0.0, 0.0, 3.0],
            [0.0, 0.0, 0.0],
        ],
        Vec::<Vec<[f64; 3]>>::new(),
    );
    let result = buffer_polygon_3d(&wall, &style(-1.0)).unwrap();
    assert_eq!(result.len(), 1);
    for p in result[0].exterior() {
        assert_close(p[0], 0.0, 1e-9);
    }
    assert_close(area_3d(&result[0]), 2.0, 1e-6);
}

#[test]
fn non_planar_and_degenerate_faces_are_refused() {
    let saddle = Polygon3D::from_rings(
        e(),
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
        ],
        Vec::<Vec<[f64; 3]>>::new(),
    );
    assert_eq!(
        buffer_polygon_3d(&saddle, &style(0.1)).unwrap_err(),
        PredicateError::NotPlanar
    );
    // A looser tolerance lets it through.
    assert!(buffer_polygon_3d(
        &saddle,
        &style(0.1).planarity(PlanarityThreshold::MaxHeight(10.0))
    )
    .is_ok());

    let collinear = Polygon3D::from_rings(
        e(),
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [0.0, 0.0, 0.0],
        ],
        Vec::<Vec<[f64; 3]>>::new(),
    );
    assert_eq!(
        buffer_polygon_3d(&collinear, &style(0.1)).unwrap_err(),
        PredicateError::NotPlanar
    );
}

#[test]
fn other_3d_leaves_are_unsupported() {
    let p = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(e(), [0.0; 3])));
    assert_eq!(
        buffer(&p, &style(1.0)).unwrap_err(),
        PredicateError::Unsupported {
            geometry: "Point3D"
        }
    );
    let l = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
        e(),
        vec![[0.0; 3], [1.0; 3]],
    )));
    assert_eq!(
        buffer(&l, &style(1.0)).unwrap_err(),
        PredicateError::Unsupported {
            geometry: "LineString3D"
        }
    );
}

#[test]
fn collections_buffer_3d_members_one_by_one() {
    let a = tilted_square();
    let b: Vec<[f64; 3]> = tilted_square()
        .into_iter()
        .map(|[x, y, z]| [x + 100.0, y, z])
        .collect();
    let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            e(),
            a,
            Vec::<Vec<[f64; 3]>>::new(),
        ))),
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            e(),
            b,
            Vec::<Vec<[f64; 3]>>::new(),
        ))),
    ])));
    let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) =
        buffer(&g, &style(0.5)).unwrap()
    else {
        panic!("expected a 3D collection");
    };
    assert_eq!(c.members().len(), 2);

    // A heterogeneous collection keeps its 2D and 3D results side by side.
    let mixed = Geometry::GeometryCollection(GeometryCollection::new([
        polygon(&rect(0.0, 0.0, 1.0, 1.0), &[]),
        polygon_3d(&tilted_square(), &[]),
    ]));
    let Geometry::GeometryCollection(c) = buffer(&mixed, &style(0.5)).unwrap() else {
        panic!("expected a geometry collection");
    };
    assert_eq!(c.members().len(), 2);
    assert!(matches!(c.members()[0], Geometry::Euclidean2D(_)));
    assert!(matches!(c.members()[1], Geometry::Euclidean3D(_)));
}

use core::f64::consts::PI;

use pretty_assertions::assert_eq;

use super::*;
use crate::line_string::LineString2D;
use crate::point::Point2D;
use crate::predicates::covers;
use crate::predicates::view::polygon2d_rings;

fn e() -> CoordinateFrame {
    CoordinateFrame::Euclidean
}

fn point(p: [f64; 2]) -> Euclidean2DGeometry {
    Euclidean2DGeometry::Point(Point2D::new(e(), p))
}

fn line(coords: &[[f64; 2]]) -> Euclidean2DGeometry {
    Euclidean2DGeometry::LineString(LineString2D::from_coords(e(), coords.to_vec()))
}

fn polygon(exterior: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Euclidean2DGeometry {
    Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
        e(),
        exterior.to_vec(),
        holes.to_vec(),
    )))
}

fn polygon_3d(exterior: &[[f64; 3]], holes: &[Vec<[f64; 3]>]) -> Polygon3D {
    Polygon3D::from_rings(e(), exterior.to_vec(), holes.to_vec())
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
}

fn rect_cw(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 2]> {
    vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0], [x0, y0]]
}

fn geometry(polygons: Vec<Polygon2D>) -> Geometry {
    Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
        polygons
            .into_iter()
            .map(|p| Euclidean2DGeometry::Polygon(Box::new(p))),
    )))
}

fn signed_area(ring: &[[f64; 2]]) -> f64 {
    signed_area_2d(ring) / 2.0
}

fn area(polygons: &[Polygon2D]) -> f64 {
    polygons
        .iter()
        .flat_map(polygon2d_rings)
        .map(signed_area)
        .sum()
}

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

fn assert_has_vertices<const N: usize>(ring: &[[f64; N]], expected: &[[f64; N]]) {
    for want in expected {
        assert!(
            ring.iter()
                .any(|p| (0..N).all(|k| (p[k] - want[k]).abs() < 1e-6)),
            "vertex {want:?} missing from {ring:?}"
        );
    }
}

fn style(distance: f64) -> BufferStyle {
    BufferStyle::new(distance)
}

#[test]
fn line_buffers_to_a_rounded_stroke() {
    let l = line(&[[0.0, 0.0], [10.0, 0.0]]);
    let result = buffer_2d(&l, &style(1.0)).unwrap();
    assert_eq!(result.len(), 1);
    assert_close(area(&result), 20.0 + PI, 0.05);
    assert!(covers(&geometry(result), &Geometry::Euclidean2D(l)).unwrap());
}

#[test]
fn polygon_expands_with_round_corners() {
    let square = polygon(&rect(0.0, 0.0, 10.0, 10.0), &[]);
    let result = buffer_2d(&square, &style(1.0)).unwrap();
    assert_eq!(result.len(), 1);
    assert_close(area(&result), 140.0 + PI, 0.05);
    assert!(signed_area(result[0].exterior()) > 0.0);
    assert!(covers(&geometry(result), &Geometry::Euclidean2D(square)).unwrap());
}

#[test]
fn polygon_contracts_exactly_and_vanishes() {
    let square = polygon(&rect(0.0, 0.0, 10.0, 10.0), &[]);
    let result = buffer_2d(&square, &style(-1.0)).unwrap();
    assert_eq!(result.len(), 1);
    assert_close(area(&result), 64.0, 1e-6);
    assert_eq!(result[0].exterior().len(), 5);
    assert_has_vertices(result[0].exterior(), &rect(1.0, 1.0, 9.0, 9.0));

    assert!(buffer_2d(&square, &style(-5.0)).unwrap().is_empty());
    assert_eq!(
        buffer(&Geometry::Euclidean2D(square), &style(-6.0)).unwrap(),
        Geometry::None
    );
}

#[test]
fn holes_shrink_on_expansion_and_grow_on_contraction() {
    let with_hole = polygon(&rect(0.0, 0.0, 10.0, 10.0), &[rect_cw(4.0, 4.0, 6.0, 6.0)]);

    let expanded = buffer_2d(&with_hole, &style(0.5)).unwrap();
    let holes: Vec<&[[f64; 2]]> = expanded[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    assert_close(signed_area(holes[0]), -1.0, 1e-6);

    let filled = buffer_2d(&with_hole, &style(2.0)).unwrap();
    assert_eq!(filled[0].interiors().count(), 0);

    let contracted = buffer_2d(&with_hole, &style(-1.0)).unwrap();
    let holes: Vec<&[[f64; 2]]> = contracted[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    assert_close(signed_area(holes[0]), -(4.0 + 8.0 + PI), 0.05);
    assert_close(signed_area(contracted[0].exterior()), 64.0, 1e-6);
}

#[test]
fn contraction_can_split_a_polygon() {
    let dumbbell = polygon(
        &[
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
        &[],
    );
    let parts = buffer_2d(&dumbbell, &style(-1.0)).unwrap();
    assert_eq!(parts.len(), 2);
    assert!(
        area(&parts) > 8.0 && area(&parts) < 8.5,
        "area {}",
        area(&parts)
    );
    assert!(matches!(
        buffer(&Geometry::Euclidean2D(dumbbell), &style(-1.0)).unwrap(),
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(_))
    ));
}

#[test]
fn collection_buffers_the_union_of_its_leaves() {
    let mixed = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
        point([-1.0, 0.0]),
        line(&[[0.0, 0.0], [5.0, 0.0]]),
        polygon(&rect(5.0, -1.0, 8.0, 1.0), &[]),
    ])));
    let result = buffer(&mixed, &style(1.0)).unwrap();
    assert!(matches!(
        result,
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(_))
    ));
    assert!(covers(&result, &mixed).unwrap());
}

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
    for p in polygon3d_rings(polygon).flatten() {
        assert_close(p[2], p[0], 1e-9);
    }
}

#[test]
fn polygon_3d_contracts_to_the_expected_vertices() {
    let face = polygon_3d(&tilted_square(), &[]);
    let result = buffer_polygon_3d(&face, &style(-0.25)).unwrap();
    assert_eq!(result.len(), 1);
    assert_on_plane_z_eq_x(&result[0]);
    let d = 0.25 / 2.0f64.sqrt();
    assert_eq!(result[0].exterior().len(), 5);
    assert_has_vertices(
        result[0].exterior(),
        &[
            [d, 0.25, d],
            [1.0 - d, 0.25, 1.0 - d],
            [1.0 - d, 0.75, 1.0 - d],
            [d, 0.75, d],
        ],
    );
    let n_in = newell(face.exterior());
    let n_out = newell(result[0].exterior());
    assert!(n_in[0] * n_out[0] + n_in[1] * n_out[1] + n_in[2] * n_out[2] > 0.0);
    assert!(buffer_polygon_3d(&face, &style(-0.6)).unwrap().is_empty());
}

#[test]
fn polygon_3d_keeps_a_downward_normal_and_its_hole() {
    let face = polygon_3d(
        &[
            [0.0, 0.0, 2.0],
            [0.0, 10.0, 2.0],
            [10.0, 10.0, 2.0],
            [10.0, 0.0, 2.0],
            [0.0, 0.0, 2.0],
        ],
        &[vec![
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
    for p in polygon3d_rings(&result[0]).flatten() {
        assert_close(p[2], 2.0, 1e-9);
    }
    assert_close(area_3d(&result[0]), 120.0 + PI / 4.0 - 1.0, 0.05);
}

#[test]
fn non_finite_distance_buffers_to_nothing() {
    let square = polygon(&rect(0.0, 0.0, 1.0, 1.0), &[]);
    let face = polygon_3d(&tilted_square(), &[]);
    for distance in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(buffer_2d(&square, &style(distance)).unwrap().is_empty());
        assert!(buffer_polygon_3d(&face, &style(distance))
            .unwrap()
            .is_empty());
    }
}

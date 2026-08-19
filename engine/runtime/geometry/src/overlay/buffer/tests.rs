use core::f64::consts::PI;

use pretty_assertions::assert_eq;

use super::*;
use crate::coordinate::EpsgCode;
use crate::line_string::LineString2D;
use crate::point::{Point2D, Point3D};
use crate::polygon_mesh::PolygonMesh2D;
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
fn point_buffers_to_a_disc_at_the_clamped_arc_step() {
    let p = point([3.0, 4.0]);
    let result = buffer_2d(&p, &style(2.0)).unwrap();
    assert_eq!(result.len(), 1);
    let ring = result[0].exterior();
    assert_eq!(ring.len() - 1, 32);
    for v in ring {
        assert_close(
            ((v[0] - 3.0).powi(2) + (v[1] - 4.0).powi(2)).sqrt(),
            2.0,
            1e-9,
        );
    }
    assert!(signed_area(ring) > 0.0);
    assert_eq!(result[0].elevation(), None);

    let coarse = buffer_2d(&p, &style(1.0).arc_step(PI)).unwrap();
    assert_eq!(coarse[0].exterior().len() - 1, 8);
    let fine = buffer_2d(&p, &style(1.0).arc_step(0.0)).unwrap();
    assert_eq!(fine[0].exterior().len() - 1, 200);
}

#[test]
fn point_and_line_vanish_under_a_non_positive_distance() {
    let l = line(&[[0.0, 0.0], [5.0, 0.0]]);
    for d in [-1.0, 0.0] {
        assert!(buffer_2d(&point([0.0, 0.0]), &style(d)).unwrap().is_empty());
        assert!(buffer_2d(&l, &style(d)).unwrap().is_empty());
    }
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
fn output_winding_follows_the_frame_not_the_input() {
    let cw = polygon(&rect_cw(0.0, 0.0, 10.0, 10.0), &[rect(4.0, 4.0, 6.0, 6.0)]);
    let result = buffer_2d(&cw, &style(0.5)).unwrap();
    assert!(signed_area(result[0].exterior()) > 0.0);
    let holes: Vec<&[[f64; 2]]> = result[0].interiors().collect();
    assert_eq!(holes.len(), 1);
    assert!(signed_area(holes[0]) < 0.0);
    assert_close(area(&result), 120.0 + PI * 0.25 - 1.0, 0.05);

    let reflected = CoordinateFrame::Crs(EpsgCode::new(4326));
    assert_eq!(reflected.orientation_sign().unwrap(), -1);
    let p = Euclidean2DGeometry::Point(Point2D::new(reflected.clone(), [0.0, 0.0]));
    let result = buffer_2d(&p, &style(0.1)).unwrap();
    assert_eq!(result[0].frame(), &reflected);
    assert!(signed_area(result[0].exterior()) < 0.0);
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

    let mixed = Euclidean2DGeometry::Collection(Collection2D::new([at_five, point([0.5, 0.5])]));
    let result = buffer_2d(&mixed, &style(1.0)).unwrap();
    assert_eq!(result[0].elevation(), None);
}

#[test]
fn mesh_buffers_like_its_dissolved_outline() {
    let mesh = PolygonMesh2D::from_parts(
        e(),
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        vec![vec![0u32, 1, 2], vec![0, 2, 3]],
    )
    .unwrap();
    let from_mesh = buffer_2d(
        &Euclidean2DGeometry::PolygonMesh(Box::new(mesh)),
        &style(0.5),
    )
    .unwrap();
    let from_polygon = buffer_2d(&polygon(&rect(0.0, 0.0, 1.0, 1.0), &[]), &style(0.5)).unwrap();
    assert_eq!(from_mesh.len(), 1);
    assert_eq!(from_mesh[0].interiors().count(), 0);
    assert_close(area(&from_mesh), area(&from_polygon), 1e-9);
}

#[test]
fn collection_buffers_the_union_of_its_leaves() {
    let far =
        Euclidean2DGeometry::Collection(Collection2D::new([point([0.0, 0.0]), point([10.0, 0.0])]));
    assert_eq!(buffer_2d(&far, &style(1.0)).unwrap().len(), 2);

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

#[test]
fn mixed_frames_are_refused() {
    let g = Euclidean2DGeometry::Collection(Collection2D::new([
        point([0.0, 0.0]),
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
fn polygon_3d_expands_in_its_own_plane() {
    let face = polygon_3d(&tilted_square(), &[]);
    let result = buffer_polygon_3d(&face, &style(0.5)).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].frame(), &e());
    assert_on_plane_z_eq_x(&result[0]);
    let s = 2.0f64.sqrt();
    assert_close(area_3d(&result[0]), s + (1.0 + s) + PI / 4.0, 0.02);
    let n_in = newell(face.exterior());
    let n_out = newell(result[0].exterior());
    assert!(n_in[0] * n_out[0] + n_in[1] * n_out[1] + n_in[2] * n_out[2] > 0.0);
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
fn non_planar_and_degenerate_faces_are_refused() {
    let saddle = polygon_3d(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
        ],
        &[],
    );
    assert_eq!(
        buffer_polygon_3d(&saddle, &style(0.1)).unwrap_err(),
        PredicateError::NotPlanar
    );
    assert!(buffer_polygon_3d(
        &saddle,
        &style(0.1).planarity(PlanarityThreshold::MaxHeight(10.0))
    )
    .is_ok());

    let collinear = polygon_3d(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [0.0, 0.0, 0.0],
        ],
        &[],
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
}

#[test]
fn collections_buffer_3d_members_one_by_one() {
    let shifted: Vec<[f64; 3]> = tilted_square()
        .into_iter()
        .map(|[x, y, z]| [x + 100.0, y, z])
        .collect();
    let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
        Euclidean3DGeometry::Polygon(Box::new(polygon_3d(&tilted_square(), &[]))),
        Euclidean3DGeometry::Polygon(Box::new(polygon_3d(&shifted, &[]))),
    ])));
    let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) =
        buffer(&g, &style(0.5)).unwrap()
    else {
        panic!("expected a 3D collection");
    };
    assert_eq!(c.members().len(), 2);

    let mixed = Geometry::GeometryCollection(GeometryCollection::new([
        Geometry::Euclidean2D(polygon(&rect(0.0, 0.0, 1.0, 1.0), &[])),
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(polygon_3d(
            &tilted_square(),
            &[],
        )))),
    ]));
    let Geometry::GeometryCollection(c) = buffer(&mixed, &style(0.5)).unwrap() else {
        panic!("expected a geometry collection");
    };
    assert_eq!(c.members().len(), 2);
    assert!(matches!(c.members()[0], Geometry::Euclidean2D(_)));
    assert!(matches!(c.members()[1], Geometry::Euclidean3D(_)));
}

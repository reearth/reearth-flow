//! Point and line sweeps. A point becomes a segment; a line becomes an open
//! wall mesh, one quad per segment. Walls of a line have no inside, so their
//! winding is simply consistent along the line.

use super::math::{cross, norm, sub};
use super::place::{place, placement};
use super::weld::{weld, weld_tolerance};
use super::{ExtrudeError, ExtrudeSpec};
use crate::coordinate::CoordinateFrame;
use crate::line_string::LineString3D;
use crate::polygon::Polygon3D;
use crate::polygon_mesh::PolygonMesh3D;
use crate::Euclidean3DGeometry;

pub(crate) fn sweep_point(
    frame: &CoordinateFrame,
    p: [f64; 3],
    spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    let (base, top) = place(&[p], &placement(frame, &spec.direction, None)?)?;
    Ok(Euclidean3DGeometry::LineString(LineString3D::from_coords(
        frame.clone(),
        [base[0], top[0]],
    )))
}

pub(crate) fn sweep_line(
    frame: &CoordinateFrame,
    coords: &[[f64; 3]],
    spec: &ExtrudeSpec,
) -> Result<Euclidean3DGeometry, ExtrudeError> {
    let resolved = placement(frame, &spec.direction, None)?;
    let tol = weld_tolerance(coords);
    let (ids, reps) = weld(coords, tol);
    let mut path: Vec<[f64; 3]> = Vec::with_capacity(ids.len());
    let mut last = None;
    for id in ids {
        if last != Some(id) {
            path.push(reps[id]);
            last = Some(id);
        }
    }
    if path.len() < 2 {
        return Err(ExtrudeError::Degenerate(
            "the line has fewer than two distinct vertices",
        ));
    }
    let (base, top) = place(&path, &resolved)?;
    let quads: Vec<Polygon3D> = (0..path.len() - 1)
        .filter_map(|i| {
            let (a, b, at, bt) = (base[i], base[i + 1], top[i], top[i + 1]);
            // A segment parallel to the sweep, or one placement collapsed
            // (flattening a vertical step), has no wall: drop its quad.
            let area = norm(cross(sub(b, a), sub(at, a)));
            (area > tol.xy * norm(sub(b, a)).max(1.0)).then(|| {
                Polygon3D::from_rings(
                    frame.clone(),
                    [a, b, bt, at, a],
                    std::iter::empty::<Vec<_>>(),
                )
            })
        })
        .collect();
    if quads.is_empty() {
        return Err(ExtrudeError::Degenerate(
            "every segment is parallel to the extrusion",
        ));
    }
    let mesh = PolygonMesh3D::from_polygons(frame.clone(), quads.iter())
        .map_err(|_| ExtrudeError::Degenerate("the walls could not be assembled"))?;
    Ok(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::extrude::test_util::{up, vertical};
    use crate::ops::extrude::{BaseMode, ExtrudeDirection, ExtrudeOutput, TopMode};

    const F: CoordinateFrame = CoordinateFrame::Euclidean;

    #[test]
    fn a_point_becomes_a_vertical_segment() {
        let g = sweep_point(&F, [1.0, 2.0, 3.0], &up(10.0)).unwrap();
        let Euclidean3DGeometry::LineString(l) = g else {
            panic!("{g:?}")
        };
        assert_eq!(l.coords(), &[[1.0, 2.0, 3.0], [1.0, 2.0, 13.0]]);
    }

    #[test]
    fn a_point_honours_a_fixed_base() {
        let spec = vertical(BaseMode::FixedElevation(0.0), TopMode::ToElevation(5.0));
        let Euclidean3DGeometry::LineString(l) = sweep_point(&F, [1.0, 2.0, 3.0], &spec).unwrap()
        else {
            panic!()
        };
        assert_eq!(l.coords(), &[[1.0, 2.0, 0.0], [1.0, 2.0, 5.0]]);
    }

    #[test]
    fn a_point_has_no_surface_normal() {
        let spec = ExtrudeSpec {
            direction: ExtrudeDirection::SurfaceNormal { distance: 1.0 },
            output: ExtrudeOutput::Solid,
        };
        assert!(matches!(
            sweep_point(&F, [0.0; 3], &spec),
            Err(ExtrudeError::UnsupportedType(_))
        ));
    }

    #[test]
    fn a_line_becomes_one_wall_quad_per_segment() {
        let coords = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]];
        let Euclidean3DGeometry::PolygonMesh(m) = sweep_line(&F, &coords, &up(3.0)).unwrap() else {
            panic!()
        };
        assert_eq!(m.num_faces(), 2);
    }

    #[test]
    fn a_vertical_segment_drops_its_zero_area_quad() {
        let coords = [[0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1.0, 0.0, 5.0]];
        let Euclidean3DGeometry::PolygonMesh(m) = sweep_line(&F, &coords, &up(1.0)).unwrap() else {
            panic!()
        };
        assert_eq!(m.num_faces(), 1);
    }

    #[test]
    fn flattening_a_vertical_step_drops_the_collapsed_quad() {
        // The step's two vertices land on one base point: that quad has no
        // area and is dropped, like any segment parallel to the sweep.
        let coords = [[0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1.0, 0.0, 5.0]];
        let spec = vertical(BaseMode::FlattenToLowest, TopMode::ByHeight(1.0));
        let Euclidean3DGeometry::PolygonMesh(m) = sweep_line(&F, &coords, &spec).unwrap() else {
            panic!()
        };
        assert_eq!(m.num_faces(), 1);
    }

    #[test]
    fn a_line_parallel_to_the_sweep_is_degenerate() {
        let coords = [[0.0, 0.0, 0.0], [0.0, 0.0, 5.0]];
        assert!(matches!(
            sweep_line(&F, &coords, &up(1.0)),
            Err(ExtrudeError::Degenerate(_))
        ));
    }

    #[test]
    fn repeated_vertices_are_ignored() {
        let coords = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let Euclidean3DGeometry::PolygonMesh(m) = sweep_line(&F, &coords, &up(1.0)).unwrap() else {
            panic!()
        };
        assert_eq!(m.num_faces(), 1);
    }
}

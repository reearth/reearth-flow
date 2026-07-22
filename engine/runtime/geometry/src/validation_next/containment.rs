//! Hole-in-exterior containment: the shared detectors behind
//! [`InteriorRingContainment`](super::ValidationType::InteriorRingContainment).

use super::measure::newell_vector_3d;
use super::{open_ring, ValidationReport};
use crate::coordinate::CoordinateFrame;
use crate::line_string::{LineString2D, LineString3D};
use crate::predicates::kernel::{coord_pos_relative_to_ring, CoordPos};
use crate::predicates::kernel3d::drop_axis;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// The position of a hole's representative point relative to the exterior
/// ring: the first hole vertex not on the exterior boundary, then the first
/// hole edge midpoint not on it, `None` when every probe sits on the boundary.
fn hole_probe_position(exterior: &[[f64; 2]], hole: &[[f64; 2]]) -> Option<CoordPos> {
    for &v in open_ring(hole) {
        match coord_pos_relative_to_ring(v, exterior) {
            CoordPos::OnBoundary => {}
            position => return Some(position),
        }
    }
    for w in hole.windows(2) {
        let midpoint = [(w[0][0] + w[1][0]) / 2.0, (w[0][1] + w[1][1]) / 2.0];
        match coord_pos_relative_to_ring(midpoint, exterior) {
            CoordPos::OnBoundary => {}
            position => return Some(position),
        }
    }
    None
}

/// Report an
/// [`InteriorRingContainment`](super::ValidationType::InteriorRingContainment)
/// problem for each hole whose representative point lies outside the exterior
/// ring; a hole whose every probe sits on the exterior boundary is treated as
/// contained. The position is the offending hole ring as a LineString. Rings
/// must be stored closed with finite coordinates, and the face's rings must
/// not cross (the check depends on
/// [`SelfIntersection`](super::ValidationType::SelfIntersection)).
pub(crate) fn check_holes_in_exterior_2d<'a>(
    frame: &CoordinateFrame,
    exterior: &[[f64; 2]],
    holes: impl IntoIterator<Item = &'a [[f64; 2]]>,
    report: &mut ValidationReport,
) {
    for hole in holes {
        if hole_probe_position(exterior, hole) == Some(CoordPos::Outside) {
            report.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
                LineString2D::from_coords(frame.clone(), hole.iter().copied()),
            )));
        }
    }
}

/// Report each hole whose representative point lies outside the exterior ring,
/// with the rings projected to 2D by dropping the dominant axis of the
/// exterior's Newell vector. A degenerate exterior (zero Newell vector) skips
/// the check. The position is the offending 3D hole ring as a LineString.
pub(crate) fn check_holes_in_exterior_3d<'a>(
    frame: &CoordinateFrame,
    exterior: &[[f64; 3]],
    holes: impl IntoIterator<Item = &'a [[f64; 3]]>,
    report: &mut ValidationReport,
) {
    let normal = newell_vector_3d(exterior);
    if normal == [0.0; 3] {
        return;
    }
    let axis = (0..3)
        .max_by(|&i, &j| normal[i].abs().total_cmp(&normal[j].abs()))
        .expect("three components");
    let project =
        |ring: &[[f64; 3]]| -> Vec<[f64; 2]> { ring.iter().map(|&p| drop_axis(p, axis)).collect() };
    let exterior_2d = project(exterior);
    for hole in holes {
        if hole_probe_position(&exterior_2d, &project(hole)) == Some(CoordPos::Outside) {
            report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
                LineString3D::from_coords(frame.clone(), hole.iter().copied()),
            )));
        }
    }
}

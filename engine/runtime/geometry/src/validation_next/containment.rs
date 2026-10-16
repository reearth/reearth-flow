//! Hole-in-exterior containment: the shared detectors behind
//! [`InteriorRingContainment`](super::ValidationType::InteriorRingContainment).

use std::cell::OnceCell;

use super::face_orientation::{dominant_axis, OrientationFrame};
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
/// with the rings projected to 2D by dropping the exterior's
/// [`dominant_axis`]. In a frame not in linear units (x and y degrees, z
/// metres) that axis is judged in local metres, so a draped footprint drops z
/// and a wall, upright or leaning, drops a horizontal axis. Only the discrete
/// axis comes from there: the inside/outside test itself runs on the exact
/// stored coordinates with that axis dropped. A degenerate exterior skips the
/// check. The position is the offending 3D hole ring as a LineString.
pub(crate) fn check_holes_in_exterior_3d<'a>(
    frame: &CoordinateFrame,
    exterior: &[[f64; 3]],
    holes: impl IntoIterator<Item = &'a [[f64; 3]]>,
    report: &mut ValidationReport,
) {
    check_holes_in_exterior_3d_with(frame, &OnceCell::new(), exterior, holes, report);
}

/// As [`check_holes_in_exterior_3d`], with the frame's [`OrientationFrame`]
/// held in `orientation` so a caller checking many faces of one frame (a
/// mesh) resolves it at most once rather than per holed face. It is filled
/// lazily, only when a face actually needs it.
pub(crate) fn check_holes_in_exterior_3d_with<'a>(
    frame: &CoordinateFrame,
    orientation: &OnceCell<OrientationFrame>,
    exterior: &[[f64; 3]],
    holes: impl IntoIterator<Item = &'a [[f64; 3]]>,
    report: &mut ValidationReport,
) {
    let mut holes = holes.into_iter().peekable();
    if holes.peek().is_none() {
        return;
    }
    // An exactly zero Newell vector is degenerate in any frame; checking it
    // first spares a degenerate face the PROJ lookup.
    if newell_vector_3d(exterior) == [0.0; 3] {
        return;
    }
    let orientation = *orientation.get_or_init(|| OrientationFrame::of(frame));
    let Some(axis) = dominant_axis(orientation, exterior) else {
        return;
    };
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

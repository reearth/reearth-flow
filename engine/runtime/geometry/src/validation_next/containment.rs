//! Hole-in-exterior containment: the shared detectors behind
//! [`InteriorRingContainment`](super::ValidationType::InteriorRingContainment).

use std::cell::OnceCell;

use super::face_orientation::{dominant_axis, OrientationFrame};
use super::measure::newell_vector_3d;
use super::{open_ring, IssuePart, ValidationReport};
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
/// problem for each hole that does not lie in the face's area: one whose
/// representative point lies outside the exterior ring, attributed to the hole,
/// or one that lies inside another hole, attributed to the hole pair. A hole
/// whose every probe sits on the other ring's boundary is treated as not
/// beyond it. The position is the offending hole ring as a LineString. Rings
/// must be stored closed with finite coordinates, and the face's rings must
/// not cross (the check depends on
/// [`SelfIntersection`](super::ValidationType::SelfIntersection)).
pub(crate) fn check_holes_in_exterior_2d<'a>(
    frame: &CoordinateFrame,
    exterior: &[[f64; 2]],
    holes: impl IntoIterator<Item = &'a [[f64; 2]]>,
    report: &mut ValidationReport,
) {
    let holes: Vec<&[[f64; 2]]> = holes.into_iter().collect();
    report_misplaced_holes(exterior, &holes, report, |k| {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            frame.clone(),
            holes[k].iter().copied(),
        )))
    });
}

/// Report each hole outside `exterior` or inside another hole, as
/// [`check_holes_in_exterior_2d`] describes, for rings already in one plane.
/// `position(k)` is the reported position of hole `k`.
fn report_misplaced_holes(
    exterior: &[[f64; 2]],
    holes: &[&[[f64; 2]]],
    report: &mut ValidationReport,
    position: impl Fn(usize) -> Geometry,
) {
    for (k, hole) in holes.iter().enumerate() {
        if hole_probe_position(exterior, hole) == Some(CoordPos::Outside) {
            report.record_part(IssuePart::Ring(k + 1), |r| r.push(position(k)));
        }
    }
    for (outer, outer_ring) in holes.iter().enumerate() {
        for (inner, inner_ring) in holes.iter().enumerate() {
            if outer != inner
                && hole_probe_position(outer_ring, inner_ring) == Some(CoordPos::Inside)
            {
                let pair = IssuePart::RingPair(outer.min(inner) + 1, outer.max(inner) + 1);
                report.record_part(pair, |r| r.push(position(inner)));
            }
        }
    }
}

/// Report each hole outside the exterior ring or inside another hole, as
/// [`check_holes_in_exterior_2d`] does, with the rings projected to 2D by
/// dropping the exterior's
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
    let holes: Vec<&[[f64; 3]]> = holes.collect();
    let holes_2d: Vec<Vec<[f64; 2]>> = holes.iter().map(|hole| project(hole)).collect();
    let holes_2d: Vec<&[[f64; 2]]> = holes_2d.iter().map(Vec::as_slice).collect();
    report_misplaced_holes(&exterior_2d, &holes_2d, report, |k| {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            frame.clone(),
            holes[k].iter().copied(),
        )))
    });
}

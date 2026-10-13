//! Unit-consistent ring normals for hole orientation and containment.
//!
//! In a frame not in linear units, x and y are degrees and z is metres, so a
//! normal computed from raw coordinates mixes units: on a draped footprint the
//! horizontal components (degrees times metres) swamp the vertical one
//! (degrees squared), and on a steep wall the reverse. The helpers here first
//! map a ring into local, roughly metric coordinates, so every face
//! orientation (flat, sloped or vertical) is judged by one rule.

use std::borrow::Cow;

use super::measure::newell_vector_3d;
use super::open_ring;
use crate::algorithm::geo_distance_converter::METRES_PER_DEGREE;

/// How far from perpendicular two unit normals must be before their
/// agreement counts: a hole agrees with (winds like) its exterior when the
/// cosine of the angle between their normals exceeds this, opposes it below
/// its negation, and is undecidable in between.
///
/// A hole lies in its face's plane, so a sound one sits at a cosine near -1
/// and a same-wound one near +1; millimetre noise on a metre-scale ring moves
/// that by about 1e-3. Only a hole tilted more than about 84 degrees off its
/// face lands in the undecidable band, and such a pair has no meaningful
/// relative winding (planarity reports it), so it is left unflagged rather
/// than guessed.
pub(crate) const COS_TOL: f64 = 0.1;

/// `ring` in local, roughly metric coordinates: unchanged when `degrees` is
/// false (a frame in linear units, so metric results stay bit-for-bit as
/// they were); otherwise relative to its first point, with both horizontal
/// axes times [`METRES_PER_DEGREE`] and z kept in metres.
///
/// One factor on both horizontal axes needs no axis-order lookup, and as a
/// positive scale per axis it never flips a winding or moves a point across
/// a ring. It ignores the cos(latitude) shrink of a degree of longitude, which
/// only skews the local frame by a bounded amount and does not change which
/// way a ring winds relative to another in the same plane.
///
/// Callers pass `degrees` as "the frame is not definitely in linear units", so
/// an [`Undeterminable`](crate::coordinate::UnitKind::Undeterminable) frame is
/// deliberately treated as degrees, matching the extrude op. The narrow
/// exposure is a metric frame with an EPSG code PROJ cannot classify: its
/// horizontal axes get scaled by about 1.1e5 against z. That skews the
/// frame but, being positive per axis, still never flips a coplanar hole's
/// winding relative to its exterior.
pub(crate) fn local_metres(ring: &[[f64; 3]], degrees: bool) -> Cow<'_, [[f64; 3]]> {
    match ring.first() {
        Some(&[x0, y0, z0]) if degrees => Cow::Owned(
            ring.iter()
                .map(|&[x, y, z]| {
                    [
                        (x - x0) * METRES_PER_DEGREE,
                        (y - y0) * METRES_PER_DEGREE,
                        z - z0,
                    ]
                })
                .collect(),
        ),
        _ => Cow::Borrowed(ring),
    }
}

/// The right-hand-rule unit normal of a planar 3D ring in its stored
/// coordinates, `None` if degenerate (collinear or zero-area).
///
/// Degeneracy must be judged here, before [`local_metres`]: the rounding-noise
/// floor of [`normal`](crate::ops::triangulation::normal) scales with the
/// ring's coordinate magnitude, which the shift to a local origin removes, so
/// a collinear ring in degrees would otherwise get an arbitrary normal.
fn stored_normal(ring: &[[f64; 3]]) -> Option<[f64; 3]> {
    crate::ops::triangulation::normal(open_ring(ring))
}

/// Right-hand-rule unit normal of a planar 3D ring in local metres (see
/// [`local_metres`]), `None` if the ring is degenerate in its stored
/// coordinates.
pub(crate) fn unit_normal(ring: &[[f64; 3]], degrees: bool) -> Option<[f64; 3]> {
    let stored = stored_normal(ring)?;
    if !degrees {
        return Some(stored);
    }
    crate::ops::triangulation::normal(open_ring(&local_metres(ring, true)))
}

/// Newell vector of a 3D ring in local metres (see [`local_metres`]), `None`
/// if the ring is degenerate: an exactly zero vector in a frame in linear
/// units (as before), or no normal in its stored coordinates otherwise (see
/// [`stored_normal`]).
pub(crate) fn newell_vector(ring: &[[f64; 3]], degrees: bool) -> Option<[f64; 3]> {
    if !degrees {
        let n = newell_vector_3d(ring);
        return (n != [0.0; 3]).then_some(n);
    }
    stored_normal(ring)?;
    Some(newell_vector_3d(&local_metres(ring, true)))
}

/// Whether a hole whose unit normal is `hole` winds like its exterior, whose
/// unit normal is `exterior`, both in local metres: true only when the cosine
/// between them exceeds [`COS_TOL`]. An undecidable pair is not flagged.
pub(crate) fn hole_agrees(exterior: [f64; 3], hole: [f64; 3]) -> bool {
    exterior[0] * hole[0] + exterior[1] * hole[1] + exterior[2] * hole[2] > COS_TOL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_metric_ring_is_left_as_it_is() {
        let ring = [[1.0e5, 2.0e5, 3.0], [1.0e5 + 1.0, 2.0e5, 3.0]];
        assert!(matches!(local_metres(&ring, false), Cow::Borrowed(r) if r == ring));
    }

    #[test]
    fn a_degree_ring_is_scaled_about_its_first_point() {
        let ring = [[35.68, 139.76, 10.0], [35.68 + 1e-5, 139.76 + 2e-5, 12.0]];
        let m = local_metres(&ring, true);
        assert_eq!(m[0], [0.0, 0.0, 0.0]);
        assert!((m[1][0] - 1e-5 * METRES_PER_DEGREE).abs() < 1e-6, "{m:?}");
        assert!((m[1][1] - 2e-5 * METRES_PER_DEGREE).abs() < 1e-6, "{m:?}");
        assert_eq!(m[1][2], 2.0);
    }

    #[test]
    fn a_degree_is_about_111_km() {
        assert!((METRES_PER_DEGREE - 111_319.49).abs() < 0.01);
    }

    #[test]
    fn a_hole_agrees_only_beyond_the_tolerance() {
        let up = [0.0, 0.0, 1.0];
        assert!(hole_agrees(up, up));
        assert!(!hole_agrees(up, [0.0, 0.0, -1.0]));
        // Within about 84 degrees of perpendicular, either way: undecidable.
        let (s, c) = 84.5f64.to_radians().sin_cos();
        assert!(!hole_agrees(up, [s, 0.0, c]));
        assert!(!hole_agrees(up, [s, 0.0, -c]));
        let (s, c) = 80.0f64.to_radians().sin_cos();
        assert!(hole_agrees(up, [s, 0.0, c]));
    }
}

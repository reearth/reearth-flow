//! Face orientation: which way a hole winds relative to its face's exterior,
//! and which axis a face is best viewed along.
//!
//! In a frame not in linear units, x and y are degrees and z is metres, so a
//! normal computed from raw coordinates mixes units: on a draped footprint the
//! horizontal components (degrees times metres) swamp the vertical one
//! (degrees squared), and on a steep wall the reverse. This module first maps
//! a ring into local, roughly metric coordinates, so every face orientation
//! (flat, sloped or vertical) is judged by one rule.
//!
//! That conversion is approximate, and stays private to this module: its
//! results are only ever used for binary, direction-level decisions (whether a
//! hole winds with or against its exterior, which axis a face's normal is
//! closest to), where a small skew of the frame cannot change the answer away
//! from a tolerance band. Predicates that decide on which side of a boundary a
//! point lies (containment, intersection, touching) need exactness right at
//! that boundary, so they keep working in the exact stored coordinates and
//! never see metres; containment only takes its projection axis from here.

use std::borrow::Cow;

use super::measure::newell_vector_3d;
use super::open_ring;
use crate::coordinate::{CoordinateFrame, UnitKind};

/// Metres per degree of arc on a sphere of the WGS84 semi-major axis.
const METRES_PER_DEGREE: f64 = 6_378_137.0 * (std::f64::consts::PI / 180.0);

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
const COS_TOL: f64 = 0.1;

/// How a frame's coordinates map to local metres for orientation decisions,
/// resolved once per frame (it may need a PROJ lookup) and then reused for
/// every ring in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OrientationFrame {
    /// Whether the frame is not definitely in linear units.
    degrees: bool,
}

impl OrientationFrame {
    /// The orientation frame of `frame`. A frame not definitely in linear
    /// units, including an
    /// [`Undeterminable`](crate::coordinate::UnitKind::Undeterminable) one, is
    /// treated as degrees, matching the extrude op. The narrow exposure is a
    /// metric frame with an EPSG code PROJ cannot classify: its horizontal
    /// axes get scaled by about 1.1e5 against z, which skews the frame but,
    /// being positive per axis, never flips a coplanar hole's winding
    /// relative to its exterior.
    pub(crate) fn of(frame: &CoordinateFrame) -> Self {
        Self {
            degrees: !matches!(frame.unit_kind(), UnitKind::Linear),
        }
    }
}

/// `ring` in local, roughly metric coordinates: unchanged in a frame in
/// linear units (so metric results stay bit-for-bit as they were); otherwise
/// relative to its first point, with both horizontal axes times
/// [`METRES_PER_DEGREE`] and z kept in metres.
///
/// A positive scale per axis never flips a winding or moves a point across a
/// ring.
fn local_metres(frame: OrientationFrame, ring: &[[f64; 3]]) -> Cow<'_, [[f64; 3]]> {
    match ring.first() {
        Some(&[x0, y0, z0]) if frame.degrees => Cow::Owned(
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

/// The right-hand-rule unit normal of a planar 3D ring in local metres, for a
/// caller that needs the face's direction; `None` if the ring is degenerate
/// in its stored coordinates.
pub(crate) fn unit_normal(frame: OrientationFrame, ring: &[[f64; 3]]) -> Option<[f64; 3]> {
    let stored = stored_normal(ring)?;
    if !frame.degrees {
        return Some(stored);
    }
    crate::ops::triangulation::normal(open_ring(&local_metres(frame, ring)))
}

/// A face's exterior, judged once so each of its holes can be compared with
/// it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ExteriorOrientation {
    frame: OrientationFrame,
    normal: [f64; 3],
}

impl ExteriorOrientation {
    /// The orientation of the exterior ring `exterior`, `None` when it has
    /// none to compare holes with (see [`unit_normal`]).
    pub(crate) fn new(frame: OrientationFrame, exterior: &[[f64; 3]]) -> Option<Self> {
        Some(Self {
            frame,
            normal: unit_normal(frame, exterior)?,
        })
    }

    /// How `hole` winds relative to this exterior: `Some(true)` when it winds
    /// the same way (a misoriented hole), `Some(false)` when it winds
    /// opposite (as it should), and `None` when that is undecidable: the hole
    /// is degenerate, or its normal is within [`COS_TOL`] of perpendicular to
    /// the exterior's.
    pub(crate) fn hole_winding(&self, hole: &[[f64; 3]]) -> Option<bool> {
        let h = unit_normal(self.frame, hole)?;
        let n = self.normal;
        let cos = n[0] * h[0] + n[1] * h[1] + n[2] * h[2];
        if cos > COS_TOL {
            Some(true)
        } else if cos < -COS_TOL {
            Some(false)
        } else {
            None
        }
    }
}

/// The axis (0, 1 or 2) along which a planar ring's Newell vector in local
/// metres is longest: dropping it projects the ring to 2D with the least
/// distortion, so a draped footprint drops z and a wall, upright or leaning,
/// drops a horizontal axis. Only the axis is taken from metres; a caller
/// projects the exact stored coordinates along it. `None` if the ring is
/// degenerate: an exactly zero Newell vector in a frame in linear units, or
/// no normal in its stored coordinates otherwise (see [`stored_normal`]).
pub(crate) fn dominant_axis(frame: OrientationFrame, ring: &[[f64; 3]]) -> Option<usize> {
    let n = if frame.degrees {
        stored_normal(ring)?;
        newell_vector_3d(&local_metres(frame, ring))
    } else {
        let n = newell_vector_3d(ring);
        if n == [0.0; 3] {
            return None;
        }
        n
    };
    (0..3).max_by(|&i, &j| n[i].abs().total_cmp(&n[j].abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINEAR: OrientationFrame = OrientationFrame { degrees: false };
    const DEGREES: OrientationFrame = OrientationFrame { degrees: true };

    #[test]
    fn a_metric_ring_is_left_as_it_is() {
        let ring = [[1.0e5, 2.0e5, 3.0], [1.0e5 + 1.0, 2.0e5, 3.0]];
        assert!(matches!(local_metres(LINEAR, &ring), Cow::Borrowed(r) if r == ring));
    }

    #[test]
    fn a_degree_ring_is_scaled_about_its_first_point() {
        let ring = [[35.68, 139.76, 10.0], [35.68 + 1e-5, 139.76 + 2e-5, 12.0]];
        let m = local_metres(DEGREES, &ring);
        assert_eq!(m[0], [0.0, 0.0, 0.0]);
        assert!((m[1][0] - 1e-5 * METRES_PER_DEGREE).abs() < 1e-6, "{m:?}");
        assert!((m[1][1] - 2e-5 * METRES_PER_DEGREE).abs() < 1e-6, "{m:?}");
        assert_eq!(m[1][2], 2.0);
    }

    #[test]
    fn a_degree_is_about_111_km() {
        assert!((METRES_PER_DEGREE - 111_319.49).abs() < 0.01);
    }

    /// A closed square ring in the z = 0 plane whose unit normal is `+z`, or
    /// `-z` when `reversed`.
    fn square(reversed: bool) -> Vec<[f64; 3]> {
        let mut ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        if reversed {
            ring.reverse();
        }
        ring
    }

    /// `square(false)` turned `degrees` about the x axis.
    fn tilted(degrees: f64) -> Vec<[f64; 3]> {
        let (s, c) = degrees.to_radians().sin_cos();
        square(false)
            .into_iter()
            .map(|[x, y, z]| [x, y * c - z * s, y * s + z * c])
            .collect()
    }

    #[test]
    fn a_hole_winding_is_decided_only_beyond_the_tolerance() {
        let ext = ExteriorOrientation::new(LINEAR, &square(false)).unwrap();
        assert_eq!(ext.hole_winding(&square(false)), Some(true));
        assert_eq!(ext.hole_winding(&square(true)), Some(false));
        // Within about 84 degrees of perpendicular, either way: undecidable.
        assert_eq!(ext.hole_winding(&tilted(84.5)), None);
        assert_eq!(ext.hole_winding(&tilted(180.0 - 84.5)), None);
        assert_eq!(ext.hole_winding(&tilted(80.0)), Some(true));
        assert_eq!(ext.hole_winding(&tilted(100.0)), Some(false));
    }

    #[test]
    fn the_dominant_axis_follows_the_largest_normal_component() {
        assert_eq!(dominant_axis(LINEAR, &square(false)), Some(2));
        assert_eq!(dominant_axis(LINEAR, &tilted(80.0)), Some(1));
        let line = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]];
        assert_eq!(dominant_axis(LINEAR, &line), None);
    }
}

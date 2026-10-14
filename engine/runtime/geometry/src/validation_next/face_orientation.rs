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

/// Metres per degree of arc on a sphere of the WGS84 semi-major axis: a
/// degree of latitude, and a degree of longitude at the equator.
const METRES_PER_DEGREE: f64 = 6_378_137.0 * (std::f64::consts::PI / 180.0);

/// The highest absolute latitude, in degrees, at which a ring in a
/// geographic frame is still judged. A degree of longitude shrinks to
/// cos(latitude) of a degree of latitude, and that factor's relative change
/// across a ring, tan(latitude) per radian of latitude, grows without bound
/// toward a pole: a single local scale stops describing the ring, and
/// meridians converge so fast that a ring can wrap the pole within a few
/// kilometres. Within a degree of a pole the factor is under 0.018 and its
/// relative rate over 57 per radian, so orientation there is left
/// undecided rather than guessed.
const MAX_ABS_LATITUDE: f64 = 89.0;

/// The widest longitude span, in degrees, of a ring still judged in a
/// geographic frame: about 111 km at the equator and less poleward, far
/// beyond any single building face, while a ring crossing the antimeridian
/// (stored longitudes jumping by about 360) or circling a pole (visiting
/// every longitude) exceeds it and is left undecided. It also bounds the
/// ring's extent so one local origin and one scale describe it.
const MAX_LONGITUDE_SPAN: f64 = 1.0;

/// The widest latitude span, in degrees, of a ring still judged in a
/// geographic frame: within it cos(latitude) varies by at most about
/// tan(latitude) * 1.7% relative, so the scale taken at the ring's origin
/// holds across the ring (about 3% at 60 degrees, and about 100% for a ring
/// of that full size at the [`MAX_ABS_LATITUDE`] limit, though a real face is
/// far smaller and its variation proportionally less).
const MAX_LATITUDE_SPAN: f64 = 1.0;

/// The smallest longitude scale factor, standing in for cos(latitude) where
/// that reaches zero or goes negative (at or past a pole, or from a
/// non-finite latitude), so a local map stays finite and positive per axis.
/// It is cos(latitude) about 0.2 arc-seconds from a pole.
const MIN_COS_LATITUDE: f64 = 1e-6;

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
/// resolved once per frame (it may need PROJ lookups) and then reused for
/// every ring in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OrientationFrame {
    metric: Metric,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Metric {
    /// A frame in linear units: coordinates are already metric.
    Linear,
    /// A geographic frame in degrees, with z in metres: stored coordinate
    /// `north` (0 or 1) holds latitude and the other horizontal one
    /// longitude.
    Geographic { north: usize },
    /// A frame not known to be linear whose latitude axis is unknown: both
    /// horizontal axes are taken as degrees of latitude.
    Uniform,
}

impl OrientationFrame {
    /// The orientation frame of `frame`.
    ///
    /// A geographic CRS has its latitude axis read from the CRS's declared
    /// axis directions (memoized per EPSG code), so a (lat, lon) frame such
    /// as EPSG:6668 and a (lon, lat) one shrink the right axis.
    ///
    /// Fallback: when that lookup fails, or the frame's units are
    /// [`Undeterminable`](crate::coordinate::UnitKind::Undeterminable), the
    /// frame is still treated as degrees but with one scale on both
    /// horizontal axes, no cos(latitude) and no latitude or span limits,
    /// since which axis is latitude (or whether the units are degrees at all)
    /// is unknown. That skews the local frame by up to 1/cos(latitude) along
    /// one axis, and for a metric frame PROJ cannot classify, scales its
    /// horizontal axes by about 1.1e5 against z; being positive per axis, it
    /// never flips a coplanar hole's winding relative to its exterior.
    pub(crate) fn of(frame: &CoordinateFrame) -> Self {
        let metric = match frame.unit_kind() {
            UnitKind::Linear => Metric::Linear,
            UnitKind::Angular => match frame {
                CoordinateFrame::Crs(epsg) => match crate::ops::north_axis_index(*epsg) {
                    Ok(north) if north < 2 => Metric::Geographic { north },
                    _ => Metric::Uniform,
                },
                _ => Metric::Uniform,
            },
            UnitKind::Undeterminable(_) => Metric::Uniform,
        };
        Self { metric }
    }
}

/// An affine map of stored coordinates to local metres: subtract `origin`,
/// then multiply each axis by its positive `scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct LocalMetres {
    origin: [f64; 3],
    scale: [f64; 3],
}

impl LocalMetres {
    /// A geographic frame's map about `origin`, whose coordinate `north`
    /// holds latitude: latitude times [`METRES_PER_DEGREE`], longitude times
    /// that and cos(latitude) at `origin`, floored at [`MIN_COS_LATITUDE`].
    fn geographic(north: usize, origin: [f64; 3]) -> Self {
        let mut scale = [METRES_PER_DEGREE, METRES_PER_DEGREE, 1.0];
        scale[1 - north] *= origin[north].to_radians().cos().max(MIN_COS_LATITUDE);
        Self { origin, scale }
    }

    /// Both horizontal axes as degrees of latitude, about `origin`.
    fn uniform(origin: [f64; 3]) -> Self {
        Self {
            origin,
            scale: [METRES_PER_DEGREE, METRES_PER_DEGREE, 1.0],
        }
    }

    fn apply<'a>(&self, ring: &'a [[f64; 3]]) -> Cow<'a, [[f64; 3]]> {
        let (o, k) = (self.origin, self.scale);
        Cow::Owned(
            ring.iter()
                .map(|&[x, y, z]| [(x - o[0]) * k[0], (y - o[1]) * k[1], (z - o[2]) * k[2]])
                .collect(),
        )
    }
}

/// How a ring maps to local metres.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Local {
    /// Already metric: used as stored, so metric results stay bit-for-bit
    /// as they were.
    Identity,
    Scaled(LocalMetres),
    /// Beyond the latitude or span limits: no meaningful local metric.
    Undecidable,
}

impl Local {
    fn apply<'a>(&self, ring: &'a [[f64; 3]]) -> Option<Cow<'a, [[f64; 3]]>> {
        match self {
            Local::Identity => Some(Cow::Borrowed(ring)),
            Local::Scaled(map) => Some(map.apply(ring)),
            Local::Undecidable => None,
        }
    }
}

/// Whether a ring in a geographic frame with latitude at coordinate `north`
/// lies within [`MAX_ABS_LATITUDE`], [`MAX_LATITUDE_SPAN`] and
/// [`MAX_LONGITUDE_SPAN`]. A ring with any non-finite horizontal
/// coordinate is not.
fn within_limits(north: usize, ring: &[[f64; 3]]) -> bool {
    let east = 1 - north;
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for p in ring {
        for k in 0..2 {
            if !p[k].is_finite() {
                return false;
            }
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    lo[north].abs().max(hi[north].abs()) <= MAX_ABS_LATITUDE
        && hi[north] - lo[north] <= MAX_LATITUDE_SPAN
        && hi[east] - lo[east] <= MAX_LONGITUDE_SPAN
}

/// The local-metres map for `ring` in `frame`, about its first point: the
/// identity in linear units; in a geographic frame, latitude times
/// [`METRES_PER_DEGREE`] and longitude times that and cos(latitude) at the
/// first point, or [`Local::Undecidable`] beyond the limits (see
/// [`within_limits`]); otherwise [`LocalMetres::uniform`]. z stays metres.
/// Every scale is positive, so it never flips a winding.
fn local_metres(frame: OrientationFrame, ring: &[[f64; 3]]) -> Local {
    let Some(&origin) = ring.first() else {
        return Local::Identity;
    };
    match frame.metric {
        Metric::Linear => Local::Identity,
        Metric::Uniform => Local::Scaled(LocalMetres::uniform(origin)),
        Metric::Geographic { north } => {
            if !within_limits(north, ring) {
                return Local::Undecidable;
            }
            Local::Scaled(LocalMetres::geographic(north, origin))
        }
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

/// The unit normal of `ring` mapped by `local`, `None` when the ring is
/// degenerate in its stored coordinates or `local` is undecidable.
fn normal_in(local: Local, ring: &[[f64; 3]]) -> Option<[f64; 3]> {
    let stored = stored_normal(ring)?;
    match local {
        Local::Identity => Some(stored),
        _ => crate::ops::triangulation::normal(open_ring(&local.apply(ring)?)),
    }
}

/// The right-hand-rule unit normal of a planar 3D ring in local metres, for a
/// caller that needs the face's direction; `None` if the ring is degenerate
/// in its stored coordinates or, in a geographic frame, beyond the latitude
/// or span limits. Only tests need a bare direction today; production
/// callers ask [`ExteriorOrientation`] instead.
#[cfg(test)]
pub(crate) fn unit_normal(frame: OrientationFrame, ring: &[[f64; 3]]) -> Option<[f64; 3]> {
    normal_in(local_metres(frame, ring), ring)
}

/// A face's exterior, judged once so each of its holes can be compared with
/// it in the same local metres.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ExteriorOrientation {
    frame: OrientationFrame,
    local: Local,
    normal: [f64; 3],
}

impl ExteriorOrientation {
    /// The orientation of the exterior ring `exterior`, `None` when it has
    /// none to compare holes with: it is degenerate in its stored coordinates
    /// (see [`stored_normal`]) or, in a geographic frame, beyond the latitude
    /// or span limits (see [`within_limits`]).
    pub(crate) fn new(frame: OrientationFrame, exterior: &[[f64; 3]]) -> Option<Self> {
        let local = local_metres(frame, exterior);
        Some(Self {
            frame,
            local,
            normal: normal_in(local, exterior)?,
        })
    }

    /// How `hole` winds relative to this exterior: `Some(true)` when it winds
    /// the same way (a misoriented hole), `Some(false)` when it winds
    /// opposite (as it should), and `None` when that is undecidable: the hole
    /// is degenerate, lies beyond the geographic limits, or its normal is
    /// within [`COS_TOL`] of perpendicular to the exterior's. The hole is
    /// mapped to metres exactly as its exterior was.
    pub(crate) fn hole_winding(&self, hole: &[[f64; 3]]) -> Option<bool> {
        if let Metric::Geographic { north } = self.frame.metric {
            if !within_limits(north, hole) {
                return None;
            }
        }
        let h = normal_in(self.local, hole)?;
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
///
/// Unlike an orientation answer, a projection axis is always needed, so the
/// latitude and span limits do not apply here: a geographic ring always uses
/// [`LocalMetres::geographic`] about its first point, whose cos(latitude)
/// still describes a small ring near a pole (the limits exist because it
/// stops describing a large or wrapping one, where the stored rings are no
/// sound basis for a 2D containment test either).
pub(crate) fn dominant_axis(frame: OrientationFrame, ring: &[[f64; 3]]) -> Option<usize> {
    let &origin = ring.first()?;
    let map = match frame.metric {
        Metric::Linear => {
            let n = newell_vector_3d(ring);
            if n == [0.0; 3] {
                return None;
            }
            return (0..3).max_by(|&i, &j| n[i].abs().total_cmp(&n[j].abs()));
        }
        Metric::Geographic { north } => LocalMetres::geographic(north, origin),
        Metric::Uniform => LocalMetres::uniform(origin),
    };
    stored_normal(ring)?;
    let n = newell_vector_3d(&map.apply(ring));
    (0..3).max_by(|&i, &j| n[i].abs().total_cmp(&n[j].abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINEAR: OrientationFrame = OrientationFrame {
        metric: Metric::Linear,
    };
    const UNIFORM: OrientationFrame = OrientationFrame {
        metric: Metric::Uniform,
    };

    fn crs(code: u16) -> OrientationFrame {
        OrientationFrame::of(&CoordinateFrame::Crs(crate::coordinate::EpsgCode::new(
            code,
        )))
    }

    /// The mapped ring, panicking when undecidable.
    fn mapped(frame: OrientationFrame, ring: &[[f64; 3]]) -> Vec<[f64; 3]> {
        local_metres(frame, ring)
            .apply(ring)
            .expect("within the limits")
            .into_owned()
    }

    #[test]
    fn frames_resolve_their_metric() {
        assert_eq!(OrientationFrame::of(&CoordinateFrame::Euclidean), LINEAR);
        assert_eq!(crs(6677), LINEAR); // JGD2011 plane rectangular IX
        assert_eq!(crs(6668).metric, Metric::Geographic { north: 0 });
        assert_eq!(crs(6697).metric, Metric::Geographic { north: 0 });
        assert_eq!(crs(7084).metric, Metric::Geographic { north: 1 });
        // Unknown to PROJ: undeterminable units, the uniform fallback.
        assert_eq!(crs(1), UNIFORM);
    }

    #[test]
    fn a_metric_ring_is_left_as_it_is() {
        let ring = [[1.0e5, 2.0e5, 3.0], [1.0e5 + 1.0, 2.0e5, 3.0]];
        assert_eq!(local_metres(LINEAR, &ring), Local::Identity);
        assert!(matches!(Local::Identity.apply(&ring), Some(Cow::Borrowed(r)) if r == ring));
    }

    #[test]
    fn an_undeterminable_ring_is_scaled_uniformly_about_its_first_point() {
        let ring = [[35.68, 139.76, 10.0], [35.68 + 1e-5, 139.76 + 2e-5, 12.0]];
        let m = mapped(UNIFORM, &ring);
        assert_eq!(m[0], [0.0, 0.0, 0.0]);
        assert!((m[1][0] - 1e-5 * METRES_PER_DEGREE).abs() < 1e-6, "{m:?}");
        assert!((m[1][1] - 2e-5 * METRES_PER_DEGREE).abs() < 1e-6, "{m:?}");
        assert_eq!(m[1][2], 2.0);
    }

    #[test]
    fn longitude_shrinks_by_cos_latitude_on_the_stored_longitude_axis() {
        // One degree-step north and one east from (lat 60, lon 20), stored
        // (lat, lon) in EPSG:6668 and (lon, lat) in EPSG:7084: the east step
        // is half the north one in metres, on whichever axis holds longitude.
        let d = 1e-5;
        let lat_first = mapped(
            crs(6668),
            &[
                [60.0, 20.0, 0.0],
                [60.0 + d, 20.0, 0.0],
                [60.0, 20.0 + d, 0.0],
            ],
        );
        let lon_first = mapped(
            crs(7084),
            &[
                [20.0, 60.0, 0.0],
                [20.0, 60.0 + d, 0.0],
                [20.0 + d, 60.0, 0.0],
            ],
        );
        let (north, east) = (d * METRES_PER_DEGREE, d * METRES_PER_DEGREE * 0.5);
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(
            close(lat_first[1][0], north) && lat_first[1][1] == 0.0,
            "{lat_first:?}"
        );
        assert!(
            close(lat_first[2][1], east) && lat_first[2][0] == 0.0,
            "{lat_first:?}"
        );
        assert!(
            close(lon_first[1][1], north) && lon_first[1][0] == 0.0,
            "{lon_first:?}"
        );
        assert!(
            close(lon_first[2][0], east) && lon_first[2][1] == 0.0,
            "{lon_first:?}"
        );
    }

    #[test]
    fn a_ring_beyond_the_limits_is_undecidable() {
        let frame = crs(6668);
        let square = |lat: f64, lon: f64, dlat: f64, dlon: f64| {
            vec![
                [lat, lon, 0.0],
                [lat + dlat, lon, 0.0],
                [lat + dlat, lon + dlon, 0.0],
                [lat, lon + dlon, 0.0],
                [lat, lon, 0.0],
            ]
        };
        let small = 1e-4;
        assert!(unit_normal(frame, &square(88.9, 20.0, small, small)).is_some());
        assert!(unit_normal(frame, &square(-88.9, 20.0, -small, small)).is_some());
        assert!(unit_normal(frame, &square(89.0, 20.0, small, small)).is_none());
        assert!(unit_normal(frame, &square(-89.5, 20.0, small, small)).is_none());
        assert!(unit_normal(frame, &square(35.0, 20.0, 0.9, 0.9)).is_some());
        assert!(unit_normal(frame, &square(35.0, 20.0, 1.1, small)).is_none());
        assert!(unit_normal(frame, &square(35.0, 20.0, small, 1.1)).is_none());
        // Across the antimeridian: the stored longitudes span almost 360.
        assert!(unit_normal(frame, &square(35.0, 179.9999, small, -359.9998)).is_none());
        // The projection axis is still found there.
        assert_eq!(
            dominant_axis(frame, &square(89.5, 20.0, small, small)),
            Some(2)
        );
        assert_eq!(
            dominant_axis(frame, &square(35.0, 179.9999, small, -359.9998)),
            Some(2)
        );
    }

    #[test]
    fn a_ring_with_a_non_finite_coordinate_is_beyond_the_limits() {
        let ok = [
            [35.0, 20.0, 0.0],
            [35.0001, 20.0, 0.0],
            [35.0, 20.0001, 0.0],
        ];
        assert!(within_limits(0, &ok));
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for k in 0..2 {
                let mut ring = ok;
                ring[1][k] = bad;
                assert!(!within_limits(0, &ring), "{bad} at {k}");
                assert!(!within_limits(1, &ring), "{bad} at {k}");
            }
        }
    }

    #[test]
    fn the_longitude_scale_stays_positive_at_and_past_a_pole() {
        for lat in [90.0, -90.0, 90.5, f64::NAN] {
            let map = LocalMetres::geographic(0, [lat, 20.0, 0.0]);
            assert!(
                map.scale.iter().all(|k| k.is_finite() && *k > 0.0),
                "{lat}: {map:?}"
            );
        }
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

//! Tolerance welding: near-coincident vertices snap to one representative so
//! a shared edge is shared by identity, and caps built from both sides meet.

use std::collections::HashMap;

/// How close two positions must be to count as one. Horizontal and vertical
/// tolerances are separate so a large elevation never loosens the horizontal
/// weld: in degrees at 3000 m a shared tolerance would be about 30 cm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tolerance {
    /// Horizontal (x, y) tolerance.
    pub(super) xy: f64,
    /// Vertical (z) tolerance.
    pub(super) z: f64,
}

impl Tolerance {
    /// Whether `a` and `b` are within tolerance horizontally and vertically.
    pub(super) fn coincide(&self, a: [f64; 3], b: [f64; 3]) -> bool {
        let dxy2 = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2);
        dxy2 <= self.xy * self.xy && (a[2] - b[2]).abs() <= self.z
    }
}

/// The smallest vertical tolerance. Elevations are metres in every frame and
/// rarely large, so a purely relative tolerance (1e-8 m at 10 m) would keep
/// apart shared vertices whose z differs only by noise while the horizontal
/// tolerance at projected magnitudes is about a millimetre.
const MIN_Z_TOLERANCE: f64 = 1e-6;

/// `1e-9` of the largest horizontal and vertical coordinate magnitudes. The
/// horizontal tolerance is floored at `1e-9`, the vertical one at 1 micron
/// ([`MIN_Z_TOLERANCE`]). At projected magnitudes (1e6 m) the horizontal
/// tolerance is 1 mm; in degrees about 1.5 cm.
pub(super) fn weld_tolerance(points: &[[f64; 3]]) -> Tolerance {
    let (xy, z) = points.iter().fold((0.0f64, 0.0f64), |(xy, z), p| {
        (xy.max(p[0].abs()).max(p[1].abs()), z.max(p[2].abs()))
    });
    Tolerance {
        xy: 1e-9 * xy.max(1.0),
        z: (1e-9 * z.max(1.0)).max(MIN_Z_TOLERANCE),
    }
}

/// Give every point an id; a point within `tol` of an earlier representative
/// takes its id. Returns ids parallel to `points` and the representatives.
pub(super) fn weld(points: &[[f64; 3]], tol: Tolerance) -> (Vec<usize>, Vec<[f64; 3]>) {
    let cell = |c: f64, size: f64| (c / size).floor() as i64;
    let mut grid: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
    let mut reps: Vec<[f64; 3]> = Vec::new();
    let mut ids = Vec::with_capacity(points.len());
    for p in points {
        let k = [cell(p[0], tol.xy), cell(p[1], tol.xy), cell(p[2], tol.z)];
        let mut found = None;
        'search: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let Some(candidates) = grid.get(&[k[0] + dx, k[1] + dy, k[2] + dz]) else {
                        continue;
                    };
                    for &id in candidates {
                        if tol.coincide(reps[id], *p) {
                            found = Some(id);
                            break 'search;
                        }
                    }
                }
            }
        }
        let id = found.unwrap_or_else(|| {
            reps.push(*p);
            grid.entry(k).or_default().push(reps.len() - 1);
            reps.len() - 1
        });
        ids.push(id);
    }
    (ids, reps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_coincident_points_share_an_id_and_a_position() {
        let pts = [
            [100_000.0, 0.0, 0.0],
            [100_000.0 + 1e-9, 0.0, 0.0],
            [100_001.0, 0.0, 0.0],
        ];
        let tol = weld_tolerance(&pts);
        let (ids, reps) = weld(&pts, tol);
        assert_eq!(ids, vec![0, 0, 1]);
        assert_eq!(reps[ids[1]], pts[0]);
    }

    #[test]
    fn tolerance_scales_with_magnitude() {
        assert_eq!(
            weld_tolerance(&[[0.5, 0.0, 0.0]]),
            Tolerance { xy: 1e-9, z: 1e-6 }
        );
        assert_eq!(
            weld_tolerance(&[[1e6, 0.0, 0.0]]),
            Tolerance { xy: 1e-3, z: 1e-6 }
        );
    }

    #[test]
    fn the_vertical_tolerance_has_an_absolute_floor() {
        let pts = [[500_000.0, 0.0, 10.0], [500_000.0, 0.0, 10.0 + 1e-7]];
        let t = weld_tolerance(&pts);
        assert_eq!(t.z, 1e-6);
        assert_eq!(weld(&pts, t).0, vec![0, 0]);
    }

    #[test]
    fn a_high_elevation_does_not_loosen_the_horizontal_tolerance() {
        let t = weld_tolerance(&[[139.76, 35.68, 3000.0]]);
        assert_eq!(t.xy, 1e-9 * 139.76);
        assert_eq!(t.z, 1e-9 * 3000.0);
        let (ids, _) = weld(
            &[[139.76, 35.68, 3000.0], [139.76 + 4.5e-7, 35.68, 3000.0]],
            t,
        );
        assert_eq!(ids, vec![0, 1]);
    }
}

//! Tolerance welding: near-coincident vertices snap to one representative so
//! a shared edge is shared by identity, and caps built from both sides meet.

use std::collections::HashMap;

/// `1e-9` of the largest coordinate magnitude, floored at `1e-9`. At
/// projected magnitudes (1e6 m) this is 1 mm; in degrees about 2 cm.
pub(super) fn weld_tolerance(points: &[[f64; 3]]) -> f64 {
    let max = points
        .iter()
        .flat_map(|p| p.iter())
        .fold(0.0f64, |m, c| m.max(c.abs()));
    1e-9 * max.max(1.0)
}

/// Give every point an id; a point within `tol` of an earlier representative
/// takes its id. Returns ids parallel to `points` and the representatives.
pub(super) fn weld(points: &[[f64; 3]], tol: f64) -> (Vec<usize>, Vec<[f64; 3]>) {
    let cell = |c: f64| (c / tol).floor() as i64;
    let mut grid: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
    let mut reps: Vec<[f64; 3]> = Vec::new();
    let mut ids = Vec::with_capacity(points.len());
    for p in points {
        let k = [cell(p[0]), cell(p[1]), cell(p[2])];
        let mut found = None;
        'search: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let Some(candidates) = grid.get(&[k[0] + dx, k[1] + dy, k[2] + dz]) else {
                        continue;
                    };
                    for &id in candidates {
                        let r = reps[id];
                        let d2 =
                            (r[0] - p[0]).powi(2) + (r[1] - p[1]).powi(2) + (r[2] - p[2]).powi(2);
                        if d2 <= tol * tol {
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
        assert_eq!(weld_tolerance(&[[0.5, 0.0, 0.0]]), 1e-9);
        assert_eq!(weld_tolerance(&[[1e6, 0.0, 0.0]]), 1e-3);
    }
}

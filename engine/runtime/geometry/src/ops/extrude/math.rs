//! Small vector helpers for the sweep. Normals are computed relative to the
//! ring's centroid so projected-CRS magnitudes (1e5 to 1e6) do not swamp them.

pub(super) fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub(super) fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub(super) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

#[allow(dead_code)] // used by Task 4
pub(super) fn centroid(points: &[[f64; 3]]) -> [f64; 3] {
    let n = points.len().max(1) as f64;
    let s = points.iter().fold([0.0; 3], |acc, p| add(acc, *p));
    scale(s, 1.0 / n)
}

/// Newell normal of an open ring (no closing vertex), right-hand rule in
/// stored coordinates; its length is twice the ring's area.
#[allow(dead_code)] // used by Task 4
pub(super) fn newell(ring: &[[f64; 3]]) -> [f64; 3] {
    let c = centroid(ring);
    let mut n = [0.0; 3];
    for i in 0..ring.len() {
        let a = sub(ring[i], c);
        let b = sub(ring[(i + 1) % ring.len()], c);
        n[0] += (a[1] - b[1]) * (a[2] + b[2]);
        n[1] += (a[2] - b[2]) * (a[0] + b[0]);
        n[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newell_of_a_ccw_unit_square_points_up_with_twice_its_area() {
        let sq = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        assert_eq!(newell(&sq), [0.0, 0.0, 2.0]);
    }

    #[test]
    fn newell_survives_projected_magnitudes() {
        let o = [-40_000.0, 3_900_000.0, 12.0];
        let sq: Vec<_> = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
        .iter()
        .map(|p| add(*p, o))
        .collect();
        let n = newell(&sq);
        assert!((n[2] - 2.0).abs() < 1e-6, "{n:?}");
    }
}

//! Measures and measure-based checks: chain length, ring area, and the
//! convex-hull planarity test behind
//! [`Degenerate`](super::ValidationType::Degenerate) and
//! [`Planarity`](super::ValidationType::Planarity). Measures are plain f64;
//! only their comparisons against caller thresholds decide anything.

use super::{open_ring, DegenerateThresholds, PlanarityThreshold, ValidationReport};
use crate::algorithm::convex_hull::quick_hull_3d;
use crate::coordinate::CoordinateFrame;
use crate::line_string::{LineString2D, LineString3D};
use crate::polygon::signed_area_2d;
use crate::types::coordinate::Coordinate3D;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// The total Euclidean length of a 2D chain (elevation ignored).
pub(crate) fn chain_length_2d(coords: &[[f64; 2]]) -> f64 {
    coords
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum()
}

/// The total Euclidean length of a 3D chain.
pub(crate) fn chain_length_3d(coords: &[[f64; 3]]) -> f64 {
    coords
        .windows(2)
        .map(|w| {
            let d = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum()
}

/// The non-unit Newell vector of a ring (closing duplicate tolerated),
/// wrapping the last vertex back to the first. Its magnitude is twice the
/// ring's planar-projected area, and it is zero for a degenerate ring.
pub(crate) fn newell_vector_3d(ring: &[[f64; 3]]) -> [f64; 3] {
    let ring = open_ring(ring);
    let n = ring.len();
    let mut acc = [0.0; 3];
    for i in 0..n {
        let a = ring[i];
        let b = ring[(i + 1) % n];
        acc[0] += (a[1] - b[1]) * (a[2] + b[2]);
        acc[1] += (a[2] - b[2]) * (a[0] + b[0]);
        acc[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    acc
}

/// Report a [`Degenerate`](super::ValidationType::Degenerate) problem when a
/// 2D chain's total length is at most `min_length`, positioned at the whole
/// chain as a LineString. At the default threshold 0.0, exactly zero-length
/// chains are flagged.
pub(crate) fn check_degenerate_chain_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    min_length: f64,
    report: &mut ValidationReport,
) {
    if chain_length_2d(coords) <= min_length {
        report.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(frame.clone(), coords.iter().copied()),
        )));
    }
}

/// Report a [`Degenerate`](super::ValidationType::Degenerate) problem when a
/// 3D chain's total length is at most `min_length`, positioned at the chain.
pub(crate) fn check_degenerate_chain_3d(
    frame: &CoordinateFrame,
    coords: &[[f64; 3]],
    min_length: f64,
    report: &mut ValidationReport,
) {
    if chain_length_3d(coords) <= min_length {
        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(frame.clone(), coords.iter().copied()),
        )));
    }
}

/// Whether a ring of the given area is degenerate under `thresholds`: its area
/// is at most `min_area`, or it is a triangle whose height over its longest
/// edge is at most `min_height`. `open` is the ring without its closing
/// duplicate.
fn is_degenerate_ring<const D: usize>(
    open: &[[f64; D]],
    area: f64,
    thresholds: &DegenerateThresholds,
) -> bool {
    if area <= thresholds.min_area {
        return true;
    }
    let [a, b, c] = open else {
        return false;
    };
    let dist = |p: &[f64; D], q: &[f64; D]| {
        p.iter()
            .zip(q)
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f64>()
            .sqrt()
    };
    let longest = dist(a, b).max(dist(b, c)).max(dist(c, a));
    longest == 0.0 || 2.0 * area / longest <= thresholds.min_height
}

/// Report a [`Degenerate`](super::ValidationType::Degenerate) problem when a
/// 2D ring (area |shoelace| / 2) is degenerate under `thresholds`, positioned
/// at the ring as a LineString. Closure is optional; the last vertex wraps to
/// the first.
pub(crate) fn check_degenerate_ring_2d(
    frame: &CoordinateFrame,
    ring: &[[f64; 2]],
    thresholds: &DegenerateThresholds,
    report: &mut ValidationReport,
) {
    let open = open_ring(ring);
    if is_degenerate_ring(open, signed_area_2d(open).abs() / 2.0, thresholds) {
        report.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(frame.clone(), ring.iter().copied()),
        )));
    }
}

/// Report a [`Degenerate`](super::ValidationType::Degenerate) problem when a
/// 3D ring (area half its Newell vector magnitude) is degenerate under
/// `thresholds`, positioned at the ring.
pub(crate) fn check_degenerate_ring_3d(
    frame: &CoordinateFrame,
    ring: &[[f64; 3]],
    thresholds: &DegenerateThresholds,
    report: &mut ValidationReport,
) {
    let n = newell_vector_3d(ring);
    let area = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0;
    if is_degenerate_ring(open_ring(ring), area, thresholds) {
        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(frame.clone(), ring.iter().copied()),
        )));
    }
}

/// Report a [`Planarity`](super::ValidationType::Planarity) problem when the
/// face's ring vertices (exterior and holes together) deviate from a common
/// plane further than `threshold` allows. `Ratio` and `MaxHeight` bound the
/// minimum height of their 3D convex hull (a scale-invariant ratio of the hull's
/// diameter, or an absolute height); a point set with no 3D hull is flat and
/// passes. `MaxZRange` bounds the spread of their z. Rings must be stored closed
/// with finite coordinates. The position is the exterior ring as a LineString.
pub(crate) fn check_planarity_3d<'a>(
    frame: &CoordinateFrame,
    exterior: &[[f64; 3]],
    interiors: impl IntoIterator<Item = &'a [[f64; 3]]>,
    threshold: PlanarityThreshold,
    report: &mut ValidationReport,
) {
    let mut points: Vec<[f64; 3]> = Vec::new();
    points.extend_from_slice(open_ring(exterior));
    for hole in interiors {
        points.extend_from_slice(open_ring(hole));
    }
    let non_planar = match threshold {
        PlanarityThreshold::Ratio(t) => exceeds_hull_height(points, |scale| t * scale),
        PlanarityThreshold::MaxHeight(h) => exceeds_hull_height(points, |_| h),
        PlanarityThreshold::MaxZRange(range) => {
            let (z_min, z_max) = points
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
                    (lo.min(p[2]), hi.max(p[2]))
                });
            z_max - z_min > range
        }
    };
    if non_planar {
        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(frame.clone(), exterior.iter().copied()),
        )));
    }
}

/// Whether the minimum height of `points`' 3D convex hull exceeds the
/// deviation `allowance` gives for a hull of the extent passed to it.
fn exceeds_hull_height(mut points: Vec<[f64; 3]>, allowance: impl Fn(f64) -> f64) -> bool {
    if points.len() < 4 {
        return false;
    }
    // Translate to the first vertex for numerical stability.
    let origin = points[0];
    for p in &mut points {
        for k in 0..3 {
            p[k] -= origin[k];
        }
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for p in &points {
        for k in 0..3 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }
    let diagonal = (0..3)
        .map(|k| (max[k] - min[k]) * (max[k] - min[k]))
        .sum::<f64>()
        .sqrt();
    if diagonal == 0.0 {
        return false;
    }
    let hull_points: Vec<Coordinate3D<f64>> = points
        .iter()
        .map(|p| Coordinate3D::new__(p[0], p[1], p[2]))
        .collect();
    // Points within 1% of the allowed deviation (bounding-box scale) are treated
    // as flat, so a near-flat hull is not built.
    let Some(hull) = quick_hull_3d(&hull_points, allowance(diagonal) * 0.01) else {
        return false;
    };
    let vertices = hull.get_vertices();
    let triangles = hull.get_triangles();
    if vertices.is_empty() || triangles.is_empty() {
        return false;
    }
    let diameter = hull_diameter(vertices);
    if diameter == 0.0 {
        return false;
    }
    // The greatest out-of-plane deviation the face may have, in coordinate units.
    let allowance = allowance(diameter);
    hull_min_height(vertices, triangles, allowance) > allowance
}

/// The minimum height (width) of a convex hull: the smallest vertex-projection
/// extent over the candidate normal directions (each face normal and each
/// edge-pair cross product). The scan stops once the running minimum reaches
/// `allowance` or below, since that already decides the planar verdict, so the
/// return is a witness at or below `allowance`, not necessarily the true minimum.
fn hull_min_height(
    vertices: &[Coordinate3D<f64>],
    triangles: &[[usize; 3]],
    allowance: f64,
) -> f64 {
    let range_along = |unit_normal: Coordinate3D<f64>| -> f64 {
        let mut min_proj = f64::INFINITY;
        let mut max_proj = f64::NEG_INFINITY;
        for v in vertices {
            let proj = v.dot(&unit_normal);
            min_proj = min_proj.min(proj);
            max_proj = max_proj.max(proj);
        }
        max_proj - min_proj
    };

    let mut edges = Vec::with_capacity(triangles.len() * 3);
    for tri in triangles {
        for k in 0..3 {
            edges.push(vertices[tri[(k + 1) % 3]] - vertices[tri[k]]);
        }
    }

    let mut min_height = f64::INFINITY;
    for tri in triangles {
        let ab = vertices[tri[1]] - vertices[tri[0]];
        let ac = vertices[tri[2]] - vertices[tri[0]];
        let normal = ab.cross(&ac);
        let norm = normal.norm();
        if norm > 0.0 {
            min_height = min_height.min(range_along(normal / norm));
            if min_height <= allowance {
                return min_height;
            }
        }
    }
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            let normal = edges[i].cross(&edges[j]);
            let norm = normal.norm();
            if norm > 0.0 {
                min_height = min_height.min(range_along(normal / norm));
                if min_height <= allowance {
                    return min_height;
                }
            }
        }
    }
    min_height
}

/// The hull's diameter: the largest pairwise distance between hull vertices,
/// which is the maximum vertex-projection extent over all directions.
fn hull_diameter(vertices: &[Coordinate3D<f64>]) -> f64 {
    let mut best = 0.0f64;
    for i in 0..vertices.len() {
        for j in (i + 1)..vertices.len() {
            best = best.max((vertices[i] - vertices[j]).norm());
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newell_vector_measures_ring_area() {
        // A closed unit square in the plane z = 5: area 1, normal +z.
        let ring = [
            [0.0, 0.0, 5.0],
            [1.0, 0.0, 5.0],
            [1.0, 1.0, 5.0],
            [0.0, 1.0, 5.0],
            [0.0, 0.0, 5.0],
        ];
        assert_eq!(newell_vector_3d(&ring), [0.0, 0.0, 2.0]);
        // A vertical square (constant y): the area lives in the y component.
        let vertical = [
            [0.0, 3.0, 0.0],
            [0.0, 3.0, 2.0],
            [2.0, 3.0, 2.0],
            [2.0, 3.0, 0.0],
            [0.0, 3.0, 0.0],
        ];
        let n = vertical_newell_norm(&vertical);
        assert_eq!(n, 8.0);
    }

    fn vertical_newell_norm(ring: &[[f64; 3]]) -> f64 {
        let n = newell_vector_3d(ring);
        (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
    }

    fn ring_is_degenerate_3d(ring: &[[f64; 3]], thresholds: &DegenerateThresholds) -> bool {
        ValidationReport::ran(|r| {
            check_degenerate_ring_3d(&CoordinateFrame::Euclidean, ring, thresholds, r)
        })
        .problem_recorded()
    }

    fn ring_is_degenerate_2d(ring: &[[f64; 2]], thresholds: &DegenerateThresholds) -> bool {
        ValidationReport::ran(|r| {
            check_degenerate_ring_2d(&CoordinateFrame::Euclidean, ring, thresholds, r)
        })
        .problem_recorded()
    }

    fn min_height(min_height: f64) -> DegenerateThresholds {
        DegenerateThresholds {
            min_height,
            ..Default::default()
        }
    }

    #[test]
    fn min_height_flags_thin_triangles() {
        // Longest edge 10, height 0.005 (area 0.025).
        let thin = [
            [0.0, 0.0, 1.0],
            [10.0, 0.0, 1.0],
            [5.0, 0.005, 1.0],
            [0.0, 0.0, 1.0],
        ];
        assert!(ring_is_degenerate_3d(&thin, &min_height(0.01)));
        assert!(ring_is_degenerate_3d(&thin, &min_height(0.005)));
        assert!(!ring_is_degenerate_3d(&thin, &min_height(0.004)));
        let thin_2d = [[0.0, 0.0], [10.0, 0.0], [5.0, 0.005]];
        assert!(ring_is_degenerate_2d(&thin_2d, &min_height(0.01)));
        assert!(!ring_is_degenerate_2d(&thin_2d, &min_height(0.004)));
    }

    #[test]
    fn min_height_ignores_rings_of_four_or_more_vertices() {
        // A thin quadrilateral: height 0.005 over a length of 10.
        let thin = [
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 0.005, 0.0],
            [0.0, 0.005, 0.0],
            [0.0, 0.0, 0.0],
        ];
        assert!(!ring_is_degenerate_3d(&thin, &min_height(0.01)));
    }

    #[test]
    fn default_thresholds_ignore_nearly_collinear_triangles() {
        let nearly = [
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [5.0, 1e-6, 0.0],
            [0.0, 0.0, 0.0],
        ];
        assert!(!ring_is_degenerate_3d(
            &nearly,
            &DegenerateThresholds::default()
        ));
        let collinear = [
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        assert!(ring_is_degenerate_3d(
            &collinear,
            &DegenerateThresholds::default()
        ));
    }

    #[test]
    fn chain_lengths() {
        assert_eq!(chain_length_2d(&[[0.0, 0.0], [3.0, 4.0]]), 5.0);
        assert_eq!(
            chain_length_3d(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 2.0, 0.0]]),
            3.0
        );
        assert_eq!(chain_length_3d(&[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]), 0.0);
    }
}

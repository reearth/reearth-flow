use crate::align_mvt::{align_mvt_features, AlignedGeometry, GeometryType};
use reearth_flow_geometry::types::multi_point::MultiPoint2D;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct MvtPointsConfig {
    pub threshold: Option<f64>,
    pub zoom: Option<(u32, u32)>,
}

#[derive(Debug)]
enum ComparisonStatus {
    BothMissing,
    Only1,
    Only2,
    Compared,
}

/// Computes the clipping-aware Hausdorff distance between two multipoint sets
/// For each point, if it's near the tile boundary, cap its distance contribution
/// to the boundary distance (since its match may have been clipped)
/// H(A, B) = max(h(A, B), h(B, A))
/// where h(A, B) = max_{a in A} min(min_{b in B} d(a, b), boundary_dist(a))
fn hausdorff_distance(points1: &MultiPoint2D<f64>, points2: &MultiPoint2D<f64>) -> f64 {
    if points1.0.is_empty() || points2.0.is_empty() {
        return 0.0;
    }

    // h(A, B): for each point in A, find closest point in B, cap by boundary distance, then take max
    let h_ab = points1
        .0
        .iter()
        .map(|p1| {
            let min_dist_to_p2 = points2
                .0
                .iter()
                .map(|p2| {
                    let dx = p1.x() - p2.x();
                    let dy = p1.y() - p2.y();
                    (dx * dx + dy * dy).sqrt()
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(f64::INFINITY);

            // Cap by boundary distance - if p1 is near edge, its match may be clipped
            let boundary_cap = distance_to_tile_boundary(p1.x(), p1.y());
            min_dist_to_p2.min(boundary_cap)
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    // h(B, A): for each point in B, find closest point in A, cap by boundary distance, then take max
    let h_ba = points2
        .0
        .iter()
        .map(|p2| {
            let min_dist_to_p1 = points1
                .0
                .iter()
                .map(|p1| {
                    let dx = p2.x() - p1.x();
                    let dy = p2.y() - p1.y();
                    (dx * dx + dy * dy).sqrt()
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(f64::INFINITY);

            // Cap by boundary distance - if p2 is near edge, its match may be clipped
            let boundary_cap = distance_to_tile_boundary(p2.x(), p2.y());
            min_dist_to_p1.min(boundary_cap)
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    // Hausdorff distance is the maximum of the two directed distances
    f64::max(h_ab, h_ba)
}

/// Computes the minimum distance from a point to the tile boundary [0,1] x [0,1]
/// This measures how close the point is to the edge (0 = on edge, 0.5 = at center for a square tile)
fn distance_to_tile_boundary(x: f64, y: f64) -> f64 {
    let dx = x.min(1.0 - x).max(0.0); // distance to nearest vertical edge (0 or 1)
    let dy = y.min(1.0 - y).max(0.0); // distance to nearest horizontal edge (0 or 1)
    dx.min(dy) // minimum distance to any edge
}

/// Computes the minimum distance to boundary across all points
/// Points near the boundary (distance ~0) are more likely to be clipping errors
fn min_distance_to_boundary(points: &MultiPoint2D<f64>) -> f64 {
    points
        .0
        .iter()
        .map(|p| distance_to_tile_boundary(p.x(), p.y()))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.5) // default to center if no points
}

/// Compares two point sets using Hausdorff distance
/// When one set is missing, use the minimum distance to tile boundary as the error metric
/// (points near the edge are more likely to be clipping errors)
fn compare_points(
    geom1: Option<MultiPoint2D<f64>>,
    geom2: Option<MultiPoint2D<f64>>,
) -> (ComparisonStatus, f64) {
    match (&geom1, &geom2) {
        (None, None) => (ComparisonStatus::BothMissing, 0.0),
        (Some(p1), None) => {
            // If points are near the tile boundary, this could be a clipping error
            // Return the minimum distance to boundary as the error metric
            let distance = min_distance_to_boundary(p1);
            (ComparisonStatus::Only1, distance)
        }
        (None, Some(p2)) => {
            // If points are near the tile boundary, this could be a clipping error
            let distance = min_distance_to_boundary(p2);
            (ComparisonStatus::Only2, distance)
        }
        (Some(p1), Some(p2)) => {
            let distance = hausdorff_distance(p1, p2);
            (ComparisonStatus::Compared, distance)
        }
    }
}

/// Tests MVT points between FME and Flow outputs
pub fn test_mvt_points(
    fme_path: &Path,
    flow_path: &Path,
    config: &MvtPointsConfig,
) -> Result<(), String> {
    let threshold = config.threshold.unwrap_or(0.0);
    let (zmin, zmax) = config.zoom.unwrap_or((0, u32::MAX));
    let zmin = if zmin == 0 { None } else { Some(zmin) };
    let zmax = if zmax == u32::MAX { None } else { Some(zmax) };

    // Fetch Point features
    let point_features = align_mvt_features(fme_path, flow_path, GeometryType::Point, zmin, zmax)?;

    let mut results = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;

    // Process Point features
    for feature in point_features {
        let (geom1, geom2) = match feature.geometry {
            AlignedGeometry::Point(g1, g2) => (g1, g2),
            _ => continue,
        };

        // Only count if at least one geometry exists
        if geom1.is_none() && geom2.is_none() {
            continue;
        }

        total += 1;
        let (status, score) = compare_points(geom1, geom2);
        worst_score = f64::max(worst_score, score);

        results.push((
            score,
            feature.tile_path,
            feature.gml_id,
            format!("{:?}", status),
        ));
    }

    let failures: Vec<_> = results
        .iter()
        .filter(|(score, _, _, _)| *score > threshold)
        .collect();

    tracing::info!(
        "MVT points: {} total, {} failures, worst={:.6}, threshold={}",
        total,
        failures.len(),
        worst_score,
        threshold
    );

    if !failures.is_empty() {
        tracing::info!("Worst 5 failures:");
        let mut sorted_failures = failures.clone();
        sorted_failures.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (score, path, gml_id, status) in sorted_failures.iter().take(5) {
            tracing::info!("  {} | {} | {:.6} | {}", path, gml_id, score, status);
        }
        return Err(format!(
            "MVT point comparison failed: {}/{} exceeded threshold {}",
            failures.len(),
            total,
            threshold
        ));
    } else {
        // Log worst 5 in debug when test passes
        let mut sorted_results = results.clone();
        sorted_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        tracing::debug!("Worst 5 scores (all below threshold):");
        for (score, path, gml_id, status) in sorted_results.iter().take(5) {
            tracing::debug!("  {} | {} | {:.6} | {}", path, gml_id, score, status);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::point::Point2D;

    #[test]
    fn test_hausdorff_identical_points() {
        let points1 = MultiPoint2D::from(vec![
            Point2D::from((0.1, 0.1)),
            Point2D::from((0.5, 0.5)),
            Point2D::from((0.9, 0.9)),
        ]);
        let points2 = MultiPoint2D::from(vec![
            Point2D::from((0.1, 0.1)),
            Point2D::from((0.5, 0.5)),
            Point2D::from((0.9, 0.9)),
        ]);

        let distance = hausdorff_distance(&points1, &points2);
        assert!(
            distance < 1e-10,
            "Identical point sets should have zero distance, got {}",
            distance
        );
    }

    #[test]
    fn test_hausdorff_small_difference() {
        // Points1: three points
        let points1 = MultiPoint2D::from(vec![
            Point2D::from((0.1, 0.1)),
            Point2D::from((0.5, 0.5)),
            Point2D::from((0.9, 0.9)),
        ]);
        // Points2: same points but one is slightly shifted
        let points2 = MultiPoint2D::from(vec![
            Point2D::from((0.1, 0.1)),
            Point2D::from((0.51, 0.51)), // shifted by 0.01 in each dimension
            Point2D::from((0.9, 0.9)),
        ]);

        let distance = hausdorff_distance(&points1, &points2);
        let expected = (0.01_f64 * 0.01 + 0.01 * 0.01).sqrt(); // ~0.0141
        assert!(
            (distance - expected).abs() < 1e-6,
            "Expected distance ~{}, got {}",
            expected,
            distance
        );
    }

    #[test]
    fn test_hausdorff_missing_point() {
        // Points1: three points
        let points1 = MultiPoint2D::from(vec![
            Point2D::from((0.1, 0.1)),
            Point2D::from((0.5, 0.5)),
            Point2D::from((0.9, 0.9)),
        ]);
        // Points2: missing the middle point
        let points2 =
            MultiPoint2D::from(vec![Point2D::from((0.1, 0.1)), Point2D::from((0.9, 0.9))]);

        let distance = hausdorff_distance(&points1, &points2);
        // The point (0.5, 0.5) in points1 is farthest from points2
        // Its closest point in points2 is either (0.1, 0.1) or (0.9, 0.9)
        // Euclidean distance would be sqrt(0.4^2 + 0.4^2) ~0.566
        // But (0.5, 0.5) is 0.5 from the nearest tile edge
        // So the distance is capped at 0.5 (clipping-aware)
        let expected = 0.5;
        assert!(
            (distance - expected).abs() < 1e-6,
            "Expected distance ~{}, got {}",
            expected,
            distance
        );
    }

    #[test]
    fn test_hausdorff_extra_point() {
        // Points1: two points
        let points1 =
            MultiPoint2D::from(vec![Point2D::from((0.1, 0.1)), Point2D::from((0.9, 0.9))]);
        // Points2: has an extra point far away
        let points2 = MultiPoint2D::from(vec![
            Point2D::from((0.1, 0.1)),
            Point2D::from((0.9, 0.9)),
            Point2D::from((0.5, 0.5)),
        ]);

        let distance = hausdorff_distance(&points1, &points2);
        // All points in points1 have exact matches in points2
        // The extra point (0.5, 0.5) in points2 is closest to either (0.1, 0.1) or (0.9, 0.9)
        // Euclidean distance would be sqrt(0.4^2 + 0.4^2) ~0.566
        // But (0.5, 0.5) is 0.5 from the nearest tile edge
        // So the distance is capped at 0.5 (clipping-aware)
        let expected = 0.5;
        assert!(
            (distance - expected).abs() < 1e-6,
            "Expected distance ~{}, got {}",
            expected,
            distance
        );
    }

    #[test]
    fn test_compare_points_both_missing() {
        let (status, distance) = compare_points(None, None);
        assert!(matches!(status, ComparisonStatus::BothMissing));
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_compare_points_only_first() {
        let points = MultiPoint2D::from(vec![Point2D::from((0.5, 0.5))]);
        let (status, distance) = compare_points(Some(points), None);
        assert!(matches!(status, ComparisonStatus::Only1));
        // Point at center has distance 0.5 to nearest edge
        assert!((distance - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compare_points_only_second() {
        let points = MultiPoint2D::from(vec![Point2D::from((0.5, 0.5))]);
        let (status, distance) = compare_points(None, Some(points));
        assert!(matches!(status, ComparisonStatus::Only2));
        // Point at center has distance 0.5 to nearest edge
        assert!((distance - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compare_points_compared() {
        let points1 =
            MultiPoint2D::from(vec![Point2D::from((0.1, 0.1)), Point2D::from((0.9, 0.9))]);
        let points2 =
            MultiPoint2D::from(vec![Point2D::from((0.1, 0.1)), Point2D::from((0.9, 0.9))]);
        let (status, distance) = compare_points(Some(points1), Some(points2));
        assert!(matches!(status, ComparisonStatus::Compared));
        assert!(distance < 1e-10);
    }
}

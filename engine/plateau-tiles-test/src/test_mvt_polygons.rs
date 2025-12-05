use crate::align_mvt::{align_mvt_features, AlignedGeometry, GeometryType};
use reearth_flow_geometry::algorithm::area2d::Area2D;
use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct MvtPolygonsConfig {
    pub threshold: Option<f64>,
    pub zoom: Option<(u32, u32)>,
}

/// Clips geometry to [0,1] x [0,1] bounds
fn clip_geometry(geom: &MultiPolygon2D<f64>) -> Option<MultiPolygon2D<f64>> {
    // Create clip bounds as a polygon [0,1] x [0,1]
    let clip_bounds = Polygon2D::new(
        LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(0.0, 1.0),
            Coordinate2D::new_(0.0, 0.0),
        ]),
        vec![],
    );

    let clipped = geom.intersection(&MultiPolygon2D::new(vec![clip_bounds]));
    if clipped.0.is_empty() {
        None
    } else {
        Some(clipped)
    }
}

#[derive(Debug)]
enum ComparisonStatus {
    BothMissing,
    Only1,
    Only2,
    Compared,
}

/// Compares two polygons using symmetric difference (XOR)
fn compare_polygons(
    geom1: Option<MultiPolygon2D<f64>>,
    geom2: Option<MultiPolygon2D<f64>>,
) -> (ComparisonStatus, f64) {
    // Clip both geometries to bounds
    let poly1 = geom1.and_then(|g| clip_geometry(&g));
    let poly2 = geom2.and_then(|g| clip_geometry(&g));

    match (poly1, poly2) {
        (None, None) => (ComparisonStatus::BothMissing, 0.0),
        (None, Some(p2)) => {
            let area = p2.unsigned_area2d();
            (ComparisonStatus::Only2, area)
        }
        (Some(p1), None) => {
            let area = p1.unsigned_area2d();
            (ComparisonStatus::Only1, area)
        }
        (Some(p1), Some(p2)) => {
            // Symmetric difference (XOR)
            let sym_diff = p1.xor(&p2);
            let area = sym_diff.unsigned_area2d();
            (ComparisonStatus::Compared, area)
        }
    }
}

/// Tests MVT polygons between FME and Flow outputs
pub fn test_mvt_polygons(
    fme_path: &Path,
    flow_path: &Path,
    config: &MvtPolygonsConfig,
) -> Result<(), String> {
    let threshold = config.threshold.unwrap_or(0.0);
    let (zmin, zmax) = config.zoom.unwrap_or((0, u32::MAX));
    let zmin = if zmin == 0 { None } else { Some(zmin) };
    let zmax = if zmax == u32::MAX { None } else { Some(zmax) };

    let aligned_features =
        align_mvt_features(fme_path, flow_path, GeometryType::Polygon, zmin, zmax)?;

    let mut results = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;

    for feature in aligned_features {
        let (geom1, geom2) = match feature.geometry {
            AlignedGeometry::Polygon(g1, g2) => (g1, g2),
            _ => continue,
        };

        // Only count if at least one geometry exists
        if geom1.is_none() && geom2.is_none() {
            continue;
        }

        total += 1;
        let (status, score) = compare_polygons(geom1, geom2);
        worst_score = f64::max(worst_score, score);

        results.push((score, feature.tile_path, feature.gml_id, format!("{:?}", status)));
    }

    let failures: Vec<_> = results.iter().filter(|(score, _, _, _)| *score > threshold).collect();

    tracing::info!(
        "MVT polygons: {} total, {} failures, worst={:.6}, threshold={}",
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
            "MVT polygon comparison failed: {}/{} exceeded threshold {}",
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

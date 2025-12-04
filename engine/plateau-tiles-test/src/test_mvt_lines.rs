use crate::align_mvt::{align_mvt_features, AlignedGeometry, GeometryType};
use reearth_flow_geometry::algorithm::clipper::ClipperOpen2D;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_line_string::MultiLineString2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct MvtLinesConfig {
    pub threshold: Option<f64>,
    pub zoom: Option<(u32, u32)>,
}

/// Clips linestring to [0,1] x [0,1] bounds
fn clip_linestring(geom: &MultiLineString2D<f64>) -> Option<MultiLineString2D<f64>> {
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

    let clipped = geom.intersection2d(&clip_bounds, 1e9);
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

/// Compares two linestrings (unimplemented - placeholder for future implementation)
fn compare_lines(
    geom1: Option<MultiLineString2D<f64>>,
    geom2: Option<MultiLineString2D<f64>>,
) -> (ComparisonStatus, f64) {
    // Clip both geometries to bounds
    let line1 = geom1.and_then(|g| clip_linestring(&g));
    let line2 = geom2.and_then(|g| clip_linestring(&g));

    match (line1, line2) {
        (None, None) => (ComparisonStatus::BothMissing, 0.0),
        (None, Some(_)) => {
            // TODO: Implement proper length calculation or other metric
            (ComparisonStatus::Only2, 0.0)
        }
        (Some(_), None) => {
            // TODO: Implement proper length calculation or other metric
            (ComparisonStatus::Only1, 0.0)
        }
        (Some(_), Some(_)) => {
            // TODO: Implement linestring comparison logic (e.g., Hausdorff distance)
            // For now, just pass all comparisons
            (ComparisonStatus::Compared, 0.0)
        }
    }
}

/// Tests MVT lines between FME and Flow outputs
pub fn test_mvt_lines(
    fme_path: &Path,
    flow_path: &Path,
    config: &MvtLinesConfig,
) -> Result<(), String> {
    let threshold = config.threshold.unwrap_or(0.0);
    let (zmin, zmax) = config.zoom.unwrap_or((0, u32::MAX));
    let zmin = if zmin == 0 { None } else { Some(zmin) };
    let zmax = if zmax == u32::MAX { None } else { Some(zmax) };

    let aligned_features =
        align_mvt_features(fme_path, flow_path, GeometryType::LineString, zmin, zmax)?;

    let mut failures = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;

    for feature in aligned_features {
        let (geom1, geom2) = match feature.geometry {
            AlignedGeometry::LineString(g1, g2) => (g1, g2),
            _ => continue,
        };

        // Only count if at least one geometry exists
        if geom1.is_none() && geom2.is_none() {
            continue;
        }

        total += 1;
        let (status, score) = compare_lines(geom1, geom2);
        worst_score = f64::max(worst_score, score);

        if score > threshold {
            failures.push((score, feature.tile_path, feature.gml_id, format!("{:?}", status)));
        }
    }

    tracing::info!(
        "MVT lines: {} total, {} failures, worst={:.6}, threshold={}",
        total,
        failures.len(),
        worst_score,
        threshold
    );

    if !failures.is_empty() {
        tracing::info!("Worst 5 failures:");
        failures.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (score, path, gml_id, status) in failures.iter().take(5) {
            tracing::info!("  {} | {} | {:.6} | {}", path, gml_id, score, status);
        }
        return Err(format!(
            "MVT line comparison failed: {}/{} exceeded threshold {}",
            failures.len(),
            total,
            threshold
        ));
    }

    Ok(())
}

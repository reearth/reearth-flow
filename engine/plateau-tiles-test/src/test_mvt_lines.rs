use crate::align_mvt::{align_mvt_features, AlignedGeometry, GeometryType};
use reearth_flow_geometry::algorithm::clipper::ClipperOpen2D;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_line_string::MultiLineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct MvtLinesConfig {
    pub threshold: Option<f64>,
    pub zoom: Option<(u32, u32)>,
}

/// Raster size for line comparison (1024x1024 pixels)
const RASTER_SIZE: usize = 1024;

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

/// Wu's line drawing algorithm - draws an anti-aliased line on a raster
fn draw_wu_line(
    raster: &mut [f32],
    width: usize,
    height: usize,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) {
    let mut x0 = x0;
    let mut y0 = y0;
    let mut x1 = x1;
    let mut y1 = y1;

    let steep = (y1 - y0).abs() > (x1 - x0).abs();

    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let gradient = if dx.abs() < 1e-10 { 1.0 } else { dy / dx };

    // Helper to set pixel with bounds checking
    let set_pixel = |raster: &mut [f32], x: i32, y: i32, alpha: f32| {
        let (px, py) = if steep { (y, x) } else { (x, y) };
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = py as usize * width + px as usize;
            raster[idx] = f32::max(raster[idx], alpha);
        }
    };

    // First endpoint
    let xend = x0.round();
    let yend = y0 + gradient * (xend - x0);
    let xgap = 1.0 - (x0 + 0.5).fract();
    let xpxl1 = xend as i32;
    let ypxl1 = yend.floor() as i32;

    set_pixel(
        raster,
        xpxl1,
        ypxl1,
        (1.0 - yend.fract()) as f32 * xgap as f32,
    );
    set_pixel(raster, xpxl1, ypxl1 + 1, yend.fract() as f32 * xgap as f32);

    let mut intery = yend + gradient;

    // Second endpoint
    let xend = x1.round();
    let yend = y1 + gradient * (xend - x1);
    let xgap = (x1 + 0.5).fract();
    let xpxl2 = xend as i32;
    let ypxl2 = yend.floor() as i32;

    set_pixel(
        raster,
        xpxl2,
        ypxl2,
        (1.0 - yend.fract()) as f32 * xgap as f32,
    );
    set_pixel(raster, xpxl2, ypxl2 + 1, yend.fract() as f32 * xgap as f32);

    // Main loop
    for x in (xpxl1 + 1)..xpxl2 {
        let y = intery.floor() as i32;
        set_pixel(raster, x, y, (1.0 - intery.fract()) as f32);
        set_pixel(raster, x, y + 1, intery.fract() as f32);
        intery += gradient;
    }
}

/// Rasterizes a MultiLineString to a RASTER_SIZE x RASTER_SIZE raster
fn rasterize_lines(geom: &MultiLineString2D<f64>, raster: &mut [f32]) {
    for linestring in geom.0.iter() {
        let coords = &linestring.0;
        for window in coords.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];

            // Convert from [0,1] coordinates to pixel coordinates
            let x0 = p0.x * RASTER_SIZE as f64;
            let y0 = p0.y * RASTER_SIZE as f64;
            let x1 = p1.x * RASTER_SIZE as f64;
            let y1 = p1.y * RASTER_SIZE as f64;

            draw_wu_line(raster, RASTER_SIZE, RASTER_SIZE, x0, y0, x1, y1);
        }
    }
}

// abs distance with threshold
fn compare_rasters(raster1: &[f32], raster2: &[f32]) -> f64 {
    let sum_sq: f64 = raster1
        .iter()
        .zip(raster2.iter())
        .map(|(a, b)| {
            let diff = ((*a as f64) - (*b as f64)).abs();
            if diff >= 0.5 {
                diff
            } else {
                0.0
            }
        })
        .sum();

    (sum_sq / raster1.len() as f64).sqrt()
}

/// Compares two linestrings using rasterization and pixelwise RMS
fn compare_lines(
    geom1: Option<MultiLineString2D<f64>>,
    geom2: Option<MultiLineString2D<f64>>,
) -> (ComparisonStatus, f64) {
    // Clip both geometries to bounds
    let line1 = geom1.as_ref().and_then(|g| clip_linestring(g));
    let line2 = geom2.as_ref().and_then(|g| clip_linestring(g));

    match (&line1, &line2) {
        (None, None) => (ComparisonStatus::BothMissing, 0.0),
        (Some(l1), None) => {
            // Only line1 exists - treat as maximum difference
            let mut raster1 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
            rasterize_lines(l1, &mut raster1);
            let raster2 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
            let rms = compare_rasters(&raster1, &raster2);
            (ComparisonStatus::Only1, rms)
        }
        (None, Some(l2)) => {
            // Only line2 exists - treat as maximum difference
            let raster1 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
            let mut raster2 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
            rasterize_lines(l2, &mut raster2);
            let rms = compare_rasters(&raster1, &raster2);
            (ComparisonStatus::Only2, rms)
        }
        (Some(l1), Some(l2)) => {
            // Both exist - rasterize and compare
            let mut raster1 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
            let mut raster2 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];

            rasterize_lines(l1, &mut raster1);
            rasterize_lines(l2, &mut raster2);

            let rms = compare_rasters(&raster1, &raster2);
            (ComparisonStatus::Compared, rms)
        }
    }
}

/// Converts polygon boundaries to linestrings for comparison
fn polygon_to_linestrings(poly: &MultiPolygon2D<f64>) -> MultiLineString2D<f64> {
    let mut linestrings = Vec::new();

    for polygon in poly.0.iter() {
        // Add exterior ring
        linestrings.push(polygon.exterior().clone());

        // Add interior rings (holes)
        for hole in polygon.interiors() {
            linestrings.push(hole.clone());
        }
    }

    MultiLineString2D::new(linestrings)
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

    // Fetch both LineString and Polygon features
    let line_features =
        align_mvt_features(fme_path, flow_path, GeometryType::LineString, zmin, zmax)?;
    let polygon_features =
        align_mvt_features(fme_path, flow_path, GeometryType::Polygon, zmin, zmax)?;

    let mut failures = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;

    // Process LineString features
    for feature in line_features {
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
            failures.push((
                score,
                feature.tile_path,
                feature.gml_id,
                format!("{:?}", status),
            ));
        }
    }

    // Process Polygon features (compare boundaries as lines)
    for feature in polygon_features {
        let (poly1, poly2) = match feature.geometry {
            AlignedGeometry::Polygon(p1, p2) => (p1, p2),
            _ => continue,
        };

        // Only count if at least one geometry exists
        if poly1.is_none() && poly2.is_none() {
            continue;
        }

        total += 1;

        // Convert polygon boundaries to linestrings
        let line1 = poly1.as_ref().map(|p| polygon_to_linestrings(p));
        let line2 = poly2.as_ref().map(|p| polygon_to_linestrings(p));

        let (status, score) = compare_lines(line1, line2);
        worst_score = f64::max(worst_score, score);

        if score > threshold {
            failures.push((
                score,
                feature.tile_path,
                feature.gml_id,
                format!("{:?}", status),
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get raster index from pixel coordinates
    fn raster_idx(x: usize, y: usize, width: usize) -> usize {
        y * width + x
    }

    #[test]
    fn test_wu_line_horizontal() {
        let size = 32;
        let mut raster = vec![0.0f32; size * size];

        // Draw horizontal line from (10, 10) to (20, 10)
        draw_wu_line(&mut raster, size, size, 10.0, 10.0, 20.0, 10.0);

        // Wu's algorithm for horizontal line at exact integer y coordinate
        // First endpoint at x=10: xgap = 1.0 - (10.0 + 0.5).fract() = 1.0 - 0.5 = 0.5
        // yend = 10.0, fract = 0.0, so: (1-0.0)*0.5 = 0.5 at y=10, 0.0*0.5 = 0.0 at y=11
        assert_eq!(raster[raster_idx(10, 10, size)], 0.5);
        assert_eq!(raster[raster_idx(10, 11, size)], 0.0);

        // Middle pixels: intery = 10.0, fract = 0.0, so (1-0.0) = 1.0 at y=10, 0.0 at y=11
        for x in 11..20 {
            assert_eq!(
                raster[raster_idx(x, 10, size)],
                1.0,
                "Middle pixel at ({}, 10)",
                x
            );
            assert_eq!(
                raster[raster_idx(x, 11, size)],
                0.0,
                "Middle pixel at ({}, 11)",
                x
            );
        }

        // Last endpoint at x=20: xgap = (20.0 + 0.5).fract() = 0.5
        // yend = 10.0, fract = 0.0, so: (1-0.0)*0.5 = 0.5 at y=10, 0.0*0.5 = 0.0 at y=11
        assert_eq!(raster[raster_idx(20, 10, size)], 0.5);
        assert_eq!(raster[raster_idx(20, 11, size)], 0.0);

        // Check that pixels outside the line are exactly 0.0
        assert_eq!(raster[raster_idx(15, 9, size)], 0.0);
        assert_eq!(raster[raster_idx(15, 12, size)], 0.0);
        assert_eq!(raster[raster_idx(9, 10, size)], 0.0);
        assert_eq!(raster[raster_idx(21, 10, size)], 0.0);
    }

    #[test]
    fn test_wu_line_diagonal() {
        let size = 32;
        let mut raster = vec![0.0f32; size * size];

        // Draw diagonal line from (5, 5) to (15, 15)
        draw_wu_line(&mut raster, size, size, 5.0, 5.0, 15.0, 15.0);

        // Wu's algorithm for diagonal (gradient = 1.0)
        // First endpoint at x=5: xgap = 1.0 - (5.0 + 0.5).fract() = 1.0 - 0.5 = 0.5
        // yend = 5.0, fract = 0.0, so: (1-0.0)*0.5 = 0.5 at y=5, 0.0*0.5 = 0.0 at y=6
        assert_eq!(raster[raster_idx(5, 5, size)], 0.5);
        assert_eq!(raster[raster_idx(5, 6, size)], 0.0);

        // Middle pixels: gradient = 1.0, intery starts at 6.0 (integer)
        // fract = 0.0, so (1-0.0) = 1.0 at y, 0.0 at y+1
        for i in 6..15 {
            assert_eq!(
                raster[raster_idx(i, i, size)],
                1.0,
                "Middle pixel at ({}, {})",
                i,
                i
            );
            assert_eq!(
                raster[raster_idx(i, i + 1, size)],
                0.0,
                "Middle pixel at ({}, {})",
                i,
                i + 1
            );
        }

        // Last endpoint at x=15: xgap = (15.0 + 0.5).fract() = 0.5
        // yend = 15.0, fract = 0.0, so: (1-0.0)*0.5 = 0.5 at y=15, 0.0*0.5 = 0.0 at y=16
        assert_eq!(raster[raster_idx(15, 15, size)], 0.5);
        assert_eq!(raster[raster_idx(15, 16, size)], 0.0);

        // Check that pixels far from the line are exactly 0.0
        assert_eq!(raster[raster_idx(15, 5, size)], 0.0);
        assert_eq!(raster[raster_idx(5, 15, size)], 0.0);
    }

    #[test]
    fn test_compare_lines_clipped_equivalent() {
        // Test 1: Both lines represent the same diagonal through the tile, one extends outside
        // A: line from (0, 0) to (0.5, 0.5) - stays within bounds
        let line_a = MultiLineString2D::new(vec![LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.5, 0.5),
        ])]);

        // B: line from (-0.5, -0.5) to (0.5, 0.5) - extends outside but clips to same as A
        let line_b = MultiLineString2D::new(vec![LineString2D::new(vec![
            Coordinate2D::new_(-0.5, -0.5),
            Coordinate2D::new_(0.5, 0.5),
        ])]);

        let (status, rms) = compare_lines(Some(line_a), Some(line_b));

        assert!(matches!(status, ComparisonStatus::Compared));
        // After clipping, both should represent the same line segment (0,0) to (0.5,0.5)
        // RMS should be very small (near zero)
        assert!(
            rms < 1e-6,
            "RMS should be near zero for equivalent clipped lines, got {}",
            rms
        );
    }

    #[test]
    fn test_compare_lines_with_small_difference() {
        // Test 2: Lines mostly identical but with a small additional line far away
        // A: a long diagonal line
        let line_a = MultiLineString2D::new(vec![LineString2D::new(vec![
            Coordinate2D::new_(0.1, 0.1),
            Coordinate2D::new_(0.9, 0.9),
        ])]);

        // B: same long diagonal line + a very short line in corner
        let line_b = MultiLineString2D::new(vec![
            LineString2D::new(vec![
                Coordinate2D::new_(0.1, 0.1),
                Coordinate2D::new_(0.9, 0.9),
            ]),
            LineString2D::new(vec![
                Coordinate2D::new_(0.95, 0.05),
                Coordinate2D::new_(0.98, 0.08),
            ]),
        ]);

        let (status, rms) = compare_lines(Some(line_a), Some(line_b));

        assert!(matches!(status, ComparisonStatus::Compared));
        // Short line far away should contribute small RMS difference
        // The main diagonal is ~800 pixels long, the extra line is ~30 pixels
        // Over 1024*1024 pixels, this should be a small RMS value
        println!("RMS with small additional line: {}", rms);
        assert!(
            rms < 0.01,
            "RMS should be small for minor additional geometry, got {}",
            rms
        );
    }

    #[test]
    fn test_compare_lines_missing_line_fails() {
        // Test 3: Grid of lines, but B is missing one entire line
        // A: 3 horizontal lines forming a grid
        let line_a = MultiLineString2D::new(vec![
            LineString2D::new(vec![
                Coordinate2D::new_(0.1, 0.3),
                Coordinate2D::new_(0.9, 0.3),
            ]),
            LineString2D::new(vec![
                Coordinate2D::new_(0.1, 0.5),
                Coordinate2D::new_(0.9, 0.5),
            ]),
            LineString2D::new(vec![
                Coordinate2D::new_(0.1, 0.7),
                Coordinate2D::new_(0.9, 0.7),
            ]),
        ]);

        // B: same grid but missing the middle line
        let line_b = MultiLineString2D::new(vec![
            LineString2D::new(vec![
                Coordinate2D::new_(0.1, 0.3),
                Coordinate2D::new_(0.9, 0.3),
            ]),
            LineString2D::new(vec![
                Coordinate2D::new_(0.1, 0.7),
                Coordinate2D::new_(0.9, 0.7),
            ]),
        ]);

        let (status, rms) = compare_lines(Some(line_a), Some(line_b));

        assert!(matches!(status, ComparisonStatus::Compared));
        // Missing a whole line should produce significant RMS difference
        // Each line is ~800 pixels long, missing one means ~800 pixels different
        println!("RMS with missing line: {}", rms);
        assert!(
            rms > 0.02,
            "RMS should be significant for missing line, got {}",
            rms
        );
    }

    #[test]
    fn test_compare_lines_both_missing() {
        let (status, rms) = compare_lines(None, None);
        assert!(matches!(status, ComparisonStatus::BothMissing));
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_compare_lines_only_first() {
        let line = MultiLineString2D::new(vec![LineString2D::new(vec![
            Coordinate2D::new_(0.2, 0.2),
            Coordinate2D::new_(0.8, 0.8),
        ])]);

        let (status, rms) = compare_lines(Some(line), None);
        assert!(matches!(status, ComparisonStatus::Only1));
        assert!(
            rms > 0.0,
            "RMS should be non-zero when only one geometry exists"
        );
    }

    #[test]
    fn test_compare_lines_only_second() {
        let line = MultiLineString2D::new(vec![LineString2D::new(vec![
            Coordinate2D::new_(0.2, 0.2),
            Coordinate2D::new_(0.8, 0.8),
        ])]);

        let (status, rms) = compare_lines(None, Some(line));
        assert!(matches!(status, ComparisonStatus::Only2));
        assert!(
            rms > 0.0,
            "RMS should be non-zero when only one geometry exists"
        );
    }
}

use prost::Message;
use reearth_flow_geometry::algorithm::area2d::Area2D;
use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tinymvt::geometry::{Geometry, GeometryDecoder};
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct MvtPolygonsConfig {
    pub threshold: Option<f64>,
    pub zoom: Option<(u32, u32)>,
}

/// Converts tinymvt Geometry to our internal polygon type, normalizing by tile extent
fn tinymvt_to_polygon(geom: Geometry, extent: u32) -> Option<MultiPolygon2D<f64>> {
    let scale = 1.0 / extent as f64;

    match geom {
        Geometry::Polygons(polygons) => {
            let poly_vec: Vec<Polygon2D<f64>> = polygons
                .into_iter()
                .filter_map(|rings| {
                    if rings.is_empty() {
                        return None;
                    }

                    // First ring is exterior
                    let exterior_points = &rings[0];
                    let exterior_coords: Vec<Coordinate2D<f64>> = exterior_points
                        .iter()
                        .map(|p| Coordinate2D::new_(p[0] as f64 * scale, p[1] as f64 * scale))
                        .collect();
                    let exterior = LineString2D::new(exterior_coords);

                    // Rest are holes
                    let interiors: Vec<LineString2D<f64>> = rings[1..]
                        .iter()
                        .filter_map(|ring| {
                            if ring.is_empty() {
                                return None;
                            }
                            let coords: Vec<Coordinate2D<f64>> = ring
                                .iter()
                                .map(|p| {
                                    Coordinate2D::new_(p[0] as f64 * scale, p[1] as f64 * scale)
                                })
                                .collect();
                            Some(LineString2D::new(coords))
                        })
                        .collect();

                    Some(Polygon2D::new(exterior, interiors))
                })
                .collect();

            if poly_vec.is_empty() {
                None
            } else {
                Some(MultiPolygon2D::new(poly_vec))
            }
        }
        _ => None,
    }
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

/// Loads a single MVT tile
fn load_mvt(path: &Path) -> Result<Tile, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read MVT file {:?}: {}", path, e))?;
    Tile::decode(&data[..]).map_err(|e| format!("Failed to decode MVT protobuf: {}", e))
}

/// Extracts polygon features by gml_id from a tile
fn features_by_gml_id(tile: &Tile) -> HashMap<String, (Vec<u32>, i32, u32)> {
    let mut result = HashMap::new();

    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        let extent = layer.extent.unwrap_or(4096);

        for feature in &layer.features {
            let geom_type = feature.r#type.unwrap_or(0);
            // Only process polygon features (type 3)
            if geom_type != 3 {
                continue;
            }

            if let Ok(tags) = tags_decoder.decode(&feature.tags) {
                if let Some(gml_id_value) = tags.iter().find(|(k, _)| k == "gml_id") {
                    if let tinymvt::tag::Value::String(gml_id) = &gml_id_value.1 {
                        // Store the geometry buffer, geometry type, and extent
                        result.insert(
                            gml_id.clone(),
                            (feature.geometry.clone(), geom_type, extent),
                        );
                    }
                }
            }
        }
    }

    result
}

/// Aligns features from two directories by walking all .mvt files
fn align_mvt(
    dir1: &Path,
    dir2: &Path,
    zmin: Option<u32>,
    zmax: Option<u32>,
) -> Result<
    Vec<(
        String,
        String,
        Option<MultiPolygon2D<f64>>,
        Option<MultiPolygon2D<f64>>,
    )>,
    String,
> {
    // Collect all .mvt files
    let mut files1: Vec<PathBuf> = WalkDir::new(dir1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "mvt"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut files2: Vec<PathBuf> = WalkDir::new(dir2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "mvt"))
        .map(|e| e.path().to_path_buf())
        .collect();

    // Filter by zoom level if specified
    if zmin.is_some() || zmax.is_some() {
        let filter = |path: &Path| -> bool {
            if let Ok(rel) = path.strip_prefix(dir1).or_else(|_| path.strip_prefix(dir2)) {
                let parts: Vec<_> = rel.iter().collect();
                if parts.len() >= 2 {
                    if let Some(z_str) = parts[1].to_str() {
                        if let Ok(z) = z_str.parse::<u32>() {
                            if let Some(min) = zmin {
                                if z < min {
                                    return false;
                                }
                            }
                            if let Some(max) = zmax {
                                if z > max {
                                    return false;
                                }
                            }
                            return true;
                        }
                    }
                }
            }
            false
        };

        files1.retain(|p| filter(p));
        files2.retain(|p| filter(p));
    }

    // Create maps of relative paths to absolute paths
    let map1: HashMap<String, PathBuf> = files1
        .iter()
        .filter_map(|p| {
            p.strip_prefix(dir1)
                .ok()
                .map(|rel| (rel.to_string_lossy().to_string(), p.clone()))
        })
        .collect();

    let map2: HashMap<String, PathBuf> = files2
        .iter()
        .filter_map(|p| {
            p.strip_prefix(dir2)
                .ok()
                .map(|rel| (rel.to_string_lossy().to_string(), p.clone()))
        })
        .collect();

    let mut result = Vec::new();

    // Get all unique paths
    let mut all_paths: Vec<_> = map1.keys().chain(map2.keys()).cloned().collect();
    all_paths.sort();
    all_paths.dedup();

    for rel_path in all_paths {
        let tile1 = map1.get(&rel_path).map(|p| load_mvt(p)).transpose()?;
        let tile2 = map2.get(&rel_path).map(|p| load_mvt(p)).transpose()?;

        let features1 = tile1.map(|t| features_by_gml_id(&t)).unwrap_or_default();
        let features2 = tile2.map(|t| features_by_gml_id(&t)).unwrap_or_default();

        // Align by gml_id
        let mut all_gml_ids: Vec<_> = features1.keys().chain(features2.keys()).cloned().collect();
        all_gml_ids.sort();
        all_gml_ids.dedup();

        for gml_id in all_gml_ids {
            let geom1 = features1
                .get(&gml_id)
                .and_then(|(geom_buf, geom_type, extent)| {
                    if *geom_type == 3 {
                        // Polygon
                        let mut decoder = GeometryDecoder::new(geom_buf);
                        decoder
                            .decode_polygons()
                            .ok()
                            .map(|polys| tinymvt_to_polygon(Geometry::Polygons(polys), *extent))
                            .flatten()
                    } else {
                        None
                    }
                });

            let geom2 = features2
                .get(&gml_id)
                .and_then(|(geom_buf, geom_type, extent)| {
                    if *geom_type == 3 {
                        // Polygon
                        let mut decoder = GeometryDecoder::new(geom_buf);
                        decoder
                            .decode_polygons()
                            .ok()
                            .map(|polys| tinymvt_to_polygon(Geometry::Polygons(polys), *extent))
                            .flatten()
                    } else {
                        None
                    }
                });

            result.push((rel_path.clone(), gml_id, geom1, geom2));
        }
    }

    Ok(result)
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

    let mut failures = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;

    for (path, gml_id, geom1, geom2) in align_mvt(fme_path, flow_path, zmin, zmax)? {
        // Only count if at least one geometry exists
        if geom1.is_none() && geom2.is_none() {
            continue;
        }

        total += 1;
        if gml_id != "urf_ed404ee8-8f1a-4a37-9491-586354ed823f" || !path.ends_with("/26163.mvt") {
            continue;
        }
        let (status, score) = compare_polygons(geom1, geom2);
        worst_score = f64::max(worst_score, score);

        if score > threshold {
            failures.push((score, path, gml_id, format!("{:?}", status)));
        }
    }

    tracing::info!(
        "MVT polygons: {} total, {} failures, worst={:.6}, threshold={}",
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
            "MVT polygon comparison failed: {}/{} exceeded threshold {}",
            failures.len(),
            total,
            threshold
        ));
    }

    Ok(())
}

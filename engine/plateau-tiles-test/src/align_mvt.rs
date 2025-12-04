use prost::Message;
use reearth_flow_geometry::types::multi_line_string::MultiLineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tinymvt::geometry::{Geometry, GeometryDecoder};
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::polygon::Polygon2D;

/// Geometry type enum for MVT features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    LineString = 2,
    Polygon = 3,
}

/// Aligned geometry pair - can be polygon or linestring
#[derive(Debug)]
pub enum AlignedGeometry {
    Polygon(Option<MultiPolygon2D<f64>>, Option<MultiPolygon2D<f64>>),
    LineString(
        Option<MultiLineString2D<f64>>,
        Option<MultiLineString2D<f64>>,
    ),
}

/// Result of aligning MVT features by gml_id
pub struct AlignedFeature {
    pub tile_path: String,
    pub gml_id: String,
    pub geometry: AlignedGeometry,
}

/// Converts tinymvt Geometry to our internal polygon type, normalizing by tile extent
pub fn tinymvt_to_polygon(geom: Geometry, extent: u32) -> Option<MultiPolygon2D<f64>> {
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

/// Converts tinymvt Geometry to our internal linestring type, normalizing by tile extent
pub fn tinymvt_to_linestring(geom: Geometry, extent: u32) -> Option<MultiLineString2D<f64>> {
    let scale = 1.0 / extent as f64;

    match geom {
        Geometry::LineStrings(linestrings) => {
            let line_vec: Vec<LineString2D<f64>> = linestrings
                .into_iter()
                .filter_map(|points| {
                    if points.is_empty() {
                        return None;
                    }
                    let coords: Vec<Coordinate2D<f64>> = points
                        .iter()
                        .map(|p| Coordinate2D::new_(p[0] as f64 * scale, p[1] as f64 * scale))
                        .collect();
                    Some(LineString2D::new(coords))
                })
                .collect();

            if line_vec.is_empty() {
                None
            } else {
                Some(MultiLineString2D::new(line_vec))
            }
        }
        _ => None,
    }
}

/// Loads a single MVT tile
fn load_mvt(path: &Path) -> Result<Tile, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read MVT file {:?}: {}", path, e))?;
    Tile::decode(&data[..]).map_err(|e| format!("Failed to decode MVT protobuf: {}", e))
}

/// Extracts features by gml_id from a tile, filtering by geometry type
fn features_by_gml_id(
    tile: &Tile,
    filter_geom_type: Option<GeometryType>,
) -> HashMap<String, (Vec<u32>, i32, u32)> {
    let mut result = HashMap::new();

    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        let extent = layer.extent.unwrap_or(4096);

        for feature in &layer.features {
            let geom_type = feature.r#type.unwrap_or(0);

            // Filter by geometry type if specified
            if let Some(filter_type) = filter_geom_type {
                if geom_type != filter_type as i32 {
                    continue;
                }
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
/// Returns aligned features with their geometries decoded
pub fn align_mvt_features(
    dir1: &Path,
    dir2: &Path,
    geometry_type: GeometryType,
    zmin: Option<u32>,
    zmax: Option<u32>,
) -> Result<Vec<AlignedFeature>, String> {
    // Collect all .mvt files
    let mut files1: Vec<PathBuf> = WalkDir::new(dir1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut files2: Vec<PathBuf> = WalkDir::new(dir2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
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

        let features1 = tile1
            .map(|t| features_by_gml_id(&t, Some(geometry_type)))
            .unwrap_or_default();
        let features2 = tile2
            .map(|t| features_by_gml_id(&t, Some(geometry_type)))
            .unwrap_or_default();

        // Align by gml_id
        let mut all_gml_ids: Vec<_> = features1.keys().chain(features2.keys()).cloned().collect();
        all_gml_ids.sort();
        all_gml_ids.dedup();

        for gml_id in all_gml_ids {
            let aligned_geom = match geometry_type {
                GeometryType::Polygon => {
                    let geom1 = decode_polygon_from_features(&features1, &gml_id);
                    let geom2 = decode_polygon_from_features(&features2, &gml_id);
                    AlignedGeometry::Polygon(geom1, geom2)
                }
                GeometryType::LineString => {
                    let geom1 = decode_linestring_from_features(&features1, &gml_id);
                    let geom2 = decode_linestring_from_features(&features2, &gml_id);
                    AlignedGeometry::LineString(geom1, geom2)
                }
            };

            result.push(AlignedFeature {
                tile_path: rel_path.clone(),
                gml_id,
                geometry: aligned_geom,
            });
        }
    }

    Ok(result)
}

fn decode_polygon_from_features(
    features: &HashMap<String, (Vec<u32>, i32, u32)>,
    gml_id: &str,
) -> Option<MultiPolygon2D<f64>> {
    features
        .get(gml_id)
        .and_then(|(geom_buf, geom_type, extent)| {
            if *geom_type == GeometryType::Polygon as i32 {
                let mut decoder = GeometryDecoder::new(geom_buf);
                decoder
                    .decode_polygons()
                    .ok()
                    .and_then(|polys| tinymvt_to_polygon(Geometry::Polygons(polys), *extent))
            } else {
                None
            }
        })
}

fn decode_linestring_from_features(
    features: &HashMap<String, (Vec<u32>, i32, u32)>,
    gml_id: &str,
) -> Option<MultiLineString2D<f64>> {
    features
        .get(gml_id)
        .and_then(|(geom_buf, geom_type, extent)| {
            if *geom_type == GeometryType::LineString as i32 {
                let mut decoder = GeometryDecoder::new(geom_buf);
                decoder
                    .decode_linestrings()
                    .ok()
                    .and_then(|lines| tinymvt_to_linestring(Geometry::LineStrings(lines), *extent))
            } else {
                None
            }
        })
}

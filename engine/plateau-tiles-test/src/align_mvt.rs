use crate::compare_attributes::make_feature_key;
use crate::test_mvt_attributes::tinymvt_value_to_json;
use prost::Message;
use reearth_flow_geometry::types::multi_line_string::MultiLineString2D;
use reearth_flow_geometry::types::multi_point::MultiPoint2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tinymvt::geometry::{Geometry, GeometryDecoder};
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::point::Point2D;
use reearth_flow_geometry::types::polygon::Polygon2D;

/// Geometry type enum for MVT features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    Point = 1,
    LineString = 2,
    Polygon = 3,
}

/// Aligned geometry pair - can be polygon, linestring, or point
#[derive(Debug)]
pub enum AlignedGeometry {
    Point(Option<MultiPoint2D<f64>>, Option<MultiPoint2D<f64>>),
    Polygon(Option<MultiPolygon2D<f64>>, Option<MultiPolygon2D<f64>>),
    LineString(
        Option<MultiLineString2D<f64>>,
        Option<MultiLineString2D<f64>>,
    ),
}

/// Result of aligning MVT features by ident
pub struct AlignedFeature {
    pub tile_path: String,
    pub ident: String,
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

/// Converts tinymvt Geometry to our internal multipoint type, normalizing by tile extent
pub fn tinymvt_to_multipoint(geom: Geometry, extent: u32) -> Option<MultiPoint2D<f64>> {
    let scale = 1.0 / extent as f64;

    match geom {
        Geometry::Points(points) => {
            if points.is_empty() {
                return None;
            }
            let point_vec: Vec<Point2D<f64>> = points
                .iter()
                .map(|p| Point2D::from((p[0] as f64 * scale, p[1] as f64 * scale)))
                .collect();
            Some(MultiPoint2D::from(point_vec))
        }
        _ => None,
    }
}

/// Loads a single MVT tile
fn load_mvt(path: &Path) -> Result<Tile, String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read MVT file {:?}: {}", path, e))?;
    Tile::decode(&data[..]).map_err(|e| format!("Failed to decode MVT protobuf: {}", e))
}

/// Extracts features by ident from a tile, filtering by geometry type
fn features_by_ident(
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

            // Decode tags to JSON object
            let tags = tags_decoder
                .decode(&feature.tags)
                .expect("Failed to decode MVT feature tags");

            let mut props = serde_json::Map::new();
            for (key, value) in tags {
                let json_value = tinymvt_value_to_json(&value);
                props.insert(key.to_string(), json_value);
            }
            let props_value = Value::Object(props);

            // Generate feature key using the same logic as attribute tests
            let feature_key = make_feature_key(&props_value, None);

            // Store the geometry buffer, geometry type, and extent
            result.insert(feature_key, (feature.geometry.clone(), geom_type, extent));
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
            .map(|t| features_by_ident(&t, Some(geometry_type)))
            .unwrap_or_default();
        let features2 = tile2
            .map(|t| features_by_ident(&t, Some(geometry_type)))
            .unwrap_or_default();

        // Align by ident
        let mut all_idents: Vec<_> = features1.keys().chain(features2.keys()).cloned().collect();
        all_idents.sort();
        all_idents.dedup();

        for ident in all_idents {
            let aligned_geom = match geometry_type {
                GeometryType::Point => {
                    let geom1 = decode_point_from_features(&features1, &ident);
                    let geom2 = decode_point_from_features(&features2, &ident);
                    AlignedGeometry::Point(geom1, geom2)
                }
                GeometryType::Polygon => {
                    let geom1 = decode_polygon_from_features(&features1, &ident);
                    let geom2 = decode_polygon_from_features(&features2, &ident);
                    AlignedGeometry::Polygon(geom1, geom2)
                }
                GeometryType::LineString => {
                    let geom1 = decode_linestring_from_features(&features1, &ident);
                    let geom2 = decode_linestring_from_features(&features2, &ident);
                    AlignedGeometry::LineString(geom1, geom2)
                }
            };

            result.push(AlignedFeature {
                tile_path: rel_path.clone(),
                ident,
                geometry: aligned_geom,
            });
        }
    }

    Ok(result)
}

fn decode_polygon_from_features(
    features: &HashMap<String, (Vec<u32>, i32, u32)>,
    ident: &str,
) -> Option<MultiPolygon2D<f64>> {
    features
        .get(ident)
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
    ident: &str,
) -> Option<MultiLineString2D<f64>> {
    features
        .get(ident)
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

fn decode_point_from_features(
    features: &HashMap<String, (Vec<u32>, i32, u32)>,
    ident: &str,
) -> Option<MultiPoint2D<f64>> {
    features
        .get(ident)
        .and_then(|(geom_buf, geom_type, extent)| {
            if *geom_type == GeometryType::Point as i32 {
                let mut decoder = GeometryDecoder::new(geom_buf);
                decoder
                    .decode_points()
                    .ok()
                    .and_then(|points| tinymvt_to_multipoint(Geometry::Points(points), *extent))
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_features_by_ident() {
        // Test MVT with 2 polygon features (Zone layer, extent 65536)
        let mvt_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/test_zone_2features.mvt");
        let tile = load_mvt(&mvt_path).expect("Failed to load test MVT");
        let features = features_by_ident(&tile, Some(GeometryType::Polygon));

        assert_eq!(features.len(), 2, "Expected 2 polygon features");
        let expected_ids = [
            "area_d14a8141-6597-42ed-a5b4-6232986df2f4",
            "area_e6b0bbf8-07f3-4afa-afa5-1a9b71ac777f",
        ];
        for id in &expected_ids {
            let (geom_buf, geom_type, extent) = features.get(*id).expect("Missing gml_id");
            assert_eq!(*geom_type, GeometryType::Polygon as i32);
            assert_eq!(*extent, 65536);
            let mut decoder = GeometryDecoder::new(geom_buf);
            let polys = decoder.decode_polygons().expect("Failed to decode");
            let multi_poly =
                tinymvt_to_polygon(Geometry::Polygons(polys), *extent).expect("Failed to convert");
            assert!(!multi_poly.0.is_empty());
        }
    }
}

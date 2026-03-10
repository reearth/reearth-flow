use crate::compare_attributes::make_feature_key;
use crate::conv::mvt::tinymvt_value_to_json;
use crate::rasterize::{
    draw_aa_circle, draw_capsule, draw_wu_line, scanline_fill, write_raster_png, RASTER_SIZE,
    STROKE_PIXELS,
};
use prost::Message;
use std::fs;
use std::path::Path;
use tinymvt::geometry::GeometryDecoder;
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

/// Rasterizes all geometry in a tile for a given feature ident into a RASTER_SIZE x RASTER_SIZE f32 raster.
/// Lines: capsule SDF per segment (analytically correct round joins/caps).
/// Polygon interiors: scanline fill. Polygon boundaries: Wu line. Points: circle SDF.
pub fn rasterize_tile_feature(tile: &Tile, ident: &str) -> Vec<f32> {
    let mut raster = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];

    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        let extent = layer.extent.unwrap_or(4096);
        let scale = 1.0 / extent as f64;

        for feature in &layer.features {
            let tags = match tags_decoder.decode(&feature.tags) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let mut props = serde_json::Map::new();
            for (key, value) in tags {
                props.insert(key.to_string(), tinymvt_value_to_json(&value));
            }
            let props_value = serde_json::Value::Object(props);
            let feature_key = make_feature_key(&props_value, None);
            if feature_key != ident {
                continue;
            }

            let geom_type = feature.r#type.unwrap_or(0);
            let mut decoder = GeometryDecoder::new(&feature.geometry);

            match geom_type {
                1 => {
                    // Point
                    if let Ok(points) = decoder.decode_points() {
                        for point in &points {
                            let x = point[0] as f64 * scale * RASTER_SIZE as f64;
                            let y = point[1] as f64 * scale * RASTER_SIZE as f64;
                            draw_aa_circle(&mut raster, x, y, STROKE_PIXELS);
                        }
                    }
                }
                2 => {
                    // LineString: capsule SDF per segment for correct round joins/caps
                    if let Ok(linestrings) = decoder.decode_linestrings() {
                        for ls in &linestrings {
                            for window in ls.windows(2) {
                                let x0 = window[0][0] as f64 * scale * RASTER_SIZE as f64;
                                let y0 = window[0][1] as f64 * scale * RASTER_SIZE as f64;
                                let x1 = window[1][0] as f64 * scale * RASTER_SIZE as f64;
                                let y1 = window[1][1] as f64 * scale * RASTER_SIZE as f64;
                                draw_capsule(&mut raster, x0, y0, x1, y1, STROKE_PIXELS);
                            }
                        }
                    }
                }
                3 => {
                    // Polygon: scanline fill interior + Wu boundary
                    if let Ok(polygons) = decoder.decode_polygons() {
                        for rings in &polygons {
                            if rings.is_empty() {
                                continue;
                            }
                            let pixel_rings: Vec<Vec<(f64, f64)>> = rings
                                .iter()
                                .map(|ring| {
                                    ring.iter()
                                        .map(|p| {
                                            (
                                                p[0] as f64 * scale * RASTER_SIZE as f64,
                                                p[1] as f64 * scale * RASTER_SIZE as f64,
                                            )
                                        })
                                        .collect()
                                })
                                .collect();

                            scanline_fill(&mut raster, &pixel_rings);

                            for ring in &pixel_rings {
                                for window in ring.windows(2) {
                                    draw_wu_line(
                                        &mut raster,
                                        window[0].0,
                                        window[0].1,
                                        window[1].0,
                                        window[1].1,
                                    );
                                }
                                if ring.len() >= 2 {
                                    let last = ring[ring.len() - 1];
                                    let first = ring[0];
                                    draw_wu_line(&mut raster, last.0, last.1, first.0, first.1);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    raster
}

/// Returns all unique feature keys present in a tile (across all layers and geometry types).
pub fn make_feature_keys_in_tile(tile: &Tile) -> Vec<String> {
    let mut keys = Vec::new();
    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        for feature in &layer.features {
            let tags = match tags_decoder.decode(&feature.tags) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let mut props = serde_json::Map::new();
            for (key, value) in tags {
                props.insert(key.to_string(), tinymvt_value_to_json(&value));
            }
            let key = make_feature_key(&serde_json::Value::Object(props), None);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    keys
}

/// For each .mvt file under `mvt_dir`, rasterizes each feature individually and writes a PNG
/// under `truth_dir/<rel_tile_path>/<ident>.png`.
///
/// If `tiles` is `Some`, only tiles whose z/x/y path (no extension) appears in the list are
/// processed.
pub fn write_png_truth(
    mvt_dir: &Path,
    truth_dir: &Path,
    tiles: Option<&[String]>,
) -> Result<(), String> {
    if !mvt_dir.exists() {
        return Err(format!("MVT directory does not exist: {:?}", mvt_dir));
    }
    for entry in WalkDir::new(mvt_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
    {
        let path = entry.path();
        let rel = path
            .strip_prefix(mvt_dir)
            .map_err(|e| e.to_string())?
            .with_extension("");

        if let Some(tile_list) = tiles {
            let rel_str = rel.to_string_lossy();
            if !tile_list.iter().any(|t| t == rel_str.as_ref()) {
                continue;
            }
        }

        let data = fs::read(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
        let tile = Tile::decode(&data[..])
            .map_err(|e| format!("Failed to decode MVT {:?}: {}", path, e))?;

        let idents = make_feature_keys_in_tile(&tile);

        for ident in &idents {
            let raster = rasterize_tile_feature(&tile, ident);

            if raster.iter().all(|&v| v == 0.0) {
                continue;
            }

            let safe_ident = sanitize_filename(ident);
            let png_path = truth_dir.join(&rel).join(format!("{}.png", safe_ident));
            write_raster_png(&raster, &png_path)?;
        }
    }

    Ok(())
}

/// Returns the expected PNG path for a given (rel_tile_path, ident) pair.
pub fn truth_png_path(truth_dir: &Path, rel_tile_path: &str, ident: &str) -> std::path::PathBuf {
    let rel = std::path::Path::new(rel_tile_path).with_extension("");
    let safe_ident = sanitize_filename(ident);
    truth_dir.join(rel).join(format!("{}.png", safe_ident))
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

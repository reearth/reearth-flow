use crate::compare_attributes::make_feature_key;
use crate::conv::mvt::tinymvt_value_to_json;
use crate::rasterize::Canvas;
use prost::Message;
use std::fs;
use std::path::Path;
use tinymvt::geometry::GeometryDecoder;
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

/// Renders a single MVT feature's geometry into `canvas`.
fn render_feature(
    canvas: &mut Canvas,
    feature: &tinymvt::vector_tile::tile::Feature,
    scale: f64,
    width: usize,
    height: usize,
    stroke: f64,
) {
    let geom_type = feature.r#type.unwrap_or(0);
    let mut decoder = GeometryDecoder::new(&feature.geometry);
    let s = |v: i32, dim: usize| v as f64 * scale * dim as f64;

    match geom_type {
        1 => {
            if let Ok(points) = decoder.decode_points() {
                for p in &points {
                    canvas.draw_aa_circle(s(p[0], width), s(p[1], height), stroke);
                }
            }
        }
        2 => {
            if let Ok(linestrings) = decoder.decode_linestrings() {
                for ls in &linestrings {
                    for w in ls.windows(2) {
                        canvas.draw_capsule(
                            s(w[0][0], width),
                            s(w[0][1], height),
                            s(w[1][0], width),
                            s(w[1][1], height),
                            stroke,
                        );
                    }
                }
            }
        }
        3 => {
            if let Ok(polygons) = decoder.decode_polygons() {
                for rings in &polygons {
                    if rings.is_empty() {
                        continue;
                    }
                    let px: Vec<Vec<(f64, f64)>> = rings
                        .iter()
                        .map(|r| {
                            r.iter()
                                .map(|p| (s(p[0], width), s(p[1], height)))
                                .collect()
                        })
                        .collect();
                    canvas.scanline_fill(&px);
                    for ring in &px {
                        for w in ring.windows(2) {
                            canvas.draw_wu_line(w[0].0, w[0].1, w[1].0, w[1].1);
                        }
                        if ring.len() >= 2 {
                            canvas.draw_wu_line(
                                ring[ring.len() - 1].0,
                                ring[ring.len() - 1].1,
                                ring[0].0,
                                ring[0].1,
                            );
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Single-pass rasterization: decodes each feature's tags once and renders directly into
/// per-ident canvases, avoiding the O(N×F) re-scan of the old make_feature_keys + rasterize_tile_feature approach.
fn rasterize_tile_to_canvases(
    tile: &Tile,
    width: usize,
    height: usize,
    stroke: f64,
) -> std::collections::HashMap<String, Canvas> {
    let mut canvases: std::collections::HashMap<String, Canvas> = std::collections::HashMap::new();
    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        let scale = 1.0 / layer.extent.unwrap_or(4096) as f64;
        for feature in &layer.features {
            let tags = match tags_decoder.decode(&feature.tags) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let props: serde_json::Map<_, _> = tags
                .into_iter()
                .map(|(k, v)| (k.to_string(), tinymvt_value_to_json(&v)))
                .collect();
            let ident = make_feature_key(&serde_json::Value::Object(props), None);
            let canvas = canvases
                .entry(ident)
                .or_insert_with(|| Canvas::new(width, height));
            render_feature(canvas, feature, scale, width, height, stroke);
        }
    }
    canvases
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
    width: usize,
    height: usize,
    stroke: f64,
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

        for (ident, canvas) in rasterize_tile_to_canvases(&tile, width, height, stroke) {
            if canvas.is_blank() {
                continue;
            }
            let png_path = truth_dir
                .join(&rel)
                .join(format!("{}.png", sanitize_filename(&ident)));
            canvas.write_png(&png_path)?;
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

use crate::align_mvt::make_feature_keys_in_tile;
use crate::raster::{rasterize_tile_feature, write_raster_png, RASTER_SIZE};
use prost::Message;
use std::fs;
use std::path::Path;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

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

            // Skip empty rasters (feature not found or no geometry)
            if raster.iter().all(|&v| v == 0.0) {
                continue;
            }

            // Sanitize ident for use as filename
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

/// Returns `RASTER_SIZE * RASTER_SIZE` zero raster (used when a feature is absent from one side).
pub fn empty_raster() -> Vec<f32> {
    vec![0.0f32; RASTER_SIZE * RASTER_SIZE]
}

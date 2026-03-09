use crate::mvt_raster::{
    make_feature_keys_in_tile, rasterize_tile_feature, write_raster_png, RASTER_SIZE,
};
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

/// Reads an 8-bit grayscale PNG file into a f32 raster.
pub fn read_raster_png(path: &std::path::Path) -> Result<Vec<f32>, String> {
    use image::GrayImage;
    let img = image::open(path)
        .map_err(|e| format!("Failed to read PNG {:?}: {}", path, e))?
        .into_luma8();
    let GrayImage { .. } = img;
    Ok(img.pixels().map(|p| p.0[0] as f32 / 255.0).collect())
}

/// Pixel-wise RMS comparison between two f32 rasters.
pub fn compare_rasters(r1: &[f32], r2: &[f32]) -> f64 {
    let sum: f64 = r1
        .iter()
        .zip(r2.iter())
        .map(|(a, b)| {
            let diff = ((*a as f64) - (*b as f64)).abs();
            if diff >= 0.5 {
                diff
            } else {
                0.0
            }
        })
        .sum();
    (sum / r1.len() as f64).sqrt()
}

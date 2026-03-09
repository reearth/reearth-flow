use crate::conv_png::{empty_raster, truth_png_path};
use crate::raster::{compare_rasters, rasterize_tile_feature, read_raster_png};
use prost::Message;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tinymvt::vector_tile::Tile;
use walkdir::WalkDir;

use crate::align_mvt::make_feature_keys_in_tile;

#[derive(Debug, Deserialize)]
pub struct RasterConfig {
    pub threshold: Option<f64>,
}

pub fn test_raster(
    truth_dir: &Path,
    flow_mvt_dir: &Path,
    config: &RasterConfig,
    tiles: Option<&[String]>,
) -> Result<(), String> {
    let threshold = config.threshold.unwrap_or(0.0);

    assert!(
        flow_mvt_dir.exists(),
        "flow_mvt_dir does not exist: {:?}",
        flow_mvt_dir
    );
    assert!(
        truth_dir.exists(),
        "truth_dir does not exist: {:?}",
        truth_dir
    );

    // Walk all .mvt files in the flow output
    let mut mvt_files: Vec<_> = WalkDir::new(flow_mvt_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "mvt"))
        .filter(|e| {
            if let Some(tile_list) = tiles {
                let rel = e
                    .path()
                    .strip_prefix(flow_mvt_dir)
                    .ok()
                    .map(|p| p.with_extension(""));
                rel.is_some_and(|r| tile_list.iter().any(|t| t == r.to_string_lossy().as_ref()))
            } else {
                true
            }
        })
        .collect();
    mvt_files.sort_by_key(|e| e.path().to_path_buf());

    // Also collect all truth PNGs to detect features present in truth but missing from flow
    let truth_idents = collect_truth_idents(truth_dir);

    let mut results: Vec<(f64, String, String)> = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;

    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    for entry in &mvt_files {
        let path = entry.path();
        let rel = path
            .strip_prefix(flow_mvt_dir)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        let data = fs::read(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
        let tile = Tile::decode(&data[..])
            .map_err(|e| format!("Failed to decode MVT {:?}: {}", path, e))?;

        let idents = make_feature_keys_in_tile(&tile);

        for ident in &idents {
            seen.insert((rel.clone(), ident.clone()));

            let flow_raster = rasterize_tile_feature(&tile, ident);
            let truth_png = truth_png_path(truth_dir, &rel, ident);

            let truth_raster = if truth_png.exists() {
                read_raster_png(&truth_png)?
            } else {
                empty_raster()
            };

            let score = compare_rasters(&truth_raster, &flow_raster);
            worst_score = f64::max(worst_score, score);
            total += 1;
            results.push((score, rel.clone(), ident.clone()));
        }
    }

    // Features present in truth but absent from flow
    for (rel, ident) in &truth_idents {
        if seen.contains(&(rel.clone(), ident.clone())) {
            continue;
        }
        let truth_png = truth_png_path(truth_dir, rel, ident);
        if !truth_png.exists() {
            continue;
        }
        let truth_raster = read_raster_png(&truth_png)?;
        let score = compare_rasters(&truth_raster, &empty_raster());
        worst_score = f64::max(worst_score, score);
        total += 1;
        results.push((score, rel.clone(), ident.clone()));
    }

    assert!(
        total > 0,
        "no features compared — truth_dir={:?}, flow_mvt_dir={:?}",
        truth_dir,
        flow_mvt_dir
    );

    let failures: Vec<_> = results
        .iter()
        .filter(|(score, _, _)| *score > threshold)
        .collect();

    tracing::info!(
        "Raster: {} total, {} failures, worst={:.6}, threshold={}",
        total,
        failures.len(),
        worst_score,
        threshold
    );

    if !failures.is_empty() {
        let mut sorted = failures.clone();
        sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        tracing::info!("Worst 5 failures:");
        for (score, path, ident) in sorted.iter().take(5) {
            tracing::info!("  {} | {} | {:.6}", path, ident, score);
        }
        return Err(format!(
            "Raster comparison failed: {}/{} exceeded threshold {}",
            failures.len(),
            total,
            threshold
        ));
    } else {
        let mut sorted = results.clone();
        sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        tracing::debug!("Worst 5 scores (all below threshold):");
        for (score, path, ident) in sorted.iter().take(5) {
            tracing::debug!("  {} | {} | {:.6}", path, ident, score);
        }
    }

    Ok(())
}

/// Collects all (rel_tile_path, ident) pairs present in the truth directory.
fn collect_truth_idents(truth_dir: &Path) -> Vec<(String, String)> {
    WalkDir::new(truth_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
        .filter_map(|e| {
            let path = e.path();
            let rel = path.strip_prefix(truth_dir).ok()?;
            // rel = <z>/<x>/<y>/<ident>.png  — parent is the tile path, stem is the ident
            let ident = path.file_stem()?.to_string_lossy().to_string();
            let tile_rel = rel.parent()?.to_string_lossy().to_string() + ".mvt";
            Some((tile_rel, ident))
        })
        .collect()
}

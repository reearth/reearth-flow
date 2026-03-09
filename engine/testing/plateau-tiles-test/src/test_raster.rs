use crate::conv_png::{compare_rasters, empty_raster, read_raster_png};
use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct RasterConfig {
    pub threshold: Option<f64>,
}

pub fn test_raster(
    truth_dir: &Path,
    flow_png_dir: &Path,
    config: &RasterConfig,
) -> Result<(), String> {
    let threshold = config.threshold.unwrap_or(0.0);

    assert!(
        flow_png_dir.exists(),
        "flow_png_dir does not exist: {:?}",
        flow_png_dir
    );
    assert!(
        truth_dir.exists(),
        "truth_dir does not exist: {:?}",
        truth_dir
    );

    let mut flow_pngs: Vec<_> = WalkDir::new(flow_png_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
        .collect();
    flow_pngs.sort_by_key(|e| e.path().to_path_buf());

    let mut results: Vec<(f64, String)> = Vec::new();
    let mut total = 0;
    let mut worst_score = 0.0;
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry in &flow_pngs {
        let path = entry.path();
        let rel = path
            .strip_prefix(flow_png_dir)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        seen.insert(rel.clone());

        let flow_raster = read_raster_png(path)?;
        let truth_png = truth_dir.join(&rel);
        let truth_raster = if truth_png.exists() {
            read_raster_png(&truth_png)?
        } else {
            empty_raster()
        };

        let score = compare_rasters(&truth_raster, &flow_raster);
        worst_score = f64::max(worst_score, score);
        total += 1;
        results.push((score, rel));
    }

    // PNGs present in truth but absent from flow
    for entry in WalkDir::new(truth_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
    {
        let path = entry.path();
        let rel = path
            .strip_prefix(truth_dir)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        if seen.contains(&rel) {
            continue;
        }
        let truth_raster = read_raster_png(path)?;
        let score = compare_rasters(&truth_raster, &empty_raster());
        worst_score = f64::max(worst_score, score);
        total += 1;
        results.push((score, rel));
    }

    assert!(
        total > 0,
        "no PNGs compared — truth_dir={:?}, flow_png_dir={:?}",
        truth_dir,
        flow_png_dir
    );

    let failures: Vec<_> = results
        .iter()
        .filter(|(score, _)| *score > threshold)
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
        for (score, path) in sorted.iter().take(5) {
            tracing::info!("  {} | {:.6}", path, score);
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
        for (score, path) in sorted.iter().take(5) {
            tracing::debug!("  {} | {:.6}", path, score);
        }
    }

    Ok(())
}

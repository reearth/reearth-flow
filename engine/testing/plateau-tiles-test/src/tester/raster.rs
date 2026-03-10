use crate::rasterize::{Canvas, RasterSize};
use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct RasterConfig {
    pub threshold: Option<f64>,
    #[serde(default)]
    pub size: RasterSize,
}

pub fn test_raster(
    truth_dir: &Path,
    flow_png_dir: &Path,
    config: &RasterConfig,
) -> Result<(), String> {
    let threshold = config.threshold.unwrap_or(0.0);
    let (default_width, default_height) = config.size.dimensions();

    if !flow_png_dir.exists() {
        return Err(format!("flow_png_dir does not exist: {:?}", flow_png_dir));
    }
    if !truth_dir.exists() {
        return Err(format!("truth_dir does not exist: {:?}", truth_dir));
    }

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

        let flow_canvas = Canvas::read_png(path)?;
        let truth_png = truth_dir.join(&rel);
        let truth_canvas = if truth_png.exists() {
            Canvas::read_png(&truth_png)?
        } else {
            Canvas::new(flow_canvas.width, flow_canvas.height)
        };

        let score = flow_canvas
            .compare(&truth_canvas)
            .map_err(|e| format!("{}: {}", rel, e))?;
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
        let truth_canvas = Canvas::read_png(path)?;
        let empty = Canvas::new(default_width, default_height);
        let score = truth_canvas
            .compare(&empty)
            .map_err(|e| format!("{}: {}", rel, e))?;
        worst_score = f64::max(worst_score, score);
        total += 1;
        results.push((score, rel));
    }

    if total == 0 {
        return Err(format!(
            "no PNG: truth={:?}, flow={:?}",
            truth_dir, flow_png_dir
        ));
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rasterize::{Canvas, RasterSize};

    fn config(threshold: f64, size: usize) -> RasterConfig {
        RasterConfig {
            threshold: Some(threshold),
            size: RasterSize::Square(size),
        }
    }

    fn write_canvas(dir: &std::path::Path, name: &str, w: usize, h: usize, fill: f32) {
        let mut c = Canvas::new(w, h);
        c.data.iter_mut().for_each(|v| *v = fill);
        c.write_png(&dir.join(name)).unwrap();
    }

    #[test]
    fn flow_dir_missing() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        std::fs::create_dir_all(&truth).unwrap();
        let err = test_raster(&truth, &td.path().join("nope"), &config(0.0, 16)).unwrap_err();
        assert!(err.contains("flow_png_dir does not exist"), "{err}");
    }

    #[test]
    fn truth_dir_missing() {
        let td = tempfile::TempDir::new().unwrap();
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&flow).unwrap();
        let err = test_raster(&td.path().join("nope"), &flow, &config(0.0, 16)).unwrap_err();
        assert!(err.contains("truth_dir does not exist"), "{err}");
    }

    #[test]
    fn no_pngs_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        std::fs::write(flow.join("not_a_png.txt"), b"data").unwrap();
        let err = test_raster(&truth, &flow, &config(0.0, 16)).unwrap_err();
        assert!(err.contains("no PNG"), "{err}");
    }

    #[test]
    fn corrupt_flow_png_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        std::fs::write(flow.join("tile.png"), b"not a png at all").unwrap();
        let err = test_raster(&truth, &flow, &config(0.0, 16)).unwrap_err();
        assert!(err.contains("Failed to read PNG"), "{err}");
    }

    #[test]
    fn size_mismatch_flow_vs_truth_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        write_canvas(&flow, "tile.png", 32, 32, 0.0);
        write_canvas(&truth, "tile.png", 16, 16, 0.0);
        let err = test_raster(&truth, &flow, &config(0.0, 16)).unwrap_err();
        assert!(err.contains("Canvas size mismatch"), "{err}");
    }

    // truth-only PNG: compared against an empty canvas of the config size.
    // If the truth PNG dimensions differ from config size → size mismatch.
    #[test]
    fn size_mismatch_truth_only_vs_empty_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        // truth PNG is 32×32 but config size (empty canvas) is 16×16
        write_canvas(&truth, "tile.png", 32, 32, 0.0);
        let err = test_raster(&truth, &flow, &config(0.0, 16)).unwrap_err();
        assert!(err.contains("Canvas size mismatch"), "{err}");
    }

    #[test]
    fn threshold_exceeded_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        // flow=white (1.0), truth=black (0.0): diff=1.0 per pixel ≥ 0.5 → score=1.0 > threshold=0.0
        write_canvas(&flow, "tile.png", 16, 16, 1.0);
        write_canvas(&truth, "tile.png", 16, 16, 0.0);
        let err = test_raster(&truth, &flow, &config(0.0, 16)).unwrap_err();
        assert!(err.contains("Raster comparison failed"), "{err}");
    }
}

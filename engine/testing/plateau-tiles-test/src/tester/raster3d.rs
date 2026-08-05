use crate::rasterize::Canvas;
use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
pub struct Raster3dConfig {
    pub threshold: f64,
}

pub fn test_raster3d(
    truth_dir: &Path,
    flow_png_dir: &Path,
    config: &Raster3dConfig,
) -> Result<(), String> {
    let threshold = config.threshold;

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

        let flow_canvas = Canvas::read_png_f32(path)?;
        let truth_png = truth_dir.join(&rel);
        if !truth_png.exists() {
            return Err(format!("truth PNG missing for {}: {:?}", rel, truth_png));
        }
        let truth_canvas = Canvas::read_png_f32(&truth_png)?;

        let score = flow_canvas
            .compare_depth(&truth_canvas)
            .map_err(|e| format!("{}: {}", rel, e))?;
        worst_score = f64::max(worst_score, score);
        total += 1;
        results.push((score, rel));
    }

    // PNGs present in truth but absent from flow: a camera that flow simply
    // failed to render is a real failure, not something to score leniently.
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
        results.push((f64::INFINITY, rel));
        total += 1;
        worst_score = f64::INFINITY;
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
        "Raster3d: {} total, {} failures, worst={:.6}, threshold={}",
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
            "Raster3d comparison failed: {}/{} exceeded threshold {}",
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

    fn config(threshold: f64) -> Raster3dConfig {
        Raster3dConfig { threshold }
    }

    fn write_canvas(dir: &std::path::Path, name: &str, w: usize, h: usize, fill: f32) {
        let mut c = Canvas::new(w, h);
        c.data.iter_mut().for_each(|v| *v = fill);
        c.write_png_f32(&dir.join(name)).unwrap();
    }

    #[test]
    fn flow_dir_missing() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        std::fs::create_dir_all(&truth).unwrap();
        let err = test_raster3d(&truth, &td.path().join("nope"), &config(0.0)).unwrap_err();
        assert!(err.contains("flow_png_dir does not exist"), "{err}");
    }

    #[test]
    fn truth_dir_missing() {
        let td = tempfile::TempDir::new().unwrap();
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&flow).unwrap();
        let err = test_raster3d(&td.path().join("nope"), &flow, &config(0.0)).unwrap_err();
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
        let err = test_raster3d(&truth, &flow, &config(0.0)).unwrap_err();
        assert!(err.contains("no PNG"), "{err}");
    }

    #[test]
    fn missing_truth_counterpart_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        write_canvas(&flow, "top_down.png", 8, 8, 0.0);
        let err = test_raster3d(&truth, &flow, &config(0.0)).unwrap_err();
        assert!(err.contains("truth PNG missing"), "{err}");
    }

    #[test]
    fn identical_canvases_pass() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        write_canvas(&flow, "top_down.png", 8, 8, 5.0);
        write_canvas(&truth, "top_down.png", 8, 8, 5.0);
        test_raster3d(&truth, &flow, &config(0.0)).unwrap();
    }

    #[test]
    fn threshold_exceeded_returns_err() {
        let td = tempfile::TempDir::new().unwrap();
        let truth = td.path().join("truth");
        let flow = td.path().join("flow");
        std::fs::create_dir_all(&truth).unwrap();
        std::fs::create_dir_all(&flow).unwrap();
        write_canvas(&flow, "top_down.png", 8, 8, 5.0);
        write_canvas(&truth, "top_down.png", 8, 8, 500.0);
        let err = test_raster3d(&truth, &flow, &config(0.0)).unwrap_err();
        assert!(err.contains("Raster3d comparison failed"), "{err}");
    }
}

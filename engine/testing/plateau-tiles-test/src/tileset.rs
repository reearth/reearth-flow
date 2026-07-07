//! Walks a Cesium 3D Tiles explicit tile tree (`content`/`contents`/
//! `children` in `tileset.json`), collecting every referenced `.glb` path
//! and the `geometricError` of the tile node it came from. Shared by
//! `conv::cesium` and `align_cesium`, which both need this.

use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct TileContent {
    pub path: PathBuf,
    pub geometric_error: f64,
}

/// `tile` is a tileset tile node — typically `tileset.json`'s `root`, as
/// obtained via `TilesetInfo::content.get("root")`.
pub fn collect_tile_contents(tileset_dir: &Path, tile: &Value) -> Result<Vec<TileContent>, String> {
    let mut out = Vec::new();
    collect(tileset_dir, tile, &mut out)?;
    Ok(out)
}

fn collect(tileset_dir: &Path, tile: &Value, out: &mut Vec<TileContent>) -> Result<(), String> {
    let geometric_error = tile
        .get("geometricError")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "Missing or invalid geometricError in tile".to_string())?;

    for uri in glb_content_uris(tile) {
        let path = tileset_dir.join(&uri);
        if !path.exists() {
            return Err(format!(
                "GLB file referenced in tileset does not exist: {:?}",
                path
            ));
        }
        out.push(TileContent { path, geometric_error });
    }

    if let Some(children) = tile.get("children").and_then(|c| c.as_array()) {
        for child in children {
            collect(tileset_dir, child, out)?;
        }
    }

    Ok(())
}

/// `content`/`contents[]` URIs on one tile node, filtered to `.glb`.
fn glb_content_uris(tile: &Value) -> Vec<String> {
    let mut uris = Vec::new();

    if let Some(uri) = tile
        .get("content")
        .and_then(|c| c.get("uri"))
        .and_then(|u| u.as_str())
    {
        if uri.ends_with(".glb") {
            uris.push(uri.to_string());
        }
    }

    if let Some(contents) = tile.get("contents").and_then(|c| c.as_array()) {
        for item in contents {
            if let Some(uri) = item.get("uri").and_then(|u| u.as_str()) {
                if uri.ends_with(".glb") {
                    uris.push(uri.to_string());
                }
            }
        }
    }

    uris
}

//! Walks a Cesium 3D Tiles tileset, collecting every referenced `.glb` path
//! and the `geometricError` of the tile it came from. Shared by
//! `conv::cesium` and `align_cesium`.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TileContent {
    pub path: PathBuf,
    pub geometric_error: f64,
}

/// `tile` is a tileset tile node — typically `tileset.json`'s `root`, as
/// obtained via `TilesetInfo::content.get("root")`. Dispatches to an
/// explicit tile-tree walk or, when `tile` carries `implicitTiling`, a
/// 3D Tiles 1.1 implicit-tiling read.
pub fn collect_tile_contents(tileset_dir: &Path, tile: &Value) -> Result<Vec<TileContent>, String> {
    match tile.get("implicitTiling") {
        Some(implicit) => collect_implicit(tileset_dir, tile, implicit),
        None => {
            let mut out = Vec::new();
            collect_explicit(tileset_dir, tile, &mut out)?;
            Ok(out)
        }
    }
}

fn collect_explicit(
    tileset_dir: &Path,
    tile: &Value,
    out: &mut Vec<TileContent>,
) -> Result<(), String> {
    let geometric_error = tile
        .get("geometricError")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "Missing or invalid geometricError in tile".to_string())?;

    for uri in glb_content_uris(tile) {
        out.push(TileContent {
            path: resolve_existing(tileset_dir, &uri)?,
            geometric_error,
        });
    }

    if let Some(children) = tile.get("children").and_then(|c| c.as_array()) {
        for child in children {
            collect_explicit(tileset_dir, child, out)?;
        }
    }

    Ok(())
}

fn resolve_existing(tileset_dir: &Path, uri: &str) -> Result<PathBuf, String> {
    let path = tileset_dir.join(uri);
    if !path.exists() {
        return Err(format!(
            "File referenced in tileset does not exist: {:?}",
            path
        ));
    }
    Ok(path)
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

/// QUADTREE-only, and reads only the root `.subtree` file — chaining across
/// a subtree boundary via `childSubtreeAvailability` is unimplemented. Both
/// limits surface as an `Err`.
fn collect_implicit(
    tileset_dir: &Path,
    root: &Value,
    implicit: &Value,
) -> Result<Vec<TileContent>, String> {
    match implicit.get("subdivisionScheme").and_then(|v| v.as_str()) {
        Some("QUADTREE") => {}
        Some(other) => {
            return Err(format!(
                "Unsupported implicitTiling subdivisionScheme: {other}"
            ))
        }
        None => return Err("implicitTiling missing subdivisionScheme".to_string()),
    }

    let subtree_levels = implicit
        .get("subtreeLevels")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "implicitTiling missing subtreeLevels".to_string())? as u32;

    // `availableLevels` beyond `subtreeLevels` implies chaining into child
    // subtree files, which this reader doesn't follow.
    if let Some(available_levels) = implicit.get("availableLevels").and_then(|v| v.as_u64()) {
        if available_levels as u32 != subtree_levels {
            return Err(format!(
                "implicitTiling.availableLevels ({available_levels}) exceeds subtreeLevels \
                 ({subtree_levels}); reading chained subtree files is unimplemented"
            ));
        }
    }

    let content_template = root
        .get("content")
        .and_then(|c| c.get("uri"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| "implicit root tile missing content.uri template".to_string())?;

    let subtree_template = implicit
        .get("subtrees")
        .and_then(|s| s.get("uri"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| "implicitTiling missing subtrees.uri template".to_string())?;

    let root_geometric_error = root
        .get("geometricError")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| "implicit root tile missing geometricError".to_string())?;

    let subtree_path = resolve_existing(tileset_dir, &expand_template(subtree_template, 0, 0, 0))?;
    let subtree_bytes = fs::read(&subtree_path)
        .map_err(|e| format!("Failed to read subtree file {:?}: {}", subtree_path, e))?;
    let content_available = parse_subtree_content_availability(&subtree_bytes)?;

    let mut out = Vec::new();
    for level in 0..subtree_levels {
        let n = 1u32 << level;
        for y in 0..n {
            for x in 0..n {
                let bit = level_offset(level) + morton2d(x, y);
                if !content_available.get(bit) {
                    continue;
                }
                let uri = expand_template(content_template, level, x, y);
                out.push(TileContent {
                    path: resolve_existing(tileset_dir, &uri)?,
                    geometric_error: root_geometric_error / (1u64 << level) as f64,
                });
            }
        }
    }
    Ok(out)
}

fn expand_template(template: &str, level: u32, x: u32, y: u32) -> String {
    template
        .replace("{level}", &level.to_string())
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string())
}

/// A bitstream read from a `.subtree` file, or the `{"constant": ...}`
/// shorthand for "every cell has (or lacks) content, don't bother listing".
enum Availability {
    Constant(bool),
    Bitstream(Vec<u8>),
}

impl Availability {
    fn get(&self, index: u64) -> bool {
        match self {
            Availability::Constant(b) => *b,
            Availability::Bitstream(bytes) => {
                let i = index as usize;
                (bytes[i / 8] >> (i % 8)) & 1 == 1
            }
        }
    }
}

/// Parses a `.subtree` file (3D Tiles 1.1 implicit tiling: a 24-byte
/// header — magic `subt`, version, JSON length, binary length — followed by
/// the two chunks) and returns just its `contentAvailability[0]` bitstream;
/// nothing here needs `tileAvailability` or `childSubtreeAvailability`.
fn parse_subtree_content_availability(bytes: &[u8]) -> Result<Availability, String> {
    if bytes.len() < 24 || &bytes[0..4] != b"subt" {
        return Err("Invalid .subtree file: bad magic".to_string());
    }
    let json_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let bin_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;

    let json_start = 24;
    let json_bytes = bytes
        .get(json_start..json_start + json_len)
        .ok_or_else(|| "Invalid .subtree file: JSON chunk out of bounds".to_string())?;
    let bin_start = json_start + json_len;
    let bin_bytes = bytes
        .get(bin_start..bin_start + bin_len)
        .ok_or_else(|| "Invalid .subtree file: binary chunk out of bounds".to_string())?;

    let json: Value = serde_json::from_slice(json_bytes)
        .map_err(|e| format!("Failed to parse .subtree JSON chunk: {e}"))?;

    let buffer_views = json
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let content_availability_def = json
        .get("contentAvailability")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| "subtree JSON missing contentAvailability".to_string())?;

    parse_availability(content_availability_def, &buffer_views, bin_bytes)
}

fn parse_availability(
    def: &Value,
    buffer_views: &[Value],
    bin: &[u8],
) -> Result<Availability, String> {
    if let Some(c) = def.get("constant").and_then(|v| v.as_u64()) {
        return Ok(Availability::Constant(c != 0));
    }

    let bitstream_idx = def
        .get("bitstream")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "availability object has neither constant nor bitstream".to_string())?
        as usize;
    let view = buffer_views
        .get(bitstream_idx)
        .ok_or_else(|| format!("bufferView {bitstream_idx} not found in subtree"))?;
    let offset = view.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let length = view
        .get("byteLength")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "subtree bufferView missing byteLength".to_string())? as usize;
    let bytes = bin
        .get(offset..offset + length)
        .ok_or_else(|| "subtree bufferView range out of bounds in binary chunk".to_string())?
        .to_vec();
    Ok(Availability::Bitstream(bytes))
}

/// Number of tiles across levels `0..level` of a complete quadtree — the bit
/// offset where `level`'s span starts in the availability bitstream. Mirrors
/// `cesium3dtiles/next/subtree.rs`'s writer-side encoding.
fn level_offset(level: u32) -> u64 {
    (4u64.pow(level) - 1) / 3
}

/// A cell's bit index in the availability bitstream: its level's offset plus
/// its Morton index within that level (3D Tiles 1.1's fixed ordering).
fn morton2d(x: u32, y: u32) -> u64 {
    spread_bits(x) | (spread_bits(y) << 1)
}

/// Spreads a 32-bit value's bits into the even bit positions of a 64-bit one.
fn spread_bits(v: u32) -> u64 {
    let mut x = v as u64;
    x = (x | (x << 16)) & 0x0000_FFFF_0000_FFFF;
    x = (x | (x << 8)) & 0x00FF_00FF_00FF_00FF;
    x = (x | (x << 4)) & 0x0F0F_0F0F_0F0F_0F0F;
    x = (x | (x << 2)) & 0x3333_3333_3333_3333;
    x = (x | (x << 1)) & 0x5555_5555_5555_5555;
    x
}

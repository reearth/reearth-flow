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

/// QUADTREE-only. Follows `childSubtreeAvailability` to chain across
/// `subtreeLevels`-sized windows, so a dataset deeper than one window (as
/// produced by the `cesium3dtiles/next` writer) is still fully readable.
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
        .ok_or_else(|| "implicitTiling missing subtreeLevels".to_string())?
        as u32;
    assert!(
        subtree_levels < 10,
        "subtreeLevels should not exceed 10, got {subtree_levels}"
    );

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

    let mut out = Vec::new();
    collect_subtree_chain(
        tileset_dir,
        content_template,
        subtree_template,
        (0, 0, 0),
        subtree_levels,
        root_geometric_error,
        &mut out,
    )?;
    Ok(out)
}

/// Reads the `.subtree` file rooted at `(level, x, y)` (all absolute,
/// dataset-wide coordinates), collects its in-window content, then recurses
/// into every subtree it chains to via `childSubtreeAvailability`.
fn collect_subtree_chain(
    tileset_dir: &Path,
    content_template: &str,
    subtree_template: &str,
    (root_level, root_x, root_y): (u32, u32, u32),
    subtree_levels: u32,
    root_geometric_error: f64,
    out: &mut Vec<TileContent>,
) -> Result<(), String> {
    let subtree_path = resolve_existing(
        tileset_dir,
        &expand_template(subtree_template, root_level, root_x, root_y),
    )?;
    let subtree_bytes = fs::read(&subtree_path)
        .map_err(|e| format!("Failed to read subtree file {:?}: {}", subtree_path, e))?;
    let subtree = parse_subtree(&subtree_bytes)?;

    for rel_level in 0..subtree_levels {
        let n = 1u32 << rel_level;
        for ry in 0..n {
            for rx in 0..n {
                let bit = level_offset(rel_level) + morton2d(rx, ry);
                if !subtree.content_availability.get(bit) {
                    continue;
                }
                let level = root_level + rel_level;
                let x = (root_x << rel_level) | rx;
                let y = (root_y << rel_level) | ry;
                let uri = expand_template(content_template, level, x, y);
                out.push(TileContent {
                    path: resolve_existing(tileset_dir, &uri)?,
                    geometric_error: root_geometric_error / (1u64 << level) as f64,
                });
            }
        }
    }

    // `childSubtreeAvailability` is indexed over the level one past this
    // file's window — where a chained subtree's root sits — not the boundary
    // level itself. Mirrors `cesium3dtiles/next/subtree.rs`'s `rel_xy` there.
    let n = 1u32 << subtree_levels;
    for cy in 0..n {
        for cx in 0..n {
            if !subtree.child_subtree_availability.get(morton2d(cx, cy)) {
                continue;
            }
            collect_subtree_chain(
                tileset_dir,
                content_template,
                subtree_template,
                (
                    root_level + subtree_levels,
                    (root_x << subtree_levels) | cx,
                    (root_y << subtree_levels) | cy,
                ),
                subtree_levels,
                root_geometric_error,
                out,
            )?;
        }
    }
    Ok(())
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

/// The two availability bitstreams `collect_subtree_chain` needs:
/// `contentAvailability[0]` for this file's own window, and
/// `childSubtreeAvailability` to find chained `.subtree` files.
/// `tileAvailability` is unused — nothing here queries tile existence
/// independent of content.
struct ParsedSubtree {
    content_availability: Availability,
    child_subtree_availability: Availability,
}

/// Parses a `.subtree` file: a 24-byte header (magic `subt`, version, JSON
/// length, binary length) followed by the two chunks.
fn parse_subtree(bytes: &[u8]) -> Result<ParsedSubtree, String> {
    if bytes.len() < 24 || &bytes[0..4] != b"subt" {
        return Err("Invalid .subtree file: bad magic".to_string());
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    assert_eq!(version, 1, "unsupported .subtree version: {version}");
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
    let content_availability =
        parse_availability(content_availability_def, &buffer_views, bin_bytes)?;

    let child_subtree_availability_def = json
        .get("childSubtreeAvailability")
        .ok_or_else(|| "subtree JSON missing childSubtreeAvailability".to_string())?;
    let child_subtree_availability =
        parse_availability(child_subtree_availability_def, &buffer_views, bin_bytes)?;

    Ok(ParsedSubtree {
        content_availability,
        child_subtree_availability,
    })
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
    let length =
        view.get("byteLength")
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bitset(len_bits: usize, set_bits: &[u64]) -> Vec<u8> {
        let mut bytes = vec![0u8; len_bits.div_ceil(8)];
        for &i in set_bits {
            bytes[(i / 8) as usize] |= 1 << (i % 8);
        }
        bytes
    }

    /// Hand-encodes a `.subtree` file with `subtreeLevels = 1` (window: level
    /// 0 only), given which in-window bit is content-available and which
    /// one-past-the-boundary bit chains to another subtree.
    fn encode_subtree(content_bit: Option<u64>, chained_bit: Option<u64>) -> Vec<u8> {
        let content_bits: Vec<u64> = content_bit.into_iter().collect();
        let content_availability = bitset(1, &content_bits);
        let child_availability_bits: Vec<u64> = chained_bit.into_iter().collect();
        let child_availability = bitset(4, &child_availability_bits);

        let mut buffers = vec![content_availability];
        let child_json = if child_availability_bits.is_empty() {
            json!({"constant": 0})
        } else {
            let bitstream = buffers.len();
            buffers.push(child_availability);
            json!({"bitstream": bitstream, "availableCount": 1})
        };

        let mut buffer_views = Vec::new();
        let mut offset = 0usize;
        for b in &buffers {
            buffer_views.push(json!({"buffer": 0, "byteOffset": offset, "byteLength": b.len()}));
            offset += b.len();
        }
        let header_json = json!({
            "buffers": [{"byteLength": offset}],
            "bufferViews": buffer_views,
            "tileAvailability": {"constant": 1},
            "contentAvailability": [{"bitstream": 0, "availableCount": 1}],
            "childSubtreeAvailability": child_json,
        });
        let mut json_bytes = serde_json::to_vec(&header_json).unwrap();
        while !json_bytes.len().is_multiple_of(8) {
            json_bytes.push(b' ');
        }
        let mut binary: Vec<u8> = buffers.into_iter().flatten().collect();
        while !binary.len().is_multiple_of(8) {
            binary.push(0);
        }

        let mut out = Vec::new();
        out.extend_from_slice(b"subt");
        out.extend_from_slice(&1u32.to_le_bytes());
        out.extend_from_slice(&(json_bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(&(binary.len() as u64).to_le_bytes());
        out.extend_from_slice(&json_bytes);
        out.extend_from_slice(&binary);
        out
    }

    #[test]
    fn collect_implicit_follows_a_chained_subtree_past_one_window() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("subtrees")).unwrap();
        fs::create_dir_all(dir.path().join("content")).unwrap();

        // Root window (subtreeLevels=1): level 0 has content, and chains to
        // a second subtree rooted at level 1, x=1, y=0.
        let root_bytes = encode_subtree(Some(0), Some(morton2d(1, 0)));
        fs::write(dir.path().join("subtrees/0.0.0.subtree"), root_bytes).unwrap();
        // Chained window rooted at (1, 1, 0): its own level 0 (= absolute
        // level 1) has content, no further chaining.
        let chained_bytes = encode_subtree(Some(0), None);
        fs::write(dir.path().join("subtrees/1.1.0.subtree"), chained_bytes).unwrap();

        for (level, x, y) in [(0, 0, 0), (1, 1, 0)] {
            let path = dir.path().join(format!("content/{level}/{x}/{y}.glb"));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"glb").unwrap();
        }

        let root_tile = json!({
            "geometricError": 100.0,
            "content": {"uri": "content/{level}/{x}/{y}.glb"},
            "implicitTiling": {
                "subdivisionScheme": "QUADTREE",
                "subtreeLevels": 1,
                "availableLevels": 2,
                "subtrees": {"uri": "subtrees/{level}.{x}.{y}.subtree"},
            },
        });

        let mut found: Vec<_> = collect_tile_contents(dir.path(), &root_tile)
            .unwrap()
            .into_iter()
            .map(|c| (c.path, c.geometric_error))
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));

        let mut expected = vec![
            (dir.path().join("content/0/0/0.glb"), 100.0),
            (dir.path().join("content/1/1/0.glb"), 50.0),
        ];
        expected.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(found, expected);
    }
}

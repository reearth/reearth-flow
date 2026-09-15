//! From-scratch Cesium 3D Tiles writer for the new geometry type.
//!
//! Appearance is painted for the default theme, front side only. Textured
//! materials across a tile share one or more embedded atlas pages (one glTF
//! primitive per page). Local-file and embedded (in-memory) rasters are both
//! supported: embedded bytes (e.g. glTF/GLB packed images) are materialized to a
//! temp file so the path-based atlas packer can read them. A tiling texture gets a
//! page of its own, bound whole so the sampler can repeat it; remote rasters fall
//! back to colour-only. Texture detail is bounded by the
//! `texel_size` option (metres per pixel); atlas pages are capped at
//! `atlas_size` and overflow spills onto further pages.

mod appearance;
mod mesh;
mod primitive;
mod quadtree;
mod stats;
mod subtree;
mod tileset;

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;
use rayon::prelude::*;

use reearth_flow_atlas::{build_atlas_multipage, TextureCache, TextureInput};
use reearth_flow_common::image::MimeType;
use reearth_flow_geometry::appearance::RasterData;
use reearth_flow_gltf::next::glb::{self, Granularity};
use reearth_flow_gltf::next::metadata;

use appearance::TextureSource;
use primitive::{Geom, TexturedPrimitive, DEFAULT_MATERIAL};
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::FEATURES_PORT;
use reearth_flow_types::Feature;

use super::sink::Cesium3DTilesWriter;
pub use super::sink::TextureCodec;
use crate::errors::SinkError;
use quadtree::{Cell, GeoBox};
pub use reearth_flow_gltf::next::metadata::MetadataOptions;

impl Cesium3DTilesWriter {
    pub(super) fn process_new_geometry(
        &mut self,
        ctx: &ExecutorContext,
    ) -> crate::errors::Result<()> {
        if ctx.port != *FEATURES_PORT {
            // The schema port has no meaning without attribute/metadata output
            // in this pass; ignore it.
            return Ok(());
        }

        let output = self
            .params
            .output
            .eval_string(&ctx.feature, Arc::clone(&ctx.variables))
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;

        self.buffer
            .entry(output)
            .or_default()
            .push(ctx.feature.clone());
        Ok(())
    }

    pub(super) fn finish_new_geometry(&self, ctx: NodeContext) -> crate::errors::Result<()> {
        let options = MetadataOptions {
            schema_key: self.params.schema_key.as_deref(),
            skip_unexposed_attributes: self.params.skip_unexposed_attributes,
            array_map_separator: self.params.array_map_separator.as_deref(),
        };
        let render = RenderOptions {
            draco: self.params.draco_compression,
            compute_flat_normal: self.params.compute_flat_normal,
            texel_size: self.params.texel_size.unwrap_or(0.0),
            atlas_size: self.params.atlas_size.unwrap_or(DEFAULT_ATLAS_SIZE),
            atlas_extrusion: self
                .params
                .atlas_extrusion
                .unwrap_or(DEFAULT_ATLAS_EXTRUSION),
            wrap_tolerance: self.params.wrap_tolerance.unwrap_or(DEFAULT_WRAP_TOLERANCE),
            texture_codec: self.params.texture_codec,
        };
        for (output, features) in &self.buffer {
            if output.ends_with(".zip") {
                let zip = reearth_flow_common::zip::StreamingZipWriter::new(std::io::Cursor::new(
                    Vec::new(),
                ));
                let write_file = |relative_path: String, bytes: Vec<u8>| {
                    zip.write_entry(&relative_path, &bytes)
                        .map_err(crate::errors::SinkError::cesium3dtiles_writer)
                };

                let target_tile_size = self
                    .params
                    .target_tile_size
                    .unwrap_or(DEFAULT_TARGET_TILE_SIZE);
                let built = build(features, options, target_tile_size, render, write_file)?;
                for (relative_path, bytes) in built.subtrees {
                    write_file(relative_path, bytes)?;
                }
                write_file("tileset.json".to_string(), built.tileset_json.into_bytes())?;

                let cursor = zip
                    .finish()
                    .map_err(|e| crate::errors::SinkError::cesium3dtiles_writer(e.to_string()))?;
                crate::SinkOutput::new(&ctx.sandbox_root, output, &ctx.storage_resolver)
                    .and_then(|out| out.write(bytes::Bytes::from(cursor.into_inner())))
                    .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
            } else {
                let write_file = |relative_path: String, bytes: Vec<u8>| {
                    crate::SinkOutput::new(
                        &ctx.sandbox_root,
                        &format!("{output}/{relative_path}"),
                        &ctx.storage_resolver,
                    )
                    .and_then(|out| out.write(bytes::Bytes::from(bytes)))
                    .map_err(crate::errors::SinkError::cesium3dtiles_writer)
                };

                // glbs stream out as they're built; only subtree/tileset outputs come back.
                let target_tile_size = self
                    .params
                    .target_tile_size
                    .unwrap_or(DEFAULT_TARGET_TILE_SIZE);
                let built = build(features, options, target_tile_size, render, write_file)?;

                for (relative_path, bytes) in built.subtrees {
                    write_file(relative_path, bytes)?;
                }

                crate::SinkOutput::new(
                    &ctx.sandbox_root,
                    &format!("{output}/tileset.json"),
                    &ctx.storage_resolver,
                )
                .and_then(|out| out.write(bytes::Bytes::from(built.tileset_json)))
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
            }
        }
        Ok(())
    }
}

/// The tileset's non-glb outputs, relative to its output directory: the
/// `tileset.json` text and one or more `.subtree` files. The content glbs are
/// streamed out through `build`'s `write_tile` callback as they are produced,
/// not held here; `tile_count` records how many were written.
pub struct BuiltTileset {
    pub tileset_json: String,
    pub subtrees: Vec<(String, Vec<u8>)>,
    pub tile_count: usize,
    /// Features that carried renderable geometry; the rest never reach a tile.
    pub rendered_features: usize,
}

/// Rendering knobs shared by every cell of a tileset.
#[derive(Clone, Copy)]
pub struct RenderOptions {
    /// Draco-compress each glb.
    pub draco: bool,
    /// Attach per-polygon flat normals for lighting.
    pub compute_flat_normal: bool,
    /// Target texel size in metres per pixel: textures finer than this are
    /// downsampled to it. `0.0` keeps full texture detail.
    pub texel_size: f64,
    /// Maximum atlas page dimension (pixels). Textures/atlases exceeding it
    /// spill onto additional pages; a single texture larger than it is
    /// force-shrunk to fit one page.
    pub atlas_size: u32,
    /// Extrusion ring (pixels) blitted around each atlas region to stop
    /// bilinear bleed between neighbours. `0` disables it.
    pub atlas_extrusion: u32,
    /// How far outside `[0, 1]` a UV may stray and still be clamped as drift;
    /// past it the texture is taken to tile and gets an atlas page of its own.
    pub wrap_tolerance: f64,
    /// Image codec for atlas pages. `Untextured` attaches no textures; textured
    /// geometry falls back to its neutral colour.
    pub texture_codec: TextureCodec,
}

/// Default atlas page size when the parameter is unset; inherited from the old
/// writer.
const DEFAULT_ATLAS_SIZE: u32 = 2048;

/// Default atlas extrusion ring when the parameter is unset; disabled by
/// default. Raise it to blit a bleed-guard ring around each packed region.
const DEFAULT_ATLAS_EXTRUSION: u32 = 0;

/// Default UV wrap tolerance when the parameter is unset: none, so any UV outside
/// `[0, 1]` is taken at face value as tiling.
const DEFAULT_WRAP_TOLERANCE: f64 = 0.0;

/// Hard safety cap on quadtree depth, well beyond any depth `target_tile_size`
/// would realistically drive placement to; guards against pathological inputs
/// (e.g. many coincident features) rather than acting as a tuning knob.
const SAFETY_MAX_DEPTH: u32 = 24;

/// Default per-tile content-size target (bytes) when `target_tile_size` is
/// unset.
const DEFAULT_TARGET_TILE_SIZE: u64 = 1_048_576;

/// Upper bound on same-tile contents a cell may split into; a cell that would
/// need more keeps contents over `target_tile_size` instead.
const MAX_CONTENTS_PER_TILE: usize = 15;

/// A free function so `gml_to_3dtiles` can drive it directly from parsed
/// CityGML, without a `Cesium3DTilesWriter`. Content glbs stream through
/// `write_tile` as each cell is built rather than being retained, so peak
/// memory stays at one cell's contents regardless of tile count.
pub fn build(
    features: &[Feature],
    options: MetadataOptions,
    target_tile_size: u64,
    render: RenderOptions,
    write_tile: impl Fn(String, Vec<u8>) -> crate::errors::Result<()> + Sync,
) -> crate::errors::Result<BuiltTileset> {
    let mut caches = mesh::ExtractCaches::default();
    let extracted: Vec<(&Feature, mesh::ExtractedMesh)> = features
        .iter()
        .filter_map(|feature| mesh::extract(&feature.geometry, &mut caches).map(|m| (feature, m)))
        .collect();

    let property_stats = stats::collect(extracted.iter().map(|(feature, _)| *feature), options);

    if extracted.is_empty() {
        tracing::warn!(
            "Cesium3DTilesWriter (new-geometry): no renderable geometry found; writing an \
             empty tileset"
        );
        return empty_tileset(&property_stats);
    }

    let root = extracted
        .iter()
        .filter_map(|(_, m)| GeoBox::of(&m.geographic_vertices))
        .reduce(GeoBox::union)
        .expect("extracted is non-empty, and mesh::extract never returns an empty vertex buffer");

    let mut by_cell: HashMap<Cell, Vec<usize>> = HashMap::new();
    for (i, (_, m)) in extracted.iter().enumerate() {
        let Some(feature_box) = GeoBox::of(&m.geographic_vertices) else {
            continue;
        };
        let cell = quadtree::place(&root, &feature_box, SAFETY_MAX_DEPTH);
        by_cell.entry(cell).or_default().push(i);
    }

    // A decode cache per build, dropped once its glb is built. PLATEAU textures
    // are per-surface, so a source image is referenced by only one cell; a
    // tileset-wide cache would grow without bound for no reuse gain.
    let build_chunk = |chunk: &[usize]| {
        let mut textures = TextureCache::default();
        let mut embedded = EmbeddedTextures::new()?;
        let members: Vec<&(&Feature, mesh::ExtractedMesh)> =
            chunk.iter().map(|&i| &extracted[i]).collect();
        build_cell_glb(&members, options, render, &mut textures, &mut embedded)
    };

    let mut units: HashMap<Cell, Vec<Unit>> = by_cell
        .into_par_iter()
        .map(
            |(cell, features)| -> crate::errors::Result<(Cell, Vec<Unit>)> {
                let bytes = build_chunk(&features)?.len() as u64;
                Ok((cell, vec![Unit { features, bytes }]))
            },
        )
        .collect::<crate::errors::Result<_>>()?;
    let overhead = measure_glb_overhead(&units, build_chunk)?;

    merge_small_cells(&mut units, target_tile_size, overhead);

    let occupied: BTreeSet<Cell> = units.keys().copied().collect();
    let available_levels = occupied.iter().map(|c| c.level).max().unwrap_or(0) + 1;

    // The glb bytes feed neither `tileset.json` nor the subtrees (those need
    // only the cell keys, already captured in `occupied`), so each cell's
    // contents stream to `write_tile` as soon as the cell is done and peak
    // memory stays at one cell's contents per worker.
    let cell_results: Vec<(Cell, usize, bool)> = units
        .into_par_iter()
        .map(
            |(cell, units)| -> crate::errors::Result<(Cell, usize, bool)> {
                let contents = split_by_size(units, target_tile_size, overhead, build_chunk)?;
                let count = contents.glbs.len();
                for (n, glb) in contents.glbs.into_iter().enumerate() {
                    write_tile(content_path(cell, n), glb)?;
                }
                Ok((cell, count, contents.capped))
            },
        )
        .collect::<crate::errors::Result<_>>()?;

    let content_counts: HashMap<Cell, usize> = cell_results
        .iter()
        .map(|(cell, count, _)| (*cell, *count))
        .collect();
    // The content URI template and the subtree `contentAvailability` array are
    // both declared once for the whole tileset, so every cell shares the same
    // slot count even where only one cell actually splits.
    let max_contents = content_counts.values().copied().max().unwrap_or(1);
    let tile_count: usize = content_counts.values().sum();
    let capped_cells = cell_results.iter().filter(|(_, _, capped)| *capped).count();
    if capped_cells > 0 {
        tracing::warn!(
            "Cesium3DTilesWriter: {capped_cells} tile(s) reached the {MAX_CONTENTS_PER_TILE} \
             contents-per-tile limit and carry contents over targetTileSize"
        );
    }

    let tileset_bytes =
        render_tileset_json(&root, available_levels, max_contents, &property_stats)?;
    let subtrees = subtree::build_all(&occupied, &content_counts, max_contents)
        .into_iter()
        .map(|(cell, bytes)| (subtree_path(cell), bytes))
        .collect();

    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        subtrees,
        tile_count,
        rendered_features: extracted.len(),
    })
}

/// A group of features whose glb, built on its own, is `bytes` long.
struct Unit {
    features: Vec<usize>,
    bytes: u64,
}

/// Bytes a glb holding all of `units` is expected to take: their measured
/// bytes less the per-glb `overhead` paid only once.
fn group_bytes(units: &[Unit], overhead: u64) -> u64 {
    let sum: u64 = units.iter().map(|u| u.bytes).sum();
    sum.saturating_sub(overhead.saturating_mul(units.len().saturating_sub(1) as u64))
}

/// Bytes every glb carries regardless of its features, measured as the amount
/// by which the two smallest units shrink when built together. Zero when there
/// are fewer than two units.
fn measure_glb_overhead(
    units: &HashMap<Cell, Vec<Unit>>,
    build_chunk: impl Fn(&[usize]) -> crate::errors::Result<Vec<u8>>,
) -> crate::errors::Result<u64> {
    let mut smallest: Vec<&Unit> = units.values().flatten().collect();
    if smallest.len() < 2 {
        return Ok(0);
    }
    smallest.sort_by_key(|u| (u.bytes, u.features[0]));
    let (a, b) = (smallest[0], smallest[1]);
    let together: Vec<usize> = a.features.iter().chain(&b.features).copied().collect();
    let joint = build_chunk(&together)?.len() as u64;
    Ok((a.bytes + b.bytes).saturating_sub(joint))
}

/// The content glbs one cell splits into, in slot order.
struct CellContents {
    glbs: Vec<Vec<u8>>,
    /// Whether [`MAX_CONTENTS_PER_TILE`] stopped a split, leaving at least one
    /// content over the target.
    capped: bool,
}

/// Partition a cell's units into fetch-parallel content chunks of at most
/// `target_tile_size` bytes each. Units are first grouped by their measured
/// bytes, then every chunk is built, measured, and cut further while it is
/// over the target: a chunk of several units is cut between units, a chunk of
/// one unit is cut into equal feature counts. A single feature is never split,
/// and no cell yields more than [`MAX_CONTENTS_PER_TILE`] chunks.
fn split_by_size(
    units: Vec<Unit>,
    target_tile_size: u64,
    overhead: u64,
    build_chunk: impl Fn(&[usize]) -> crate::errors::Result<Vec<u8>> + Sync,
) -> crate::errors::Result<CellContents> {
    let expected = group_bytes(&units, overhead);
    let parts = if target_tile_size == 0 {
        MAX_CONTENTS_PER_TILE
    } else {
        (expected.div_ceil(target_tile_size) as usize).clamp(1, MAX_CONTENTS_PER_TILE)
    };
    let mut pending: Vec<Vec<Unit>> = if parts >= 2 && units.len() >= 2 {
        split_into(units, |u| u.bytes, parts)
    } else {
        vec![units]
    };

    let mut glbs: Vec<Vec<u8>> = Vec::new();
    let mut capped = false;
    while !pending.is_empty() {
        let built: Vec<(Vec<Unit>, Vec<u8>)> = std::mem::take(&mut pending)
            .into_par_iter()
            .map(|chunk| {
                let features: Vec<usize> = chunk
                    .iter()
                    .flat_map(|u| u.features.iter().copied())
                    .collect();
                build_chunk(&features).map(|glb| (chunk, glb))
            })
            .collect::<crate::errors::Result<_>>()?;
        let total = built.len();
        for (k, (chunk, glb)) in built.into_iter().enumerate() {
            let bytes = glb.len() as u64;
            let feature_count: usize = chunk.iter().map(|u| u.features.len()).sum();
            if bytes <= target_tile_size || feature_count == 1 {
                glbs.push(glb);
                continue;
            }
            let unprocessed = total - k - 1;
            let room = MAX_CONTENTS_PER_TILE - glbs.len() - pending.len() - unprocessed;
            if room < 2 {
                capped = true;
                glbs.push(glb);
                continue;
            }
            let wanted = if target_tile_size == 0 {
                room
            } else {
                bytes.div_ceil(target_tile_size) as usize
            };
            let parts = wanted.clamp(2, room).min(feature_count);
            if chunk.len() >= 2 {
                let parts = parts.min(chunk.len());
                pending.extend(split_into(chunk, |u| u.bytes, parts));
            } else {
                let unit = chunk
                    .into_iter()
                    .next()
                    .expect("a chunk holds at least one unit");
                let per_unit = bytes / parts as u64;
                pending.extend(split_into(unit.features, |_| 0, parts).into_iter().map(
                    |features| {
                        vec![Unit {
                            features,
                            bytes: per_unit,
                        }]
                    },
                ));
            }
        }
    }
    Ok(CellContents { glbs, capped })
}

/// Cut `items` into between two and `parts` contiguous groups of roughly equal
/// summed `weight`. Both `items` and `parts` must be at least two. Falls back
/// to equal counts when every weight is zero.
fn split_into<T>(items: Vec<T>, weight: impl Fn(&T) -> u64, parts: usize) -> Vec<Vec<T>> {
    debug_assert!(items.len() >= 2 && parts >= 2);
    let total: u64 = items.iter().map(&weight).sum();
    if total == 0 {
        let per = items.len().div_ceil(parts).max(1);
        let mut groups: Vec<Vec<T>> = Vec::new();
        for item in items {
            match groups.last_mut() {
                Some(last) if last.len() < per => last.push(item),
                _ => groups.push(vec![item]),
            }
        }
        return groups;
    }
    let budget = total as f64 / parts as f64;
    let mut groups: Vec<Vec<T>> = Vec::new();
    let mut current: Vec<T> = Vec::new();
    let mut current_weight = 0u64;
    for item in items {
        let w = weight(&item);
        if !current.is_empty() && groups.len() + 1 < parts && (current_weight + w) as f64 > budget {
            groups.push(std::mem::take(&mut current));
            current_weight = 0;
        }
        current.push(item);
        current_weight += w;
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

/// Fold small sibling cells upward into their parent while the folded glb is
/// expected to stay within `target_tile_size`, deepest level first so a fold
/// can cascade to the root.
fn merge_small_cells(units: &mut HashMap<Cell, Vec<Unit>>, target_tile_size: u64, overhead: u64) {
    let max_level = units.keys().map(|c| c.level).max().unwrap_or(0);
    for level in (1..=max_level).rev() {
        let mut by_parent: HashMap<Cell, Vec<Cell>> = HashMap::new();
        for cell in units.keys().filter(|c| c.level == level) {
            if let Some(parent) = cell.parent() {
                by_parent.entry(parent).or_default().push(*cell);
            }
        }
        for (parent, mut children) in by_parent {
            children.sort_by_key(|c| (group_bytes(&units[c], overhead), *c));
            for child in children {
                let mut folded: Vec<&Unit> = Vec::new();
                folded.extend(units.get(&parent).into_iter().flatten());
                folded.extend(&units[&child]);
                let sum: u64 = folded.iter().map(|u| u.bytes).sum();
                let expected = sum.saturating_sub(overhead * (folded.len() as u64 - 1));
                if expected > target_tile_size {
                    continue;
                }
                let moved = units.remove(&child).unwrap();
                units.entry(parent).or_default().extend(moved);
            }
        }
    }
}

/// Render one feature into a glb, untiled. Where [`build`] splits a whole
/// dataset across a quadtree, this renders a single picked feature, which is
/// what a viewer wants when a table row is clicked.
///
/// Positions are local to an origin chosen from the rendered geometry, carried
/// on the scene node's translation exactly as tile content is, so the result
/// drops into a georeferenced viewer unchanged.
///
/// `Ok(None)` means the feature carried no renderable geometry.
///
// TODO: Relocate this after GltfWriter is implemented (for the new geometry)
pub fn build_glb(
    feature: &Feature,
    options: MetadataOptions,
    render: RenderOptions,
) -> crate::errors::Result<Option<Vec<u8>>> {
    let mut caches = mesh::ExtractCaches::default();
    let Some(extracted) = mesh::extract(&feature.geometry, &mut caches) else {
        return Ok(None);
    };

    let mut textures = TextureCache::default();
    let mut embedded = EmbeddedTextures::new()?;
    build_cell_glb(
        &[&(feature, extracted)],
        options,
        render,
        &mut textures,
        &mut embedded,
    )
    .map(Some)
}

fn empty_tileset(
    property_stats: &IndexMap<String, stats::PropertyStats>,
) -> crate::errors::Result<BuiltTileset> {
    let root = GeoBox {
        west: 0.0,
        south: 0.0,
        east: 0.0,
        north: 0.0,
        min_height: 0.0,
        max_height: 0.0,
    };
    let tileset_bytes = render_tileset_json(&root, 1, 1, property_stats)?;
    let subtrees = subtree::build_all(&BTreeSet::new(), &HashMap::new(), 1)
        .into_iter()
        .map(|(cell, bytes)| (subtree_path(cell), bytes))
        .collect();
    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        subtrees,
        tile_count: 0,
        rendered_features: 0,
    })
}

fn render_tileset_json(
    root: &GeoBox,
    available_levels: u32,
    max_contents: usize,
    property_stats: &IndexMap<String, stats::PropertyStats>,
) -> crate::errors::Result<String> {
    let tileset_json = tileset::build(root, available_levels, max_contents, property_stats);
    serde_json::to_string_pretty(&tileset_json)
        .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))
}

/// Content slot `n` of a cell: slot 0 is the plain `{y}.glb`, further slots
/// are `{y}_{n}.glb`.
fn content_path(cell: Cell, n: usize) -> String {
    if n == 0 {
        format!("content/{}/{}/{}.glb", cell.level, cell.x, cell.y)
    } else {
        format!("content/{}/{}/{}_{}.glb", cell.level, cell.x, cell.y, n)
    }
}

fn subtree_path(cell: Cell) -> String {
    format!("subtrees/{}.{}.{}.subtree", cell.level, cell.x, cell.y)
}

/// Materializes embedded texture bytes (e.g. glTF/GLB packed images) to temp
/// files so the path-based atlas packer can read them. Deduplicated by content
/// hash within a cell; the temp dir and its files drop once the cell's glb is
/// built.
struct EmbeddedTextures {
    dir: tempfile::TempDir,
    by_hash: HashMap<u64, PathBuf>,
}

impl EmbeddedTextures {
    fn new() -> crate::errors::Result<Self> {
        Ok(Self {
            dir: tempfile::tempdir().map_err(|e| {
                SinkError::Cesium3DTilesWriter(format!("failed to create texture temp dir: {e}"))
            })?,
            by_hash: HashMap::new(),
        })
    }

    /// Write `data` to a temp file (once per distinct content) and return its path.
    fn materialize(&mut self, data: &RasterData) -> crate::errors::Result<PathBuf> {
        let mut hasher = DefaultHasher::new();
        data.bytes.hash(&mut hasher);
        let hash = hasher.finish();
        if let Some(path) = self.by_hash.get(&hash) {
            return Ok(path.clone());
        }
        let ext = match data.mime_type {
            MimeType::ImagePng => "png",
            MimeType::ImageJpeg => "jpg",
            MimeType::ImageWebp => "webp",
        };
        let path = self.dir.path().join(format!("{hash:016x}.{ext}"));
        std::fs::write(&path, &data.bytes).map_err(|e| {
            SinkError::Cesium3DTilesWriter(format!("failed to write embedded texture: {e}"))
        })?;
        self.by_hash.insert(hash, path.clone());
        Ok(path)
    }
}

/// A packed page holds many textures side by side, so a repeating wrap would bleed
/// across sub-images; a page the packer gave to one tiling texture wraps that texture.
fn page_sampler(wrap: reearth_flow_atlas::PageWrap) -> glb::SamplerDesc {
    let (wrap_s, wrap_t) = match wrap {
        reearth_flow_atlas::PageWrap::Clamp => (glb::Wrap::ClampToEdge, glb::Wrap::ClampToEdge),
        reearth_flow_atlas::PageWrap::Repeat => (glb::Wrap::Repeat, glb::Wrap::Repeat),
    };
    glb::SamplerDesc {
        wrap_s,
        wrap_t,
        mag: glb::MagFilter::Linear,
        min: glb::MinFilter::LinearMipmap,
    }
}

/// Render one occupied cell to a glb: one primitive per resolved colour-only
/// material, plus one textured primitive per atlas page covering the cell's
/// textured faces (see [`primitive::collect`], [`build_textured_pages`]).
/// `render.compute_flat_normal` attaches per-polygon flat normals;
/// `render.draco` Draco-compresses the output.
fn build_cell_glb(
    cell_members: &[&(&Feature, mesh::ExtractedMesh)],
    options: MetadataOptions,
    render: RenderOptions,
    textures: &mut TextureCache,
    embedded: &mut EmbeddedTextures,
) -> crate::errors::Result<Vec<u8>> {
    let cells = primitive::collect(cell_members);

    let cell_features: Vec<&Feature> = cell_members.iter().map(|(f, _)| *f).collect();
    let table = metadata::build_table(&cell_features, options);

    // Per-tile local origin keeps the f32 positions small next to ECEF's
    // ~6.378e6 m magnitude (see [`push_geom`]).
    let origin = cell_origin(&cells);

    let mut builder = glb::Builder::new();
    // Each primitive keeps its own per-vertex feature IDs (its vertex buffer is
    // compacted independently), attached together in `metadata::encode`.
    let mut primitives: Vec<(glb::PrimitiveHandle, Vec<u32>)> = Vec::new();

    if let Some(textured) = cells.textured {
        // `Untextured` skips texturing entirely and renders the textured
        // geometry in the neutral fallback colour.
        let pages = match render.texture_codec {
            TextureCodec::Untextured => None,
            _ => build_textured_pages(&mut builder, &textured, render, textures, embedded)?,
        };
        match pages {
            Some(pages) => {
                for page in pages {
                    let material = glb::MaterialDesc {
                        base_color_factor: [1.0, 1.0, 1.0, 1.0],
                        metallic_factor: 0.0,
                        roughness_factor: 1.0,
                        base_color_texture: Some(page.texture),
                    };
                    let handle = push_geom(
                        &mut builder,
                        &page.geom,
                        origin,
                        material,
                        Some(page.corner_uv),
                        render.compute_flat_normal,
                    );
                    primitives.push((handle, page.geom.feature_ids));
                }
            }
            // Packing failed or produced no image: render the textured geometry
            // in the neutral fallback colour rather than dropping it.
            None => {
                let handle = push_geom(
                    &mut builder,
                    &textured.geom,
                    origin,
                    color_material(DEFAULT_MATERIAL),
                    None,
                    render.compute_flat_normal,
                );
                primitives.push((handle, textured.geom.feature_ids));
            }
        }
    }

    for color in cells.color {
        let material = color_material(color.factors);
        let handle = push_geom(
            &mut builder,
            &color.geom,
            origin,
            material,
            None,
            render.compute_flat_normal,
        );
        primitives.push((handle, color.geom.feature_ids));
    }

    let refs: Vec<(glb::PrimitiveHandle, &[u32])> = primitives
        .iter()
        .map(|(h, ids)| (*h, ids.as_slice()))
        .collect();
    metadata::encode(&table, &mut builder, &refs);

    let gltf_origin = [origin[0], origin[2], -origin[1]];
    let glb = builder.build(gltf_origin);

    if render.draco {
        reearth_flow_gltf::next::draco::compress(&glb)
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("draco compression failed: {e:?}")))
    } else {
        Ok(glb)
    }
}

fn color_material(factors: primitive::MaterialFactors) -> glb::MaterialDesc {
    glb::MaterialDesc {
        base_color_factor: factors.base_color_factor,
        metallic_factor: factors.metallic_factor,
        roughness_factor: factors.roughness_factor,
        base_color_texture: None,
    }
}

/// One atlas page realized as a glTF primitive: its embedded texture, the
/// subset of textured geometry whose UVs landed on that page, and those faces'
/// atlas-space per-corner UVs (parallel to the sub-geometry's corners).
struct TexturedPage {
    texture: glb::TextureRef,
    geom: Geom,
    corner_uv: Vec<[f32; 2]>,
}

/// Pack the cell's textured faces into one or more atlas pages, embed each page
/// with the configured [`TextureCodec`], and split the textured geometry so each
/// returned page carries only the faces whose UVs live on it (glTF binds one
/// texture per primitive). `Ok(None)` when packing produced no image, so the
/// caller falls back to colour-only.
fn build_textured_pages(
    builder: &mut glb::Builder,
    textured: &TexturedPrimitive,
    render: RenderOptions,
    textures: &mut TextureCache,
    embedded: &mut EmbeddedTextures,
) -> crate::errors::Result<Option<Vec<TexturedPage>>> {
    // Resolve each polygon's texture to a local file path: on-disk sources pass
    // through; embedded (in-memory) sources are materialized to a temp file (once
    // per distinct content) so the path-based atlas packer can read them.
    let mut polygon_paths: Vec<PathBuf> = Vec::with_capacity(textured.polygon_texture.len());
    for source in &textured.polygon_texture {
        polygon_paths.push(match source {
            TextureSource::File(path) => path.clone(),
            TextureSource::Embedded(data) => embedded.materialize(data)?,
        });
    }

    // Group polygons by source texture, one atlas polygon per source polygon;
    // `slots[p] = (input, polygon-within-input)` locates polygon `p`'s entry in
    // the atlas result.
    let mut inputs: Vec<TextureInput> = Vec::new();
    let mut path_index: HashMap<PathBuf, usize> = HashMap::new();
    let mut slots: Vec<(usize, usize)> = Vec::new();
    let mut tri_off = 0usize;
    for (polygon, &tris) in textured.geom.polygon_tris.iter().enumerate() {
        let corners = tris as usize * 3;
        let corner_off = tri_off * 3;
        let path = &polygon_paths[polygon];
        let pi = *path_index.entry(path.clone()).or_insert_with(|| {
            inputs.push(TextureInput {
                path: path.clone(),
                uvs: Vec::new(),
                scale: 1.0,
            });
            inputs.len() - 1
        });
        let poly = inputs[pi].uvs.len();
        inputs[pi]
            .uvs
            .push(textured.geom.corner_uv[corner_off..corner_off + corners].to_vec());
        slots.push((pi, poly));
        tri_off += tris as usize;
    }

    let scales = texture_target_scales(textured, &polygon_paths, &inputs, render.texel_size);
    for (input, scale) in inputs.iter_mut().zip(scales) {
        input.scale = scale;
    }

    let codec = codec_for(render.texture_codec);
    let built = match build_atlas_multipage(
        &inputs,
        render.atlas_size,
        render.atlas_extrusion,
        codec.block_align(),
        render.wrap_tolerance,
        textures,
    )
    .map_err(SinkError::cesium3dtiles_writer)?
    {
        Some(built) => built,
        None => return Ok(None),
    };

    let mut page_textures = Vec::with_capacity(built.pages.len());
    for (page, wrap) in built.pages.iter().zip(&built.wrap) {
        let texture = builder
            .push_atlas_texture(page, codec.as_ref(), page_sampler(*wrap))
            .map_err(SinkError::cesium3dtiles_writer)?;
        page_textures.push(texture);
    }

    Ok(Some(split_textured_by_page(
        textured,
        &built.remapped,
        &slots,
        page_textures,
    )))
}

/// Resolve the user-facing codec parameter to its glTF codec implementation.
fn codec_for(codec: TextureCodec) -> Box<dyn glb::Codec> {
    use reearth_flow_gltf::next::ktx2::{Ktx2Codec, Supercompression};
    match codec {
        TextureCodec::Ktx2Etc1s => Box::new(Ktx2Codec {
            supercompression: Supercompression::Etc1s,
        }),
        TextureCodec::Ktx2Uastc => Box::new(Ktx2Codec {
            supercompression: Supercompression::Uastc,
        }),
        TextureCodec::Png => Box::new(glb::PngCodec),
        TextureCodec::Jpeg => Box::new(glb::JpegCodec),
        // The `Untextured` cell is filtered out before texturing (see `build`),
        // so a codec is never resolved for it.
        TextureCodec::Untextured => unreachable!("Untextured cells are not textured"),
    }
}

/// Per input texture, the fraction of native resolution to keep so its
/// highest-density (finest metres-per-pixel) face is downsampled to
/// `texel_size` metres per pixel. One scale per texture: the coarser faces
/// sharing it may end up below `texel_size`, the accepted cost of not splitting
/// a texture by face. `texel_size == 0.0` disables downsampling (scale `1.0`).
fn texture_target_scales(
    textured: &TexturedPrimitive,
    polygon_paths: &[PathBuf],
    inputs: &[TextureInput],
    texel_size: f64,
) -> Vec<f64> {
    if texel_size <= 0.0 {
        return vec![1.0; inputs.len()];
    }
    // Native dimensions per input (header read only); `None` if unreadable.
    let dims: Vec<Option<(u32, u32)>> = inputs
        .iter()
        .map(|input| image::image_dimensions(&input.path).ok())
        .collect();
    let path_input: HashMap<&PathBuf, usize> = inputs
        .iter()
        .enumerate()
        .map(|(i, input)| (&input.path, i))
        .collect();

    // Finest metres-per-pixel over every face using each input.
    let mut min_mpp = vec![f64::INFINITY; inputs.len()];
    let mut tri_off = 0usize;
    for (polygon, &tris) in textured.geom.polygon_tris.iter().enumerate() {
        let tris = tris as usize;
        let range = tri_off..tri_off + tris;
        tri_off += tris;
        let pi = path_input[&polygon_paths[polygon]];
        let Some(size) = dims[pi] else { continue };
        if let Some(mpp) = polygon_metres_per_pixel(&textured.geom, range, size) {
            min_mpp[pi] = min_mpp[pi].min(mpp);
        }
    }

    min_mpp
        .into_iter()
        .map(|mpp| {
            if mpp.is_finite() {
                (mpp / texel_size).min(1.0)
            } else {
                1.0
            }
        })
        .collect()
}

/// Average metres-per-pixel over a polygon's triangle edges: world edge length
/// (ECEF metres) divided by its length in source-texture pixels. `None` when no
/// edge has a measurable pixel length.
fn polygon_metres_per_pixel(
    geom: &Geom,
    tris: std::ops::Range<usize>,
    (tw, th): (u32, u32),
) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for tri in tris {
        let idx = geom.indices[tri];
        for e in 0..3 {
            let (a, b) = (e, (e + 1) % 3);
            let pa = geom.positions[idx[a] as usize];
            let pb = geom.positions[idx[b] as usize];
            let world =
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                    .sqrt();
            let ua = geom.corner_uv[tri * 3 + a];
            let ub = geom.corner_uv[tri * 3 + b];
            let du = (ua[0] - ub[0]) * tw as f64;
            let dv = (ua[1] - ub[1]) * th as f64;
            let px = (du * du + dv * dv).sqrt();
            if px > 1e-6 && world.is_finite() {
                sum += world / px;
                n += 1;
            }
        }
    }
    (n > 0).then(|| sum / n as f64)
}

/// Split the single textured [`Geom`] into one [`Geom`] per atlas page, each
/// holding only the polygons whose UVs landed on that page and carrying those
/// polygons' atlas-space per-corner UVs. Vertices are re-welded per page (each
/// page's vertex buffer is compacted independently).
fn split_textured_by_page(
    textured: &TexturedPrimitive,
    remapped: &[Vec<reearth_flow_atlas::PolygonPlacement>],
    slots: &[(usize, usize)],
    textures: Vec<glb::TextureRef>,
) -> Vec<TexturedPage> {
    let pages = textures.len();
    let mut geoms: Vec<Geom> = (0..pages).map(|_| Geom::default()).collect();
    let mut corner_uvs: Vec<Vec<[f32; 2]>> = vec![Vec::new(); pages];
    // Per page, weld source vertex index -> that page's local vertex index.
    let mut remap: Vec<HashMap<u32, u32>> = vec![HashMap::new(); pages];

    let geom = &textured.geom;
    let mut tri_off = 0usize;
    for (polygon, &tris) in geom.polygon_tris.iter().enumerate() {
        let tris = tris as usize;
        let range = tri_off..tri_off + tris;
        tri_off += tris;

        let (pi, poly) = slots[polygon];
        let placement = &remapped[pi][poly];
        let page = placement.page;
        let out = &mut geoms[page];
        let page_remap = &mut remap[page];

        // `placement.uvs` is parallel to this polygon's source corners, in the
        // same triangle-corner order we emit below.
        let mut local_corner = 0usize;
        for tri in range {
            let mut out_tri = [0u32; 3];
            for (c, &orig) in geom.indices[tri].iter().enumerate() {
                let local = *page_remap.entry(orig).or_insert_with(|| {
                    let idx = out.positions.len() as u32;
                    out.positions.push(geom.positions[orig as usize]);
                    out.feature_ids.push(geom.feature_ids[orig as usize]);
                    idx
                });
                out_tri[c] = local;
                let [u, v] = placement.uvs[local_corner];
                corner_uvs[page].push([u as f32, v as f32]);
                local_corner += 1;
            }
            out.indices.push(out_tri);
        }
        out.polygon_normals.push(geom.polygon_normals[polygon]);
        out.polygon_tris.push(tris as u32);
    }

    textures
        .into_iter()
        .zip(geoms)
        .zip(corner_uvs)
        .map(|((texture, geom), corner_uv)| TexturedPage {
            texture,
            geom,
            corner_uv,
        })
        .collect()
}

/// Push one primitive from a [`Geom`], localizing positions to `origin` and
/// converting to glTF (Y-up -> Z-up) convention. `uv`, when present, is a
/// per-corner `TEXCOORD_0` parallel to the geometry's corners.
fn push_geom(
    builder: &mut glb::Builder,
    geom: &Geom,
    origin: [f64; 3],
    material: glb::MaterialDesc,
    uv: Option<Vec<[f32; 2]>>,
    compute_flat_normal: bool,
) -> glb::PrimitiveHandle {
    // 3D Tiles renderers rotate bare-glTF content Y-up -> Z-up on load; our
    // input is already Z-up (ECEF-relative), so pre-apply the inverse and the
    // renderer's rotation cancels out.
    let positions: Vec<[f32; 3]> = geom
        .positions
        .iter()
        .map(|p| {
            [
                (p[0] - origin[0]) as f32,
                (p[2] - origin[2]) as f32,
                -((p[1] - origin[1]) as f32),
            ]
        })
        .collect();

    let mut dedup_attrs = Vec::new();
    if compute_flat_normal {
        // Same axis swap as position, no translation (a normal is a direction).
        let normals: Vec<[f32; 3]> = geom
            .polygon_normals
            .iter()
            .map(|&[x, y, z]| [x as f32, z as f32, -y as f32])
            .collect();
        dedup_attrs.push(glb::normal(Granularity::PerPolygon, normals));
    }
    let corner_src: Vec<u32> = if uv.is_some() {
        (0..geom.indices.len() as u32 * 3).collect()
    } else {
        Vec::new()
    };
    if let Some(uv) = uv {
        dedup_attrs.push(glb::texcoord(uv));
    }

    builder.push_primitive(
        positions,
        geom.indices.clone(),
        material,
        &geom.polygon_tris,
        &corner_src,
        dedup_attrs,
    )
}

/// Centroid of every primitive's vertices — a tile-local origin near the
/// geometry. Shared vertices count once per primitive, which is immaterial to a
/// centroid used only to keep f32 positions small.
fn cell_origin(cells: &primitive::CellPrimitives) -> [f64; 3] {
    let mut sum = [0.0f64; 3];
    let mut count = 0usize;
    let mut add = |positions: &[[f64; 3]]| {
        for p in positions {
            sum[0] += p[0];
            sum[1] += p[1];
            sum[2] += p[2];
        }
        count += positions.len();
    };
    for color in &cells.color {
        add(&color.geom.positions);
    }
    if let Some(textured) = &cells.textured {
        add(&textured.geom.positions);
    }
    if count == 0 {
        [0.0, 0.0, 0.0]
    } else {
        [
            sum[0] / count as f64,
            sum[1] / count as f64,
            sum[2] / count as f64,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_types::Attributes;
    use std::io::Cursor;
    use std::sync::Mutex;

    /// A minimal red 2x2 PNG, encoded in memory.
    fn tiny_png() -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .expect("encode png");
        png
    }

    // An embedded (in-memory) texture — what a glTF/GLB packed image decodes to —
    // must be materialized to a temp file, atlased, and embedded as an atlas page,
    // not dropped. Exercises the whole `Raster::InMemory` writer path.
    #[test]
    fn embedded_texture_is_materialized_and_atlased() {
        let textured = TexturedPrimitive {
            geom: Geom {
                positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                indices: vec![[0, 1, 2]],
                polygon_normals: vec![[0.0, 0.0, 1.0]],
                polygon_tris: vec![1],
                corner_uv: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                feature_ids: vec![0, 0, 0],
            },
            polygon_texture: vec![TextureSource::Embedded(RasterData {
                mime_type: MimeType::ImagePng,
                bytes: bytes::Bytes::from(tiny_png()),
            })],
        };
        let render = RenderOptions {
            draco: false,
            compute_flat_normal: false,
            texel_size: 0.0,
            atlas_size: 1024,
            atlas_extrusion: 0,
            wrap_tolerance: 0.0,
            texture_codec: TextureCodec::Png,
        };

        let mut builder = glb::Builder::new();
        let mut cache = TextureCache::default();
        let mut embedded = EmbeddedTextures::new().expect("temp dir");

        let pages =
            build_textured_pages(&mut builder, &textured, render, &mut cache, &mut embedded)
                .expect("build textured pages")
                .expect("an in-memory texture must produce an atlas page, not colour-only");

        assert_eq!(pages.len(), 1, "one atlas page for the single texture");
        assert_eq!(
            pages[0].geom.indices.len(),
            1,
            "the textured triangle is kept"
        );
        // The embedded bytes were written to a temp file and cached by content hash.
        assert_eq!(embedded.by_hash.len(), 1);
    }

    // End to end through the public `build`: a CRS-framed TriangularMesh carrying an
    // embedded (in-memory) base-colour texture must produce a content glb that embeds
    // the texture image — proving the whole chain (extract -> appearance resolve ->
    // collect -> materialize -> atlas -> glb embed) works for in-memory textures.
    // Uses a geographic CRS so the mesh isn't skipped and the Png codec so the
    // embedded image carries a recognizable `image/png` mime; independent of the
    // (separate) glTF georeferencing concern.
    #[test]
    fn embedded_texture_round_trips_through_build() {
        use reearth_flow_geometry::appearance::{
            AlphaMode, ChannelId, Material, PbrMaterial, Raster, RasterData, Sampler, Texture,
            ThemeId, UvSource,
        };
        use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
        use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
        use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
        use reearth_flow_types::Attributes;
        use std::sync::Mutex;

        let frame = CoordinateFrame::Crs(EpsgCode::new(4979));
        let mut mesh = TriangularMesh3D::from_soup(
            frame,
            [
                [35.0, 139.0, 10.0],
                [35.0, 139.001, 10.0],
                [35.001, 139.0, 10.0],
            ],
        );
        mesh.set_appearance(
            ThemeId(Arc::from("default")),
            Material::Pbr(PbrMaterial {
                base_color: [1.0, 1.0, 1.0, 1.0],
                metallic: 1.0,
                roughness: 1.0,
                emissive: [0.0, 0.0, 0.0],
                base_color_map: Some(Texture {
                    raster: Arc::new(Raster::InMemory(RasterData {
                        mime_type: MimeType::ImagePng,
                        bytes: bytes::Bytes::from(tiny_png()),
                    })),
                    sampler: Sampler::default(),
                    transform: None,
                    uv_channel: ChannelId(0),
                }),
                metallic_roughness_map: None,
                normal_map: None,
                occlusion_map: None,
                emissive_map: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
            }),
            Some(UvSource::Explicit(
                vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]].into_boxed_slice(),
            )),
        )
        .expect("attach embedded texture");

        let feature = reearth_flow_types::Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh))),
        );
        let render = RenderOptions {
            draco: false,
            compute_flat_normal: false,
            texel_size: 0.0,
            atlas_size: 1024,
            atlas_extrusion: 0,
            wrap_tolerance: 0.0,
            texture_codec: TextureCodec::Png,
        };
        let options = MetadataOptions {
            schema_key: None,
            skip_unexposed_attributes: false,
            array_map_separator: Some("_"),
        };

        // `build` streams each content glb to the callback; capture them.
        let tiles = Mutex::new(Vec::new());
        let built = build(
            &[feature],
            options,
            DEFAULT_TARGET_TILE_SIZE,
            render,
            |_path: String, glb| {
                tiles.lock().unwrap().push(glb);
                Ok(())
            },
        )
        .expect("build tileset");

        assert_eq!(built.tile_count, 1, "textured mesh produced a content glb");
        let tiles = tiles.into_inner().unwrap();
        assert_eq!(tiles.len(), 1);
        let glb = &tiles[0];
        let needle = b"image/png";
        assert!(
            glb.windows(needle.len()).any(|w| w == needle),
            "the embedded texture is emitted as an image/png texture in the glb"
        );
    }

    /// A geocentric (EPSG:4978) mesh must produce actual tile content: a
    /// Euclidean mesh, lacking a georeferenced frame, yields an empty
    /// tileset instead. This also proves the writer accepts a geocentric
    /// declared frame.
    #[test]
    fn geocentric_mesh_produces_tile_content() {
        // Small triangle near lat 35.908, lon 140.102 in ECEF metres.
        let base = [-3958731.9, -3309830.0, 3736419.1];
        let soup = vec![
            base,
            [base[0] + 30.0, base[1], base[2]],
            [base[0], base[1] + 30.0, base[2]],
        ];
        let mesh = TriangularMesh3D::from_soup(CoordinateFrame::Crs(EpsgCode::new(4978)), soup);
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh))),
        );

        let tiles = Mutex::new(Vec::new());
        let built = build(
            &[feature],
            plain_metadata_options(),
            DEFAULT_TARGET_TILE_SIZE,
            plain_render_options(),
            |name: String, glb| {
                tiles.lock().unwrap().push((name, glb));
                Ok(())
            },
        )
        .expect("build tileset");

        assert!(
            built.tile_count > 0,
            "a geocentric mesh must produce tile content"
        );
        let written = tiles.into_inner().unwrap();
        assert!(
            !written.is_empty(),
            "a geocentric mesh must produce tile content"
        );
        assert!(
            written.iter().all(|(_, bytes)| !bytes.is_empty()),
            "tiles must not be empty: {:?}",
            written
                .iter()
                .map(|(n, b)| (n.clone(), b.len()))
                .collect::<Vec<_>>()
        );
    }

    /// A plain untextured triangle at `lat, lon`, far enough from other test
    /// features to land in its own quadtree cell at deep placement levels.
    fn untextured_feature(lat: f64, lon: f64) -> Feature {
        let frame = CoordinateFrame::Crs(EpsgCode::new(4979));
        let mesh = TriangularMesh3D::from_soup(
            frame,
            [
                [lat, lon, 10.0],
                [lat, lon + 0.0001, 10.0],
                [lat + 0.0001, lon, 10.0],
            ],
        );
        Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh))),
        )
    }

    fn plain_render_options() -> RenderOptions {
        RenderOptions {
            draco: false,
            compute_flat_normal: false,
            texel_size: 0.0,
            atlas_size: 1024,
            atlas_extrusion: 0,
            wrap_tolerance: 0.0,
            texture_codec: TextureCodec::Png,
        }
    }

    fn plain_metadata_options() -> MetadataOptions<'static> {
        MetadataOptions {
            schema_key: None,
            skip_unexposed_attributes: false,
            array_map_separator: Some("_"),
        }
    }

    // Two features far enough apart to place in different leaf cells must still
    // land in one content glb once `target_tile_size` is large relative to their
    // (tiny) combined cost — proving `merge_small_cells` folds undersized sibling
    // cells upward instead of leaving the tileset needlessly fragmented.
    #[test]
    fn small_cells_merge_into_one_tile_under_a_large_target_size() {
        let features = [
            untextured_feature(35.0, 139.0),
            untextured_feature(36.0, 140.0),
        ];
        let tiles = Mutex::new(Vec::new());
        let built = build(
            &features,
            plain_metadata_options(),
            DEFAULT_TARGET_TILE_SIZE,
            plain_render_options(),
            |_path: String, glb| {
                tiles.lock().unwrap().push(glb);
                Ok(())
            },
        )
        .expect("build tileset");

        assert_eq!(
            built.tile_count, 1,
            "two spatially separate but tiny-cost features merge into one tile"
        );
        assert_eq!(tiles.into_inner().unwrap().len(), 1);
    }

    // The same two features, but with `target_tile_size` set below what even one
    // of them costs alone, so no merge/co-placement can fit them together —
    // proving `split_by_size` (and, since they'd otherwise share a cell, its
    // per-cell same-tile-content splitting) keeps every chunk under the target.
    #[test]
    fn oversized_cell_splits_into_multiple_same_tile_contents() {
        // Same location twice: both are forced into the same leaf cell
        // regardless of `SAFETY_MAX_DEPTH`, isolating `split_by_size`'s
        // same-tile-content behaviour from `merge_small_cells`.
        let features = [
            untextured_feature(35.0, 139.0),
            untextured_feature(35.0, 139.0),
        ];
        let paths = Mutex::new(Vec::new());
        let built = build(
            &features,
            plain_metadata_options(),
            1,
            plain_render_options(),
            |path: String, _glb| {
                paths.lock().unwrap().push(path);
                Ok(())
            },
        )
        .expect("build tileset");

        assert_eq!(
            built.tile_count, 2,
            "target size of 1 byte forces each feature into its own content chunk"
        );
        let paths = paths.into_inner().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(
            paths.iter().any(|p| p.ends_with("/0.glb"))
                && paths.iter().any(|p| p.ends_with("/0_1.glb")),
            "slot 0 keeps the plain name and slot 1 gets a suffix once a cell splits: {paths:?}"
        );
    }

    /// Bytes of every content glb written for `features` under `target`.
    fn content_sizes(features: &[Feature], target: u64) -> Vec<usize> {
        let sizes = Mutex::new(Vec::new());
        build(
            features,
            plain_metadata_options(),
            target,
            plain_render_options(),
            |_path: String, glb| {
                sizes.lock().unwrap().push(glb.len());
                Ok(())
            },
        )
        .expect("build tileset");
        sizes.into_inner().unwrap()
    }

    // The split decision follows the bytes the writer emits, not the estimate:
    // a target one byte under the pair's glb splits the cell into one content
    // per feature, while a target equal to it keeps the cell whole.
    #[test]
    fn split_follows_measured_glb_bytes() {
        let one = content_sizes(&[untextured_feature(35.0, 139.0)], u64::MAX);
        assert_eq!(one.len(), 1);
        let single = one[0];

        let pair = [
            untextured_feature(35.0, 139.0),
            untextured_feature(35.0, 139.0),
        ];
        let whole = content_sizes(&pair, u64::MAX);
        assert_eq!(whole.len(), 1);
        let both = whole[0] as u64;
        assert!(both > single as u64);

        let split = content_sizes(&pair, both - 1);
        assert_eq!(split.len(), 2, "pair glb exceeds the target");
        assert!(split.iter().all(|&b| b == single));

        let kept = content_sizes(&pair, both);
        assert_eq!(kept.len(), 1, "pair glb fits the target exactly");
    }

    // A cell that would need more contents than `MAX_CONTENTS_PER_TILE` stops
    // splitting at the cap and keeps oversized contents instead.
    #[test]
    fn contents_per_tile_are_capped() {
        let features: Vec<Feature> = (0..MAX_CONTENTS_PER_TILE * 3)
            .map(|_| untextured_feature(35.0, 139.0))
            .collect();
        let sizes = content_sizes(&features, 1);
        assert_eq!(sizes.len(), MAX_CONTENTS_PER_TILE);
    }

    #[test]
    fn split_into_weights_items_and_never_yields_one_group() {
        let weights = [10u64, 10, 10, 10, 100, 1, 1];
        let all: Vec<usize> = (0..weights.len()).collect();
        let w = |i: &usize| weights[*i];
        assert_eq!(
            split_into(all.clone(), w, 3),
            vec![vec![0, 1, 2, 3], vec![4], vec![5, 6]]
        );
        assert_eq!(split_into(all, w, 2), vec![vec![0, 1, 2, 3], vec![4, 5, 6]]);
        // The dominant item first: the rest still forms a second group.
        let weights = [100u64, 1, 1];
        assert_eq!(
            split_into(vec![0usize, 1, 2], |i| weights[*i], 2),
            vec![vec![0], vec![1, 2]]
        );
        // No weight signal at all: equal counts.
        assert_eq!(
            split_into((0..5).collect::<Vec<usize>>(), |_| 0, 2),
            vec![vec![0, 1, 2], vec![3, 4]]
        );
    }

    // Merging follows measured bytes: two far-apart features fold into one
    // tile exactly when their joint glb fits the target, which the writer
    // predicts from the two leaf glbs less the per-glb overhead it measured.
    #[test]
    fn merge_follows_measured_glb_bytes() {
        let features = [
            untextured_feature(35.0, 139.0),
            untextured_feature(36.0, 140.0),
        ];
        let whole = content_sizes(&features, u64::MAX);
        assert_eq!(whole.len(), 1);
        let joint = whole[0] as u64;

        assert_eq!(content_sizes(&features, joint).len(), 1);
        assert_eq!(content_sizes(&features, joint - 1).len(), 2);
    }
}

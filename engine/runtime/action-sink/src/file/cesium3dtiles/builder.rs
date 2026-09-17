use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::path::PathBuf;

use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;

use reearth_flow_atlas::{build_atlas_multipage, TextureCache, TextureInput};
use reearth_flow_gltf::tiles::glb::{self, Granularity};
use reearth_flow_gltf::tiles::metadata;
pub use reearth_flow_gltf::tiles::metadata::MetadataOptions;
use reearth_flow_types::geometry::GeometryValue;
use reearth_flow_types::Feature;

use super::appearance::TextureSource;
use super::mesh;
use super::primitive::{self, Geom, TexturedPrimitive, DEFAULT_MATERIAL};
use super::quadtree::{self, Cell, GeoBox};
use super::sink::TextureCodec;
use super::{subtree, tileset};
use crate::errors::SinkError;

/// The tileset's non-glb outputs, relative to its output directory: the
/// `tileset.json` text and one or more `.subtree` files. The content glbs are
/// streamed out through `build`'s `write_tile` callback as they are produced,
/// not held here; `tile_count` records how many were written.
pub(super) struct BuiltTileset {
    pub(super) tileset_json: String,
    pub(super) subtrees: Vec<(String, Vec<u8>)>,
    pub(super) tile_count: usize,
}

/// Rendering knobs shared by every cell of a tileset.
#[derive(Clone, Copy)]
pub(super) struct RenderOptions {
    /// Draco-compress each glb.
    pub(super) draco: bool,
    /// Attach per-polygon flat normals for lighting.
    pub(super) compute_flat_normal: bool,
    /// Target texel size in metres per pixel: textures finer than this are
    /// downsampled to it. `0.0` keeps full texture detail.
    pub(super) texel_size: f64,
    /// Maximum atlas page dimension (pixels). Textures/atlases exceeding it
    /// spill onto additional pages; a single texture larger than it is
    /// force-shrunk to fit one page.
    pub(super) atlas_size: u32,
    /// Extrusion ring (pixels) blitted around each atlas region to stop
    /// bilinear bleed between neighbours. `0` disables it.
    pub(super) atlas_extrusion: u32,
    /// How far outside `[0, 1]` a UV may stray and still be clamped as drift;
    /// past it the texture is taken to tile and gets an atlas page of its own.
    pub(super) wrap_tolerance: f64,
    /// Image codec for atlas pages. `Untextured` attaches no textures; textured
    /// geometry falls back to its neutral colour.
    pub(super) texture_codec: TextureCodec,
}

/// Hard safety cap on quadtree depth, well beyond any depth `target_tile_size`
/// would realistically drive placement to; guards against pathological inputs
/// (e.g. many coincident features) rather than acting as a tuning knob.
const SAFETY_MAX_DEPTH: u32 = 24;

/// Upper bound on same-tile contents a cell may split into; a cell that would
/// need more keeps contents over `target_tile_size` instead.
const MAX_CONTENTS_PER_TILE: usize = 15;

/// Content glbs stream through `write_tile` as each cell is built rather than
/// being retained, so peak memory stays at one cell's contents regardless of
/// tile count.
pub(super) fn build(
    features: &[Feature],
    schema: &nusamai_citygml::schema::Schema,
    options: MetadataOptions,
    target_tile_size: u64,
    render: RenderOptions,
    write_tile: impl Fn(String, Vec<u8>) -> crate::errors::Result<()> + Sync,
) -> crate::errors::Result<BuiltTileset> {
    let extracted: Vec<(&Feature, mesh::ExtractedMesh)> = features
        .iter()
        .filter_map(|feature| match &feature.geometry.value {
            GeometryValue::CityGmlGeometry(city_gml) => {
                mesh::extract(city_gml).map(|m| (feature, m))
            }
            _ => None,
        })
        .collect();

    let property_stats = super::stats::collect(
        extracted.iter().map(|(feature, _)| *feature),
        schema,
        options,
    );

    if extracted.is_empty() {
        tracing::warn!(
            "Cesium3DTilesWriter: no renderable geometry found; writing an empty tileset"
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

    // Callers own the caches, so a cell's candidate builds decode its sources
    // once between them. PLATEAU textures are per-surface, so a source image is
    // referenced by only one cell; a tileset-wide cache would grow without
    // bound for no reuse gain.
    let build_chunk = |chunk: &[usize], textures: &TextureCache| {
        let members: Vec<&(&Feature, mesh::ExtractedMesh)> =
            chunk.iter().map(|&i| &extracted[i]).collect();
        build_cell_glb(&members, schema, options, render, textures)
    };

    let mut units: HashMap<Cell, Vec<Unit>> = by_cell
        .into_par_iter()
        .map(
            |(cell, features)| -> crate::errors::Result<(Cell, Vec<Unit>)> {
                let textures = TextureCache::default();
                let bytes = transfer_bytes(&build_chunk(&features, &textures)?);
                Ok((cell, vec![Unit { features, bytes }]))
            },
        )
        .collect::<crate::errors::Result<_>>()?;

    merge_small_cells(&mut units, target_tile_size);

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
                let textures = TextureCache::default();
                let contents = split_by_size(units, target_tile_size, &textures, build_chunk)?;
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
    })
}

/// A group of features whose glb, built on its own, takes `bytes` in transfer.
struct Unit {
    features: Vec<usize>,
    bytes: u64,
}

/// Transfer bytes of a glb holding all of `units`, taken as their sum.
fn group_bytes(units: &[Unit]) -> u64 {
    units.iter().map(|u| u.bytes).sum()
}

/// A [`Write`] sink that keeps only the number of bytes written to it.
struct ByteCounter(u64);

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0 += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Bytes `glb` occupies in transfer: its gzipped length, tile contents being
/// served with `Content-Encoding: gzip`.
fn transfer_bytes(glb: &[u8]) -> u64 {
    const COUNTER_NEVER_FAILS: &str = "writing to a byte counter never fails";
    let mut encoder = GzEncoder::new(ByteCounter(0), Compression::fast());
    encoder.write_all(glb).expect(COUNTER_NEVER_FAILS);
    encoder.finish().expect(COUNTER_NEVER_FAILS).0
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
    textures: &TextureCache,
    build_chunk: impl Fn(&[usize], &TextureCache) -> crate::errors::Result<Vec<u8>> + Sync,
) -> crate::errors::Result<CellContents> {
    let expected = group_bytes(&units);
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
                build_chunk(&features, textures).map(|glb| (chunk, glb))
            })
            .collect::<crate::errors::Result<_>>()?;
        let total = built.len();
        for (k, (chunk, glb)) in built.into_iter().enumerate() {
            let bytes = transfer_bytes(&glb);
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

/// Fold small sibling cells upward into their parent while their summed
/// transfer bytes stay within `target_tile_size`, deepest level first so a
/// fold can cascade to the root.
fn merge_small_cells(units: &mut HashMap<Cell, Vec<Unit>>, target_tile_size: u64) {
    let max_level = units.keys().map(|c| c.level).max().unwrap_or(0);
    for level in (1..=max_level).rev() {
        let mut by_parent: HashMap<Cell, Vec<Cell>> = HashMap::new();
        for cell in units.keys().filter(|c| c.level == level) {
            if let Some(parent) = cell.parent() {
                by_parent.entry(parent).or_default().push(*cell);
            }
        }
        for (parent, mut children) in by_parent {
            children.sort_by_key(|c| (group_bytes(&units[c]), *c));
            for child in children {
                let parent_bytes: u64 = units
                    .get(&parent)
                    .map(|units| group_bytes(units))
                    .unwrap_or(0);
                if parent_bytes + group_bytes(&units[&child]) > target_tile_size {
                    continue;
                }
                let moved = units.remove(&child).unwrap();
                units.entry(parent).or_default().extend(moved);
            }
        }
    }
}

fn empty_tileset(
    property_stats: &indexmap::IndexMap<String, super::stats::PropertyStats>,
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
    })
}

fn render_tileset_json(
    root: &GeoBox,
    available_levels: u32,
    max_contents: usize,
    property_stats: &indexmap::IndexMap<String, super::stats::PropertyStats>,
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

/// A packed page holds many textures side by side, so a repeating wrap would
/// bleed across sub-images; a page the packer gave to one tiling texture
/// wraps that texture.
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
    schema: &nusamai_citygml::schema::Schema,
    options: MetadataOptions,
    render: RenderOptions,
    textures: &TextureCache,
) -> crate::errors::Result<Vec<u8>> {
    let cells = primitive::collect(cell_members);

    let cell_features: Vec<&Feature> = cell_members.iter().map(|(f, _)| *f).collect();
    let feature_types: BTreeSet<String> = cell_features
        .iter()
        .filter_map(|f| f.feature_type())
        .collect();
    let schemas: Vec<&nusamai_citygml::schema::Map> = feature_types
        .iter()
        .filter_map(|ft| crate::schema::schema_attributes(ft, schema))
        .collect();
    let table = metadata::build_table(&cell_features, &schemas, options);

    // Per-tile local origin keeps the f32 positions small next to ECEF's
    // ~6.378e6 m magnitude (see [`push_geom`]).
    let origin = cell_origin(&cells);

    let mut builder = glb::Builder::new();
    // Each primitive keeps its own per-vertex feature IDs (its vertex buffer is
    // compacted independently), pushed with it as a dedup attribute.
    let mut primitives: Vec<glb::PrimitiveHandle> = Vec::new();

    if let Some(textured) = cells.textured {
        // `Untextured` skips texturing entirely and renders the textured
        // geometry in the neutral fallback colour.
        let pages = match render.texture_codec {
            TextureCodec::Untextured => None,
            _ => build_textured_pages(&mut builder, &textured, render, textures)?,
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
                    primitives.push(handle);
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
                primitives.push(handle);
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
        primitives.push(handle);
    }

    metadata::encode(&table, &mut builder, &primitives);

    let gltf_origin = [origin[0], origin[2], -origin[1]];
    let glb = builder.build(gltf_origin);

    if render.draco {
        reearth_flow_gltf::tiles::draco::compress(&glb)
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

/// Pack the cell's textured faces into one or more atlas pages, embed each
/// page with the configured [`TextureCodec`], and split the textured geometry
/// so each returned page carries only the faces whose UVs live on it (glTF
/// binds one texture per primitive). `Ok(None)` when packing produced no
/// image, so the caller falls back to colour-only.
fn build_textured_pages(
    builder: &mut glb::Builder,
    textured: &TexturedPrimitive,
    render: RenderOptions,
    textures: &TextureCache,
) -> crate::errors::Result<Option<Vec<TexturedPage>>> {
    let polygon_paths: Vec<PathBuf> = textured
        .polygon_texture
        .iter()
        .map(|TextureSource::File(path)| path.clone())
        .collect();

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
    use reearth_flow_gltf::tiles::ktx2::{Ktx2Codec, Supercompression};
    match codec {
        TextureCodec::Ktx2Etc1s => Box::new(Ktx2Codec {
            supercompression: Supercompression::Etc1s,
        }),
        TextureCodec::Ktx2Uastc => Box::new(Ktx2Codec {
            supercompression: Supercompression::Uastc,
        }),
        TextureCodec::Png => Box::new(glb::PngCodec),
        TextureCodec::Jpeg => Box::new(glb::JpegCodec),
        // The `Untextured` cell is filtered out before texturing (see `build_cell_glb`),
        // so a codec is never resolved for it.
        TextureCodec::Untextured => unreachable!("Untextured cells are not textured"),
    }
}

/// Per input texture, the fraction of native resolution to keep so its
/// highest-density (finest metres-per-pixel) face is downsampled to
/// `texel_size` metres per pixel. One scale per texture: the coarser faces
/// sharing it may end up below `texel_size`, the accepted cost of not
/// splitting a texture by face. `texel_size == 0.0` disables downsampling
/// (scale `1.0`).
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
/// (ECEF metres) divided by its length in source-texture pixels. `None` when
/// no edge has a measurable pixel length.
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
    dedup_attrs.push(glb::scalar_u32(
        "FEATURE_ID_0",
        Granularity::PerVertex,
        geom.feature_ids.clone(),
    ));

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
/// geometry. Shared vertices count once per primitive, which is immaterial to
/// a centroid used only to keep f32 positions small.
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
    use nusamai_citygml::schema::Schema;
    use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
    use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
    use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_types::geometry::{CityGmlGeometry, GeometryType, GmlGeometry};
    use reearth_flow_types::metadata::Metadata;
    use reearth_flow_types::{Attributes, Geometry};
    use std::sync::Mutex;

    const DEFAULT_TARGET_TILE_SIZE: u64 = 1_048_576;

    /// A plain untextured triangle at `lat, lon`, far enough from other test
    /// features to land in its own quadtree cell at deep placement levels.
    fn untextured_feature(lat: f64, lon: f64) -> Feature {
        attributed_feature(lat, lon, &[])
    }

    /// [`untextured_feature`] carrying `attributes`, so cells can be given
    /// disjoint metadata schemas.
    fn attributed_feature(lat: f64, lon: f64, attributes: &[(&str, &str)]) -> Feature {
        let exterior = LineString3D::new(vec![
            Coordinate3D::new__(lon, lat, 10.0),
            Coordinate3D::new__(lon + 0.0001, lat, 10.0),
            Coordinate3D::new__(lon, lat + 0.0001, 10.0),
        ]);
        let uv_exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(0.0, 1.0),
        ]);
        let city_gml = CityGmlGeometry {
            gml_geometries: vec![GmlGeometry {
                id: None,
                ty: GeometryType::Surface,
                gml_trait: None,
                lod: Some(2),
                pos: 0,
                len: 1,
                polygons: vec![Polygon3D::new(exterior, vec![])],
                line_strings: vec![],
                points: vec![],
                feature_id: None,
                feature_type: None,
            }],
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![None],
            polygon_textures: vec![None],
            polygon_uvs: MultiPolygon2D::new(vec![Polygon2D::new(uv_exterior, vec![])]),
        };
        let mut attrs = Attributes::new();
        for (key, value) in attributes {
            attrs.insert(
                reearth_flow_types::Attribute::new(*key),
                reearth_flow_types::AttributeValue::String((*value).to_string()),
            );
        }
        Feature::new_with_attributes_and_geometry(
            attrs,
            Geometry {
                epsg: Some(4979),
                value: GeometryValue::CityGmlGeometry(city_gml),
            },
            Metadata::default(),
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
            &Schema::default(),
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
            &Schema::default(),
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

    /// Transfer bytes of every content glb written for `features` under
    /// `target`.
    fn content_sizes(features: &[Feature], target: u64) -> Vec<u64> {
        let sizes = Mutex::new(Vec::new());
        build(
            features,
            &Schema::default(),
            plain_metadata_options(),
            target,
            plain_render_options(),
            |_path: String, glb| {
                sizes.lock().unwrap().push(transfer_bytes(&glb));
                Ok(())
            },
        )
        .expect("build tileset");
        sizes.into_inner().unwrap()
    }

    // The split decision follows the transfer bytes the writer emits: a target
    // one byte under the pair's gzipped glb splits the cell into one content
    // per feature, while a target equal to it keeps the cell whole.
    #[test]
    fn split_follows_measured_transfer_bytes() {
        let one = content_sizes(&[untextured_feature(35.0, 139.0)], u64::MAX);
        assert_eq!(one.len(), 1);
        let single = one[0];

        let pair = [
            untextured_feature(35.0, 139.0),
            untextured_feature(35.0, 139.0),
        ];
        let whole = content_sizes(&pair, u64::MAX);
        assert_eq!(whole.len(), 1);
        let both = whole[0];
        assert!(both > single);

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

    // Merging follows the summed transfer bytes of the cells being folded:
    // two far-apart features share a tile exactly when their separate contents
    // together fit the target.
    #[test]
    fn merge_follows_summed_transfer_bytes() {
        let features = [
            untextured_feature(35.0, 139.0),
            untextured_feature(36.0, 140.0),
        ];
        let apart = content_sizes(&features, 1);
        assert_eq!(apart.len(), 2);
        let summed: u64 = apart.iter().sum();

        assert_eq!(content_sizes(&features, summed).len(), 1);
        assert_eq!(content_sizes(&features, summed - 1).len(), 2);
    }

    #[test]
    fn content_slot_zero_keeps_the_plain_name() {
        let cell = Cell {
            level: 3,
            x: 1,
            y: 2,
        };
        assert_eq!(content_path(cell, 0), "content/3/1/2.glb");
        assert_eq!(content_path(cell, 1), "content/3/1/2_1.glb");
    }
}

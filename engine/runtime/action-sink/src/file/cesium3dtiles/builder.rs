use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

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

/// Build one tileset (tileset.json + subtrees + streamed content glbs) from a
/// batch of features already resolved to one output path. Content glbs stream
/// through `write_tile` as built rather than being retained, so peak memory
/// stays at one glb per rayon worker regardless of tile count. Features
/// without a `CityGmlGeometry` are skipped.
pub(super) fn build(
    features: &[Feature],
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

    if extracted.is_empty() {
        tracing::warn!("Cesium3DTilesWriter: no renderable geometry found; writing an empty tileset");
        return empty_tileset();
    }

    let root = extracted
        .iter()
        .filter_map(|(_, m)| GeoBox::of(&m.geographic_vertices))
        .reduce(GeoBox::union)
        .expect("extracted is non-empty, and mesh::extract never returns an empty vertex buffer");

    let mut cost_caches = super::cost::CostCaches::default();
    let mut by_cell: HashMap<Cell, Vec<usize>> = HashMap::new();
    let mut cell_cost: HashMap<Cell, u64> = HashMap::new();
    let mut feature_cost: Vec<u64> = vec![0; extracted.len()];
    for (i, (feature, m)) in extracted.iter().enumerate() {
        let Some(feature_box) = GeoBox::of(&m.geographic_vertices) else {
            continue;
        };
        let cell = quadtree::place(&root, &feature_box, SAFETY_MAX_DEPTH);
        let cost = super::cost::estimate(feature, m, &mut cost_caches);
        by_cell.entry(cell).or_default().push(i);
        *cell_cost.entry(cell).or_default() += cost;
        feature_cost[i] = cost;
    }

    merge_small_cells(&mut by_cell, &mut cell_cost, target_tile_size);

    let occupied: BTreeSet<Cell> = by_cell.keys().copied().collect();
    let available_levels = occupied.iter().map(|c| c.level).max().unwrap_or(0) + 1;

    // A cell over `target_tile_size` splits into several same-tile contents
    // (3D Tiles 1.1 multiple contents) rather than growing an oversized glb;
    // this only aids fetch parallelism, since the union of contents holds
    // exactly the cell's features either way.
    let cell_contents: Vec<(Cell, Vec<Vec<usize>>)> = by_cell
        .into_iter()
        .map(|(cell, indices)| {
            (
                cell,
                split_by_cost(&indices, &feature_cost, target_tile_size),
            )
        })
        .collect();
    let content_counts: HashMap<Cell, usize> = cell_contents
        .iter()
        .map(|(cell, chunks)| (*cell, chunks.len()))
        .collect();
    // The content URI template and the subtree `contentAvailability` array are
    // both declared once for the whole tileset, so every cell shares the same
    // slot count even where only one cell actually splits.
    let max_contents = content_counts.values().copied().max().unwrap_or(1);
    let tile_count: usize = cell_contents.iter().map(|(_, chunks)| chunks.len()).sum();

    // Cells are independent (own texture cache, own glb(s), unique output
    // path), so render them across the rayon pool. Each glb streams straight
    // to `write_tile` as it is built, so peak memory stays at one glb per
    // worker rather than the whole tileset.
    cell_contents
        .par_iter()
        .try_for_each(|(cell, chunks)| -> crate::errors::Result<()> {
            let mut textures = TextureCache::default();
            for (n, indices) in chunks.iter().enumerate() {
                let cell_members: Vec<&(&Feature, mesh::ExtractedMesh)> =
                    indices.iter().map(|&i| &extracted[i]).collect();
                let glb = build_cell_glb(&cell_members, options, render, &mut textures)?;
                write_tile(content_path(*cell, n, max_contents > 1), glb)?;
            }
            Ok(())
        })?;

    let tileset_bytes = render_tileset_json(&root, available_levels, max_contents)?;
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

/// Partition a cell's features into fetch-parallel content chunks, each kept
/// under `target_tile_size` where possible; a single feature already over the
/// target is kept whole in its own chunk (features are never split).
fn split_by_cost(indices: &[usize], feature_cost: &[u64], target_tile_size: u64) -> Vec<Vec<usize>> {
    let mut chunks: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut current_cost = 0u64;
    for &i in indices {
        let cost = feature_cost[i];
        if !current.is_empty() && current_cost + cost > target_tile_size {
            chunks.push(std::mem::take(&mut current));
            current_cost = 0;
        }
        current.push(i);
        current_cost += cost;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Fold small sibling cells upward into their parent while staying within
/// `target_tile_size`, deepest level first so a fold can cascade to the root.
fn merge_small_cells(
    by_cell: &mut HashMap<Cell, Vec<usize>>,
    cell_cost: &mut HashMap<Cell, u64>,
    target_tile_size: u64,
) {
    let max_level = by_cell.keys().map(|c| c.level).max().unwrap_or(0);
    for level in (1..=max_level).rev() {
        let mut by_parent: HashMap<Cell, Vec<Cell>> = HashMap::new();
        for cell in by_cell.keys().filter(|c| c.level == level) {
            if let Some(parent) = cell.parent() {
                by_parent.entry(parent).or_default().push(*cell);
            }
        }
        for (parent, mut children) in by_parent {
            children.sort_by_key(|c| cell_cost[c]);
            let mut parent_cost = cell_cost.get(&parent).copied().unwrap_or(0);
            for child in children {
                let child_cost = cell_cost[&child];
                if parent_cost + child_cost > target_tile_size {
                    continue;
                }
                let features = by_cell.remove(&child).unwrap();
                by_cell.entry(parent).or_default().extend(features);
                cell_cost.remove(&child);
                parent_cost += child_cost;
                cell_cost.insert(parent, parent_cost);
            }
        }
    }
}

fn empty_tileset() -> crate::errors::Result<BuiltTileset> {
    let root = GeoBox {
        west: 0.0,
        south: 0.0,
        east: 0.0,
        north: 0.0,
        min_height: 0.0,
        max_height: 0.0,
    };
    let tileset_bytes = render_tileset_json(&root, 1, 1)?;
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
) -> crate::errors::Result<String> {
    let tileset_json = tileset::build(root, available_levels, max_contents);
    serde_json::to_string_pretty(&tileset_json)
        .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))
}

/// `multi` picks the naming scheme: plain `{y}.glb` when every cell in the
/// dataset has a single content (the common case), else `{y}_{n}.glb` for
/// every cell, since the content URI template is declared once for the whole
/// tileset.
fn content_path(cell: Cell, n: usize, multi: bool) -> String {
    if multi {
        format!("content/{}/{}/{}_{}.glb", cell.level, cell.x, cell.y, n)
    } else {
        format!("content/{}/{}/{}.glb", cell.level, cell.x, cell.y)
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
    options: MetadataOptions,
    render: RenderOptions,
    textures: &mut TextureCache,
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
    textures: &mut TextureCache,
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

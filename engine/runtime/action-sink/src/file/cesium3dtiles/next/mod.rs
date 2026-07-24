//! From-scratch Cesium 3D Tiles writer for the new geometry type.
//!
//! Appearance is painted for the default theme, front side only. Textured
//! materials across a tile share one or more embedded atlas pages (one glTF
//! primitive per page); wrapping textures and remote/in-memory rasters aren't
//! handled yet and fall back to colour-only. Texture detail is bounded by the
//! `texel_size` option (metres per pixel); atlas pages are capped at
//! `atlas_size` and overflow spills onto further pages.

mod appearance;
mod mesh;
mod primitive;
mod quadtree;
mod subtree;
mod tileset;

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use rayon::prelude::*;

use reearth_flow_atlas::{build_atlas_multipage, TextureCache, TextureInput};
use reearth_flow_gltf::next::glb::{self, Granularity};
use reearth_flow_gltf::next::metadata;

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
            .eval_string(&ctx.feature, Arc::clone(&ctx.env_vars))
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;

        self.buffer
            .entry((output, None, None))
            .or_default()
            .push(ctx.feature.clone());
        Ok(())
    }

    pub(super) fn finish_new_geometry(&self, ctx: NodeContext) -> crate::errors::Result<()> {
        let options = MetadataOptions {
            schema_key: self.params.schema_key.as_deref(),
            skip_unexposed_attributes: self.params.skip_unexposed_attributes,
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
            texture_codec: self.params.texture_codec,
        };
        for ((output, _, _), features) in &self.buffer {
            let write_file = |relative_path: String, bytes: Vec<u8>| {
                crate::SinkOutput::new(
                    &ctx.sandbox_root,
                    &format!("{output}/{relative_path}"),
                    &ctx.storage_resolver,
                )
                .and_then(|out| out.write(bytes::Bytes::from(bytes)))
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)
            };

            // glbs stream to disk here as they are built; only the small
            // subtree/tileset outputs come back for writing below.
            let built = build(features, options, self.params.max_zoom, render, &write_file)?;

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
    /// Image codec for atlas pages. `None` attaches no textures; textured
    /// geometry falls back to its neutral colour.
    pub texture_codec: Option<TextureCodec>,
}

/// Default atlas page size when the parameter is unset; inherited from the old
/// writer.
const DEFAULT_ATLAS_SIZE: u32 = 2048;

/// Default atlas extrusion ring when the parameter is unset; disabled by
/// default. Raise it to blit a bleed-guard ring around each packed region.
const DEFAULT_ATLAS_EXTRUSION: u32 = 0;

/// Extract and reproject every feature's mesh, place each into the deepest
/// quadtree cell (bounded by `max_zoom`) that fully contains it, and render
/// the result to a [`BuiltTileset`]. A free function so `gml_to_3dtiles` can
/// drive it directly from parsed CityGML, without a `Cesium3DTilesWriter`.
///
/// Each content glb is handed to `write_tile` (relative path, bytes) the moment
/// it is built and is not retained, so peak memory does not grow with the tile
/// count. The returned [`BuiltTileset`] carries only the small `tileset.json`
/// and subtree outputs, which the caller writes after `build` returns.
pub fn build(
    features: &[Feature],
    options: MetadataOptions,
    max_zoom: u8,
    render: RenderOptions,
    write_tile: impl Fn(String, Vec<u8>) -> crate::errors::Result<()> + Sync,
) -> crate::errors::Result<BuiltTileset> {
    let mut caches = mesh::ExtractCaches::default();
    let extracted: Vec<(&Feature, mesh::ExtractedMesh)> = features
        .iter()
        .filter_map(|feature| mesh::extract(&feature.geometry, &mut caches).map(|m| (feature, m)))
        .collect();

    if extracted.is_empty() {
        tracing::warn!(
            "Cesium3DTilesWriter (new-geometry): no renderable geometry found; writing an \
             empty tileset"
        );
        return empty_tileset();
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
        let cell = quadtree::place(&root, &feature_box, max_zoom as u32);
        by_cell.entry(cell).or_default().push(i);
    }

    let occupied: BTreeSet<Cell> = by_cell.keys().copied().collect();
    let available_levels = occupied.iter().map(|c| c.level).max().unwrap_or(0) + 1;

    // A decode cache per cell, dropped once the cell's glb is built. PLATEAU
    // textures are per-surface, so a source image is referenced by only one
    // cell; a tileset-wide cache would grow without bound for no reuse gain.
    // Stream each cell's glb straight to the caller as it is built, so peak
    // memory stays at one glb rather than the whole tileset. The glb bytes feed
    // neither `tileset.json` nor the subtrees (those need only the cell keys,
    // already captured in `occupied`), so nothing downstream needs them retained.
    // Cells are independent (own texture cache, own glb, unique output path), so
    // render them across the rayon pool. Each glb still streams straight to
    // `write_tile` as it is built, so peak memory stays at one glb per worker.
    let cells: Vec<(Cell, Vec<usize>)> = by_cell.into_iter().collect();
    let tile_count = cells.len();
    cells
        .par_iter()
        .try_for_each(|(cell, indices)| -> crate::errors::Result<()> {
            let cell_members: Vec<&(&Feature, mesh::ExtractedMesh)> =
                indices.iter().map(|&i| &extracted[i]).collect();
            let mut textures = TextureCache::default();
            let glb = build_cell_glb(&cell_members, options, render, &mut textures)?;
            write_tile(content_path(*cell), glb)?;
            Ok(())
        })?;

    let tileset_bytes = render_tileset_json(&root, available_levels)?;
    let subtrees = subtree::build_all(&occupied)
        .into_iter()
        .map(|(cell, bytes)| (subtree_path(cell), bytes))
        .collect();

    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        subtrees,
        tile_count,
    })
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
    let tileset_bytes = render_tileset_json(&root, 1)?;
    let subtrees = subtree::build_all(&BTreeSet::new())
        .into_iter()
        .map(|(cell, bytes)| (subtree_path(cell), bytes))
        .collect();
    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        subtrees,
        tile_count: 0,
    })
}

fn render_tileset_json(root: &GeoBox, available_levels: u32) -> crate::errors::Result<String> {
    let tileset_json = tileset::build(root, available_levels);
    serde_json::to_string_pretty(&tileset_json)
        .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))
}

fn content_path(cell: Cell) -> String {
    format!("content/{}/{}/{}.glb", cell.level, cell.x, cell.y)
}

fn subtree_path(cell: Cell) -> String {
    format!("subtrees/{}.{}.{}.subtree", cell.level, cell.x, cell.y)
}

/// The atlas holds many textures side by side, so a repeating wrap would bleed
/// across sub-images; clamp instead.
const ATLAS_SAMPLER: glb::SamplerDesc = glb::SamplerDesc {
    wrap_s: glb::Wrap::ClampToEdge,
    wrap_t: glb::Wrap::ClampToEdge,
    mag: glb::MagFilter::Linear,
    min: glb::MinFilter::LinearMipmap,
};

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
        // With no codec configured, skip texturing entirely and render the
        // textured geometry in the neutral fallback colour.
        let pages = match render.texture_codec {
            Some(_) => build_textured_pages(&mut builder, &textured, render, textures)?,
            None => None,
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
) -> crate::errors::Result<Option<Vec<TexturedPage>>> {
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
        let path = &textured.polygon_texture[polygon];
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

    let scales = texture_target_scales(textured, &inputs, render.texel_size);
    for (input, scale) in inputs.iter_mut().zip(scales) {
        input.scale = scale;
    }

    // Only reached with a codec set (see `build_cell_glb`); default to the
    // enum's `KTX2/ETC1S`.
    let codec = codec_for(render.texture_codec.unwrap_or_default());
    let built = match build_atlas_multipage(
        &inputs,
        render.atlas_size,
        render.atlas_extrusion,
        codec.block_align(),
        textures,
    )
    .map_err(SinkError::cesium3dtiles_writer)?
    {
        Some(built) => built,
        None => return Ok(None),
    };

    let mut page_textures = Vec::with_capacity(built.pages.len());
    for page in built.pages {
        let texture = builder
            .push_atlas_texture(&page, codec.as_ref(), ATLAS_SAMPLER)
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
    }
}

/// Per input texture, the fraction of native resolution to keep so its
/// highest-density (finest metres-per-pixel) face is downsampled to
/// `texel_size` metres per pixel. One scale per texture: the coarser faces
/// sharing it may end up below `texel_size`, the accepted cost of not splitting
/// a texture by face. `texel_size == 0.0` disables downsampling (scale `1.0`).
fn texture_target_scales(
    textured: &TexturedPrimitive,
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
        let pi = path_input[&textured.polygon_texture[polygon]];
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

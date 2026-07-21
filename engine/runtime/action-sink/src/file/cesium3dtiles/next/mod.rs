//! From-scratch Cesium 3D Tiles writer for the new geometry type.
//!
//! Appearance is painted for the default theme, front side only. Textured
//! materials across a tile share one embedded atlas; wrapping textures and
//! remote/in-memory rasters aren't handled yet and fall back to colour-only.

mod appearance;
mod mesh;
mod primitive;
mod quadtree;
mod subtree;
mod tileset;

use std::collections::{BTreeSet, HashMap};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use reearth_flow_atlas::{build_atlas, TextureInput};
use reearth_flow_gltf::next::glb::{self, Granularity};
use reearth_flow_gltf::next::metadata;

use primitive::{Geom, TexturedPrimitive, DEFAULT_MATERIAL};
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::FEATURES_PORT;
use reearth_flow_types::Feature;

use super::sink::Cesium3DTilesWriter;
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
        // Both default to true (see `Cesium3DTilesWriterParam`).
        let draco = self.params.draco_compression.unwrap_or(true);
        let compute_flat_normal = self.params.compute_flat_normal.unwrap_or(true);
        for ((output, _, _), features) in &self.buffer {
            let built = build(
                features,
                options,
                self.params.max_zoom,
                draco,
                compute_flat_normal,
            )?;

            for (relative_path, bytes) in built.tiles.into_iter().chain(built.subtrees) {
                crate::SinkOutput::new(
                    &ctx.sandbox_root,
                    &format!("{output}/{relative_path}"),
                    &ctx.storage_resolver,
                )
                .and_then(|out| out.write(bytes::Bytes::from(bytes)))
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
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

/// Every file a built tileset is made of, relative to the tileset's output
/// directory: one content glb per occupied cell, one or more `.subtree`
/// files, and the `tileset.json` text itself.
pub struct BuiltTileset {
    pub tileset_json: String,
    pub tiles: Vec<(String, Vec<u8>)>,
    pub subtrees: Vec<(String, Vec<u8>)>,
}

/// Extract and reproject every feature's mesh, place each into the deepest
/// quadtree cell (bounded by `max_zoom`) that fully contains it, and render
/// the result to a [`BuiltTileset`]. A free function so `gml_to_3dtiles` can
/// drive it directly from parsed CityGML, without a `Cesium3DTilesWriter`.
pub fn build(
    features: &[Feature],
    options: MetadataOptions,
    max_zoom: u8,
    draco: bool,
    compute_flat_normal: bool,
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

    let tiles = by_cell
        .into_iter()
        .map(|(cell, indices)| {
            let cell_members: Vec<&(&Feature, mesh::ExtractedMesh)> =
                indices.iter().map(|&i| &extracted[i]).collect();
            let glb = build_cell_glb(&cell_members, options, draco, compute_flat_normal)?;
            Ok((content_path(cell), glb))
        })
        .collect::<crate::errors::Result<_>>()?;

    let tileset_bytes = render_tileset_json(&root, available_levels)?;
    let subtrees = subtree::build_all(&occupied)
        .into_iter()
        .map(|(cell, bytes)| (subtree_path(cell), bytes))
        .collect();

    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        tiles,
        subtrees,
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
        tiles: Vec::new(),
        subtrees,
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

/// Largest atlas the packer may emit; inherited from the old writer. Textures
/// exceeding this are globally downsampled (see [`reearth_flow_atlas`]).
const MAX_ATLAS_SIZE: u32 = 8192;

/// The atlas holds many textures side by side, so a repeating wrap would bleed
/// across sub-images; clamp instead.
const ATLAS_SAMPLER: glb::SamplerDesc = glb::SamplerDesc {
    wrap_s: glb::Wrap::ClampToEdge,
    wrap_t: glb::Wrap::ClampToEdge,
    mag: glb::MagFilter::Linear,
    min: glb::MinFilter::LinearMipmap,
};

/// Render one occupied cell to a glb: one primitive per resolved colour-only
/// material, plus one shared atlas primitive for all textured faces (see
/// [`primitive::collect`]). `compute_flat_normal` attaches per-polygon flat
/// normals; `draco` Draco-compresses the output.
fn build_cell_glb(
    cell_members: &[&(&Feature, mesh::ExtractedMesh)],
    options: MetadataOptions,
    draco: bool,
    compute_flat_normal: bool,
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
        let (material, uv) = match build_atlas_texture(&mut builder, &textured)? {
            Some((texture, uv)) => (
                glb::MaterialDesc {
                    base_color_factor: [1.0, 1.0, 1.0, 1.0],
                    metallic_factor: 0.0,
                    roughness_factor: 1.0,
                    base_color_texture: Some(texture),
                },
                Some(uv),
            ),
            // Packing failed or produced no image: render the textured geometry
            // in the neutral fallback colour rather than dropping it.
            None => (color_material(DEFAULT_MATERIAL), None),
        };
        let handle = push_geom(
            &mut builder,
            &textured.geom,
            origin,
            material,
            uv,
            compute_flat_normal,
        );
        primitives.push((handle, textured.geom.feature_ids));
    }

    for color in cells.color {
        let material = color_material(color.factors);
        let handle = push_geom(
            &mut builder,
            &color.geom,
            origin,
            material,
            None,
            compute_flat_normal,
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

    if draco {
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

/// Pack the textured primitive's per-polygon UVs into one atlas, embed it as a
/// WebP texture, and return the handle with the atlas-remapped per-corner UVs
/// (parallel to `textured.geom.corner_uv`). `Ok(None)` if packing produced no
/// image.
fn build_atlas_texture(
    builder: &mut glb::Builder,
    textured: &TexturedPrimitive,
) -> crate::errors::Result<Option<(glb::TextureRef, Vec<[f32; 2]>)>> {
    // Group polygons by texture path, feeding one atlas polygon per source
    // polygon; `slots[p]` locates polygon `p`'s remapped UVs afterward.
    let mut inputs: Vec<TextureInput> = Vec::new();
    let mut path_index: HashMap<PathBuf, usize> = HashMap::new();
    let mut slots: Vec<(usize, usize, usize)> = Vec::new(); // (path, poly, corner offset)
    let mut offset = 0usize;
    for (polygon, &tris) in textured.geom.polygon_tris.iter().enumerate() {
        let corners = tris as usize * 3;
        let path = &textured.polygon_texture[polygon];
        let pi = *path_index.entry(path.clone()).or_insert_with(|| {
            inputs.push(TextureInput {
                path: path.clone(),
                uvs: Vec::new(),
            });
            inputs.len() - 1
        });
        let poly = inputs[pi].uvs.len();
        inputs[pi]
            .uvs
            .push(textured.geom.corner_uv[offset..offset + corners].to_vec());
        slots.push((pi, poly, offset));
        offset += corners;
    }

    let built = match build_atlas(&inputs, MAX_ATLAS_SIZE) {
        Ok(Some(built)) => built,
        Ok(None) => return Ok(None),
        Err(e) => {
            tracing::error!("Cesium3DTilesWriter: atlas packing failed: {e}; textures dropped");
            return Ok(None);
        }
    };

    let mut corner_uv = vec![[0.0f32, 0.0]; textured.geom.corner_uv.len()];
    for &(pi, poly, offset) in &slots {
        for (k, &[u, v]) in built.remapped_uvs[pi][poly].iter().enumerate() {
            corner_uv[offset + k] = [u as f32, v as f32];
        }
    }

    let mut webp = Vec::new();
    image::DynamicImage::ImageRgba8(built.image)
        .write_to(&mut Cursor::new(&mut webp), image::ImageFormat::WebP)
        .map_err(|e| SinkError::Cesium3DTilesWriter(format!("atlas WebP encode failed: {e}")))?;
    let image = builder.push_image(&webp, "image/webp");
    // WebP has no core-glTF fallback image, so the extension is required.
    builder.require_extension("EXT_texture_webp");
    let texture = builder.push_texture(
        None,
        ATLAS_SAMPLER,
        vec![(
            "EXT_texture_webp",
            serde_json::json!({ "source": image.index() }),
        )],
    );
    Ok(Some((texture, corner_uv)))
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

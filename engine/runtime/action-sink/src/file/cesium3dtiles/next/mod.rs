//! From-scratch Cesium 3D Tiles writer for the new geometry type.
//!
//! Appearance is painted for the default theme, front side only. Textured
//! materials across a tile share one embedded atlas; wrapping textures and
//! remote/in-memory rasters aren't handled yet and fall back to colour-only.

mod appearance;
mod mesh;
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

use appearance::ResolvedMaterial;
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

/// Fallback for a face bound to no material: flat gray (the old writer's
/// X3DMaterial default), which keeps adjacent buildings from merging into the
/// glTF-default white.
const DEFAULT_MATERIAL: MaterialFactors = MaterialFactors {
    base_color_factor: [0.7, 0.7, 0.7, 1.0],
    metallic_factor: 0.0,
    roughness_factor: 0.9,
};

/// The atlas holds many textures side by side, so a repeating wrap would bleed
/// across sub-images; clamp instead.
const ATLAS_SAMPLER: glb::SamplerDesc = glb::SamplerDesc {
    wrap_s: glb::Wrap::ClampToEdge,
    wrap_t: glb::Wrap::ClampToEdge,
    mag: glb::MagFilter::Linear,
    min: glb::MinFilter::LinearMipmap,
};

/// The PBR factors that key a colour-only primitive; textures multiply these.
#[derive(Clone, Copy, PartialEq)]
struct MaterialFactors {
    base_color_factor: [f32; 4],
    metallic_factor: f32,
    roughness_factor: f32,
}

impl MaterialFactors {
    fn of(material: Option<&ResolvedMaterial>) -> Self {
        match material {
            Some(m) => Self {
                base_color_factor: m.base_color_factor,
                metallic_factor: m.metallic_factor,
                roughness_factor: m.roughness_factor,
            },
            None => DEFAULT_MATERIAL,
        }
    }

    /// A hashable identity for grouping (f32 has no `Eq`/`Hash`).
    fn key(&self) -> [u32; 6] {
        let [r, g, b, a] = self.base_color_factor;
        [
            r.to_bits(),
            g.to_bits(),
            b.to_bits(),
            a.to_bits(),
            self.metallic_factor.to_bits(),
            self.roughness_factor.to_bits(),
        ]
    }
}

/// Merged, tile-local geometry plus its resolved appearance, all in glTF
/// convention (Z-up pre-rotated, positions localized to the tile origin).
struct CellMesh {
    positions: Vec<[f32; 3]>,
    origin: [f64; 3],
    indices: Vec<[u32; 3]>,
    /// Per-vertex feature row in the tile's property table.
    feature_ids: Vec<u32>,
    /// Per-triangle flat normal.
    triangle_normals: Vec<[f32; 3]>,
    /// Per-triangle resolved material (`None` = unbound → [`DEFAULT_MATERIAL`]).
    triangle_material: Vec<Option<ResolvedMaterial>>,
    /// Per-corner base-map UV, length `3 * indices.len()`.
    corner_uv: Vec<[f32; 2]>,
}

/// Concatenate one occupied cell's features (index-offset) and flatten their
/// per-triangle appearance, converting to glTF convention.
fn merge_cell(cell_members: &[&(&Feature, mesh::ExtractedMesh)]) -> CellMesh {
    let mut ecef_vertices: Vec<[f64; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    let mut feature_ids: Vec<u32> = Vec::new();
    let mut triangle_normals: Vec<[f32; 3]> = Vec::new();
    let mut triangle_material: Vec<Option<ResolvedMaterial>> = Vec::new();
    let mut corner_uv: Vec<[f32; 2]> = Vec::new();

    for (row, (_, m)) in cell_members.iter().enumerate() {
        let base = ecef_vertices.len() as u32;
        indices.extend(
            m.indices
                .iter()
                .map(|&[a, b, c]| [a + base, b + base, c + base]),
        );
        ecef_vertices.extend(&m.ecef_vertices);
        feature_ids.extend(std::iter::repeat_n(row as u32, m.ecef_vertices.len()));
        // A source polygon's flat normal is shared by all the triangles it
        // split into; expand it to one per triangle.
        for (polygon, &count) in m.polygon_tris.iter().enumerate() {
            let [x, y, z] = m.polygon_normals[polygon];
            triangle_normals.extend(std::iter::repeat_n(
                [x as f32, z as f32, -y as f32],
                count as usize,
            ));
        }
        triangle_material.extend(
            m.triangle_material
                .iter()
                .map(|&bound| bound.and_then(|mi| m.materials.get(mi as usize).cloned())),
        );
        corner_uv.extend(m.corner_uv.iter().map(|&[u, v]| [u as f32, v as f32]));
    }

    // Per-tile local origin keeps the f32 positions small next to ECEF's
    // ~6.378e6 m magnitude. 3D Tiles renderers rotate bare-glTF content
    // Y-up -> Z-up on load; our input is already Z-up (ECEF-relative), so
    // pre-apply the inverse and the renderer's rotation cancels out.
    let origin = centroid(&ecef_vertices);
    let positions: Vec<[f32; 3]> = ecef_vertices
        .iter()
        .map(|p| {
            [
                (p[0] - origin[0]) as f32,
                (p[2] - origin[2]) as f32,
                -((p[1] - origin[1]) as f32),
            ]
        })
        .collect();

    CellMesh {
        positions,
        origin,
        indices,
        feature_ids,
        triangle_normals,
        triangle_material,
        corner_uv,
    }
}

/// Render one occupied cell's merged mesh to a glb: one primitive per resolved
/// colour-only material, plus one shared atlas primitive for all textured
/// faces. `compute_flat_normal` attaches per-triangle flat normals; `draco`
/// Draco-compresses the output.
fn build_cell_glb(
    cell_members: &[&(&Feature, mesh::ExtractedMesh)],
    options: MetadataOptions,
    draco: bool,
    compute_flat_normal: bool,
) -> crate::errors::Result<Vec<u8>> {
    let cell = merge_cell(cell_members);

    let cell_features: Vec<&Feature> = cell_members.iter().map(|(f, _)| *f).collect();
    let table = metadata::build_table(&cell_features, options);

    let mut builder = glb::Builder::new();
    let mut primitives = Vec::new();

    // Partition triangles into the shared atlas (textured, non-wrapping) and
    // colour-only groups keyed by PBR factors.
    let mut atlas = AtlasBatch::default();
    let mut color_groups: HashMap<[u32; 6], (MaterialFactors, Vec<usize>)> = HashMap::new();
    for triangle in 0..cell.indices.len() {
        let material = cell.triangle_material[triangle].as_ref();
        let texture = material
            .and_then(|m| m.base_texture.as_ref())
            .filter(|_| !uv_wraps(&cell, triangle));
        match texture {
            Some(source) => {
                let uvs: Vec<[f64; 2]> = cell.corner_uv[triangle * 3..triangle * 3 + 3]
                    .iter()
                    .map(|&[u, v]| [u as f64, v as f64])
                    .collect();
                atlas.push(triangle, source, uvs);
            }
            None => {
                let factors = MaterialFactors::of(material);
                color_groups
                    .entry(factors.key())
                    .or_insert_with(|| (factors, Vec::new()))
                    .1
                    .push(triangle);
            }
        }
    }

    // Build the atlas primitive, or fold its triangles back into colour-only
    // groups if packing yields no image (empty or failed).
    match atlas.build(&mut builder)? {
        Some((texture, remapped)) => {
            let material = glb::MaterialDesc {
                base_color_factor: [1.0, 1.0, 1.0, 1.0],
                metallic_factor: 0.0,
                roughness_factor: 1.0,
                base_color_texture: Some(texture),
            };
            let uv = atlas
                .textured
                .iter()
                .flat_map(|&(_, path, poly)| {
                    remapped[path][poly]
                        .iter()
                        .map(|&[u, v]| [u as f32, v as f32])
                })
                .collect();
            let tris: Vec<usize> = atlas.textured.iter().map(|&(t, _, _)| t).collect();
            primitives.push(push_group(
                &mut builder,
                &cell,
                &tris,
                material,
                Some(uv),
                compute_flat_normal,
            ));
        }
        None => {
            for &(triangle, _, _) in &atlas.textured {
                let factors = MaterialFactors::of(cell.triangle_material[triangle].as_ref());
                color_groups
                    .entry(factors.key())
                    .or_insert_with(|| (factors, Vec::new()))
                    .1
                    .push(triangle);
            }
        }
    }

    for (factors, tris) in color_groups.into_values() {
        let material = glb::MaterialDesc {
            base_color_factor: factors.base_color_factor,
            metallic_factor: factors.metallic_factor,
            roughness_factor: factors.roughness_factor,
            base_color_texture: None,
        };
        primitives.push(push_group(
            &mut builder,
            &cell,
            &tris,
            material,
            None,
            compute_flat_normal,
        ));
    }

    metadata::encode(&table, &mut builder, &primitives, &cell.feature_ids);
    let gltf_origin = [cell.origin[0], cell.origin[2], -cell.origin[1]];
    let glb = builder.build(gltf_origin);

    if draco {
        reearth_flow_gltf::next::draco::compress(&glb)
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("draco compression failed: {e:?}")))
    } else {
        Ok(glb)
    }
}

/// Whether triangle `t`'s corner UVs fall outside `[0,1]` (with the same 0.1
/// tolerance the old writer used): such a texture can't be atlased, so the face
/// falls back to colour-only.
fn uv_wraps(cell: &CellMesh, triangle: usize) -> bool {
    const TOLERANCE: f32 = 0.1;
    cell.corner_uv[triangle * 3..triangle * 3 + 3]
        .iter()
        .any(|&[u, v]| {
            u < -TOLERANCE || u > 1.0 + TOLERANCE || v < -TOLERANCE || v > 1.0 + TOLERANCE
        })
}

/// Textured triangles grouped by texture path for one atlas pass.
#[derive(Default)]
struct AtlasBatch {
    /// One [`TextureInput`] per distinct path; `uvs[i]` is a triangle's 3 UVs.
    inputs: Vec<TextureInput>,
    path_index: HashMap<PathBuf, usize>,
    /// `(triangle, path index, polygon within that path)`, in insertion order.
    textured: Vec<(usize, usize, usize)>,
}

impl AtlasBatch {
    fn push(&mut self, triangle: usize, source: &appearance::TextureSource, uvs: Vec<[f64; 2]>) {
        let path = *self
            .path_index
            .entry(source.path.clone())
            .or_insert_with(|| {
                self.inputs.push(TextureInput {
                    path: source.path.clone(),
                    uvs: Vec::new(),
                });
                self.inputs.len() - 1
            });
        let poly = self.inputs[path].uvs.len();
        self.textured.push((triangle, path, poly));
        self.inputs[path].uvs.push(uvs);
    }

    /// Pack the batch, embed the resulting image, and return its texture handle
    /// with the remapped UVs (per path, per polygon). `Ok(None)` if there was
    /// nothing to pack; UVs are filled in [`AtlasBatch::fill_uvs`] first.
    fn build(
        &mut self,
        builder: &mut glb::Builder,
    ) -> crate::errors::Result<Option<(glb::TextureRef, Vec<reearth_flow_atlas::TextureUVs>)>> {
        if self.textured.is_empty() {
            return Ok(None);
        }
        let built = match build_atlas(&self.inputs, MAX_ATLAS_SIZE) {
            Ok(Some(built)) => built,
            Ok(None) => return Ok(None),
            Err(e) => {
                tracing::error!("Cesium3DTilesWriter: atlas packing failed: {e}; textures dropped");
                return Ok(None);
            }
        };

        let mut webp = Vec::new();
        image::DynamicImage::ImageRgba8(built.image)
            .write_to(&mut Cursor::new(&mut webp), image::ImageFormat::WebP)
            .map_err(|e| {
                SinkError::Cesium3DTilesWriter(format!("atlas WebP encode failed: {e}"))
            })?;
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
        Ok(Some((texture, built.remapped_uvs)))
    }
}

/// Push one primitive for the triangles in `tris`, keyed by a single material.
/// Positions are shared with the whole cell (dedup drops the unreferenced ones);
/// `uv`, when present, is a `TEXCOORD_0` value per corner in `tris` order.
fn push_group(
    builder: &mut glb::Builder,
    cell: &CellMesh,
    tris: &[usize],
    material: glb::MaterialDesc,
    uv: Option<Vec<[f32; 2]>>,
    compute_flat_normal: bool,
) -> glb::PrimitiveHandle {
    let indices: Vec<[u32; 3]> = tris.iter().map(|&t| cell.indices[t]).collect();
    // One "polygon" per triangle, so the per-triangle normal is looked up
    // directly; coplanar triangles share a normal and still weld at shared edges.
    let polygon_tris = vec![1u32; indices.len()];

    let mut dedup_attrs = Vec::new();
    if compute_flat_normal {
        let normals: Vec<[f32; 3]> = tris.iter().map(|&t| cell.triangle_normals[t]).collect();
        dedup_attrs.push(glb::normal(Granularity::PerPolygon, normals));
    }
    let corner_src: Vec<u32> = if uv.is_some() {
        (0..indices.len() as u32 * 3).collect()
    } else {
        Vec::new()
    };
    if let Some(uv) = uv {
        dedup_attrs.push(glb::texcoord(uv));
    }

    builder.push_primitive(
        cell.positions.clone(),
        indices,
        material,
        &polygon_tris,
        &corner_src,
        dedup_attrs,
    )
}

fn centroid(points: &[[f64; 3]]) -> [f64; 3] {
    if points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = points.len() as f64;
    let mut sum = [0.0; 3];
    for p in points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

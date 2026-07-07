//! Minimal, from-scratch Cesium 3D Tiles writer for the new geometry type
//! (`reearth_flow_geometry`). Pass-1 scope, deliberately narrow:
//!
//! - Only a bare `PolygonMesh` leaf is read from each feature's geometry
//!   (see `mesh.rs`); every other shape is skipped with a warning.
//! - No appearance / materials / textures: every mesh is emitted untextured
//!   (see `glb.rs`).
//! - No same-tile content splitting (`maxGlbSize`) and no texture atlasing:
//!   both are appearance-adjacent concerns this pass has nothing to feed
//!   them (§6.2.3/§6.2.4 of the geometry design doc).
//!
//! Nothing here references the old `pipeline.rs` / `slice.rs` / `tiling.rs` /
//! `b3dm.rs` modules; this reuses only the generic sink I/O helpers
//! (`crate::SinkOutput`, `NodeContext`) shared by every sink in this crate,
//! plus `reearth_flow_gltf::next` for the actual glb bytes (`glb.rs`'s
//! former home — moved there since it has nothing 3D-Tiles-specific about
//! it and a future glTF-writing sink will want it too).

mod mesh;
mod quadtree;
mod subtree;
mod tileset;

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use reearth_flow_gltf::next::{glb, metadata};
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::DEFAULT_PORT;
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
        if ctx.port != *DEFAULT_PORT {
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
        for ((output, _, _), features) in &self.buffer {
            let built = build(features, options)?;

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

/// Generous hard cap on quadtree depth, protecting containment placement
/// (`quadtree::place`) against a pathological zero/near-zero-extent feature
/// forcing it to the bottom of the loop. `maxDepth` (§6.2.1 of the geometry
/// design doc) will make this a real, user-facing parameter; until the
/// writer's params are wired up, this stands in.
const MAX_DEPTH: u32 = 21;

/// Every file a built tileset is made of, relative to the tileset's output
/// directory: one content glb per occupied cell, one or more `.subtree`
/// files, and the `tileset.json` text itself.
pub struct BuiltTileset {
    pub tileset_json: String,
    pub tiles: Vec<(String, Vec<u8>)>,
    pub subtrees: Vec<(String, Vec<u8>)>,
}

/// Extract and reproject every feature's mesh, place each into the deepest
/// quadtree cell that fully contains it (§6.2.2 of the geometry design doc),
/// and render the result to a [`BuiltTileset`].
///
/// A free function, independent of any `Cesium3DTilesWriter` instance or
/// `NodeContext`, so it doubles as the entry point for the `gml_to_3dtiles`
/// example, which drives it from a real parsed CityGML file instead of the
/// sink's buffered features.
pub fn build(
    features: &[Feature],
    options: MetadataOptions,
) -> crate::errors::Result<BuiltTileset> {
    let extracted: Vec<(&Feature, mesh::ExtractedMesh)> = features
        .iter()
        .filter_map(|feature| mesh::extract(&feature.geometry).map(|m| (feature, m)))
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
        let cell = quadtree::place(&root, &feature_box, MAX_DEPTH);
        by_cell.entry(cell).or_default().push(i);
    }

    let occupied: BTreeSet<Cell> = by_cell.keys().copied().collect();
    let subtree_levels = occupied.iter().map(|c| c.level).max().unwrap_or(0) + 1;

    let tiles = by_cell
        .into_iter()
        .map(|(cell, indices)| {
            let cell_members: Vec<&(&Feature, mesh::ExtractedMesh)> =
                indices.iter().map(|&i| &extracted[i]).collect();
            (content_path(cell), build_cell_glb(&cell_members, options))
        })
        .collect();

    let tileset_bytes = render_tileset_json(&root, subtree_levels)?;
    let subtree_bytes = subtree::build(&occupied, subtree_levels);

    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        tiles,
        subtrees: vec![(subtree_path(Cell::root()), subtree_bytes)],
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
    let subtree_bytes = subtree::build(&BTreeSet::new(), 1);
    Ok(BuiltTileset {
        tileset_json: tileset_bytes,
        tiles: Vec::new(),
        subtrees: vec![(subtree_path(Cell::root()), subtree_bytes)],
    })
}

fn render_tileset_json(root: &GeoBox, subtree_levels: u32) -> crate::errors::Result<String> {
    let tileset_json = tileset::build(root, subtree_levels);
    serde_json::to_string_pretty(&tileset_json)
        .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))
}

fn content_path(cell: Cell) -> String {
    format!("content/{}/{}/{}.glb", cell.level, cell.x, cell.y)
}

fn subtree_path(cell: Cell) -> String {
    format!("subtrees/{}.{}.{}.subtree", cell.level, cell.x, cell.y)
}

/// Merge one occupied cell's features into a single glb, index-offset
/// concatenated (mirrors the pre-tiling pass's whole-dataset merge, just
/// scoped to one cell's features instead of all of them), tagging each
/// vertex with its feature's row in the cell's property table.
fn build_cell_glb(
    cell_members: &[&(&Feature, mesh::ExtractedMesh)],
    options: MetadataOptions,
) -> Vec<u8> {
    let mut ecef_vertices: Vec<[f64; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    let mut feature_ids: Vec<u32> = Vec::new();
    for (row, (_, m)) in cell_members.iter().enumerate() {
        let base = ecef_vertices.len() as u32;
        indices.extend(
            m.indices
                .iter()
                .map(|&[a, b, c]| [a + base, b + base, c + base]),
        );
        ecef_vertices.extend(&m.ecef_vertices);
        feature_ids.extend(std::iter::repeat_n(row as u32, m.ecef_vertices.len()));
    }

    // Per-tile local origin: keeps the f32 GLB positions small relative to
    // ECEF's ~6.378e6 m magnitude.
    let origin = centroid(&ecef_vertices);
    let local_positions: Vec<[f32; 3]> = ecef_vertices
        .iter()
        .map(|p| {
            [
                (p[0] - origin[0]) as f32,
                (p[1] - origin[1]) as f32,
                (p[2] - origin[2]) as f32,
            ]
        })
        .collect();

    let cell_features: Vec<&Feature> = cell_members.iter().map(|(f, _)| *f).collect();
    let table = metadata::build_table(&cell_features, options);

    glb::write(&local_positions, &indices, origin, &feature_ids, &table)
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

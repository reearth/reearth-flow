//! Minimal, from-scratch Cesium 3D Tiles writer for the new geometry type
//! (`reearth_flow_geometry`). Pass-1 scope, deliberately narrow:
//!
//! - Only a bare `PolygonMesh` leaf is read from each feature's geometry
//!   (see `mesh.rs`); every other shape is skipped with a warning.
//! - No appearance / materials / textures: every mesh is emitted untextured
//!   (see `glb.rs`).
//! - No tiling: every feature in a group lands in one root tile. Containment
//!   placement / quadtree subdivision is a separate, later pass.
//! - No `.subtree` / implicit tiling (see `tileset.rs`): the tileset is a
//!   plain, explicit tree with exactly one tile.
//!
//! Nothing here references the old `pipeline.rs` / `slice.rs` / `tiling.rs` /
//! `b3dm.rs` modules; this is a self-contained implementation, reusing only
//! the generic sink I/O helpers (`crate::SinkOutput`, `NodeContext`) shared by
//! every sink in this crate.

mod glb;
mod mesh;
mod tileset;

use std::sync::Arc;

use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::DEFAULT_PORT;
use reearth_flow_types::Feature;

use super::sink::Cesium3DTilesWriter;
use crate::errors::SinkError;

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
        for ((output, _, _), features) in &self.buffer {
            self.write_one_tileset(&ctx, output, features)?;
        }
        Ok(())
    }

    fn write_one_tileset(
        &self,
        node_ctx: &NodeContext,
        output: &str,
        features: &[Feature],
    ) -> crate::errors::Result<()> {
        // Merge every feature's mesh into one combined buffer: pass 1 has no
        // tiling, so a "group" (one evaluated `output` path) is one tile.
        let mut ecef_vertices: Vec<[f64; 3]> = Vec::new();
        let mut geographic_vertices: Vec<[f64; 3]> = Vec::new();
        let mut indices: Vec<[u32; 3]> = Vec::new();

        for feature in features {
            let Some(extracted) = mesh::extract(&feature.geometry) else {
                continue;
            };
            let base = ecef_vertices.len() as u32;
            indices.extend(
                extracted
                    .indices
                    .into_iter()
                    .map(|[a, b, c]| [a + base, b + base, c + base]),
            );
            ecef_vertices.extend(extracted.ecef_vertices);
            geographic_vertices.extend(extracted.geographic_vertices);
        }

        if ecef_vertices.is_empty() {
            tracing::warn!(
                "Cesium3DTilesWriter (new-geometry): no renderable geometry for output \
                 `{output}`; writing an empty tileset"
            );
        }

        // Per-tile (= the single root tile, in this pass) local origin: keeps
        // the f32 GLB positions small relative to ECEF's ~6.378e6 m magnitude,
        // regardless of how many tiles the eventual tiling pass introduces.
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

        let glb_bytes = glb::write(&local_positions, &indices, origin);
        let tileset_json = tileset::build(&geographic_vertices, "tile.glb");
        let tileset_bytes = serde_json::to_string_pretty(&tileset_json)
            .map_err(|e| SinkError::Cesium3DTilesWriter(format!("{e:?}")))?;

        crate::SinkOutput::new(
            &node_ctx.sandbox_root,
            &format!("{output}/tile.glb"),
            &node_ctx.storage_resolver,
        )
        .and_then(|out| out.write(bytes::Bytes::from(glb_bytes)))
        .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

        crate::SinkOutput::new(
            &node_ctx.sandbox_root,
            &format!("{output}/tileset.json"),
            &node_ctx.storage_resolver,
        )
        .and_then(|out| out.write(bytes::Bytes::from(tileset_bytes)))
        .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

        Ok(())
    }
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

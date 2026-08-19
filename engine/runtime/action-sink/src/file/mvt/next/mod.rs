mod extract;
mod slice;
mod tile;

use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Cursor};
use std::sync::Arc;

use rayon::prelude::*;
use reearth_flow_geometry::ops::ReprojectionCache;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::FEATURES_PORT;
use reearth_flow_types::{Attribute, Feature};

use super::sink::{MVTWriter, MVTWriterCompiledParam};
use super::tileid::TileIdMethod;
use super::tiling::{TileContent, TileMetadata, VectorLayer};
use extract::extract;
use slice::{slice_leaves, TileKey, TiledGeom};
use tile::{make_tile, SlicedFeature, SlicedGeom};

impl MVTWriter {
    pub(super) fn process_new_geometry(
        &mut self,
        ctx: &ExecutorContext,
    ) -> crate::errors::Result<()> {
        if ctx.port != *FEATURES_PORT {
            return Ok(());
        }

        let variables = ctx.variables.clone();
        let eval = |c: &reearth_flow_types::CompiledCode| {
            c.eval_string(&ctx.feature, Arc::clone(&variables))
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
        };
        let output = eval(&self.params.output)?;
        let layer_name = eval(&self.params.layer_name)?;
        let compress_output = self.params.compress_output.as_ref().map(eval).transpose()?;

        let feature = {
            let mut attrs = crate::schema::filter_and_cast_attributes(
                &ctx.feature,
                &self.schema,
                self.params.schema_key.as_deref(),
            );
            let skip_unexp = self.params.skip_unexposed_attributes;
            attrs.retain(|k, _| {
                let key = k.as_ref();
                !(skip_unexp && key.starts_with("__"))
                    && self.params.schema_key.as_deref() != Some(key)
            });
            if self.params.colon_to_underscore {
                attrs = attrs
                    .into_iter()
                    .map(|(k, v)| (Attribute::new(k.inner().replace(':', "_")), v))
                    .collect();
            }
            ctx.feature.with_attributes(attrs)
        };

        self.buffer
            .entry((output, compress_output))
            .or_default()
            .push((feature, layer_name));
        Ok(())
    }

    pub(super) fn finish_new_geometry(&self, ctx: NodeContext) -> crate::errors::Result<()> {
        for ((output, compress_output), buffer) in &self.buffer {
            write_tileset(
                &ctx,
                buffer,
                output,
                compress_output.as_deref(),
                &self.params,
            )?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct SliceAccum {
    content: TileContent,
    layer_names: HashSet<String>,
    by_tile: HashMap<TileKey, Vec<SlicedFeature>>,
}

impl SliceAccum {
    fn merge(mut self, other: Self) -> Self {
        self.content = self.content.union(other.content);
        self.layer_names.extend(other.layer_names);
        for (key, feats) in other.by_tile {
            self.by_tile.entry(key).or_default().extend(feats);
        }
        self
    }
}

fn write_tileset(
    ctx: &NodeContext,
    upstream: &[(Feature, String)],
    output: &str,
    compress_output: Option<&str>,
    params: &MVTWriterCompiledParam,
) -> crate::errors::Result<()> {
    let min_zoom = params.min_zoom;
    let max_zoom = params.max_zoom;
    let extent = params.extent;
    let max_tile_bytes = params.max_tile_bytes;

    let accum = upstream
        .par_iter()
        .fold(SliceAccum::default, |mut acc, (feature, layer_name)| {
            acc.layer_names.insert(layer_name.clone());
            let mut cache = ReprojectionCache::new();
            let leaves = extract(&feature.geometry, &mut cache);
            let (content, tiled) = slice_leaves(leaves, min_zoom, max_zoom, extent as u32);
            acc.content = std::mem::take(&mut acc.content).union(content);
            for tiled_leaf in tiled {
                let sliced = to_sliced_feature(layer_name, tiled_leaf.geom, feature);
                acc.by_tile.entry(tiled_leaf.key).or_default().push(sliced);
            }
            acc
        })
        .reduce(SliceAccum::default, SliceAccum::merge);

    accum.by_tile.par_iter().try_for_each(|(&key, feats)| {
        write_tile(ctx, output, key, feats, extent, max_tile_bytes)
    })?;

    write_tilejson(
        ctx,
        output,
        min_zoom,
        max_zoom,
        &accum.content,
        &accum.layer_names,
    )?;

    if let Some(compress_rel) = compress_output {
        compress_tileset(ctx, output, compress_rel)?;
    }
    Ok(())
}

fn to_sliced_feature(layer_name: &str, geom: TiledGeom, feature: &Feature) -> SlicedFeature {
    let geom = match geom {
        TiledGeom::Polygon(parts) => SlicedGeom::Polygon(parts),
        TiledGeom::LineString(lines) => SlicedGeom::LineString(lines),
        TiledGeom::Point(points) => SlicedGeom::Point(points),
    };
    SlicedFeature {
        layer_name: layer_name.to_string(),
        geom,
        properties: feature.attributes.clone(),
    }
}

fn write_tile(
    ctx: &NodeContext,
    output_rel: &str,
    (zoom, x, y): TileKey,
    feats: &[SlicedFeature],
    extent: i32,
    max_tile_bytes: u64,
) -> crate::errors::Result<()> {
    let bytes = make_tile(extent, feats, max_tile_bytes)?;
    let tile_rel = format!("{output_rel}/{zoom}/{x}/{y}.mvt");
    crate::SinkOutput::new(&ctx.sandbox_root, &tile_rel, &ctx.storage_resolver)
        .and_then(|out| out.write(bytes::Bytes::from(bytes)))
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
}

fn write_tilejson(
    ctx: &NodeContext,
    output_rel: &str,
    min_zoom: u8,
    max_zoom: u8,
    content: &TileContent,
    layer_names: &HashSet<String>,
) -> crate::errors::Result<()> {
    let basename = std::path::Path::new(output_rel)
        .file_name()
        .map(|s| s.to_string_lossy().to_string());

    let tiles = vec!["/{z}/{x}/{y}.mvt".to_string()];
    let vector_layers: Vec<_> = layer_names
        .iter()
        .map(|id| VectorLayer {
            id: id.clone(),
            fields: HashMap::new(),
        })
        .collect();
    let metadata = TileMetadata::from_tile_content(
        basename,
        min_zoom,
        max_zoom,
        content,
        tiles,
        vector_layers,
    );

    let metadata = serde_json::to_string_pretty(&metadata)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
    let tilejson_rel = format!("{output_rel}/tilejson.json");
    crate::SinkOutput::new(&ctx.sandbox_root, &tilejson_rel, &ctx.storage_resolver)
        .and_then(|out| out.write(bytes::Bytes::from(metadata)))
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
}

fn compress_tileset(
    ctx: &NodeContext,
    output_rel: &str,
    compress_rel: &str,
) -> crate::errors::Result<()> {
    let output_uri = crate::SinkOutput::new(&ctx.sandbox_root, output_rel, &ctx.storage_resolver)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?
        .uri()
        .clone();
    let abs_path = output_uri.path().as_path().to_path_buf();

    let compress_sink_out =
        crate::SinkOutput::new(&ctx.sandbox_root, compress_rel, &ctx.storage_resolver)
            .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;

    let mut cursor = Cursor::new(Vec::new());
    let writer = BufWriter::new(&mut cursor);
    reearth_flow_common::zip::write(writer, abs_path.as_path())
        .map_err(|e| crate::errors::SinkError::MvtWriter(e.to_string()))?;

    compress_sink_out
        .write(bytes::Bytes::from(cursor.into_inner()))
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;

    std::fs::remove_dir_all(abs_path.as_path())
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
    Ok(())
}

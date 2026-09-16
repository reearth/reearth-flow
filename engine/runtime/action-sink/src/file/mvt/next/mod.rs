//! Vector-tile writer for the new geometry type.
//!
//! [`build`] is the whole pipeline: geometry is reprojected to WGS84, sliced
//! into a `min_zoom..=max_zoom` pyramid and encoded one tile at a time. It
//! takes no executor context, so both the sink and the intermediate-data view
//! drive it.

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

/// One feature to tile, with the layer it lands in.
pub struct TileFeature<'a> {
    pub feature: &'a Feature,
    pub layer_name: &'a str,
    /// Vector-tile feature id, repeated in every tile the feature reaches.
    /// `None` leaves the encoded feature unidentified.
    pub id: Option<u64>,
}

/// Knobs shared by every tile of a tileset.
#[derive(Clone, Copy)]
pub struct TileOptions<'a> {
    /// Lowest zoom sliced; must not exceed `max_zoom`.
    pub min_zoom: u8,
    /// Highest zoom sliced, inclusive.
    pub max_zoom: u8,
    /// Coordinate grid resolution within a tile.
    pub extent: i32,
    /// Size cap per tile; the visually smallest features are dropped until a
    /// tile fits under it.
    pub max_tile_bytes: u64,
    /// Joins array and map attribute values into one string tag. `None` leaves
    /// them out of the tile.
    pub array_map_separator: Option<&'a str>,
    /// `name` of the tilejson document.
    pub name: Option<&'a str>,
}

/// A tileset's non-tile output. The tiles stream out through [`build`]'s
/// `write_tile` callback as they are encoded, so only their count is kept here.
#[derive(Debug)]
pub struct BuiltTiles {
    pub tilejson: String,
    pub tile_count: usize,
    /// Features that reached at least one tile; the rest are absent from the
    /// output.
    pub rendered_features: usize,
}

/// Slice `features` into a vector tile pyramid, handing each tile to
/// `write_tile` as `{z}/{x}/{y}.mvt` relative to the tileset root.
///
/// Geometry carrying no geographic CRS, and 3D geometry, are skipped with a
/// warning rather than tiled. Fails before any tile is written when
/// `options` describe no pyramid: an inverted zoom range or a non-positive
/// extent.
pub fn build(
    features: &[TileFeature<'_>],
    options: TileOptions<'_>,
    write_tile: impl Fn(String, Vec<u8>) -> crate::errors::Result<()> + Sync,
) -> crate::errors::Result<BuiltTiles> {
    validate(&options)?;
    let accum = features
        .par_iter()
        .fold(SliceAccum::default, |mut acc, input| {
            acc.layer_names.insert(input.layer_name.to_string());
            let mut cache = ReprojectionCache::new();
            let leaves = extract(&input.feature.geometry, &mut cache);
            let (content, tiled) = slice_leaves(
                leaves,
                options.min_zoom,
                options.max_zoom,
                options.extent as u32,
            );
            acc.content = std::mem::take(&mut acc.content).union(content);
            acc.rendered_features += !tiled.is_empty() as usize;
            for tiled_leaf in tiled {
                let sliced = to_sliced_feature(input, tiled_leaf.geom);
                acc.by_tile.entry(tiled_leaf.key).or_default().push(sliced);
            }
            acc
        })
        .reduce(SliceAccum::default, SliceAccum::merge);

    accum
        .by_tile
        .par_iter()
        .try_for_each(|(&(zoom, x, y), feats)| {
            let bytes = make_tile(
                options.extent,
                feats,
                options.max_tile_bytes,
                options.array_map_separator,
            )?;
            write_tile(format!("{zoom}/{x}/{y}.mvt"), bytes)
        })?;

    Ok(BuiltTiles {
        tilejson: tilejson(&options, &accum.content, &accum.layer_names)?,
        tile_count: accum.by_tile.len(),
        rendered_features: accum.rendered_features,
    })
}

/// Reject options that describe no pyramid.
fn validate(options: &TileOptions<'_>) -> crate::errors::Result<()> {
    if options.min_zoom > options.max_zoom {
        return Err(crate::errors::SinkError::MvtWriter(format!(
            "minZoom {} exceeds maxZoom {}",
            options.min_zoom, options.max_zoom
        )));
    }
    if options.extent <= 0 {
        return Err(crate::errors::SinkError::MvtWriter(format!(
            "extent must be positive, got {}",
            options.extent
        )));
    }
    Ok(())
}

#[derive(Default)]
struct SliceAccum {
    content: TileContent,
    layer_names: HashSet<String>,
    by_tile: HashMap<TileKey, Vec<SlicedFeature>>,
    rendered_features: usize,
}

impl SliceAccum {
    fn merge(mut self, other: Self) -> Self {
        self.content = self.content.union(other.content);
        self.layer_names.extend(other.layer_names);
        self.rendered_features += other.rendered_features;
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
    let features: Vec<TileFeature<'_>> = upstream
        .iter()
        .map(|(feature, layer_name)| TileFeature {
            feature,
            layer_name,
            id: None,
        })
        .collect();

    let built = build(
        &features,
        TileOptions {
            min_zoom: params.min_zoom,
            max_zoom: params.max_zoom,
            extent: params.extent,
            max_tile_bytes: params.max_tile_bytes,
            array_map_separator: params.array_map_separator.as_deref(),
            name: std::path::Path::new(output)
                .file_name()
                .and_then(|name| name.to_str()),
        },
        |relative_path, bytes| write_output(ctx, &format!("{output}/{relative_path}"), bytes),
    )?;

    write_output(
        ctx,
        &format!("{output}/tilejson.json"),
        built.tilejson.into_bytes(),
    )?;

    if let Some(compress_rel) = compress_output {
        compress_tileset(ctx, output, compress_rel)?;
    }
    Ok(())
}

fn write_output(ctx: &NodeContext, path: &str, bytes: Vec<u8>) -> crate::errors::Result<()> {
    crate::SinkOutput::new(&ctx.sandbox_root, path, &ctx.storage_resolver)
        .and_then(|out| out.write(bytes::Bytes::from(bytes)))
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
}

fn to_sliced_feature(input: &TileFeature<'_>, geom: TiledGeom) -> SlicedFeature {
    let geom = match geom {
        TiledGeom::Polygon(parts) => SlicedGeom::Polygon(parts),
        TiledGeom::LineString(lines) => SlicedGeom::LineString(lines),
        TiledGeom::Point(points) => SlicedGeom::Point(points),
    };
    SlicedFeature {
        layer_name: input.layer_name.to_string(),
        geom,
        properties: input.feature.attributes.clone(),
        id: input.id,
    }
}

fn tilejson(
    options: &TileOptions<'_>,
    content: &TileContent,
    layer_names: &HashSet<String>,
) -> crate::errors::Result<String> {
    let tiles = vec!["/{z}/{x}/{y}.mvt".to_string()];
    let vector_layers: Vec<_> = layer_names
        .iter()
        .map(|id| VectorLayer {
            id: id.clone(),
            fields: HashMap::new(),
        })
        .collect();
    let metadata = TileMetadata::from_tile_content(
        options.name.map(str::to_string),
        options.min_zoom,
        options.max_zoom,
        content,
        tiles,
        vector_layers,
    );

    serde_json::to_string_pretty(&metadata)
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn options(min_zoom: u8, max_zoom: u8, extent: i32) -> TileOptions<'static> {
        TileOptions {
            min_zoom,
            max_zoom,
            extent,
            max_tile_bytes: 500_000,
            array_map_separator: None,
            name: None,
        }
    }

    /// Options that describe no pyramid are refused up front rather than
    /// producing an empty one, and nothing is written for them.
    #[test]
    fn options_that_describe_no_pyramid_are_refused() {
        let written = std::sync::atomic::AtomicUsize::new(0);
        let write = |_path: String, _bytes: Vec<u8>| {
            written.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        };

        let inverted = build(&[], options(16, 15, 4096), write).expect_err("min above max");
        assert!(inverted
            .to_string()
            .contains("minZoom 16 exceeds maxZoom 15"));

        let flat = build(&[], options(0, 15, 0), write).expect_err("no extent");
        assert!(flat.to_string().contains("extent must be positive"));

        build(&[], options(15, 15, 4096), write).expect("a one-level pyramid is a pyramid");
        assert_eq!(written.load(std::sync::atomic::Ordering::Relaxed), 0);
    }
}

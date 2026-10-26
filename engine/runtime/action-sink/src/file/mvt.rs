//! Vector-tile writer.
//!
//! [`build`] is the whole pipeline: geometry is sliced into a
//! `min_zoom..=max_zoom` pyramid in memory and encoded one tile at a time. A
//! tile over the size cap is coarsened under a single geometric error budget
//! until it fits (see `tile::make_tile`).

mod extract;
pub(crate) mod sink;
mod slice;
pub(crate) mod tags;
mod tile;
pub(crate) mod tiling;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufWriter, Cursor};
use std::path::Path;

use rayon::prelude::*;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_types::Feature;

use extract::extract;
use slice::{slice_leaves, SlicedGeom, TileKey};
use tile::{make_tile, SlicedFeature, FULL_DETAIL};
use tiling::{TileContent, TileMetadata, VectorLayer};

// Screen pixels across a rendered tile, for reporting the error budget.
const TILE_PIXELS: f64 = 512.0;

// Feature ids listed in the size-cap warning.
const SIZE_LIMITED_SAMPLE: usize = 10;

/// One feature to tile, with the layer it lands in.
pub(super) struct TileFeature<'a> {
    pub(super) feature: &'a Feature,
    pub(super) layer_name: &'a str,
}

/// Knobs shared by every tile of a tileset.
#[derive(Clone, Copy)]
pub(super) struct TileOptions<'a> {
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
    pub(super) extent: i32,
    /// Coordinate grid resolution of the `max_zoom` tiles.
    pub(super) extent_at_max_zoom: i32,
    /// Size cap per tile, before compression, at every zoom. A larger tile is
    /// written with a coarser geometric error until it fits under it.
    pub(super) max_tile_bytes: u64,
    /// Exempts `max_zoom` tiles from `max_tile_bytes`, writing them at full detail.
    pub(super) keep_all_at_max_zoom: bool,
    /// `name` of the tilejson document.
    pub(super) name: Option<&'a str>,
}

/// A tileset's non-tile output; the tiles stream out through [`build`]'s
/// `write_tile` callback as they are encoded.
#[derive(Debug)]
pub(super) struct BuiltTiles {
    pub(super) tilejson: String,
    pub(super) tile_count: usize,
    /// Features written into at least one tile; the rest are absent from the
    /// output.
    pub(super) rendered_features: usize,
    /// Indices into `build`'s input of the features the size cap left out of
    /// at least one tile at `max_zoom`, ascending.
    pub(super) size_limited_features: Vec<usize>,
    /// Tiles, at any zoom, the size cap left at least one feature out of.
    pub(super) size_limited_tiles: usize,
    /// Per zoom, over the tiles that needed more than full detail.
    pub(super) coarsening: BTreeMap<u8, ZoomCoarsening>,
}

/// Slice `features` into a vector tile pyramid, handing each tile to
/// `write_tile` as `{z}/{x}/{y}.mvt` relative to the tileset root.
pub(super) fn build(
    features: &[TileFeature<'_>],
    options: TileOptions<'_>,
    write_tile: impl Fn(String, Vec<u8>) -> crate::errors::Result<()> + Sync,
) -> crate::errors::Result<BuiltTiles> {
    validate(&options)?;
    let accum = features
        .par_iter()
        .enumerate()
        .fold(SliceAccum::default, |mut acc, (source, input)| {
            acc.layer_names.insert(input.layer_name.to_string());
            let leaves = extract(&input.feature.geometry.value);
            let (content, tiled) = slice_leaves(
                leaves,
                options.min_zoom,
                options.max_zoom,
                options.extent.max(options.extent_at_max_zoom) as u32,
            );
            acc.content = std::mem::take(&mut acc.content).union(content);
            for tiled_leaf in tiled {
                let sliced = to_sliced_feature(input, source, tiled_leaf.geom);
                acc.by_tile.entry(tiled_leaf.key).or_default().push(sliced);
            }
            acc
        })
        .reduce(SliceAccum::default, SliceAccum::merge);

    let tiles = accum
        .by_tile
        .par_iter()
        .try_fold(TileAccum::default, |mut acc, (&(zoom, x, y), feats)| {
            let tile = make_tile(
                options.extent_at(zoom),
                feats,
                options.max_tile_bytes_at(zoom),
            );
            if tile.error > FULL_DETAIL {
                acc.coarsening
                    .entry(zoom)
                    .or_default()
                    .add(&ZoomCoarsening {
                        coarsened: 1,
                        max_budget: tile.error,
                    });
            }
            write_tile(format!("{zoom}/{x}/{y}.mvt"), tile.bytes)?;
            acc.rendered.extend(tile.written);
            acc.size_limited_tiles += !tile.size_limited.is_empty() as usize;
            if zoom == options.max_zoom {
                acc.size_limited_features.extend(tile.size_limited);
            }
            Ok::<_, crate::errors::SinkError>(acc)
        })
        .try_reduce(TileAccum::default, |a, b| Ok(a.merge(b)))?;

    Ok(BuiltTiles {
        tilejson: tilejson(&options, &accum.content, &accum.layer_names)?,
        tile_count: accum.by_tile.len(),
        rendered_features: tiles.rendered.len(),
        size_limited_features: tiles.size_limited_features.into_iter().collect(),
        size_limited_tiles: tiles.size_limited_tiles,
        coarsening: tiles.coarsening,
    })
}

impl TileOptions<'_> {
    fn extent_at(&self, zoom: u8) -> i32 {
        if zoom == self.max_zoom {
            self.extent_at_max_zoom
        } else {
            self.extent
        }
    }

    fn max_tile_bytes_at(&self, zoom: u8) -> u64 {
        if zoom == self.max_zoom && self.keep_all_at_max_zoom {
            u64::MAX
        } else {
            self.max_tile_bytes
        }
    }
}

/// Reject options that describe no pyramid.
fn validate(options: &TileOptions<'_>) -> crate::errors::Result<()> {
    if options.min_zoom > options.max_zoom {
        return Err(crate::errors::SinkError::MvtWriter(format!(
            "minZoom {} exceeds maxZoom {}",
            options.min_zoom, options.max_zoom
        )));
    }
    if options.extent <= 0 || options.extent_at_max_zoom <= 0 {
        return Err(crate::errors::SinkError::MvtWriter(format!(
            "extent must be positive, got {} ({} at maxZoom)",
            options.extent, options.extent_at_max_zoom
        )));
    }
    Ok(())
}

#[derive(Default)]
struct SliceAccum {
    content: TileContent,
    layer_names: BTreeSet<String>,
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

/// What the encoded tiles hold, by input feature index.
#[derive(Default)]
struct TileAccum {
    rendered: HashSet<usize>,
    size_limited_features: BTreeSet<usize>,
    size_limited_tiles: usize,
    coarsening: BTreeMap<u8, ZoomCoarsening>,
}

/// How far the size cap pushed one zoom's tiles from full detail.
#[derive(Debug, Default)]
pub(super) struct ZoomCoarsening {
    /// Tiles written with more than full detail's error budget.
    pub(super) coarsened: usize,
    /// Largest error budget among them, in nominal extent units.
    pub(super) max_budget: f64,
}

impl ZoomCoarsening {
    fn add(&mut self, other: &Self) {
        self.coarsened += other.coarsened;
        self.max_budget = self.max_budget.max(other.max_budget);
    }
}

impl TileAccum {
    fn merge(mut self, other: Self) -> Self {
        self.rendered.extend(other.rendered);
        self.size_limited_features
            .extend(other.size_limited_features);
        self.size_limited_tiles += other.size_limited_tiles;
        for (zoom, coarsening) in other.coarsening {
            self.coarsening.entry(zoom).or_default().add(&coarsening);
        }
        self
    }
}

/// Write one tileset (tiles, tilejson.json, and the optional zip) for
/// already attribute-filtered `upstream`.
pub(super) fn write_tileset(
    ctx: &Context,
    upstream: &[(Feature, String)],
    output: &Uri,
    compress_output: Option<&Uri>,
    options: TileOptions<'_>,
) -> crate::errors::Result<()> {
    let storage = ctx
        .storage_resolver
        .resolve(output)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
    let write = |relative_path: &str, bytes: Vec<u8>| {
        let path = output
            .join(Path::new(relative_path))
            .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
        storage
            .put_sync(&path.path(), bytes::Bytes::from(bytes))
            .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
    };

    let features: Vec<TileFeature<'_>> = upstream
        .iter()
        .map(|(feature, layer_name)| TileFeature {
            feature,
            layer_name,
        })
        .collect();
    let name = output.file_name().map(|s| s.to_string_lossy().to_string());
    let options = TileOptions {
        name: name.as_deref(),
        ..options
    };
    let built = build(&features, options, |relative_path, bytes| {
        write(&relative_path, bytes)
    })?;
    tracing::info!(
        "MVTWriter: wrote {} tiles to {output}; {} of {} features rendered",
        built.tile_count,
        built.rendered_features,
        upstream.len()
    );
    for (zoom, coarsening) in &built.coarsening {
        tracing::info!(
            "MVTWriter: z{zoom}: {} tiles coarsened to fit, max budget {:.2} px",
            coarsening.coarsened,
            coarsening.max_budget * TILE_PIXELS / options.extent_at(*zoom) as f64
        );
    }
    if !built.size_limited_features.is_empty() {
        let sample: Vec<String> = built
            .size_limited_features
            .iter()
            .take(SIZE_LIMITED_SAMPLE)
            .map(|&index| upstream[index].0.id.to_string())
            .collect();
        ctx.event_hub.warn_log(
            None,
            format!(
                "MVTWriter: tile size cap of {} bytes left {} features out of maxZoom tiles ({} tiles trimmed at any zoom); e.g. {}",
                options.max_tile_bytes,
                built.size_limited_features.len(),
                built.size_limited_tiles,
                sample.join(", ")
            ),
        );
    }

    write("tilejson.json", built.tilejson.into_bytes())?;

    if let Some(compress_output) = compress_output {
        compress_tileset(ctx, output, compress_output)?;
    }
    Ok(())
}

fn to_sliced_feature(input: &TileFeature<'_>, source: usize, geom: SlicedGeom) -> SlicedFeature {
    SlicedFeature {
        layer_name: input.layer_name.to_string(),
        geom,
        properties: input.feature.attributes.clone(),
        source,
    }
}

fn tilejson(
    options: &TileOptions<'_>,
    content: &TileContent,
    layer_names: &BTreeSet<String>,
) -> crate::errors::Result<String> {
    // Absolute path: the parent directory of tilejson.json is the HTTP root.
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
    ctx: &Context,
    output: &Uri,
    compress_output: &Uri,
) -> crate::errors::Result<()> {
    let storage = ctx
        .storage_resolver
        .resolve(compress_output)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;

    let mut cursor = Cursor::new(Vec::new());
    let writer = BufWriter::new(&mut cursor);
    reearth_flow_common::zip::write(writer, output.path().as_path())
        .map_err(|e| crate::errors::SinkError::MvtWriter(e.to_string()))?;

    storage
        .put_sync(
            compress_output.path().as_path(),
            bytes::Bytes::from(cursor.into_inner()),
        )
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;

    std::fs::remove_dir_all(output.path().as_path())
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::coordinate::Coordinate2D;
    use reearth_flow_geometry::types::geometry::Geometry2D;
    use reearth_flow_geometry::types::line_string::LineString2D;
    use reearth_flow_geometry::types::polygon::Polygon2D;
    use reearth_flow_types::geometry::{Geometry, GeometryValue};

    use super::*;

    fn options(min_zoom: u8, max_zoom: u8, extent: i32) -> TileOptions<'static> {
        TileOptions {
            min_zoom,
            max_zoom,
            extent,
            extent_at_max_zoom: extent,
            max_tile_bytes: 500_000,
            keep_all_at_max_zoom: false,
            name: None,
        }
    }

    fn square(lng: f64, lat: f64, size: f64) -> Feature {
        let ring: Vec<Coordinate2D<f64>> = [
            [lng, lat],
            [lng + size, lat],
            [lng + size, lat + size],
            [lng, lat + size],
            [lng, lat],
        ]
        .iter()
        .map(|&[x, y]| Coordinate2D::new_(x, y))
        .collect();
        let poly = Polygon2D::new(LineString2D::new(ring), vec![]);
        Feature::from(Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)),
        })
    }

    #[test]
    fn options_that_describe_no_pyramid_are_refused() {
        let write = |_path: String, _bytes: Vec<u8>| Ok(());
        let inverted = build(&[], options(16, 15, 4096), write).expect_err("min above max");
        assert!(inverted
            .to_string()
            .contains("minZoom 16 exceeds maxZoom 15"));
        let flat = build(&[], options(0, 15, 0), write).expect_err("no extent");
        assert!(flat.to_string().contains("extent must be positive"));
    }

    #[test]
    fn a_polygon_is_written_into_one_tile_per_zoom() {
        let feature = square(139.7, 35.6, 1e-4);
        let input = [TileFeature {
            feature: &feature,
            layer_name: "layer",
        }];
        let written = std::sync::Mutex::new(Vec::new());
        let built = build(&input, options(10, 12, 4096), |path, _bytes| {
            written.lock().unwrap().push(path);
            Ok(())
        })
        .expect("build");

        let mut written = written.into_inner().unwrap();
        written.sort();
        assert_eq!(built.tile_count, 3);
        assert_eq!(
            written.iter().map(|p| &p[..3]).collect::<Vec<_>>(),
            vec!["10/", "11/", "12/"]
        );
        assert!(built.size_limited_features.is_empty());
    }

    #[test]
    fn max_zoom_tiles_are_written_at_their_own_extent() {
        let feature = square(139.7, 35.6, 1e-4);
        let input = [TileFeature {
            feature: &feature,
            layer_name: "layer",
        }];
        let extents = std::sync::Mutex::new(std::collections::BTreeMap::new());
        build(
            &input,
            TileOptions {
                extent_at_max_zoom: 65536,
                ..options(10, 12, 4096)
            },
            |path, bytes| {
                let tile: tinymvt::vector_tile::Tile =
                    prost::Message::decode(bytes.as_slice()).expect("a tile");
                extents
                    .lock()
                    .unwrap()
                    .insert(path[..2].to_string(), tile.layers[0].extent);
                Ok(())
            },
        )
        .expect("build");

        let extents = extents.into_inner().unwrap();
        assert_eq!(extents["10"], Some(4096));
        assert_eq!(extents["11"], Some(4096));
        assert_eq!(extents["12"], Some(65536));
    }

    /// Under a cap too small for any of them, a grid of squares is thinned
    /// at every zoom, unless the maximum zoom keeps them all.
    #[test]
    fn keeping_all_at_max_zoom_exempts_only_max_zoom_from_the_cap() {
        let features: Vec<Feature> = (0..100)
            .map(|i| {
                square(
                    139.7 + (i % 10) as f64 * 2e-4,
                    35.6 + (i / 10) as f64 * 2e-4,
                    1e-4,
                )
            })
            .collect();
        let input: Vec<TileFeature<'_>> = features
            .iter()
            .map(|feature| TileFeature {
                feature,
                layer_name: "layer",
            })
            .collect();
        let counts = |keep_all_at_max_zoom| {
            let counts = std::sync::Mutex::new(std::collections::BTreeMap::new());
            let options = TileOptions {
                max_tile_bytes: 200,
                keep_all_at_max_zoom,
                ..options(14, 16, 4096)
            };
            let built = build(&input, options, |path, bytes| {
                let tile: tinymvt::vector_tile::Tile =
                    prost::Message::decode(bytes.as_slice()).expect("a tile");
                let features: usize = tile.layers.iter().map(|l| l.features.len()).sum();
                *counts
                    .lock()
                    .unwrap()
                    .entry(path[..2].to_string())
                    .or_insert(0) += features;
                Ok(())
            })
            .expect("build");
            (built, counts.into_inner().unwrap())
        };

        let (capped, capped_counts) = counts(false);
        assert!(capped_counts["16"] < 100);
        assert!(!capped.size_limited_features.is_empty());

        let (kept, kept_counts) = counts(true);
        assert_eq!(kept_counts["16"], 100);
        assert!(kept.size_limited_features.is_empty());
        assert_eq!(kept_counts["14"], capped_counts["14"]);
    }
}

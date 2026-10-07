//! Vector-tile writer for the new geometry type.
//!
//! [`build`] is the whole pipeline: geometry is reprojected to WGS84, sliced
//! into a `min_zoom..=max_zoom` pyramid and encoded one tile at a time. It
//! takes no executor context, so both the sink and the intermediate-data view
//! drive it.

mod extract;
mod slice;
mod tile;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{BufWriter, Cursor};
use std::sync::Arc;

use rayon::prelude::*;
use reearth_flow_diagnostics::ErrorCode;
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
    /// The tilejson `tiles` URL template, which must contain `{z}`, `{x}` and
    /// `{y}`. `None` writes `/{z}/{x}/{y}.mvt`, which resolves only when the
    /// tileset directory is served as the HTTP root.
    pub tiles_url: Option<&'a str>,
}

/// A tileset's non-tile output. The tiles stream out through [`build`]'s
/// `write_tile` callback as they are encoded, so only their count is kept here.
#[derive(Debug)]
pub struct BuiltTiles {
    pub tilejson: String,
    pub tile_count: usize,
    /// Features written into at least one tile; the rest are absent from the
    /// output.
    pub rendered_features: usize,
    /// Features sliced into at least one tile, before any were left out of it
    /// for being under a pixel or over the size cap. At least
    /// `rendered_features`.
    pub sliced_features: usize,
    /// Indices into `build`'s input of the features the size cap left out of
    /// at least one tile at `max_zoom`, ascending. A viewer zoomed in past
    /// `max_zoom` shows the `max_zoom` tile, so these are missing there however
    /// far it zooms in. A feature left out only at lower zooms is not here.
    pub size_limited_features: Vec<usize>,
    /// Tiles, at any zoom, the size cap left at least one feature out of.
    pub size_limited_tiles: usize,
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
        .enumerate()
        .fold(SliceAccum::default, |mut acc, (source, input)| {
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
            acc.sliced_features += !tiled.is_empty() as usize;
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
                options.extent,
                feats,
                options.max_tile_bytes,
                options.array_map_separator,
            )?;
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
        sliced_features: accum.sliced_features,
        size_limited_features: tiles.size_limited_features.into_iter().collect(),
        size_limited_tiles: tiles.size_limited_tiles,
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
    sliced_features: usize,
}

impl SliceAccum {
    fn merge(mut self, other: Self) -> Self {
        self.content = self.content.union(other.content);
        self.layer_names.extend(other.layer_names);
        self.sliced_features += other.sliced_features;
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
}

impl TileAccum {
    fn merge(mut self, other: Self) -> Self {
        self.rendered.extend(other.rendered);
        self.size_limited_features
            .extend(other.size_limited_features);
        self.size_limited_tiles += other.size_limited_tiles;
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
            tiles_url: None,
        },
        |relative_path, bytes| write_output(ctx, &format!("{output}/{relative_path}"), bytes),
    )?;
    for &index in &built.size_limited_features {
        ctx.report_drop(
            ErrorCode::MvtTileSizeLimit,
            Some(upstream[index].0.id),
            Some(true),
        );
    }

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

fn to_sliced_feature(input: &TileFeature<'_>, source: usize, geom: TiledGeom) -> SlicedFeature {
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
        source,
    }
}

fn tilejson(
    options: &TileOptions<'_>,
    content: &TileContent,
    layer_names: &HashSet<String>,
) -> crate::errors::Result<String> {
    let tiles = vec![options.tiles_url.unwrap_or("/{z}/{x}/{y}.mvt").to_string()];
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
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::line_string::LineString2D;
    use reearth_flow_geometry::point::Point2D;
    use reearth_flow_geometry::polygon::Polygon2D;
    use std::str::FromStr;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_geometry::{Euclidean2DGeometry, Geometry};
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::node::NodeHandle;
    use reearth_flow_types::{AttributeValue, CompiledCode};

    use super::*;

    fn options(min_zoom: u8, max_zoom: u8, extent: i32) -> TileOptions<'static> {
        TileOptions {
            min_zoom,
            max_zoom,
            extent,
            max_tile_bytes: 500_000,
            array_map_separator: None,
            name: None,
            tiles_url: None,
        }
    }

    fn tiles_of(tilejson: &str) -> Vec<String> {
        let doc: serde_json::Value = serde_json::from_str(tilejson).expect("tilejson parses");
        serde_json::from_value(doc["tiles"].clone()).expect("tiles is a list of strings")
    }

    /// Without a URL the template stays root-relative, which is the documented
    /// MVT Writer output; with one, the tilejson carries it verbatim.
    #[test]
    fn the_tiles_template_is_the_given_url_or_root_relative() {
        let write = |_path: String, _bytes: Vec<u8>| Ok(());

        let default = build(&[], options(0, 15, 4096), write).expect("build");
        assert_eq!(tiles_of(&default.tilejson), vec!["/{z}/{x}/{y}.mvt"]);

        let url = "https://example.com/views/abc/{z}/{x}/{y}.mvt";
        let given = build(
            &[],
            TileOptions {
                tiles_url: Some(url),
                ..options(0, 15, 4096)
            },
            write,
        )
        .expect("build");
        assert_eq!(tiles_of(&given.tilejson), vec![url]);
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

    fn crs() -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(4326))
    }

    fn feature(name: String, geometry: Euclidean2DGeometry) -> Feature {
        let mut attributes = reearth_flow_types::Attributes::new();
        attributes.insert(Attribute::new("name"), AttributeValue::String(name));
        let mut feature = Feature::from(attributes);
        feature.set_geometry(Geometry::Euclidean2D(geometry));
        feature
    }

    /// Square ring of side `size` degrees with its corner at `[lat, lng]`,
    /// latitude first as a geographic CRS stores it.
    fn square(lat: f64, lng: f64, size: f64) -> Euclidean2DGeometry {
        let ring = [
            [lat, lng],
            [lat + size, lng],
            [lat + size, lng + size],
            [lat, lng + size],
            [lat, lng],
        ];
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            crs(),
            ring,
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    /// A cluster of points and polygons of many sizes, crossed by one line,
    /// dense enough to overflow `CAPPED_TILE_BYTES` at every zoom.
    fn crowded_features() -> Vec<Feature> {
        let mut features = Vec::new();
        for i in 0..200 {
            let (row, col) = ((i / 20) as f64, (i % 20) as f64);
            let point = Point2D::new(crs(), [35.6 + row * 1e-4, 139.7 + col * 1e-4]);
            features.push(feature(
                format!("point {i}"),
                Euclidean2DGeometry::Point(point),
            ));
        }
        for i in 0..60 {
            let size = 1e-5 * (i + 1) as f64;
            let (row, col) = ((i / 10) as f64, (i % 10) as f64);
            features.push(feature(
                format!("polygon {i}"),
                square(35.6 + row * 4e-4, 139.7 + col * 4e-4, size),
            ));
        }
        let line = LineString2D::from_coords(crs(), [[35.6, 139.7], [35.603, 139.704]]);
        features.push(feature(
            "line".to_string(),
            Euclidean2DGeometry::LineString(line),
        ));
        features
    }

    const CAPPED_TILE_BYTES: u64 = 2_000;

    /// Build `features` over zooms 12 to 16, collecting every tile written by
    /// path. With `with_ids`, each feature's tile id is its index.
    fn build_tiles(
        features: &[Feature],
        max_tile_bytes: u64,
        with_ids: bool,
    ) -> (BuiltTiles, std::collections::BTreeMap<String, Vec<u8>>) {
        let inputs: Vec<TileFeature<'_>> = features
            .iter()
            .enumerate()
            .map(|(index, feature)| TileFeature {
                feature,
                layer_name: "layer",
                id: with_ids.then_some(index as u64),
            })
            .collect();
        let tiles = std::sync::Mutex::new(std::collections::BTreeMap::new());
        let built = build(
            &inputs,
            TileOptions {
                max_tile_bytes,
                ..options(12, 16, 4096)
            },
            |path, bytes| {
                tiles.lock().unwrap().insert(path, bytes);
                Ok(())
            },
        )
        .expect("build");
        (built, tiles.into_inner().unwrap())
    }

    fn decode(bytes: &[u8]) -> tinymvt::vector_tile::Tile {
        prost::Message::decode(bytes).expect("a tile")
    }

    /// The ids found in one tile.
    fn ids_of(bytes: &[u8]) -> BTreeSet<u64> {
        decode(bytes)
            .layers
            .into_iter()
            .flat_map(|layer| layer.features)
            .filter_map(|feature| feature.id)
            .collect()
    }

    /// The ids found in any of `tiles`.
    fn ids_in(tiles: &std::collections::BTreeMap<String, Vec<u8>>) -> BTreeSet<u64> {
        tiles.values().flat_map(|bytes| ids_of(bytes)).collect()
    }

    /// The size cap drops points first, so points that crowd a tile at every
    /// zoom reach no tile at all and are not rendered.
    #[test]
    fn features_the_size_cap_removes_from_every_tile_are_not_rendered() {
        let points: Vec<Feature> = crowded_features()
            .into_iter()
            .filter(|f| {
                matches!(
                    *f.geometry,
                    Geometry::Euclidean2D(Euclidean2DGeometry::Point(_))
                )
            })
            .collect();
        let (built, tiles) = build_tiles(&points, CAPPED_TILE_BYTES, true);

        let drawn = ids_in(&tiles);
        assert!(drawn.len() < points.len(), "the cap removed no point");
        assert_eq!(built.rendered_features, drawn.len());
    }

    /// A line shorter than a pixel at every zoom is sliced into tiles but
    /// written into none of them.
    #[test]
    fn a_feature_below_a_pixel_at_every_zoom_is_not_rendered() {
        let line = LineString2D::from_coords(crs(), [[35.6, 139.7], [35.6, 139.700_000_1]]);
        let features = [feature(
            "line".to_string(),
            Euclidean2DGeometry::LineString(line),
        )];
        let (built, tiles) = build_tiles(&features, u64::MAX, true);

        assert!(!tiles.is_empty(), "the line was not sliced");
        assert!(ids_in(&tiles).is_empty());
        assert_eq!(built.rendered_features, 0);
        assert!(built.size_limited_features.is_empty());
        assert_eq!(built.size_limited_tiles, 0);
    }

    /// The ids in each of `tiles`, by path.
    fn ids_by_tile(
        tiles: &std::collections::BTreeMap<String, Vec<u8>>,
    ) -> std::collections::BTreeMap<&str, BTreeSet<u64>> {
        tiles
            .iter()
            .map(|(path, bytes)| (path.as_str(), ids_of(bytes)))
            .collect()
    }

    /// Without a cap, a tile still leaves out features under a pixel, so what
    /// an uncapped tile holds and the capped one does not is exactly what the
    /// cap left out. A feature is size-limited when that happened to it in any
    /// tile at the highest zoom, and counted once however many such tiles it
    /// spans; a tile is size-limited when it happened at any zoom.
    #[test]
    fn features_the_size_cap_leaves_out_at_max_zoom_are_size_limited() {
        let features = crowded_features();
        let (built, capped) = build_tiles(&features, CAPPED_TILE_BYTES, true);
        let (_, uncapped) = build_tiles(&features, u64::MAX, true);
        let (capped, uncapped) = (ids_by_tile(&capped), ids_by_tile(&uncapped));

        let mut left_out_at_max_zoom = BTreeSet::new();
        let mut size_limited_tiles = 0;
        for (path, all) in &uncapped {
            let left_out: Vec<u64> = all.difference(&capped[path]).copied().collect();
            size_limited_tiles += !left_out.is_empty() as usize;
            if path.starts_with("16/") {
                left_out_at_max_zoom.extend(left_out);
            }
        }
        let expected: Vec<usize> = left_out_at_max_zoom
            .into_iter()
            .map(|id| id as usize)
            .collect();

        assert!(!expected.is_empty(), "the cap left nothing out at max zoom");
        assert_eq!(built.size_limited_features, expected);
        assert_eq!(built.size_limited_tiles, size_limited_tiles);
    }

    /// Lower zooms are overviews, where thinning is expected: a feature the
    /// cap leaves out only there is still drawn at the highest zoom.
    #[test]
    fn a_feature_left_out_only_below_max_zoom_is_not_size_limited() {
        // A 5 by 5 grid 0.01 degrees apart: one point to a tile at zoom 16,
        // all of them in one tile at zoom 12.
        let features: Vec<Feature> = (0..25)
            .map(|i| {
                let (row, col) = ((i / 5) as f64, (i % 5) as f64);
                let point = Point2D::new(crs(), [35.61 + row * 0.01, 139.71 + col * 0.01]);
                feature(format!("point {i}"), Euclidean2DGeometry::Point(point))
            })
            .collect();
        let (built, _) = build_tiles(&features, 200, true);

        assert!(built.size_limited_tiles > 0, "the cap left nothing out");
        assert!(built.size_limited_features.is_empty());
        assert_eq!(built.rendered_features, features.len());
    }

    /// An MVT Writer over zooms 12 to 16 writing one layer to `tiles`.
    fn writer_params(max_tile_bytes: u64) -> MVTWriterCompiledParam {
        MVTWriterCompiledParam {
            output: CompiledCode::Literal("tiles".to_string()),
            layer_name: CompiledCode::Literal("layer".to_string()),
            min_zoom: 12,
            max_zoom: 16,
            compress_output: None,
            skip_unexposed_attributes: false,
            colon_to_underscore: false,
            extent: 4096,
            schema_key: None,
            max_tile_bytes,
            array_map_separator: None,
        }
    }

    /// Write `features` as the MVT Writer does under the default policy,
    /// returning the diagnostics its node collected.
    fn write_with_diagnostics(
        features: &[Feature],
        max_tile_bytes: u64,
    ) -> Vec<reearth_flow_diagnostics::Diagnostic> {
        let dir = tempfile::tempdir().expect("tempdir");
        let handle = Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "MVT Writer".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            true,
        ));
        let ctx = NodeContext {
            sandbox_root: Uri::from_str(&format!("file://{}", dir.path().display()))
                .expect("a file uri"),
            diagnostics: Some(handle.clone()),
            ..NodeContext::default()
        };
        let upstream: Vec<(Feature, String)> = features
            .iter()
            .map(|feature| (feature.clone(), "layer".to_string()))
            .collect();

        write_tileset(
            &ctx,
            &upstream,
            "tiles",
            None,
            &writer_params(max_tile_bytes),
        )
        .expect("write");
        assert!(dir.path().join("tiles/tilejson.json").exists());
        handle.inner.drain_summaries()
    }

    /// Each feature the size cap left out at the highest zoom is reported as
    /// a drop, by its id, under one code.
    #[test]
    fn the_writer_reports_each_feature_the_size_cap_left_out() {
        let features = crowded_features();
        let expected: BTreeSet<uuid::Uuid> = build_tiles(&features, CAPPED_TILE_BYTES, false)
            .0
            .size_limited_features
            .iter()
            .map(|&index| features[index].id)
            .collect();
        assert!(!expected.is_empty(), "the cap left nothing out");

        let summaries = write_with_diagnostics(&features, CAPPED_TILE_BYTES);

        assert_eq!(summaries.len(), 1, "{summaries:?}");
        assert_eq!(summaries[0].code, ErrorCode::MvtTileSizeLimit);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(reearth_flow_diagnostics::Disposition::WarnDrop)
        );
        let aggregated = summaries[0].aggregated.as_ref().expect("aggregated");
        assert_eq!(aggregated.count, expected.len() as u64);
        assert!(aggregated
            .sample_feature_ids
            .iter()
            .all(|id| expected.contains(id)));
    }

    #[test]
    fn the_writer_reports_nothing_when_every_tile_fits() {
        let summaries = write_with_diagnostics(&crowded_features(), u64::MAX);
        assert!(summaries.is_empty(), "{summaries:?}");
    }
}

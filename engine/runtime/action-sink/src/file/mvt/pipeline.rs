//! Disk-backed tileset building. Features are recorded on disk as they arrive;
//! writing slices them into a store of per-tile parts, indexed in memory by
//! tile, then encodes each tile from its parts read back in feature order. Peak
//! memory is the index plus one tile per worker.

use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};

use flate2::{write::ZlibEncoder, Compression};
use flatgeom::{LineString2, MultiLineString as NMultiLineString};
use flatgeom::{MultiPoint as NMultiPoint, MultiPolygon as NMultiPolygon};
use nusamai_citygml::schema::Schema;
use prost::Message;
use rayon::prelude::*;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_types::{Attribute, Feature};
use serde::{de::DeserializeOwned, Serialize};
use tinymvt::geometry::GeometryEncoder;
use tinymvt::tag::TagsEncoder;
use tinymvt::vector_tile;

use super::sink::MVTWriterCompiledParam;
use super::slice::{self, Leaf, SlicedFeature, SlicedGeom, TileKey};
use super::store::{RecordReader, RecordStore};
use super::tags::convert_properties;
use super::tiling::{TileContent, TileMetadata, VectorLayer};
use crate::errors::SinkError;
use crate::file::record::{self, Reader};

/// Tiles over this many zlib-compressed bytes are re-encoded at half the
/// extent, down to [`MIN_EXTENT`].
const MAX_COMPRESSED_TILE_BYTES: usize = 500_000;
const MIN_EXTENT: i32 = 512;

type Result<T> = std::result::Result<T, SinkError>;

fn mvt_err(e: impl std::fmt::Debug) -> SinkError {
    SinkError::MvtWriter(format!("{e:?}"))
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serde::encode_to_vec(value, bincode::config::standard()).map_err(mvt_err)
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .map(|(value, _)| value)
        .map_err(mvt_err)
}

/// The features of one output tileset, kept on disk until it is written.
#[derive(Debug)]
pub(super) struct TilesetBuffer {
    dir: PathBuf,
    /// Per feature: layer name, schema key and attributes. Attributes are
    /// filtered against the schema only when tiles are written, as schema
    /// features may still arrive after them.
    props: RecordStore,
    /// Per feature: its `Vec<Leaf>`.
    geoms: RecordStore,
    content: TileContent,
    layer_names: HashSet<String>,
}

impl TilesetBuffer {
    pub(super) fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
            props: RecordStore::new(dir),
            geoms: RecordStore::new(dir),
            content: TileContent::default(),
            layer_names: HashSet::new(),
        }
    }

    pub(super) fn push(&mut self, feature: &Feature, layer_name: String) -> Result<()> {
        let (content, leaves) = slice::extract(feature)?;
        self.content.union(&content);

        let mut props = Vec::new();
        record::put_str(&mut props, &layer_name);
        match crate::schema::schema_key(feature) {
            Some(key) => {
                record::put_u8(&mut props, 1);
                record::put_str(&mut props, &key);
            }
            None => record::put_u8(&mut props, 0),
        }
        record::encode_attributes(&mut props, &feature.attributes);
        self.props.push(&props).map_err(mvt_err)?;
        self.geoms.push(&encode(&leaves)?).map_err(mvt_err)?;
        self.layer_names.insert(layer_name);
        Ok(())
    }
}

pub(super) fn write_tileset(
    ctx: &Context,
    buffer: &TilesetBuffer,
    output: &Uri,
    compress_output: Option<&Uri>,
    schema: &Schema,
    params: &MVTWriterCompiledParam,
) -> Result<()> {
    let storage = ctx.storage_resolver.resolve(output).map_err(mvt_err)?;
    let (slices, tiles) = slice_features(buffer, params.min_zoom, params.max_zoom)?;

    tiles.into_par_iter().try_for_each_init(
        || (buffer.props.reader(), slices.reader()),
        |readers, ((zoom, x, y), members)| {
            let props = readers.0.as_mut().map_err(|e| mvt_err(&*e))?;
            let slices = readers.1.as_mut().map_err(|e| mvt_err(&*e))?;
            let mut extent = params.extent;
            let bytes = loop {
                let feats = members.iter().map(|&(seq, id)| {
                    read_member(&mut *props, &mut *slices, seq, id, schema, params)
                });
                let bytes = make_tile(extent, feats)?;
                let compressed_size = compressed_len(&bytes)?;
                if compressed_size > MAX_COMPRESSED_TILE_BYTES && extent > MIN_EXTENT {
                    tracing::warn!(
                        "Tile z:{} x:{} y:{} with extent {} is too large ({} bytes), retrying with smaller extent",
                        zoom, x, y, extent, compressed_size
                    );
                    extent /= 2;
                    continue;
                }
                break bytes;
            };
            let path = output
                .join(Path::new(&format!("{zoom}/{x}/{y}.mvt")))
                .map_err(mvt_err)?;
            storage
                .put_sync(&path.path(), bytes::Bytes::from(bytes))
                .map_err(mvt_err)
        },
    )?;

    let tilejson = tilejson(output, buffer, params)?;
    let path = output.join(Path::new("tilejson.json")).map_err(mvt_err)?;
    storage
        .put_sync(&path.path(), bytes::Bytes::from(tilejson))
        .map_err(mvt_err)?;

    if let Some(compress_output) = compress_output {
        let storage = ctx
            .storage_resolver
            .resolve(compress_output)
            .map_err(mvt_err)?;
        let mut cursor = Cursor::new(Vec::new());
        reearth_flow_common::zip::write(BufWriter::new(&mut cursor), output.path().as_path())
            .map_err(mvt_err)?;
        storage
            .put_sync(
                compress_output.path().as_path(),
                bytes::Bytes::from(cursor.into_inner()),
            )
            .map_err(mvt_err)?;
        std::fs::remove_dir_all(output.path().as_path()).map_err(mvt_err)?;
    }
    Ok(())
}

/// Slice every feature of `buffer` into a new store, returning it with each
/// tile's parts as `(feature seq, part id)` in feature order.
#[allow(clippy::type_complexity)]
fn slice_features(
    buffer: &TilesetBuffer,
    min_zoom: u8,
    max_zoom: u8,
) -> Result<(RecordStore, Vec<(TileKey, Vec<(u32, u32)>)>)> {
    let slices = parking_lot::Mutex::new(RecordStore::new(&buffer.dir));
    let tiles = (0..buffer.geoms.len() as u32)
        .into_par_iter()
        .map_init(
            || buffer.geoms.reader(),
            |geoms, seq| {
                let geoms = geoms.as_mut().map_err(|e| mvt_err(&*e))?;
                let leaves: Vec<Leaf> = decode(geoms.read(seq).map_err(mvt_err)?)?;
                let parts = slice::slice(&leaves, min_zoom, max_zoom)
                    .into_iter()
                    .map(|(key, geom)| Ok((key, encode(&geom)?)))
                    .collect::<Result<Vec<_>>>()?;
                // One lock per feature keeps its parts' ids ascending in slice order.
                let mut slices = slices.lock();
                parts
                    .into_iter()
                    .map(|(key, bytes)| Ok((key, seq, slices.push(&bytes).map_err(mvt_err)?)))
                    .collect::<Result<Vec<_>>>()
            },
        )
        .try_fold(
            HashMap::<TileKey, Vec<(u32, u32)>>::new,
            |mut tiles, parts| -> Result<_> {
                for (key, seq, id) in parts? {
                    tiles.entry(key).or_default().push((seq, id));
                }
                Ok(tiles)
            },
        )
        .try_reduce(HashMap::new, |mut a, b| {
            for (key, members) in b {
                a.entry(key).or_default().extend(members);
            }
            Ok(a)
        })?;

    let mut tiles: Vec<_> = tiles.into_iter().collect();
    for (_, members) in &mut tiles {
        members.sort_unstable();
    }
    Ok((slices.into_inner(), tiles))
}

fn read_member(
    props: &mut RecordReader<'_>,
    slices: &mut RecordReader<'_>,
    seq: u32,
    id: u32,
    schema: &Schema,
    params: &MVTWriterCompiledParam,
) -> Result<SlicedFeature> {
    let mut r = Reader::new(props.read(seq).map_err(mvt_err)?);
    let typename = r.string().map_err(mvt_err)?;
    let schema_key = match r.u8().map_err(mvt_err)? {
        0 => None,
        _ => Some(r.string().map_err(mvt_err)?),
    };
    let attributes = record::decode_attributes(&mut r).map_err(mvt_err)?;

    let mut properties = crate::schema::filter_and_cast(&attributes, schema_key.as_deref(), schema);
    if params.skip_unexposed_attributes {
        properties.retain(|k, _| !k.as_ref().starts_with("__"));
    }
    if params.colon_to_underscore {
        properties = properties
            .into_iter()
            .map(|(k, v)| (Attribute::new(k.inner().replace(':', "_")), v))
            .collect();
    }
    Ok(SlicedFeature {
        typename,
        geom: decode(slices.read(id).map_err(mvt_err)?)?,
        properties,
    })
}

fn compressed_len(bytes: &[u8]) -> Result<usize> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes).map_err(mvt_err)?;
    Ok(e.finish().map_err(mvt_err)?.len())
}

fn tilejson(
    output: &Uri,
    buffer: &TilesetBuffer,
    params: &MVTWriterCompiledParam,
) -> Result<String> {
    let basename = output.file_name().map(|s| s.to_string_lossy().to_string());
    if basename.is_none() {
        tracing::warn!("Basename extraction failed from output path: {}", output);
    }
    // Absolute tile path: the parent of tilejson.json is the HTTP root.
    let tiles = vec!["/{z}/{x}/{y}.mvt".to_string()];
    let vector_layers = buffer
        .layer_names
        .iter()
        .map(|id| VectorLayer {
            id: id.clone(),
            fields: HashMap::new(),
        })
        .collect();
    let metadata = TileMetadata::from_tile_content(
        basename,
        params.min_zoom,
        params.max_zoom,
        &buffer.content,
        tiles,
        vector_layers,
    );
    serde_json::to_string_pretty(&metadata).map_err(mvt_err)
}

#[derive(Default)]
struct LayerData {
    features: Vec<vector_tile::tile::Feature>,
    tags_enc: TagsEncoder,
}

fn make_tile(extent: i32, feats: impl Iterator<Item = Result<SlicedFeature>>) -> Result<Vec<u8>> {
    let mut layers: HashMap<String, LayerData> = HashMap::new();
    let mut int_ring_buf = Vec::new();
    let mut int_ring_buf2 = Vec::new();
    let to_int = |[x, y]: [f64; 2]| {
        let x = (x * extent as f64 + 0.5) as i32;
        let y = (y * extent as f64 + 0.5) as i32;
        [x, y]
    };

    for feature in feats {
        let feature = feature?;
        let mut geom_enc = GeometryEncoder::new();
        let geom_type = match &feature.geom {
            SlicedGeom::Polygons(mpoly) => {
                let mut int_mpoly = NMultiPolygon::<[i32; 2]>::new();
                for poly in mpoly {
                    for (ri, ring) in poly.rings().enumerate() {
                        int_ring_buf.clear();
                        int_ring_buf.extend(ring.into_iter().map(to_int));

                        // some simplification
                        int_ring_buf2.clear();
                        int_ring_buf2.push(int_ring_buf[0]);
                        for c in int_ring_buf.windows(3) {
                            let &[prev, curr, next] = c else {
                                unreachable!("windows(3) yields three points")
                            };

                            // Remove duplicate points
                            if prev == curr {
                                continue;
                            }

                            // Skip collinear points (cast to i64 to avoid overflow)
                            if curr != next {
                                let dx1 = (curr[0] - prev[0]) as i64;
                                let dy1 = (curr[1] - prev[1]) as i64;
                                let dx2 = (next[0] - prev[0]) as i64;
                                let dy2 = (next[1] - prev[1]) as i64;
                                if dx1 * dy2 - dy1 * dx2 == 0 {
                                    continue;
                                }
                            }

                            int_ring_buf2.push(curr);
                        }
                        int_ring_buf2.push(*int_ring_buf.last().unwrap());

                        match ri {
                            0 => int_mpoly.add_exterior(int_ring_buf2.drain(..)),
                            _ => int_mpoly.add_interior(int_ring_buf2.drain(..)),
                        }
                    }
                }
                for poly in &int_mpoly {
                    let exterior = poly.exterior();
                    if exterior.signed_ring_area() > 0.0 {
                        geom_enc.add_ring(&exterior);
                        for interior in poly.interiors() {
                            if interior.is_cw() {
                                geom_enc.add_ring(&interior);
                            }
                        }
                    }
                }
                vector_tile::tile::GeomType::Polygon
            }
            SlicedGeom::LineStrings(mline_string) => {
                let mut int_line_string = NMultiLineString::<[i32; 2]>::new();
                for line_string in mline_string {
                    let coords: Vec<_> = line_string.into_iter().map(to_int).collect();
                    int_line_string.add_linestring(&LineString2::from_raw(coords.into()));
                }
                for line_string in &int_line_string {
                    if line_string.len() >= 2 {
                        geom_enc.add_linestring(&line_string);
                    }
                }
                vector_tile::tile::GeomType::Linestring
            }
            SlicedGeom::Points(mpoints) => {
                let mut int_multi_point = NMultiPoint::<[i32; 2]>::new();
                for point in mpoints {
                    int_multi_point.push(to_int(point));
                }
                geom_enc.add_points(&int_multi_point);
                vector_tile::tile::GeomType::Point
            }
        };
        let geometry = geom_enc.into_vec();
        if geometry.is_empty() {
            continue;
        }

        let layer = layers.entry(feature.typename).or_default();
        for (key, value) in &feature.properties {
            convert_properties(&mut layer.tags_enc, &key.inner(), value);
        }
        layer.features.push(vector_tile::tile::Feature {
            id: None,
            tags: layer.tags_enc.take_tags(),
            r#type: Some(geom_type as i32),
            geometry,
        });
    }

    let layers = layers
        .into_iter()
        .filter(|(_, layer_data)| !layer_data.features.is_empty())
        .map(|(name, layer_data)| {
            let (keys, values) = layer_data.tags_enc.into_keys_and_values();
            vector_tile::tile::Layer {
                version: 2,
                name,
                features: layer_data.features,
                keys,
                values,
                extent: Some(extent as u32),
            }
        })
        .collect();

    Ok(vector_tile::Tile { layers }.encode_to_vec())
}

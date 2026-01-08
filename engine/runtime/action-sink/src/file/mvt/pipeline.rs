use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use flate2::{write::ZlibEncoder, Compression};
use flatgeom::LineString2;
use flatgeom::MultiLineString as NMultiLineString;
use flatgeom::MultiLineString2;
use flatgeom::MultiPoint as NMultiPoint;
use flatgeom::MultiPoint2;
use flatgeom::MultiPolygon as NMultiPolygon;
use flatgeom::MultiPolygon2;
use itertools::Itertools;
use prost::Message;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_types::Feature;
use tinymvt::geometry::GeometryEncoder;
use tinymvt::tag::TagsEncoder;
use tinymvt::vector_tile;

use super::slice::slice_cityobj_geoms;
use super::tags::convert_properties;
use super::tileid::TileIdMethod;
use super::tiling::TileContent;
use super::tiling::TileMetadata;
use super::tiling::VectorLayer;

#[allow(clippy::too_many_arguments)]
pub(super) fn geometry_slicing_stage(
    ctx: Context,
    upstream: &[(Feature, String)],
    tile_id_conv: TileIdMethod,
    sender_sliced: std::sync::mpsc::SyncSender<(u64, Vec<u8>)>,
    output_path: &Uri,
    min_zoom: u8,
    max_zoom: u8,
) -> crate::errors::Result<()> {
    let tile_contents = Arc::new(Mutex::new(Vec::new()));
    let layer_names = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let storage = ctx
        .storage_resolver
        .resolve(output_path)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;

    // Convert CityObjects to sliced features
    upstream
        .iter()
        .par_bridge()
        .try_for_each(|(feature, layer_name)| {
            let max_detail = 12; // 4096
            let buffer_pixels = 5;

            // Collect unique layer names
            layer_names
                .lock()
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("Mutex poisoned: {e}")))?
                .insert(layer_name.clone());

            let tile_content = slice_cityobj_geoms(
                feature,
                layer_name,
                min_zoom,
                max_zoom,
                max_detail,
                buffer_pixels,
                |(z, x, y, typename), mpoly| {
                    let feature = super::slice::SlicedFeature {
                        typename,
                        multi_polygons: mpoly,
                        multi_line_strings: MultiLineString2::new(),
                        multi_points: MultiPoint2::new(),
                        properties: feature.attributes.clone(),
                    };
                    let bytes = serde_json::to_vec(&feature).map_err(|err| {
                        crate::errors::SinkError::MvtWriter(format!(
                            "Failed to serialize a sliced feature: {err:?}"
                        ))
                    })?;
                    let tile_id = tile_id_conv.zxy_to_id(z, x, y);
                    if sender_sliced.send((tile_id, bytes)).is_err() {
                        return Err(crate::errors::SinkError::MvtWriter("Canceled".to_string()));
                    };
                    Ok(())
                },
                |(z, x, y, typename), line_strings| {
                    let feature = super::slice::SlicedFeature {
                        typename,
                        multi_polygons: MultiPolygon2::new(),
                        multi_line_strings: line_strings,
                        multi_points: MultiPoint2::new(),
                        properties: feature.attributes.clone(),
                    };
                    let bytes = serde_json::to_vec(&feature).map_err(|err| {
                        crate::errors::SinkError::MvtWriter(format!(
                            "Failed to serialize a sliced feature: {err:?}"
                        ))
                    })?;
                    let tile_id = tile_id_conv.zxy_to_id(z, x, y);
                    if sender_sliced.send((tile_id, bytes)).is_err() {
                        return Err(crate::errors::SinkError::MvtWriter("Canceled".to_string()));
                    };
                    Ok(())
                },
                |(z, x, y, typename), points| {
                    let feature = super::slice::SlicedFeature {
                        typename,
                        multi_polygons: MultiPolygon2::new(),
                        multi_line_strings: MultiLineString2::new(),
                        multi_points: points,
                        properties: feature.attributes.clone(),
                    };
                    let bytes = serde_json::to_vec(&feature).map_err(|err| {
                        crate::errors::SinkError::MvtWriter(format!(
                            "Failed to serialize a sliced feature: {err:?}"
                        ))
                    })?;
                    let tile_id = tile_id_conv.zxy_to_id(z, x, y);
                    if sender_sliced.send((tile_id, bytes)).is_err() {
                        return Err(crate::errors::SinkError::MvtWriter("Canceled".to_string()));
                    };
                    Ok(())
                },
            )?;
            tile_contents
                .lock()
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("Mutex poisoned: {e}")))?
                .push(tile_content);
            Ok::<(), crate::errors::SinkError>(())
        })?;

    let mut tile_content = TileContent::default();
    for content in tile_contents
        .lock()
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("Mutex poisoned: {e}")))?
        .iter()
    {
        tile_content.min_lng = tile_content.min_lng.min(content.min_lng);
        tile_content.max_lng = tile_content.max_lng.max(content.max_lng);
        tile_content.min_lat = tile_content.min_lat.min(content.min_lat);
        tile_content.max_lat = tile_content.max_lat.max(content.max_lat);
    }

    // Get tileset name from output path
    let basename = output_path
        .file_name()
        .ok_or_else(|| {
            crate::errors::SinkError::MvtWriter(format!(
                "Failed to get tileset name from output path {:?}",
                output_path
            ))
        })?
        .to_str()
        .ok_or_else(|| {
            crate::errors::SinkError::MvtWriter(format!(
                "Failed to parse output path basename as UTF-8: {:?}",
                output_path
            ))
        })?;

    // Construct absolute path for tiles (parent of tilejson.json is root)
    let tiles = vec!["/{{z}}/{{x}}/{{y}}.mvt".to_string()];
    let vector_layers: Vec<_> = layer_names
        .lock()
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("Mutex poisoned: {e}")))?
        .iter()
        .map(|id| VectorLayer {
            id: id.clone(),
            fields: HashMap::new(),
        })
        .collect();
    let metadata = TileMetadata::from_tile_content(
        basename.to_string(),
        min_zoom,
        max_zoom,
        &TileContent {
            min_lng: tile_content.min_lng,
            max_lng: tile_content.max_lng,
            min_lat: tile_content.min_lat,
            max_lat: tile_content.max_lat,
        },
        tiles,
        vector_layers,
    );

    serde_json::to_string_pretty(&metadata)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
        .and_then(|metadata| {
            storage
                .put_sync(
                    &output_path
                        .join(Path::new("tilejson.json"))
                        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?
                        .path(),
                    bytes::Bytes::from(metadata),
                )
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))
        })?;

    Ok(())
}

pub(super) fn feature_sorting_stage(
    receiver_sliced: std::sync::mpsc::Receiver<(u64, Vec<u8>)>,
    sender_sorted: std::sync::mpsc::SyncSender<(u64, Vec<Vec<u8>>)>,
) -> crate::errors::Result<()> {
    let config = kv_extsort::SortConfig::default().max_chunk_bytes(256 * 1024 * 1024);

    let sorted_iter = kv_extsort::sort(
        receiver_sliced
            .into_iter()
            .map(|(tile_id, body)| std::result::Result::<_, Infallible>::Ok((tile_id, body))),
        config,
    );

    for ((_, tile_id), grouped) in &sorted_iter.chunk_by(|feat| match feat {
        Ok((tile_id, _)) => (false, *tile_id),
        Err(_) => (true, 0),
    }) {
        let grouped = grouped
            .into_iter()
            .map_ok(|(_, serialized_feats)| serialized_feats)
            .collect::<kv_extsort::Result<Vec<_>, _>>();
        match grouped {
            Ok(serialized_feats) => {
                if sender_sorted.send((tile_id, serialized_feats)).is_err() {
                    return Err(crate::errors::SinkError::MvtWriter("Canceled".to_string()));
                }
            }
            Err(kv_extsort::Error::Canceled) => {
                return Err(crate::errors::SinkError::MvtWriter("Canceled".to_string()));
            }
            Err(err) => {
                return Err(crate::errors::SinkError::MvtWriter(format!(
                    "Failed to sort features: {err:?}"
                )));
            }
        }
    }

    Ok(())
}

#[derive(Default)]
pub(super) struct LayerData {
    pub(super) features: Vec<vector_tile::tile::Feature>,
    pub(super) tags_enc: TagsEncoder,
}

pub(super) fn tile_writing_stage(
    ctx: Context,
    output_path: &Uri,
    receiver_sorted: std::sync::mpsc::Receiver<(u64, Vec<Vec<u8>>)>,
    tile_id_conv: TileIdMethod,
    skip_unexposed_attributes: bool,
    colon_to_underscore: bool,
    default_extent: i32,
) -> crate::errors::Result<()> {
    let min_extent: i32 = 512;

    let storage = ctx
        .storage_resolver
        .resolve(output_path)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;

    receiver_sorted
        .into_iter()
        .par_bridge()
        .try_for_each(|(tile_id, serialized_feats)| {
            let (zoom, x, y) = tile_id_conv.id_to_zxy(tile_id);

            let path = output_path
                .join(Path::new(&format!("{zoom}/{x}/{y}.mvt")))
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
            let mut extent = default_extent;
            while extent >= min_extent {
                let bytes = make_tile(
                    extent,
                    &serialized_feats,
                    skip_unexposed_attributes,
                    colon_to_underscore,
                )?;
                let compressed_size = {
                    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                    e.write_all(&bytes)
                        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
                    let compressed_bytes = e
                        .finish()
                        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
                    compressed_bytes.len()
                };
                if compressed_size > 500_000 && extent > min_extent {
                    tracing::warn!(
                        "Tile z:{} x:{} y:{} with extent {} is too large ({} bytes), retrying with smaller extent",
                        zoom, x, y, extent, compressed_size
                    );
                    extent /= 2;
                    continue;
                }
                storage
                    .put_sync(&path.path(), bytes::Bytes::from(bytes))
                    .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{e:?}")))?;
                break;
            }
            Ok::<(), crate::errors::SinkError>(())
        })?;
    Ok(())
}

pub(super) fn make_tile(
    extent: i32,
    serialized_feats: &[Vec<u8>],
    skip_unexposed_attributes: bool,
    colon_to_underscore: bool,
) -> crate::errors::Result<Vec<u8>> {
    let mut layers: HashMap<String, LayerData> = HashMap::new();
    let mut int_ring_buf = Vec::new();
    let mut int_ring_buf2 = Vec::new();

    for serialized_feat in serialized_feats {
        let feature: super::slice::SlicedFeature = serde_json::from_slice(serialized_feat)
            .map_err(|err| {
                crate::errors::SinkError::MvtWriter(format!(
                    "Failed to deserialize a sliced feature: {err:?}"
                ))
            })?;

        let mpoly = feature.multi_polygons;
        let mut int_mpoly = NMultiPolygon::<[i32; 2]>::new();

        for poly in &mpoly {
            for (ri, ring) in poly.rings().enumerate() {
                int_ring_buf.clear();
                int_ring_buf.extend(ring.into_iter().map(|[x, y]| {
                    let x = (x * extent as f64 + 0.5) as i32;
                    let y = (y * extent as f64 + 0.5) as i32;
                    [x, y]
                }));

                // some simplification
                {
                    int_ring_buf2.clear();
                    int_ring_buf2.push(int_ring_buf[0]);
                    for c in int_ring_buf.windows(3) {
                        let &[prev, curr, next] = c.try_into().map_err(|_| {
                            crate::errors::SinkError::MvtWriter("Failed to convert".to_string())
                        })?;

                        // Remove duplicate points
                        if prev == curr {
                            continue;
                        }

                        // Skip collinear points (cast to i64 to avoid overflow)
                        let [curr_x, curr_y] = curr;
                        let [prev_x, prev_y] = prev;
                        let [next_x, next_y] = next;
                        if curr != next {
                            let dx1 = (curr_x - prev_x) as i64;
                            let dy1 = (curr_y - prev_y) as i64;
                            let dx2 = (next_x - prev_x) as i64;
                            let dy2 = (next_y - prev_y) as i64;
                            let cross_product = dx1 * dy2 - dy1 * dx2;
                            if cross_product == 0 {
                                continue;
                            }
                        }

                        int_ring_buf2.push(curr);
                    }
                    int_ring_buf2.push(*int_ring_buf.last().ok_or(
                        crate::errors::SinkError::MvtWriter("Failed to get last".to_string()),
                    )?);
                }

                match ri {
                    0 => int_mpoly.add_exterior(int_ring_buf2.drain(..)),
                    _ => int_mpoly.add_interior(int_ring_buf2.drain(..)),
                }
            }
        }

        let mut int_line_string = NMultiLineString::<[i32; 2]>::new();
        let mline_string = feature.multi_line_strings;

        let mut int_line_string_buf = Vec::new();
        for line_string in &mline_string {
            int_line_string_buf.clear();
            int_line_string_buf.extend(line_string.into_iter().map(|[x, y]| {
                let x = (x * extent as f64 + 0.5) as i32;
                let y = (y * extent as f64 + 0.5) as i32;
                [x, y]
            }));
            int_line_string.add_linestring(&LineString2::from_raw(
                int_line_string_buf.drain(..).collect(),
            ));
        }

        let mut int_multi_point = NMultiPoint::<[i32; 2]>::new();
        let mpoints = feature.multi_points;

        for point in &mpoints {
            let [x, y] = point;
            let x = (x * extent as f64 + 0.5) as i32;
            let y = (y * extent as f64 + 0.5) as i32;
            int_multi_point.push([x, y]);
        }

        // encode geometry
        let mut geom_enc = GeometryEncoder::new();
        let has_polygons = !int_mpoly.is_empty();
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

        let has_linestrings = !int_line_string.is_empty();
        for line_string in &int_line_string {
            if line_string.len() >= 2 {
                geom_enc.add_linestring(&line_string);
            }
        }

        let has_points = !int_multi_point.is_empty();
        if has_points {
            geom_enc.add_points(&int_multi_point);
        }
        let geometry = geom_enc.into_vec();
        if geometry.is_empty() {
            continue;
        }

        let layer = {
            let layer = layers.entry(feature.typename).or_default();

            // Encode attributes as MVT tags
            for (key, value) in &feature.properties {
                // skip keys starting with "__"
                if skip_unexposed_attributes && key.as_ref().starts_with("__") {
                    continue;
                }
                let key_string = if colon_to_underscore {
                    key.inner().replace(":", "_")
                } else {
                    key.inner().to_string()
                };
                convert_properties(&mut layer.tags_enc, &key_string, value);
            }
            layer
        };

        // MVT Feature only supports one geometry type per feature.
        // Priority: Polygons > LineStrings > Points
        let geometry_count = has_polygons as u8 + has_linestrings as u8 + has_points as u8;
        if geometry_count > 1 {
            tracing::warn!("Feature has mixed geometry types, some geometries will be skipped.");
        }

        if has_polygons {
            layer.features.push(vector_tile::tile::Feature {
                id: None,
                tags: layer.tags_enc.take_tags(),
                r#type: Some(vector_tile::tile::GeomType::Polygon as i32),
                geometry,
            });
        } else if has_linestrings {
            layer.features.push(vector_tile::tile::Feature {
                id: None,
                tags: layer.tags_enc.take_tags(),
                r#type: Some(vector_tile::tile::GeomType::Linestring as i32),
                geometry,
            });
        } else if has_points {
            layer.features.push(vector_tile::tile::Feature {
                id: None,
                tags: layer.tags_enc.take_tags(),
                r#type: Some(vector_tile::tile::GeomType::Point as i32),
                geometry,
            });
        } else {
            tracing::warn!("Feature has no valid geometry, skipping.");
        }
    }

    let layers = layers
        .into_iter()
        .flat_map(|(name, layer_data)| {
            if layer_data.features.is_empty() {
                return None;
            }
            let (keys, values) = layer_data.tags_enc.into_keys_and_values();
            Some(vector_tile::tile::Layer {
                version: 2,
                name: name.to_string(),
                features: layer_data.features,
                keys,
                values,
                extent: Some(extent as u32),
            })
        })
        .collect();

    let tile = vector_tile::Tile { layers };

    let bytes = tile.encode_to_vec();
    Ok(bytes)
}

use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec;

use flate2::{write::ZlibEncoder, Compression};
use flatgeom::LineString2;
use flatgeom::MultiLineString as NMultiLineString;
use flatgeom::MultiLineString2;
use flatgeom::MultiPolygon as NMultiPolygon;
use flatgeom::MultiPolygon2;
use itertools::Itertools;
use prost::Message;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::Event;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::geometry as geometry_types;
use reearth_flow_types::Expr;
use reearth_flow_types::Feature;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tinymvt::geometry::GeometryEncoder;
use tinymvt::tag::TagsEncoder;
use tinymvt::vector_tile;

use super::slice::slice_cityobj_geoms;
use super::tags::convert_properties;
use super::tileid::TileIdMethod;
use super::tiling::TileContent;
use super::tiling::TileMetadata;
use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub struct MVTSinkFactory;

impl SinkFactory for MVTSinkFactory {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn description(&self) -> &str {
        "Writes features to a file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MVTWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, JsonValue>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: MVTWriterParam = if let Some(with) = with.clone() {
            let value: JsonValue = serde_json::to_value(with).map_err(|e| {
                SinkError::MvtWriterFactory(format!("Failed to serialize `with` parameter: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::MvtWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(SinkError::MvtWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let expr_output = &params.output;
        let output = expr_engine
            .compile(expr_output.as_ref())
            .map_err(|e| SinkError::MvtWriterFactory(format!("{:?}", e)))?;
        let expr_layer_name = &params.layer_name;
        let layer_name = expr_engine
            .compile(expr_layer_name.as_ref())
            .map_err(|e| SinkError::MvtWriterFactory(format!("{:?}", e)))?;

        let sink = MVTWriter {
            global_params: with,
            buffer: HashMap::new(),
            params: MVTWriterCompiledParam {
                output,
                layer_name,
                min_zoom: params.min_zoom,
                max_zoom: params.max_zoom,
            },
        };
        Ok(Box::new(sink))
    }
}

type BufferKey = (Uri, String);

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) global_params: Option<HashMap<String, serde_json::Value>>,
    pub(super) params: MVTWriterCompiledParam,
    pub(super) buffer: HashMap<BufferKey, Vec<Feature>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MVTWriterParam {
    pub(super) output: Expr,
    pub(super) layer_name: Expr,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
}

#[derive(Debug, Clone)]
pub struct MVTWriterCompiledParam {
    pub(super) output: rhai::AST,
    pub(super) layer_name: rhai::AST,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
}

impl Sink for MVTWriter {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let geometry = &ctx.feature.geometry;
        if geometry.is_empty() {
            return Err(Box::new(SinkError::MvtWriter(
                "Unsupported input".to_string(),
            )));
        };

        let feature = ctx.feature;
        match feature.geometry.value {
            geometry_types::GeometryValue::CityGmlGeometry(_) => {
                let output = self.params.output.clone();
                let scope = feature.new_scope(ctx.expr_engine.clone(), &self.global_params);
                let path = scope
                    .eval_ast::<String>(&output)
                    .map_err(|e| SinkError::MvtWriter(format!("{:?}", e)))?;
                let output = Uri::from_str(path.as_str())?;
                let layer_name = scope
                    .eval_ast::<String>(&self.params.layer_name)
                    .map_err(|e| SinkError::MvtWriter(format!("{:?}", e)))?;
                let buffer = self.buffer.entry((output, layer_name)).or_default();
                buffer.push(feature);
            }
            _ => {
                return Err(Box::new(SinkError::MvtWriter(
                    "Unsupported input".to_string(),
                )));
            }
        }

        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        for ((output, layer_name), buffer) in &self.buffer {
            self.write(ctx.as_context(), buffer, output, layer_name)?;
        }
        Ok(())
    }
}

impl MVTWriter {
    pub fn write(
        &self,
        ctx: Context,
        upstream: &[Feature],
        output: &Uri,
        layer_name: &str,
    ) -> crate::errors::Result<()> {
        let tile_id_conv = TileIdMethod::Hilbert;
        let name = self.name().to_string();
        std::thread::scope(|scope| {
            let (sender_sliced, receiver_sliced) = std::sync::mpsc::sync_channel(2000);
            let (sender_sorted, receiver_sorted) = std::sync::mpsc::sync_channel(2000);
            scope.spawn(|| {
                let result = geometry_slicing_stage(
                    ctx.clone(),
                    upstream,
                    tile_id_conv,
                    sender_sliced,
                    output,
                    layer_name,
                    self.params.min_zoom,
                    self.params.max_zoom,
                );
                if let Err(err) = result {
                    ctx.event_hub.error_log(
                        None,
                        format!("Failed to geometry_slicing_stage with error =  {:?}", err),
                    );
                    ctx.event_hub
                        .send(Event::SinkFinishFailed { name: name.clone() });
                }
            });
            scope.spawn(|| {
                let result = feature_sorting_stage(receiver_sliced, sender_sorted);
                if let Err(err) = result {
                    ctx.event_hub.error_log(
                        None,
                        format!("Failed to feature_sorting_stage with error =  {:?}", err),
                    );
                    ctx.event_hub
                        .send(Event::SinkFinishFailed { name: name.clone() });
                }
            });
            scope.spawn(|| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .use_current_thread()
                    .build()
                    .unwrap();
                pool.install(|| {
                    let result =
                        tile_writing_stage(ctx.clone(), output, receiver_sorted, tile_id_conv);
                    if let Err(err) = result {
                        ctx.event_hub.error_log(
                            None,
                            format!("Failed to tile_writing_stage with error =  {:?}", err),
                        );
                        ctx.event_hub
                            .send(Event::SinkFinishFailed { name: name.clone() });
                    }
                })
            });
        });
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn geometry_slicing_stage(
    ctx: Context,
    upstream: &[Feature],
    tile_id_conv: TileIdMethod,
    sender_sliced: std::sync::mpsc::SyncSender<(u64, Vec<u8>)>,
    output_path: &Uri,
    layer_name: &str,
    min_zoom: u8,
    max_zoom: u8,
) -> crate::errors::Result<()> {
    let bincode_config = bincode::config::standard();
    let tile_contents = Arc::new(Mutex::new(Vec::new()));
    let storage = ctx
        .storage_resolver
        .resolve(output_path)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?;

    // Convert CityObjects to sliced features
    upstream.iter().par_bridge().try_for_each(|feature| {
        let max_detail = 12; // 4096
        let buffer_pixels = 5;
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
                    properties: feature.attributes.clone(),
                };
                let bytes =
                    bincode::serde::encode_to_vec(&feature, bincode_config).map_err(|err| {
                        crate::errors::SinkError::MvtWriter(format!(
                            "Failed to serialize a sliced feature: {:?}",
                            err
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
                    properties: feature.attributes.clone(),
                };
                let bytes =
                    bincode::serde::encode_to_vec(&feature, bincode_config).map_err(|err| {
                        crate::errors::SinkError::MvtWriter(format!(
                            "Failed to serialize a sliced feature: {:?}",
                            err
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
            .map_err(|e| crate::errors::SinkError::MvtWriter(format!("Mutex poisoned: {}", e)))?
            .push(tile_content);
        Ok(())
    })?;

    let mut tile_content = TileContent::default();
    for content in tile_contents
        .lock()
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("Mutex poisoned: {}", e)))?
        .iter()
    {
        tile_content.min_lng = tile_content.min_lng.min(content.min_lng);
        tile_content.max_lng = tile_content.max_lng.max(content.max_lng);
        tile_content.min_lat = tile_content.min_lat.min(content.min_lat);
        tile_content.max_lat = tile_content.max_lat.max(content.max_lat);
    }
    let metadata = TileMetadata::from_tile_content(
        layer_name.to_string(),
        min_zoom,
        max_zoom,
        &TileContent {
            min_lng: tile_content.min_lng,
            max_lng: tile_content.max_lng,
            min_lat: tile_content.min_lat,
            max_lat: tile_content.max_lat,
        },
    );

    serde_json::to_string_pretty(&metadata)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))
        .and_then(|metadata| {
            storage
                .put_sync(
                    &output_path
                        .join(Path::new("metadata.json"))
                        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?
                        .path(),
                    bytes::Bytes::from(metadata),
                )
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))
        })?;

    Ok(())
}

fn feature_sorting_stage(
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
                    "Failed to sort features: {:?}",
                    err
                )));
            }
        }
    }

    Ok(())
}

#[derive(Default)]
struct LayerData {
    pub features: Vec<vector_tile::tile::Feature>,
    pub tags_enc: TagsEncoder,
}

fn tile_writing_stage(
    ctx: Context,
    output_path: &Uri,
    receiver_sorted: std::sync::mpsc::Receiver<(u64, Vec<Vec<u8>>)>,
    tile_id_conv: TileIdMethod,
) -> crate::errors::Result<()> {
    let default_detail = 12;
    let min_detail = 9;

    let storage = ctx
        .storage_resolver
        .resolve(output_path)
        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?;

    receiver_sorted
        .into_iter()
        .par_bridge()
        .try_for_each(|(tile_id, serialized_feats)| {
            let (zoom, x, y) = tile_id_conv.id_to_zxy(tile_id);

            let path = output_path
                .join(Path::new(&format!("{zoom}/{x}/{y}.pbf")))
                .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?;
            for detail in (min_detail..=default_detail).rev() {
                // Make a MVT tile binary
                let bytes = make_tile(detail, &serialized_feats)?;

                // Retry with a lower detail level if the compressed tile size is too large
                let compressed_size = {
                    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                    e.write_all(&bytes)
                        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?;
                    let compressed_bytes = e
                        .finish()
                        .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?;
                    compressed_bytes.len()
                };
                if detail != min_detail && compressed_size > 500_000 {
                    // If the tile is too large, try a lower detail level
                    continue;
                }
                storage
                    .put_sync(&path.path(), bytes::Bytes::from(bytes))
                    .map_err(|e| crate::errors::SinkError::MvtWriter(format!("{:?}", e)))?;
                break;
            }
            Ok::<(), crate::errors::SinkError>(())
        })?;
    Ok(())
}

fn make_tile(default_detail: i32, serialized_feats: &[Vec<u8>]) -> crate::errors::Result<Vec<u8>> {
    let mut layers: HashMap<String, LayerData> = HashMap::new();
    let mut int_ring_buf = Vec::new();
    let mut int_ring_buf2 = Vec::new();
    let extent = 1 << default_detail;
    let bincode_config = bincode::config::standard();

    for serialized_feat in serialized_feats {
        let (feature, _): (super::slice::SlicedFeature, _) =
            bincode::serde::decode_from_slice(serialized_feat, bincode_config).map_err(|err| {
                crate::errors::SinkError::MvtWriter(format!(
                    "Failed to deserialize a sliced feature: {:?}",
                    err
                ))
            })?;

        let mpoly = feature.multi_polygons;
        let mut int_mpoly = NMultiPolygon::<[i16; 2]>::new();

        for poly in &mpoly {
            for (ri, ring) in poly.rings().enumerate() {
                int_ring_buf.clear();
                int_ring_buf.extend(ring.into_iter().map(|[x, y]| {
                    let x = (x * extent as f64 + 0.5) as i16;
                    let y = (y * extent as f64 + 0.5) as i16;
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

                        // Reject collinear points
                        let [curr_x, curr_y] = curr;
                        let [prev_x, prev_y] = prev;
                        let [next_x, next_y] = next;
                        if curr != next
                            && ((next_y - prev_y) as i32 * (curr_x - prev_x) as i32).abs()
                                == ((curr_y - prev_y) as i32 * (next_x - prev_x) as i32).abs()
                        {
                            continue;
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

        let mut int_line_string = NMultiLineString::<[i16; 2]>::new();
        let mline_string = feature.multi_line_strings;

        let mut int_line_string_buf = Vec::new();
        for line_string in &mline_string {
            int_line_string_buf.clear();
            int_line_string_buf.extend(line_string.into_iter().map(|[x, y]| {
                let x = (x * extent as f64 + 0.5) as i16;
                let y = (y * extent as f64 + 0.5) as i16;
                [x, y]
            }));
            int_line_string.add_linestring(&LineString2::from_raw(
                int_line_string_buf.drain(..).collect(),
            ));
        }

        // encode geometry
        let mut geom_enc = GeometryEncoder::new();
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

        for line_string in &int_line_string {
            let area = line_string.signed_ring_area();
            if area as i32 != 0 {
                geom_enc.add_linestring(&line_string);
            }
        }
        let geometry = geom_enc.into_vec();
        if geometry.is_empty() {
            continue;
        }

        let layer = {
            let layer = layers.entry(feature.typename).or_default();

            // Encode attributes as MVT tags
            for (key, value) in &feature.properties {
                convert_properties(&mut layer.tags_enc, key.as_ref(), value);
            }
            layer
        };

        layer.features.push(vector_tile::tile::Feature {
            id: None,
            tags: layer.tags_enc.take_tags(),
            r#type: Some(vector_tile::tile::GeomType::Polygon as i32),
            geometry,
        });
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
                extent: Some(extent),
            })
        })
        .collect();

    let tile = vector_tile::Tile { layers };

    let bytes = tile.encode_to_vec();
    Ok(bytes)
}

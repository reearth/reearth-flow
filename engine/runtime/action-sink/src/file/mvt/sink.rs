use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use flate2::{write::ZlibEncoder, Compression};
use flatgeom::MultiPolygon as NMultiPolygon;
use itertools::Itertools;
use nusamai_mvt::geometry::GeometryEncoder;
use nusamai_mvt::tag::TagsEncoder;
use nusamai_mvt::tileid::TileIdMethod;
use nusamai_mvt::vector_tile;
use prost::Message;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
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

use super::slice::slice_cityobj_geoms;
use super::tags::convert_properties;
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
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, JsonValue>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params = if let Some(with) = with {
            let value: JsonValue = serde_json::to_value(with).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to serialize `with` parameter: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to deserialize `with` parameter: {}", e))
            })?
        } else {
            return Err(
                SinkError::BuildFactory("Missing required parameter `with`".to_string()).into(),
            );
        };

        let sink = MVTWriter {
            buffer: Vec::new(),
            params,
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) params: MVTWriterParam,
    pub(super) buffer: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MVTWriterCommonParam {
    pub(super) output: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MVTWriterParam {
    pub(super) output: Expr,
    pub(super) min_zoom: u8,
    pub(super) max_zoom: u8,
}

impl Sink for MVTWriter {
    fn name(&self) -> &str {
        "MVTWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let Some(geometry) = ctx.feature.geometry.as_ref() else {
            return Err(Box::new(SinkError::FileWriter(
                "Unsupported input".to_string(),
            )));
        };
        let geometry_value = geometry.value.clone();
        match geometry_value {
            geometry_types::GeometryValue::CityGmlGeometry(_) => {
                self.buffer.push(ctx.feature.clone());
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
        let upstream = &self.buffer;
        let tile_id_conv = TileIdMethod::Hilbert;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;

        std::thread::scope(|scope| {
            let (sender_sliced, receiver_sliced) = std::sync::mpsc::sync_channel(2000);
            let (sender_sorted, receiver_sorted) = std::sync::mpsc::sync_channel(2000);
            scope.spawn(|| {
                let _ = geometry_slicing_stage(upstream, tile_id_conv, sender_sliced, &self.params);
            });
            scope.spawn(|| {
                let _ = feature_sorting_stage(receiver_sliced, sender_sorted);
            });
            scope.spawn(|| {
                let pool = rayon::ThreadPoolBuilder::new()
                    .use_current_thread()
                    .build()
                    .unwrap();
                pool.install(|| {
                    let _ = tile_writing_stage(
                        ctx.as_context(),
                        &output,
                        receiver_sorted,
                        tile_id_conv,
                    );
                })
            });
        });
        Ok(())
    }
}

fn geometry_slicing_stage(
    upstream: &[Feature],
    tile_id_conv: TileIdMethod,
    sender_sliced: std::sync::mpsc::SyncSender<(u64, Vec<u8>)>,
    mvt_options: &MVTWriterParam,
) -> crate::errors::Result<()> {
    let bincode_config = bincode::config::standard();

    // Convert CityObjects to sliced features
    upstream.iter().par_bridge().try_for_each(|feature| {
        let max_detail = 12; // 4096
        let buffer_pixels = 5;
        slice_cityobj_geoms(
            feature,
            mvt_options.min_zoom,
            mvt_options.max_zoom,
            max_detail,
            buffer_pixels,
            |(z, x, y, typename), mpoly| {
                let feature = super::slice::SlicedFeature {
                    typename,
                    geometry: mpoly,
                    properties: feature.attributes.clone(),
                };
                let bytes = bincode::serde::encode_to_vec(&feature, bincode_config).unwrap();
                let tile_id = tile_id_conv.zxy_to_id(z, x, y);
                if sender_sliced.send((tile_id, bytes)).is_err() {
                    return Err(crate::errors::SinkError::MvtWriter("Canceled".to_string()));
                };
                Ok(())
            },
        )
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

        let mpoly = feature.geometry;
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
                        let &[prev, curr, next] = c.try_into().unwrap();

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
                    int_ring_buf2.push(*int_ring_buf.last().unwrap());
                }

                match ri {
                    0 => int_mpoly.add_exterior(int_ring_buf2.drain(..)),
                    _ => int_mpoly.add_interior(int_ring_buf2.drain(..)),
                }
            }
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
        let geometry = geom_enc.into_vec();
        if geometry.is_empty() {
            continue;
        }

        let mut tags: Vec<u32> = Vec::new();

        let layer = {
            let layer = layers.entry(feature.typename).or_default();

            // Encode attributes as MVT tags
            for (key, value) in &feature.properties {
                convert_properties(&mut tags, &mut layer.tags_enc, key.as_ref(), value);
            }
            layer
        };

        layer.features.push(vector_tile::tile::Feature {
            id: None,
            tags,
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

use super::tree::TileContent;
use super::vector_tile::tile;
use crate::errors::SinkError;
use ahash::RandomState;
use flate2::{write::ZlibEncoder, Compression};
use hashbrown::HashMap as BrownHashMap;
use indexmap::IndexSet;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::polygon::Polygon;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry as geomotry_types;
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec;

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
            params,
            contents: Arc::new(Mutex::new(Vec::new())),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) params: MVTWriterParam,
    pub(super) contents: Arc<Mutex<Vec<TileContent>>>,
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
    fn initialize(&self, _ctx: NodeContext) {}
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let geometry = ctx.feature.geometry.as_ref().unwrap();
        let geometry_value = geometry.value.clone();
        match geometry_value {
            geomotry_types::GeometryValue::None => {
                return Err(Box::new(SinkError::FileWriter(
                    "Unsupported input".to_string(),
                )));
            }
            geomotry_types::GeometryValue::CityGmlGeometry(city_gml) => {
                let storage_resolver = Arc::clone(&ctx.storage_resolver);
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let output = self.params.output.clone();
                let scope = expr_engine.new_scope();
                let path = scope
                    .eval::<String>(output.as_ref())
                    .unwrap_or_else(|_| output.as_ref().to_string());
                let output = Uri::from_str(path.as_str())?;
                let contents = match handle_city_gml_geometry(
                    &output,
                    storage_resolver.clone(),
                    city_gml,
                    self.params.min_zoom,
                    self.params.max_zoom,
                ) {
                    Ok(contents) => contents,
                    Err(e) => {
                        return Err(Box::new(SinkError::FileWriter(format!(
                            "CityGmlGeometry handle Error: {:?}",
                            e
                        ))))
                    }
                };
                self.contents
                    .lock()
                    .unwrap()
                    .extend(contents.lock().unwrap().iter().cloned());
            }
            geomotry_types::GeometryValue::FlowGeometry2D(_flow_geom_2d) => {
                return Err(Box::new(SinkError::FileWriter(
                    "Unsupported input".to_string(),
                )));
            }
            geomotry_types::GeometryValue::FlowGeometry3D(_flow_geom_3d) => {
                return Err(Box::new(SinkError::FileWriter(
                    "Unsupported input".to_string(),
                )));
            }
        }

        Ok(())
    }
    fn finish(&self, _ctx: NodeContext) -> Result<(), BoxedError> {
        Ok(())
    }
}

fn handle_city_gml_geometry(
    output: &Uri,
    storage_resolver: Arc<StorageResolver>,
    city_gml: geomotry_types::CityGmlGeometry,
    min_zoom: u8,
    max_zoom: u8,
) -> Result<Arc<Mutex<std::vec::Vec<TileContent>>>, crate::errors::SinkError> {
    let contents: Arc<Mutex<Vec<TileContent>>> = Default::default();

    let features = city_gml.features.clone();
    for feature in features {
        match handle_feature(
            output,
            storage_resolver.clone(),
            feature,
            min_zoom,
            max_zoom,
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(crate::errors::SinkError::file_writer(format!(
                    "Feature handle Error: {:?}",
                    e
                )))
            }
        }
    }
    Ok(contents)
}

fn handle_feature(
    output: &Uri,
    storage_resolver: Arc<StorageResolver>,
    feature: geomotry_types::GeometryFeature,
    min_zoom: u8,
    max_zoom: u8,
) -> Result<(), crate::errors::SinkError> {
    // let min_zoom = min_zoom.unwrap_or(12);
    // let max_zoom = max_zoom.unwrap_or(18);

    let default_detail = 12;
    let min_detail = 9;

    for zoom in min_zoom..=max_zoom {
        let xi: i32 = 0; // TODO
        let yi: i32 = 0; // TODO

        let (zoom, x, y) = (
            zoom,
            xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
            yi,
        );

        let mpoly = feature.polygons.clone();

        let feature = SlicedFeature {
            geometry: mpoly,
            properties: nusamai_citygml::Value::String(
                feature.id.clone().unwrap_or("".to_string()),
            ), // TODO
        };

        let bincode_config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(&feature, bincode_config).unwrap();

        let serialized_feats = [bytes];

        let storage = storage_resolver
            .resolve(output)
            .map_err(crate::errors::SinkError::file_writer)?;
        let output_path = output
            .path()
            .join(Path::new(&format!("{zoom}/{x}/{y}.pbf")));
        if let Some(dir) = output_path.parent() {
            fs::create_dir_all(dir).map_err(crate::errors::SinkError::file_writer)?;
        }
        let path = Path::new(&output_path);

        for detail in (min_detail..=default_detail).rev() {
            // Make a MVT tile binary
            let bytes = make_tile(detail, &serialized_feats)?;

            // Retry with a lower detail level if the compressed tile size is too large
            let compressed_size = {
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&bytes)
                    .map_err(crate::errors::SinkError::file_writer)?;
                let compressed_bytes = e.finish().map_err(crate::errors::SinkError::file_writer)?;
                compressed_bytes.len()
            };
            if detail != min_detail && compressed_size > 500_000 {
                // If the tile is too large, try a lower detail level
                continue;
            }
            storage
                .put_sync(path, bytes::Bytes::from(bytes))
                .map_err(crate::errors::SinkError::file_writer)?;
            break;
        }
    }
    Ok(())
}

fn make_tile(default_detail: i32, serialized_feats: &[Vec<u8>]) -> Result<Vec<u8>, SinkError> {
    let mut layers: BrownHashMap<String, LayerData> = BrownHashMap::new();
    // let mut int_ring_buf = Vec::new();
    // let mut int_ring_buf2 = Vec::new();
    let extent = 1 << default_detail;
    let bincode_config = bincode::config::standard();

    for serialized_feat in serialized_feats {
        let (feature, _): (SlicedFeature, _) =
            bincode::serde::decode_from_slice(serialized_feat, bincode_config).map_err(|err| {
                SinkError::FileWriter(format!("Failed to deserialize a sliced feature: {:?}", err))
            })?;

        // let mpoly = feature.geometry;
        // let mut int_mpoly = Polygon::from(value);

        // for poly in &mpoly {
        //     for (ri, ring) in poly.rings().into_iter().enumerate() {
        //         int_ring_buf.clear();
        //         int_ring_buf.extend(ring.into_iter().map(|c| {
        //             let x = (c.x * extent as f64 + 0.5) as i16;
        //             let y = (c.y * extent as f64 + 0.5) as i16;
        //             [x, y]
        //         }));

        //         // some simplification
        //         {
        //             int_ring_buf2.clear();
        //             int_ring_buf2.push(int_ring_buf[0]);
        //             for c in int_ring_buf.windows(3) {
        //                 let &[prev, curr, next] = c.try_into().unwrap();

        //                 // Remove duplicate points
        //                 if prev == curr {
        //                     continue;
        //                 }

        //                 // Reject collinear points
        //                 let [curr_x, curr_y] = curr;
        //                 let [prev_x, prev_y] = prev;
        //                 let [next_x, next_y] = next;
        //                 if curr != next
        //                     && ((next_y - prev_y) as i32 * (curr_x - prev_x) as i32).abs()
        //                         == ((curr_y - prev_y) as i32 * (next_x - prev_x) as i32).abs()
        //                 {
        //                     continue;
        //                 }

        //                 int_ring_buf2.push(curr);
        //             }
        //             int_ring_buf2.push(*int_ring_buf.last().unwrap());
        //         }

        //         match ri {
        //             0 => int_mpoly.add_exterior(int_ring_buf2.drain(..)),
        //             _ => int_mpoly.add_interior(int_ring_buf2.drain(..)),
        //         }
        //     }
        // }

        // let mut int_mpoly = mpoly.clone();

        // // encode geometry
        // let mut geom_enc = GeometryEncoder::new();
        // for poly in &int_mpoly {
        //     let exterior = poly.exterior();
        //     // if exterior.signed_ring_area() > 0.0 {
        //         geom_enc.add_ring(&exterior);
        //         for interior in poly.interiors() {
        //             if interior.is_cw() {
        //                 geom_enc.add_ring(&interior);
        //             }
        //         }
        //     // }
        // }
        // let geometry = geom_enc.into_vec();
        // if geometry.is_empty() {
        //     continue;
        // }

        let geom_enc = GeometryEncoder::new();
        let geometry = geom_enc.into_vec(); // TODO

        let mut id = None;
        let mut tags: Vec<u32> = Vec::new();

        let layer = if let nusamai_citygml::object::Value::Object(obj) = &feature.properties {
            let layer = layers.entry_ref(obj.typename.as_ref()).or_default();

            // Encode attributes as MVT tags
            for (key, value) in &obj.attributes {
                convert_properties(&mut tags, &mut layer.tags_enc, key, value);
            }

            // Make a MVT feature id (u64) by hashing the original feature id string.
            id = obj.stereotype.id().map(|id| {
                id.as_bytes()
                    .iter()
                    .fold(5381u64, |a, c| a.wrapping_mul(33) ^ *c as u64)
            });

            layer
        } else {
            layers.entry_ref("Unknown").or_default()
        };

        layer.features.push(vector_tile::tile::Feature {
            id,
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

const GEOM_COMMAND_MOVE_TO: u32 = 1;
const GEOM_COMMAND_LINE_TO: u32 = 2;
const GEOM_COMMAND_CLOSE_PATH: u32 = 7;

const GEOM_COMMAND_MOVE_TO_WITH_COUNT1: u32 = 1 << 3 | GEOM_COMMAND_MOVE_TO;
const GEOM_COMMAND_CLOSE_PATH_WITH_COUNT1: u32 = 1 << 3 | GEOM_COMMAND_CLOSE_PATH;

pub struct GeometryEncoder {
    buf: Vec<u32>,
    prev_x: i16,
    prev_y: i16,
}

/// Utility for encoding MVT geometries.
impl GeometryEncoder {
    // TODO: with_capacity
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            prev_x: 0,
            prev_y: 0,
        }
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u32> {
        self.buf
    }

    pub fn add_ring(&mut self, iterable: impl IntoIterator<Item = [i16; 2]>) {
        let mut iter = iterable.into_iter();
        let Some([first_x, first_y]) = iter.next() else {
            return;
        };
        let dx = (first_x - self.prev_x) as i32;
        let dy = (first_y - self.prev_y) as i32;
        (self.prev_x, self.prev_y) = (first_x, first_y);

        // move to
        self.buf
            .extend([GEOM_COMMAND_MOVE_TO_WITH_COUNT1, zigzag(dx), zigzag(dy)]);

        // line to
        let lineto_cmd_pos = self.buf.len();
        self.buf.push(GEOM_COMMAND_LINE_TO); // length will be updated later
        let mut count = 0;
        for [x, y] in iter {
            let dx = (x - self.prev_x) as i32;
            let dy = (y - self.prev_y) as i32;
            (self.prev_x, self.prev_y) = (x, y);
            if dx != 0 || dy != 0 {
                self.buf.extend([zigzag(dx), zigzag(dy)]);
                count += 1;
            }
        }
        debug_assert!(count >= 2);
        self.buf[lineto_cmd_pos] = GEOM_COMMAND_LINE_TO | count << 3;

        // close path
        self.buf.push(GEOM_COMMAND_CLOSE_PATH_WITH_COUNT1);
    }
}

impl Default for GeometryEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn zigzag(v: i32) -> u32 {
    ((v << 1) ^ (v >> 31)) as u32
}

#[derive(Default)]
pub struct TagsEncoder {
    keys: IndexSet<String, RandomState>,
    values: IndexSet<Value, RandomState>,
}

/// Utility for encoding MVT attributes (tags).
impl TagsEncoder {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn add(&mut self, key: &str, value: Value) -> [u32; 2] {
        let key_idx = match self.keys.get_index_of(key) {
            None => self.keys.insert_full(key.to_string()).0,
            Some(idx) => idx,
        };
        let value_idx = match self.values.get_index_of(&value) {
            None => self.values.insert_full(value).0,
            Some(idx) => idx,
        };
        [key_idx as u32, value_idx as u32]
    }

    #[inline]
    pub fn into_keys_and_values(self) -> (Vec<String>, Vec<tile::Value>) {
        let keys = self.keys.into_iter().collect();
        let values = self
            .values
            .into_iter()
            .map(|v| v.into_tile_value())
            .collect();
        (keys, values)
    }
}

/// Wrapper for MVT Values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    String(String),
    Float([u8; 4]),
    Double([u8; 8]),
    Int(i64),
    Uint(u64),
    SInt(i64),
    Bool(bool),
}

impl Value {
    pub fn into_tile_value(self) -> tile::Value {
        use Value::*;
        match self {
            String(v) => tile::Value {
                string_value: Some(v),
                ..Default::default()
            },
            Float(v) => tile::Value {
                float_value: Some(f32::from_ne_bytes(v)),
                ..Default::default()
            },
            Double(v) => tile::Value {
                double_value: Some(f64::from_ne_bytes(v)),
                ..Default::default()
            },
            Int(v) => tile::Value {
                int_value: Some(v),
                ..Default::default()
            },
            Uint(v) => tile::Value {
                uint_value: Some(v),
                ..Default::default()
            },
            SInt(v) => tile::Value {
                sint_value: Some(v),
                ..Default::default()
            },
            Bool(v) => tile::Value {
                bool_value: Some(v),
                ..Default::default()
            },
        }
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}
impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Uint(v)
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Uint(v as u64)
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        if v >= 0 {
            Value::Uint(v as u64)
        } else {
            Value::SInt(v)
        }
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        if v >= 0 {
            Value::Uint(v as u64)
        } else {
            Value::SInt(v as i64)
        }
    }
}
impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v.to_ne_bytes())
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Double(v.to_ne_bytes())
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

#[derive(Default)]
struct LayerData {
    pub features: Vec<vector_tile::tile::Feature>,
    pub tags_enc: TagsEncoder,
}

#[derive(Serialize, Deserialize)]
struct SlicedFeature {
    geometry: Vec<Polygon>,
    properties: nusamai_citygml::object::Value,
}
use super::vector_tile;
use prost::Message;

pub fn convert_properties(
    tags: &mut Vec<u32>,
    tags_enc: &mut TagsEncoder,
    name: &str,
    tree: &nusamai_citygml::object::Value,
) {
    match &tree {
        nusamai_citygml::Value::String(v) => {
            tags.extend(tags_enc.add(name, v.clone().into()));
        }
        nusamai_citygml::Value::Code(v) => {
            tags.extend(tags_enc.add(name, v.value().into()));
        }
        nusamai_citygml::Value::Integer(v) => {
            tags.extend(tags_enc.add(name, (*v).into()));
        }
        nusamai_citygml::Value::NonNegativeInteger(v) => {
            tags.extend(tags_enc.add(name, (*v).into()));
        }
        nusamai_citygml::Value::Double(v) => {
            tags.extend(tags_enc.add(name, (*v).into()));
        }
        nusamai_citygml::Value::Measure(v) => {
            tags.extend(tags_enc.add(name, v.value().into()));
        }
        nusamai_citygml::Value::Boolean(v) => {
            tags.extend(tags_enc.add(name, (*v).into()));
        }
        nusamai_citygml::Value::Uri(v) => {
            tags.extend(tags_enc.add(name, v.value().to_string().into()));
        }
        nusamai_citygml::Value::Date(v) => {
            tags.extend(tags_enc.add(name, v.to_string().into()));
        }
        nusamai_citygml::Value::Point(v) => {
            tags.extend(tags_enc.add(name, format!("{:?}", v).into())); // FIXME
        }
        nusamai_citygml::Value::Array(_arr) => {
            // ignore non-root attributes
        }
        nusamai_citygml::Value::Object(_obj) => {
            // ignore non-root attributes
        }
    }
}

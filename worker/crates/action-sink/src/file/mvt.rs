use super::vector_tile::tile;
use crate::errors::SinkError;
use ahash::RandomState;
use flate2::{write::ZlibEncoder, Compression};
use geomotry_types::GeometryType;
use hashbrown::HashMap as BrownHashMap;
use indexmap::IndexSet;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::algorithm::area2d::Area2D;
use reearth_flow_geometry::algorithm::coords_iter::CoordsIter;
use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::line_string::LineString;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry as geomotry_types;
use reearth_flow_types::Attribute;
use reearth_flow_types::AttributeValue;
use reearth_flow_types::Expr;
use rhai::Variant;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
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

        let sink = MVTWriter { params };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub struct MVTWriter {
    pub(super) params: MVTWriterParam,
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
        let attributes = ctx.feature.attributes.clone();
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
                match self.handle_city_gml_geometry(
                    &output,
                    storage_resolver.clone(),
                    attributes,
                    city_gml,
                    self.params.min_zoom,
                    self.params.max_zoom,
                ) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(Box::new(SinkError::FileWriter(format!(
                            "CityGmlGeometry handle Error: {:?}",
                            e
                        ))))
                    }
                };
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

impl MVTWriter {
    fn handle_city_gml_geometry(
        &self,
        output: &Uri,
        storage_resolver: Arc<StorageResolver>,
        attributes: HashMap<Attribute, AttributeValue>,
        city_gml: geomotry_types::CityGmlGeometry,
        min_zoom: u8,
        max_zoom: u8,
    ) -> Result<(), crate::errors::SinkError> {
        let bincode_config = bincode::config::standard();

        let max_detail = 12; // 4096
        let buffer_pixels = 5;

        let mut tiled_mpolys = HashMap::new();

        let extent = 1 << max_detail;
        let buffer = extent * buffer_pixels / 256;

        for entry in city_gml.gml_geometries.iter() {
            match entry.ty {
                GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                    for idx_poly in city_gml.clone().polygon_uvs {
                        // Early rejection of polygons that are not front-facing.
                        if idx_poly.exterior().signed_area2d() < 0.0 {
                            continue;
                        }

                        let area = idx_poly.exterior().signed_area2d();

                        // Slice for each zoom level
                        for zoom in min_zoom..=max_zoom {
                            // Skip if the polygon is smaller than 4 square subpixels
                            //
                            // TODO: emulate the 'tiny-polygon-reduction' of tippecanoe
                            if area * (4u64.pow(zoom as u32 + max_detail) as f64) < 4.0 {
                                continue;
                            }

                            slice_polygon(zoom, extent, buffer, &idx_poly, &mut tiled_mpolys);
                        }
                    }
                }
                GeometryType::Curve => {
                    // TODO: implement
                }
                GeometryType::Point => {
                    // TODO: implement
                }
                _ => {
                    // TODO: implement
                }
            }
        }

        for ((zoom, x, y), mpoly) in tiled_mpolys {
            if mpoly.is_empty() {
                continue;
            }
            let feature = SlicedFeature {
                geometry: mpoly,
                properties: attributes.clone(),
            };

            let bytes = match bincode::serde::encode_to_vec(&feature, bincode_config) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Err(SinkError::FileWriter(format!(
                        "Failed to serialize a sliced feature: {:?}",
                        e
                    )));
                }
            };

            let default_detail = 12;
            let min_detail = 9;

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
                    let compressed_bytes =
                        e.finish().map_err(crate::errors::SinkError::file_writer)?;
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
}

fn make_tile(default_detail: i32, serialized_feats: &[Vec<u8>]) -> Result<Vec<u8>, SinkError> {
    let mut layers: BrownHashMap<String, LayerData> = BrownHashMap::new();
    let mut int_ring_buf = Vec::new();
    let mut int_ring_buf2 = Vec::new();
    let extent = 1 << default_detail;
    let bincode_config = bincode::config::standard();

    for serialized_feat in serialized_feats {
        let (feature, _): (SlicedFeature, _) =
            bincode::serde::decode_from_slice(serialized_feat, bincode_config).map_err(|err| {
                SinkError::FileWriter(format!("Failed to deserialize a sliced feature: {:?}", err))
            })?;

        let mpoly = feature.geometry;
        let mut int_mpoly: MultiPolygon2D<i16> = Default::default(); // 2D only

        for poly in &mpoly {
            for (ri, ring) in poly.rings().into_iter().enumerate() {
                int_ring_buf.clear();
                int_ring_buf.extend(ring.into_iter().map(|c| {
                    let x = (c.x as f64 * extent as f64 + 0.5) as i16;
                    let y = (c.y as f64 * extent as f64 + 0.5) as i16;
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

                let ls = LineString::new(
                    int_ring_buf2
                        .iter()
                        .map(|c| Coordinate::new_(c[0], c[1]))
                        .collect(),
                );
                match ri {
                    0 => int_mpoly.add_exterior(ls),
                    _ => int_mpoly.add_interior(ls),
                }
            }
        }

        // encode geometry
        let mut geom_enc = GeometryEncoder::new();
        for poly in &int_mpoly {
            let exterior = poly.exterior();
            if exterior.signed_area2d() < 0 {
                geom_enc.add_ring(exterior.into_iter().map(|c| [c.x, c.y]));
                for interior in poly.interiors() {
                    if interior.signed_area2d() > 0 {
                        geom_enc.add_ring(interior.into_iter().map(|c| [c.x, c.y]));
                    }
                }
            }
        }
        let geometry = geom_enc.into_vec();
        if geometry.is_empty() {
            continue;
        }

        let tags: Vec<u32> = Vec::new();

        for (key, value) in &feature.properties {
            let layer = layers.entry_ref(key.type_name()).or_default();
            let mut tags_clone = tags.clone();
            convert_properties(&mut tags_clone, &mut layer.tags_enc, key.type_name(), value);

            let geomery_cloned = geometry.clone();
            layer.features.push(vector_tile::tile::Feature {
                id: Some(
                    key.type_name()
                        .as_bytes()
                        .iter()
                        .fold(5381u64, |a, c| a.wrapping_mul(33) ^ *c as u64),
                ),
                tags: tags_clone,
                r#type: Some(vector_tile::tile::GeomType::Polygon as i32),
                geometry: geomery_cloned,
            });
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
    #[allow(dead_code)]
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
    geometry: MultiPolygon2D<i16>,
    properties: HashMap<Attribute, AttributeValue>,
}
use super::vector_tile;
use prost::Message;

pub fn convert_properties(
    tags: &mut Vec<u32>,
    tags_enc: &mut TagsEncoder,
    name: &str,
    tree: &AttributeValue,
) {
    match &tree {
        AttributeValue::Null => {
            // ignore
        }
        AttributeValue::String(v) => {
            tags.extend(tags_enc.add(name, v.clone().into()));
        }
        AttributeValue::Bool(v) => {
            tags.extend(tags_enc.add(name, (*v).into()));
        }
        AttributeValue::Number(v) => {
            if let Some(v) = v.as_u64() {
                tags.extend(tags_enc.add(name, v.into()));
            } else if let Some(v) = v.as_i64() {
                tags.extend(tags_enc.add(name, v.into()));
            } else if let Some(v) = v.as_f64() {
                tags.extend(tags_enc.add(name, v.into()));
            }
        }
        AttributeValue::Array(_arr) => {
            // ignore non-root attributes
        }
        AttributeValue::Bytes(_v) => {
            // ignore non-root attributes
        }
        AttributeValue::Map(obj) => {
            for (key, value) in obj {
                convert_properties(tags, tags_enc, key, value);
            }
        }
        AttributeValue::DateTime(v) => {
            tags.extend(tags_enc.add(name, v.to_string().into()));
        }
    }
}

fn slice_polygon(
    zoom: u8,
    extent: u32,
    buffer: u32,
    poly: &Polygon2D<f64>,
    out: &mut HashMap<(u8, u32, u32), MultiPolygon2D<i16>>,
) {
    let z_scale = (1 << zoom) as f64;
    let buf_width = buffer as f64 / extent as f64;
    let mut new_ring_buffer: Vec<[f64; 2]> =
        Vec::with_capacity(poly.exterior().into_iter().len() + 1);

    // Slice along Y-axis
    let y_range = {
        let (min_y, max_y) = poly
            .exterior()
            .into_iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), c| {
                (min_y.min(c.y), max_y.max(c.y))
            });
        (min_y * z_scale).floor() as u32..(max_y * z_scale).ceil() as u32
    };

    let mut y_sliced_polys = Vec::with_capacity(y_range.len());

    for yi in y_range.clone() {
        let k1 = (yi as f64 - buf_width) / z_scale;
        let k2 = ((yi + 1) as f64 + buf_width) / z_scale;

        let mut y_sliced_poly: Polygon2D<f64> =
            Polygon2D::<f64>::new(LineString::new(vec![]), vec![]);

        for ring in poly.rings() {
            if ring.coords_count() == 0 {
                continue;
            }

            new_ring_buffer.clear();
            ring.into_iter()
                .fold(None, |a, b| {
                    let Some(a) = a else { return Some(b) };

                    if a.y < k1 {
                        if b.y > k1 {
                            let x = (b.x - a.x) * (k1 - a.y) / (b.y - a.y) + a.x;
                            new_ring_buffer.push([x, k1])
                        }
                    } else if a.y > k2 {
                        if b.y < k2 {
                            let x = (b.x - a.x) * (k2 - a.y) / (b.y - a.y) + a.x;
                            new_ring_buffer.push([x, k2])
                        }
                    } else {
                        new_ring_buffer.push([a.x, a.y])
                    }

                    if b.y < k1 && a.y > k1 {
                        let x = (b.x - a.x) * (k1 - a.y) / (b.y - a.y) + a.x;
                        new_ring_buffer.push([x, k1])
                    } else if b.y > k2 && a.y < k2 {
                        let x = (b.x - a.x) * (k2 - a.y) / (b.y - a.y) + a.x;
                        new_ring_buffer.push([x, k2])
                    }

                    Some(b)
                })
                .unwrap();

            let coordinates: Vec<Coordinate2D<f64>> = new_ring_buffer
                .clone()
                .into_iter()
                .map(|[x, y]| Coordinate { x, y, z: 00 })
                .map(|c| Coordinate2D::new_(c.x, c.y))
                .collect();

            let linestring = LineString::from(coordinates);

            y_sliced_poly.interiors_push(linestring);
        }

        y_sliced_polys.push(y_sliced_poly);
    }

    let mut norm_coords_buf = Vec::new();

    // Slice along X-axis
    for (yi, y_sliced_poly) in y_range.zip(y_sliced_polys.iter()) {
        let x_range = {
            let (min_x, max_x) = y_sliced_poly
                .exterior()
                .into_iter()
                .fold((f64::MAX, f64::MIN), |(min_x, max_x), c| {
                    (min_x.min(c.x), max_x.max(c.x))
                });
            (min_x * z_scale).floor() as i32..(max_x * z_scale).ceil() as i32
        };

        for xi in x_range {
            let k1 = (xi as f64 - buf_width) / z_scale;
            let k2 = ((xi + 1) as f64 + buf_width) / z_scale;

            let key = (
                zoom,
                xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
                yi,
            );
            let tile_mpoly = out.entry(key).or_default();

            for (ri, ring) in y_sliced_poly.rings().into_iter().enumerate() {
                if ring.coords_count() == 0 {
                    continue;
                }

                new_ring_buffer.clear();
                ring.into_iter()
                    .fold(None, |a, b| {
                        let Some(a) = a else { return Some(b) };

                        if a.x < k1 {
                            if b.x > k1 {
                                let y = (b.y - a.y) * (k1 - a.x) / (b.x - a.x) + a.y;
                                new_ring_buffer.push([k1, y])
                            }
                        } else if a.x > k2 {
                            if b.x < k2 {
                                let y = (b.y - a.y) * (k2 - a.x) / (b.x - a.x) + a.y;
                                new_ring_buffer.push([k2, y])
                            }
                        } else {
                            new_ring_buffer.push([a.x, a.y])
                        }

                        if b.x < k1 && a.x > k1 {
                            let y = (b.y - a.y) * (k1 - a.x) / (b.x - a.x) + a.y;
                            new_ring_buffer.push([k1, y])
                        } else if b.x > k2 && a.x < k2 {
                            let y = (b.y - a.y) * (k2 - a.x) / (b.x - a.x) + a.y;
                            new_ring_buffer.push([k2, y])
                        }

                        Some(b)
                    })
                    .unwrap();

                // get integer coordinates and simplify the ring
                {
                    norm_coords_buf.clear();
                    norm_coords_buf.extend(new_ring_buffer.iter().map(|&[x, y]| {
                        let tx = (x * z_scale - xi as f64) as i16;
                        let ty = (y * z_scale - yi as f64) as i16;
                        [tx, ty]
                    }));

                    // remove closing point if exists
                    if norm_coords_buf.len() >= 2
                        && norm_coords_buf[0] == *norm_coords_buf.last().unwrap()
                    {
                        norm_coords_buf.pop();
                    }

                    if norm_coords_buf.len() < 3 {
                        continue;
                    }
                }

                // let mut ring = LineString::from(norm_coords_buf);
                let mut ls = LineString::new(
                    norm_coords_buf
                        .iter()
                        .map(|c| Coordinate::new_(c[0], c[1]))
                        .collect(),
                );
                ls.reverse_inplace();

                match ri {
                    0 => tile_mpoly.add_exterior(ls),
                    _ => tile_mpoly.add_interior(ls),
                };
            }
        }
    }
}

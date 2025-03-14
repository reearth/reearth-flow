use std::collections::HashMap;

use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::rect::{Rect, Rect2D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct JPStandardGridAccumulatorFactory;

impl ProcessorFactory for JPStandardGridAccumulatorFactory {
    fn name(&self) -> &str {
        "JPStandardGridAccumulator"
    }

    fn description(&self) -> &str {
        "Creates a convex partition based on a group of input features."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JPStandardGridAccumulatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: JPStandardGridAccumulatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::JPStandardGridAccumulatorFactory(format!(
                    "Failed to serialize 'with' parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::JPStandardGridAccumulatorFactory(format!(
                    "Failed to deserialize 'with' parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::JPStandardGridAccumulatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = JPStandardGridAccumulator {
            group_by: param.group_by,
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JPStandardGridAccumulatorParam {
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct JPStandardGridAccumulator {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for JPStandardGridAccumulator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                let key = if let Some(group_by) = &self.group_by {
                    AttributeValue::Array(
                        group_by
                            .iter()
                            .filter_map(|attr| feature.attributes.get(attr).cloned())
                            .collect(),
                    )
                } else {
                    AttributeValue::Null
                };

                if !self.buffer.contains_key(&key) {
                    for partition in self.devide_into_grid() {
                        fw.send(ctx.new_with_feature_and_port(partition, DEFAULT_PORT.clone()));
                    }
                    self.buffer.clear();
                }

                self.buffer
                    .entry(key.clone())
                    .or_default()
                    .push(feature.clone());
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        for partition in self.devide_into_grid() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                partition,
                DEFAULT_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "JPStandardGridAccumulator"
    }
}

impl JPStandardGridAccumulator {
    fn devide_into_grid(&self) -> Vec<Feature> {
        vec![]
    }
}

#[derive(Debug, Clone, Copy)]
enum JPMeshType {
    /// 第1次地域区画
    Mesh80km,
    /// 第2次地域区画
    Mesh10km,
    /// 基準地域メッシュ
    Mesh1km,
    /// 2分の1地域メッシュ
    Mesh500m,
    /// 4分の1地域メッシュ
    Mesh250m,
    /// 8分の1地域メッシュ
    Mesh125m,
}

impl JPMeshType {
    fn from_string(s: &str) -> Option<JPMeshType> {
        match s {
            "Mesh80km" => Some(JPMeshType::Mesh80km),
            "Mesh10km" => Some(JPMeshType::Mesh10km),
            "Mesh1km" => Some(JPMeshType::Mesh1km),
            "Mesh500m" => Some(JPMeshType::Mesh500m),
            "Mesh250m" => Some(JPMeshType::Mesh250m),
            "Mesh125m" => Some(JPMeshType::Mesh125m),
            _ => None,
        }
    }

    fn lat_interval_seconds(&self) -> f64 {
        match self {
            JPMeshType::Mesh80km => 2400.0,
            JPMeshType::Mesh10km => 300.0,
            JPMeshType::Mesh1km => 30.0,
            JPMeshType::Mesh500m => 15.0,
            JPMeshType::Mesh250m => 7.5,
            JPMeshType::Mesh125m => 3.75,
        }
    }

    fn lng_interval_seconds(&self) -> f64 {
        match self {
            JPMeshType::Mesh80km => 3600.0,
            JPMeshType::Mesh10km => 450.0,
            JPMeshType::Mesh1km => 45.0,
            JPMeshType::Mesh500m => 22.5,
            JPMeshType::Mesh250m => 11.25,
            JPMeshType::Mesh125m => 5.625,
        }
    }

    fn lat_interval_minutes(&self) -> f64 {
        self.lat_interval_seconds() / 60.0
    }

    fn lng_interval_minutes(&self) -> f64 {
        self.lng_interval_seconds() / 60.0
    }

    fn lat_interval_degrees(&self) -> f64 {
        self.lat_interval_seconds() / 3600.0
    }

    fn lng_interval_degrees(&self) -> f64 {
        self.lng_interval_seconds() / 3600.0
    }
}

struct JPMeshCode {
    mesh_code_type: JPMeshType,
    seed: JPMeshCodeSeed,
}

impl JPMeshCode {
    fn new(coords: Coordinate2D<f64>, mesh_code_type: JPMeshType) -> Self {
        let seed = JPMeshCodeSeed::new(coords);
        JPMeshCode {
            mesh_code_type,
            seed,
        }
    }

    fn to_slice(&self) -> &[u8] {
        match self.mesh_code_type {
            JPMeshType::Mesh80km => &self.seed.code_bin[..4],
            JPMeshType::Mesh10km => &self.seed.code_bin[..6],
            JPMeshType::Mesh1km => &self.seed.code_bin[..8],
            JPMeshType::Mesh500m => &self.seed.code_bin[..9],
            JPMeshType::Mesh250m => &self.seed.code_bin[..10],
            JPMeshType::Mesh125m => &self.seed.code_bin[..11],
        }
    }

    fn to_string(&self) -> String {
        self.to_slice()
            .iter()
            .map(|&digit| digit.to_string())
            .collect()
    }

    fn to_number(&self) -> u64 {
        let mut result = 0;
        for &digit in self.to_slice() {
            result = result * 10 + digit as u64;
        }
        result
    }

    /// メッシュコードの値に対して、その地域を表す座標の形をRectで表現する
    fn into_bounds(&self) -> Rect2D<f64> {
        self.seed.into_bounds(self.mesh_code_type)
    }
}

struct JPMeshCodeSeed {
    code_bin: [u8; 11],
}

impl JPMeshCodeSeed {
    // 座標に対するメッシュコードの計算
    fn new(coords: Coordinate2D<f64>) -> Self {
        // 緯度の計算
        // 緯度 / first_lat_degrees = p 余り a
        let p = (coords.y / JPMeshType::Mesh80km.lat_interval_degrees()).floor() as u8;
        let a_degrees = coords.y % JPMeshType::Mesh80km.lat_interval_degrees();

        // a / second_lat_degrees = q 余り b
        let q = (a_degrees / JPMeshType::Mesh10km.lat_interval_degrees()).floor() as u8;
        let b_degrees = a_degrees % JPMeshType::Mesh10km.lat_interval_degrees();

        // b / third_lat_degrees = r 余り c
        let r = (b_degrees / JPMeshType::Mesh1km.lat_interval_degrees()).floor() as u8;
        let c_degrees = b_degrees % JPMeshType::Mesh1km.lat_interval_degrees();

        // c / half_lat_degrees = s 余り d
        let s = (c_degrees / JPMeshType::Mesh500m.lat_interval_degrees()).floor() as u8;
        let d_degrees = c_degrees % JPMeshType::Mesh500m.lat_interval_degrees();

        // d / quarter_lat_degrees = t 余り e
        let t = (d_degrees / JPMeshType::Mesh250m.lat_interval_degrees()).floor() as u8;

        // d / eighth_lat_degrees = tt
        let tt = (d_degrees / JPMeshType::Mesh125m.lat_interval_degrees()).floor() as u8;

        // 経度の計算
        // 経度 - 100度 = u 余り f
        let u = (coords.x - 100.0).floor() as u8;
        let f_degrees = coords.x - 100.0 - u as f64;

        // f / second_lng_degrees = v 余り g
        let v = (f_degrees / JPMeshType::Mesh10km.lng_interval_degrees()).floor() as u8;
        let g_degrees = f_degrees % JPMeshType::Mesh10km.lng_interval_degrees();

        // g / third_lng_degrees = w 余り h
        let w = (g_degrees / JPMeshType::Mesh1km.lng_interval_degrees()).floor() as u8;
        let h_degrees = g_degrees % JPMeshType::Mesh1km.lng_interval_degrees();

        // h / half_lng_degrees = x 余り i
        let x = (h_degrees / JPMeshType::Mesh500m.lng_interval_degrees()).floor() as u8;
        let i_degrees = h_degrees % JPMeshType::Mesh500m.lng_interval_degrees();

        // i / quarter_lng_degrees = y 余り j
        let y = (i_degrees / JPMeshType::Mesh250m.lng_interval_degrees()).floor() as u8;

        // i / eighth_lng_degrees = yy
        let yy = (i_degrees / JPMeshType::Mesh125m.lng_interval_degrees()).floor() as u8;

        // 最終計算
        // (s * 2)+(x + 1)= m
        let m = (s * 2) + (x + 1);

        // (t * 2)+(y + 1)= n
        let n = (t * 2) + (y + 1);

        // (tt * 2)+(yy + 1)= nn
        let nn = (tt * 2) + (yy + 1);

        // 上位6桁 (第1次地域区画, 第2次地域区画)
        let head = {
            let p1 = (p / 10) % 10;
            let p2 = p % 10;
            let u1 = (u / 10) % 10;
            let u2 = u % 10;
            [p1, p2, u1, u2, q, v]
        };

        // 下位5桁 (基準地域メッシュ, {2,4,8}分の1地域メッシュ)
        let tail_bin = { [r, w, m, n, nn] };

        let mut code_bin = [0u8; 11];
        code_bin[..6].copy_from_slice(&head);
        code_bin[6..11].copy_from_slice(&tail_bin);

        JPMeshCodeSeed { code_bin }
    }

    // メッシュコードの値に対して、その地域を表す座標の形をRectで表現する
    fn into_bounds(&self, mesh_code_type: JPMeshType) -> Rect2D<f64> {
        // メッシュコードから緯度経度の範囲を計算
        let p = (self.code_bin[0] * 10 + self.code_bin[1]) as f64;
        let u = (self.code_bin[2] * 10 + self.code_bin[3]) as f64;
        let q = self.code_bin[4] as f64;
        let v = self.code_bin[5] as f64;
        let r = self.code_bin[6] as f64;
        let w = self.code_bin[7] as f64;
        let m = self.code_bin[8] as f64;
        let n = self.code_bin[9] as f64;
        let nn = self.code_bin[10] as f64;

        // 緯度の計算（南西端）
        let lat_base = p * JPMeshType::Mesh80km.lat_interval_degrees();
        let lat_q = q * JPMeshType::Mesh10km.lat_interval_degrees();
        let lat_r = r * JPMeshType::Mesh1km.lat_interval_degrees();
        let lat_m = ((m - 1.0) % 2.0) * JPMeshType::Mesh500m.lat_interval_degrees();
        let lat_n = ((n - 1.0) % 2.0) * JPMeshType::Mesh250m.lat_interval_degrees();
        let lat_nn = ((nn - 1.0) % 2.0) * JPMeshType::Mesh125m.lat_interval_degrees();

        // 経度の計算（南西端）
        let lng_base = 100.0 + u;
        let lng_v = v * JPMeshType::Mesh10km.lng_interval_degrees();
        let lng_w = w * JPMeshType::Mesh1km.lng_interval_degrees();
        let lng_m = ((m - 1.0) / 2.0) * JPMeshType::Mesh500m.lng_interval_degrees();
        let lng_n = ((n - 1.0) / 2.0) * JPMeshType::Mesh250m.lng_interval_degrees();
        let lng_nn = ((nn - 1.0) / 2.0) * JPMeshType::Mesh125m.lng_interval_degrees();

        // 南西端（左下）の座標
        let min_lng = lng_base + lng_v + lng_w + lng_m + lng_n + lng_nn;
        let min_lat = lat_base + lat_q + lat_r + lat_m + lat_n + lat_nn;

        // 北東端（右上）の座標
        let max_lat = min_lat + mesh_code_type.lat_interval_degrees();
        let max_lng = min_lng + mesh_code_type.lng_interval_degrees();

        Rect::new(
            Coordinate2D::new_(min_lng, min_lat),
            Coordinate2D::new_(max_lng, max_lat),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    #[macro_export]
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert!(
                ($a - $b).abs() < EPSILON,
                "assertion failed: `(left ≈ right)`\n  left: `{}`\n right: `{}`\n",
                $a,
                $b
            );
        };
    }

    #[derive(Debug)]
    struct TestCase {
        inner_latitude: f64,
        inner_longitude: f64,
        mesh_code: u64,
        left_bottom_latitude: f64,
        left_bottom_longitude: f64,
    }
    const TEST_CASES: [TestCase; 4] = [
        TestCase {
            inner_latitude: 43.058336,
            inner_longitude: 141.337503,
            mesh_code: 64414277,
            left_bottom_latitude: 43.058333,
            left_bottom_longitude: 141.3375,
        },
        TestCase {
            inner_latitude: 40.81667,
            inner_longitude: 140.737503,
            mesh_code: 61401589,
            left_bottom_latitude: 40.816667,
            left_bottom_longitude: 140.7375,
        },
        TestCase {
            inner_latitude: 39.700003,
            inner_longitude: 141.150003,
            mesh_code: 59414142,
            left_bottom_latitude: 39.7,
            left_bottom_longitude: 141.15,
        },
        TestCase {
            inner_latitude: 38.26667,
            inner_longitude: 140.862503,
            mesh_code: 57403629,
            left_bottom_latitude: 38.266667,
            left_bottom_longitude: 140.8625,
        },
    ];

    #[test]
    fn test_mesh_code_generation() {
        for test_case in TEST_CASES {
            let coords = Coordinate2D::new_(test_case.inner_longitude, test_case.inner_latitude);
            let mesh_code = JPMeshCode::new(coords, JPMeshType::Mesh1km);

            let actual_number = mesh_code.to_number();
            assert_eq!(actual_number, test_case.mesh_code);

            let actual_string = mesh_code.to_string();
            assert_eq!(actual_string, test_case.mesh_code.to_string());
        }
    }

    #[test]
    fn test_mesh_code_into_bounds() {
        for test_case in TEST_CASES {
            let coords = Coordinate2D::new_(test_case.inner_longitude, test_case.inner_latitude);
            let mesh_code = JPMeshCode::new(coords, JPMeshType::Mesh1km);

            let bounds = mesh_code.into_bounds();
            let min_coord = bounds.min();

            // check if the left bottom coordinate is correct
            assert_approx_eq!(min_coord.x, test_case.left_bottom_longitude);
            assert_approx_eq!(min_coord.y, test_case.left_bottom_latitude);

            // check if the size of the area is correct
            let max_coord = bounds.max();
            assert_approx_eq!(max_coord.x - min_coord.x, 45.0 / 3600.0);
            assert_approx_eq!(max_coord.y - min_coord.y, 30.0 / 3600.0);
        }
    }
}

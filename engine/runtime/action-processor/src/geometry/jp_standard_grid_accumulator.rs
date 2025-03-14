use std::collections::HashMap;

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

enum MeshCodeType {
    /// 第1次地域区画
    First,
    /// 第2次地域区画
    Second,
    /// 基準地域メッシュ
    Third,
    /// 2分の1地域メッシュ
    Half,
    /// 4分の1地域メッシュ
    Quarter,
}

impl MeshCodeType {
    fn from_string(s: &str) -> Option<MeshCodeType> {
        match s {
            "First" => Some(MeshCodeType::First),
            "Second" => Some(MeshCodeType::Second),
            "Third" => Some(MeshCodeType::Third),
            "Half" => Some(MeshCodeType::Half),
            "Quarter" => Some(MeshCodeType::Quarter),
            _ => None,
        }
    }
}

struct MeshCode {
    mesh_code_type: MeshCodeType,
    code_bin: [u8; 10],
}

impl MeshCode {
    fn new(lon_degrees: f64, lat_degrees: f64, mesh_code_type: MeshCodeType) -> Self {
        let seed = MeshCodeSeed::new(lon_degrees, lat_degrees);
        let mut code_bin = [0u8; 10];
        code_bin[..6].copy_from_slice(&seed.head);
        code_bin[6..10].copy_from_slice(&seed.tail_bin);
        MeshCode {
            code_bin,
            mesh_code_type,
        }
    }

    fn to_slice(&self) -> &[u8] {
        match self.mesh_code_type {
            MeshCodeType::First => &self.code_bin[..4],
            MeshCodeType::Second => &self.code_bin[..6],
            MeshCodeType::Third => &self.code_bin[..8],
            MeshCodeType::Half => &self.code_bin[..9],
            MeshCodeType::Quarter => &self.code_bin[..10],
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
}

struct MeshCodeSeed {
    /// 上位6桁 (第1次地域区画, 第2次地域区画)
    head: [u8; 6],
    /// 下位4桁 (基準地域メッシュ, {2,4,8}分の1地域メッシュ)
    tail_bin: [u8; 4],
}

impl MeshCodeSeed {
    fn new(lon_degrees: f64, lat_degrees: f64) -> Self {
        // 緯度の計算
        // 緯度 × 60分 ÷ 40分 ＝ p 余り a
        let lat_minutes = lat_degrees * 60.0;
        let p = (lat_minutes / 40.0).floor() as u8;
        let a_minutes = lat_minutes % 40.0;

        // a ÷ 5分 ＝ q 余り b
        let q = (a_minutes / 5.0).floor() as u8;
        let b_minutes = a_minutes % 5.0;

        // b × 60秒 ÷ 30秒 ＝ r 余り c
        let b_seconds = b_minutes * 60.0;
        let r = (b_seconds / 30.0).floor() as u8;
        let c_seconds = b_seconds % 30.0;

        // c ÷ 15秒 ＝ s 余り d
        let s = (c_seconds / 15.0).floor() as u8;
        let d_seconds = c_seconds % 15.0;

        // d ÷ 7.5秒 ＝ t 余り e
        let t = (d_seconds / 7.5).floor() as u8;
        // e は使用しないので計算しない

        // 経度の計算
        // 経度 － 100度 ＝ u 余り f
        let u = (lon_degrees - 100.0).floor() as u8;
        let f_degrees = lon_degrees - 100.0 - u as f64;

        // f × 60分 ÷ 7分30秒 ＝ v 余り g
        let f_minutes = f_degrees * 60.0;
        let v = (f_minutes / 7.5).floor() as u8;
        let g_minutes = f_minutes % 7.5;

        // g × 60秒 ÷ 45秒 ＝ w 余り h
        let g_seconds = g_minutes * 60.0;
        let w = (g_seconds / 45.0).floor() as u8;
        let h_seconds = g_seconds % 45.0;

        // h ÷ 22.5秒 ＝ x 余り i
        let x = (h_seconds / 22.5).floor() as u8;
        let i_seconds = h_seconds % 22.5;

        // i ÷ 11.25秒 ＝ y 余り j
        let y = (i_seconds / 11.25).floor() as u8;
        // j は使用しないので計算しない

        // 最終計算
        // (s × 2)＋(x ＋ 1)＝ m
        let m = (s * 2) + (x + 1);

        // (t × 2)＋(y ＋ 1)＝ n
        let n = (t * 2) + (y + 1);

        let head = {
            let v1 = (p / 10) % 10;
            let v2 = p % 10;
            let v3 = (u / 10) % 10;
            let v4 = u % 10;
            [v1, v2, v3, v4, q, v]
        };

        let tail_bin = { [r, w, m, n] };

        MeshCodeSeed { head, tail_bin }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCase {
        inner_latitude: f64,
        inner_longitude: f64,
        mesh_code: u64,
        left_bottom_latitude: f64,
        left_bottom_longitude: f64,
    }

    #[test]
    fn test_mesh_code_generation() {
        // テストケースの作成
        let test_cases = vec![
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

        for test_case in test_cases {
            let mesh_code = MeshCode::new(
                test_case.inner_longitude,
                test_case.inner_latitude,
                MeshCodeType::Third,
            );

            let actual = mesh_code.to_number();

            assert_eq!(
                actual,
                test_case.mesh_code,
                "Failed to generate mesh code from latitude: {}, longitude: {}. Expected: {}, Actual: {}",
                test_case.inner_latitude,
                test_case.inner_longitude,
                test_case.mesh_code,
                actual
            );
        }
    }
}

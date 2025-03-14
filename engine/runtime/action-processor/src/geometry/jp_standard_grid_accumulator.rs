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

#[derive(Debug, Clone, PartialEq, Eq)]
enum MeshCode {
    /// 第1次地域区画
    First([u8; 4]),
    /// 第2次地域区画
    Second([u8; 6]),
    /// 基準地域メッシュ
    Third([u8; 8]),
    /// 2分の1地域メッシュ
    Half([u8; 9]),
    /// 4分の1地域メッシュ
    Quarter([u8; 10]),
}

impl MeshCode {
    fn new(lon_degrees: f64, lat_degrees: f64, mesh_code_type: MeshCodeType) -> MeshCode {
        let s = calculate_mesh_code_seed(lon_degrees, lat_degrees);

        match mesh_code_type {
            MeshCodeType::First => {
                let mut code = [0u8; 4];
                code.copy_from_slice(&s.head[..4]);
                MeshCode::First(code)
            }
            MeshCodeType::Second => {
                let mut code = [0u8; 6];
                code.copy_from_slice(&s.head[..6]);
                MeshCode::Second(code)
            }
            MeshCodeType::Third => {
                let mut code = [0u8; 8];
                code[..6].copy_from_slice(&s.head[..6]);
                code[6..8].copy_from_slice(&s.tail_bin[..2]);
                MeshCode::Third(code)
            }
            MeshCodeType::Half => {
                let mut code = [0u8; 9];
                code[..6].copy_from_slice(&s.head[..6]);
                code[6..9].copy_from_slice(&s.tail_bin[..3]);
                MeshCode::Half(code)
            }
            MeshCodeType::Quarter => {
                let mut code = [0u8; 10];
                code[..6].copy_from_slice(&s.head[..6]);
                code[6..10].copy_from_slice(&s.tail_bin[..4]);
                MeshCode::Quarter(code)
            }
        }
    }

    fn to_vec(&self) -> &[u8] {
        match self {
            MeshCode::First(code) => code,
            MeshCode::Second(code) => code,
            MeshCode::Third(code) => code,
            MeshCode::Half(code) => code,
            MeshCode::Quarter(code) => code,
        }
    }

    fn to_string(&self) -> String {
        self.to_vec()
            .iter()
            .map(|&digit| digit.to_string())
            .collect()
    }

    fn to_number(&self) -> i32 {
        let mut result = 0;
        for &digit in self.to_vec() {
            result = result * 10 + digit as i32;
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

fn calculate_mesh_code_seed(lon_degrees: f64, lat_degrees: f64) -> MeshCodeSeed {
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

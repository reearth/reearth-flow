use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::bool_ops::BooleanOps, types::multi_polygon::MultiPolygon2D,
};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));

#[derive(Debug, Clone, Default)]
pub struct UnifierFactory;

impl ProcessorFactory for UnifierFactory {
    fn name(&self) -> &str {
        "Unifier"
    }

    fn description(&self) -> &str {
        "unifies features grouped by specified attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(UnifierParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![AREA_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: UnifierParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::UnifierFactory(format!(
                    "Failed to serialize 'with' parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::UnifierFactory(format!(
                    "Failed to deserialize 'with' parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::UnifierFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = Unifier {
            group_by: param.group_by,
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnifierParam {
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct Unifier {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for Unifier {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }
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
                    // グループが切り替わったタイミングで、バッファ内の地物について Unite 処理を実行
                    for unified in self.unify() {
                        fw.send(ctx.new_with_feature_and_port(unified, AREA_PORT.clone()));
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

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for unified in self.unify() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                unified,
                AREA_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Unifier"
    }
}

impl Unifier {
    fn unify(&self) -> Vec<Feature> {
        let mut unified = Vec::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();

            let features = self.unify_2d(buffered_features_2d);
            unified.extend(features);
        }
        unified
    }

    fn unify_2d(&self, buffered_features_2d: Vec<&Feature>) -> Vec<Feature> {
        // let multi_polygon_2d = buffered_features_2d.iter().fold(
        //     None,
        //     |multi_polygon_acc: Option<_>, feature_incoming| {
        //         let geometry_incoming = feature_incoming.geometry.value.as_flow_geometry_2d()?;
        //         let multi_polygon_incoming =
        //             if let Some(multi_polygon) = geometry_incoming.as_multi_polygon() {
        //                 multi_polygon
        //             } else if let Some(polygon) = geometry_incoming.as_polygon() {
        //                 MultiPolygon2D::new(vec![polygon])
        //             } else {
        //                 return multi_polygon_acc;
        //             };

        //         let mutli_polygon_acc = if let Some(mutli_polygon_acc) = multi_polygon_acc {
        //             mutli_polygon_acc
        //         } else {
        //             return Some(multi_polygon_incoming);
        //         };

        //         let unite = multi_polygon_incoming.union(&mutli_polygon_acc);
        //         Some(unite)
        //     },
        // );

        let mut unified_polygons = Vec::new();
        for geometry_incoming in buffered_features_2d
            .iter()
            .filter_map(|f| f.geometry.value.as_flow_geometry_2d())
        {
            let multi_polygon_incoming =
                if let Some(multi_polygon) = geometry_incoming.as_multi_polygon() {
                    multi_polygon
                } else if let Some(polygon) = geometry_incoming.as_polygon() {
                    MultiPolygon2D::new(vec![polygon])
                } else {
                    continue;
                };

            let mut new_unified_polygons = Vec::new();

            if unified_polygons.is_empty() {
                new_unified_polygons.push(multi_polygon_incoming);
            } else {
                let mut multi_polygon_incoming_rest = multi_polygon_incoming.clone();
                for unified_polygon in &unified_polygons {
                    multi_polygon_incoming_rest =
                        multi_polygon_incoming_rest.difference(unified_polygon);
                }
                new_unified_polygons.push(multi_polygon_incoming_rest);
                for unified_polygon in &unified_polygons {
                    let intersected = multi_polygon_incoming.intersection(unified_polygon);
                    new_unified_polygons.push(intersected);
                    let difference = unified_polygon.difference(&multi_polygon_incoming);
                    new_unified_polygons.push(difference);
                }
            }

            unified_polygons = new_unified_polygons;
        }

        let mut features = Vec::new();
        for multi_polygon_2d in unified_polygons {
            let mut feature = Feature::new();
            feature.geometry.value = GeometryValue::FlowGeometry2D(multi_polygon_2d.into());
            features.push(feature);
        }
        features
    }
}

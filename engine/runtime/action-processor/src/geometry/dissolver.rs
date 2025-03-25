use std::collections::HashMap;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::bool_ops::BooleanOps, types::multi_polygon::MultiPolygon2D,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));

#[derive(Debug, Clone, Default)]
pub struct DissolverFactory;

impl ProcessorFactory for DissolverFactory {
    fn name(&self) -> &str {
        "Dissolver"
    }

    fn description(&self) -> &str {
        "Dissolves features grouped by specified attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DissolverParam))
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
        let param: DissolverParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::DissolverFactory(format!(
                    "Failed to serialize 'with' parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::DissolverFactory(format!(
                    "Failed to deserialize 'with' parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::DissolverFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = Dissolver {
            group_by: param.group_by,
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DissolverParam {
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct Dissolver {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for Dissolver {
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
                    for dissolved in self.dissolve() {
                        fw.send(ctx.new_with_feature_and_port(dissolved, AREA_PORT.clone()));
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
        for dissolved in self.dissolve() {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                dissolved,
                AREA_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Dissolver"
    }
}

impl Dissolver {
    fn dissolve(&self) -> Vec<Feature> {
        let mut dissolved = Vec::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();

            if let Some(dissolved_2d) = self.dissolve_2d(buffered_features_2d) {
                dissolved.push(dissolved_2d);
            }
        }
        dissolved
    }

    fn dissolve_2d(&self, buffered_features_2d: Vec<&Feature>) -> Option<Feature> {
        let multi_polygon_2d = buffered_features_2d.iter().fold(
            None,
            |multi_polygon_acc: Option<_>, feature_incoming| {
                let geometry_incoming = feature_incoming.geometry.value.as_flow_geometry_2d()?;
                let multi_polygon_incoming =
                    if let Some(multi_polygon) = geometry_incoming.as_multi_polygon() {
                        multi_polygon
                    } else if let Some(polygon) = geometry_incoming.as_polygon() {
                        MultiPolygon2D::new(vec![polygon])
                    } else {
                        return multi_polygon_acc;
                    };

                let mutli_polygon_acc = if let Some(mutli_polygon_acc) = multi_polygon_acc {
                    mutli_polygon_acc
                } else {
                    return Some(multi_polygon_incoming);
                };

                let unite = multi_polygon_incoming.union(&mutli_polygon_acc);
                Some(unite)
            },
        );

        if let Some(multi_polygon_2d) = multi_polygon_2d {
            let mut feature = Feature::new();
            if let (Some(group_by), Some(last_feature)) =
                (&self.group_by, buffered_features_2d.last())
            {
                feature.attributes = group_by
                    .iter()
                    .filter_map(|attr| {
                        let value = last_feature.attributes.get(attr).cloned()?;
                        Some((attr.clone(), value))
                    })
                    .collect::<IndexMap<_, _>>();
            } else {
                feature.attributes = IndexMap::new();
            }
            feature.geometry.value = GeometryValue::FlowGeometry2D(multi_polygon_2d.into());
            Some(feature)
        } else {
            None
        }
    }
}

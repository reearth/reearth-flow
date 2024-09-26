use std::collections::hash_map::Entry;
use std::collections::HashMap;

use itertools::Itertools;
use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::AttributeValue;
use reearth_flow_types::{Attribute, Geometry};
use reearth_flow_types::{Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));
pub static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

#[derive(Debug, Clone, Default)]
pub struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }

    fn description(&self) -> &str {
        "Overlays an area on another area"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaOnAreaOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            AREA_PORT.clone(),
            REMNANTS_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AreaOnAreaOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::AreaOnAreaOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(AreaOnAreaOverlayer {
            params,
            buffer: HashMap::new(),
            previous_group_key: None,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AreaOnAreaOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct AreaOnAreaOverlayer {
    params: AreaOnAreaOverlayerParam,
    buffer: HashMap<String, (bool, Vec<Feature>)>, // (complete_grouped, features)
    previous_group_key: Option<String>,
}

impl Processor for AreaOnAreaOverlayer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(geometry) = &feature.geometry else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::FlowGeometry2D(_) => {
                let key = if let Some(group_by) = &self.params.group_by {
                    group_by
                        .iter()
                        .map(|k| feature.get(&k).map(|v| v.to_string()).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    "_all".to_string()
                };

                match self.buffer.entry(key.clone()) {
                    Entry::Occupied(mut entry) => {
                        self.previous_group_key = Some(key.clone());
                        {
                            let (_, buffer) = entry.get_mut();
                            buffer.push(feature.clone());
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((false, vec![feature.clone()]));
                        if let Some(previous_group_key) = &self.previous_group_key {
                            if let Entry::Occupied(mut entry) =
                                self.buffer.entry(previous_group_key.clone())
                            {
                                let (complete_grouped_change, _) = entry.get_mut();
                                *complete_grouped_change = true;
                            }
                            self.change_group(ctx, fw);
                        }
                        self.previous_group_key = Some(key.clone());
                    }
                }
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (_, (_, features)) in self.buffer.iter() {
            self.handle_2d_geometry(features, ctx.as_context(), fw);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }
}

impl AreaOnAreaOverlayer {
    fn handle_2d_geometry(
        &self,
        targets: &[Feature],
        ctx: Context,
        fw: &mut dyn ProcessorChannelForwarder,
    ) {
        for comb in targets.iter().combinations(2) {
            let (target_feature, other_feature) = (comb[0], comb[1]);
            let target = self.handle_2d_polygon_and_multi_polygon(target_feature);
            let other = self.handle_2d_polygon_and_multi_polygon(other_feature);
            let (Some(target), Some(other)) = (target, other) else {
                continue;
            };
            for target in target.iter() {
                for other in other.iter() {
                    let inter = target.intersection(other);
                    if inter.is_empty() {
                        continue;
                    }
                    let diff = target.difference(other);
                    let mut feature = Feature::default();
                    feature.refresh_id();
                    feature.geometry = Some(Geometry {
                        epsg: target_feature.geometry.as_ref().unwrap().epsg,
                        value: GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(diff)),
                    });
                    feature.insert(
                        self.params.output_attribute.clone(),
                        AttributeValue::Number(2.into()),
                    );
                    feature.insert(
                        "features",
                        AttributeValue::Array(vec![
                            AttributeValue::Map(target_feature.to_map()),
                            AttributeValue::Map(other_feature.to_map()),
                        ]),
                    );
                    fw.send(ExecutorContext::new_with_context_feature_and_port(
                        &ctx,
                        feature,
                        AREA_PORT.clone(),
                    ));
                }
            }
        }
        for target in targets {
            let mut feature = target.clone();
            feature.attributes.insert(
                self.params.output_attribute.clone(),
                AttributeValue::Number(1.into()),
            );
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &ctx,
                feature,
                AREA_PORT.clone(),
            ));
        }
    }

    fn handle_2d_polygon_and_multi_polygon(
        &self,
        feature: &Feature,
    ) -> Option<MultiPolygon2D<f64>> {
        feature
            .geometry
            .as_ref()
            .and_then(|geometry| match &geometry.value {
                GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)) => {
                    Some(MultiPolygon2D::new(vec![poly.clone()]))
                }
                GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(mpoly)) => {
                    Some(mpoly.clone())
                }
                _ => None,
            })
    }

    fn change_group(&mut self, ctx: ExecutorContext, fw: &mut dyn ProcessorChannelForwarder) {
        let mut remove_keys = Vec::new();
        for (key, (complete_grouped, features)) in self.buffer.iter() {
            if !*complete_grouped {
                continue;
            }
            remove_keys.push(key.clone());
            self.handle_2d_geometry(features, ctx.as_context(), fw);
        }
        for key in remove_keys.iter() {
            self.buffer.remove(key);
        }
    }
}

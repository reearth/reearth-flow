use std::collections::HashMap;

use reearth_flow_geometry::algorithm::winding_order::Winding;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::{algorithm::winding_order::WindingOrder, types::geometry::Geometry2D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

type WindingOrderResult = &'static str;

const NO_ORIENTATION: WindingOrderResult = "no_orientation";
const INVALID_ORIENTATION: WindingOrderResult = "invalid_orientation";
const CLOCKWISE_ORIENTATION: WindingOrderResult = "clockwise";
const COUNTER_CLOCKWISE_ORIENTATION: WindingOrderResult = "counter_clockwise";

#[derive(Debug, Clone, Default)]
pub struct OrientationExtractorFactory;

impl ProcessorFactory for OrientationExtractorFactory {
    fn name(&self) -> &str {
        "OrientationExtractor"
    }

    fn description(&self) -> &str {
        "Extracts the orientation of a geometry from a feature and adds it as an attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(OrientationExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: OrientationExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::OrientationExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::OrientationExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::OrientationExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(OrientationExtractor {
            output_attribute: params.output_attribute,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrientationExtractorParam {
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct OrientationExtractor {
    output_attribute: Attribute,
}

impl Processor for OrientationExtractor {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            let mut feature = feature.clone();
            feature.attributes.insert(
                self.output_attribute.clone(),
                AttributeValue::String(NO_ORIENTATION.to_string()),
            );
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::String(NO_ORIENTATION.to_string()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geometry) => match geometry {
                Geometry2D::Polygon(polygon) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygon
                        .rings()
                        .iter()
                        .map(|ring| ring.winding_order())
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes.insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                Geometry2D::MultiPolygon(polygons) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygons
                        .iter()
                        .flat_map(|polygon| {
                            polygon
                                .rings()
                                .iter()
                                .map(|ring| ring.winding_order())
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes.insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                _ => unimplemented!(),
            },
            GeometryValue::FlowGeometry3D(geometry) => match geometry {
                Geometry3D::Polygon(polygon) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygon
                        .rings()
                        .iter()
                        .map(|ring| ring.winding_order())
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes.insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                Geometry3D::MultiPolygon(polygons) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygons
                        .iter()
                        .flat_map(|polygon| {
                            polygon
                                .rings()
                                .iter()
                                .map(|ring| ring.winding_order())
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes.insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone())),
            },
            GeometryValue::CityGmlGeometry(_) => unimplemented!(),
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "OrientationExtractor"
    }
}

fn detect_orientation_by_ring_winding_orders(
    ring_winding_orders: Vec<Option<WindingOrder>>,
) -> WindingOrderResult {
    if ring_winding_orders.is_empty() {
        return NO_ORIENTATION;
    }
    if !ring_winding_orders
        .iter()
        .all(|winding_order| winding_order.is_some())
    {
        return INVALID_ORIENTATION;
    }
    let ring_winding_orders = ring_winding_orders.iter().flatten().collect::<Vec<_>>();
    for ring_winding_order in ring_winding_orders.iter() {
        let orientation = match ring_winding_order {
            WindingOrder::Clockwise => CLOCKWISE_ORIENTATION,
            WindingOrder::CounterClockwise => COUNTER_CLOCKWISE_ORIENTATION,
            WindingOrder::None => NO_ORIENTATION,
        };
        if orientation == NO_ORIENTATION {
            return orientation;
        }
    }
    match ring_winding_orders.first().unwrap() {
        WindingOrder::Clockwise => CLOCKWISE_ORIENTATION,
        WindingOrder::CounterClockwise => COUNTER_CLOCKWISE_ORIENTATION,
        WindingOrder::None => NO_ORIENTATION,
    }
}

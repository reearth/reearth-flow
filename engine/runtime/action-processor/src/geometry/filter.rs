use std::collections::HashMap;

use inflector::cases::camelcase::to_camel_case;
use once_cell::sync::Lazy;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Feature, Geometry, GeometryType, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub struct GeometryFilterFactory;

impl ProcessorFactory for GeometryFilterFactory {
    fn name(&self) -> &str {
        "GeometryFilter"
    }

    fn description(&self) -> &str {
        "Filter geometry by type"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        let mut result = vec![UNFILTERED_PORT.clone()];
        result.extend(GeometryFilterParam::all_ports());
        result
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: GeometryFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryFilterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = GeometryFilter { params };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct GeometryFilter {
    params: GeometryFilterParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "filterType", rename_all = "camelCase")]
pub enum GeometryFilterParam {
    None,
    Multiple,
    GeometryType,
}

impl GeometryFilterParam {
    fn output_port(&self) -> Port {
        match self {
            GeometryFilterParam::None => Port::new("none"),
            GeometryFilterParam::Multiple => Port::new("contains"),
            GeometryFilterParam::GeometryType => unreachable!(),
        }
    }

    fn all_feature_type_ports() -> Vec<Port> {
        let mut result = reearth_flow_geometry::types::geometry::all_type_names()
            .iter()
            .map(|name| Port::new(to_camel_case(name)))
            .collect::<Vec<Port>>();
        result.extend(
            GeometryType::all_type_names()
                .iter()
                .map(|name| Port::new(to_camel_case(name)))
                .collect::<Vec<Port>>(),
        );
        result
    }

    fn all_ports() -> Vec<Port> {
        let mut result = vec![
            GeometryFilterParam::None.output_port(),
            GeometryFilterParam::Multiple.output_port(),
        ];
        result.extend(GeometryFilterParam::all_feature_type_ports());
        result
    }
}

impl Processor for GeometryFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        match self.params {
            GeometryFilterParam::None => match &feature.geometry.value {
                GeometryValue::None => fw.send(ctx.new_with_feature_and_port(
                    feature.clone(),
                    GeometryFilterParam::None.output_port(),
                )),
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
                }
            },
            GeometryFilterParam::Multiple => {
                if feature.geometry.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
                } else {
                    filter_multiple_geometry(&ctx, fw, feature, &feature.geometry)
                }
            }
            GeometryFilterParam::GeometryType => {
                if feature.geometry.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
                } else {
                    filter_geometry_type(&ctx, fw, feature, &feature.geometry)
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometryFilter"
    }
}

fn filter_multiple_geometry(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    geometry: &Geometry,
) {
    match &geometry.value {
        GeometryValue::None => {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
        }
        GeometryValue::FlowGeometry3D(geometry) => match geometry {
            Geometry3D::MultiPolygon(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            Geometry3D::GeometryCollection(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone())),
        },
        GeometryValue::FlowGeometry2D(geometry) => match geometry {
            Geometry2D::MultiPolygon(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            Geometry2D::GeometryCollection(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone())),
        },
        GeometryValue::CityGmlGeometry(geometry) => {
            if geometry.gml_geometries.len() > 1 {
                fw.send(ctx.new_with_feature_and_port(
                    feature.clone(),
                    GeometryFilterParam::Multiple.output_port(),
                ))
            } else {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
            }
        }
    }
}

fn filter_geometry_type(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    geometry: &Geometry,
) {
    match &geometry.value {
        GeometryValue::None => {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
        }
        GeometryValue::FlowGeometry3D(geometry) => {
            let geometry_type: GeometryType = geometry.into();
            fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                Port::new(to_camel_case(geometry_type.name())),
            ))
        }
        GeometryValue::FlowGeometry2D(geometry) => {
            let geometry_type: GeometryType = geometry.into();
            fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                Port::new(to_camel_case(geometry_type.name())),
            ))
        }
        GeometryValue::CityGmlGeometry(geometry) => {
            if geometry.gml_geometries.len() != 1 {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
            } else {
                let Some(first_feature) = geometry.gml_geometries.first() else {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()),
                    );
                    return;
                };
                fw.send(ctx.new_with_feature_and_port(
                    feature.clone(),
                    Port::new(to_camel_case(first_feature.name())),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    use crate::tests::utils::create_default_execute_context;

    use super::*;

    #[test]
    fn test_filter_multiple_geometry_null() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::default();
        let geometry = Geometry {
            value: GeometryValue::None,
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &geometry);
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(UNFILTERED_PORT.clone())
            );
        }
    }

    #[test]
    fn test_filter_multiple_geometry_3d_multipolygon() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(Default::default())),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &feature.geometry.clone());
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(GeometryFilterParam::Multiple.output_port())
            );
        }
    }

    #[test]
    fn test_filter_multiple_geometry_3d_geometry_collection() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::GeometryCollection(
                    Default::default(),
                )),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &feature.geometry.clone());
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(GeometryFilterParam::Multiple.output_port())
            );
        }
    }

    #[test]
    fn test_filter_multiple_geometry_3d_other_geometry() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Default::default())),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &feature.geometry.clone());
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(UNFILTERED_PORT.clone())
            );
        }
    }

    // Add more tests for other scenarios...
}

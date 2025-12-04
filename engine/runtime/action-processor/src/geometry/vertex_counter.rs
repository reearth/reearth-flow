use std::collections::HashMap;

use reearth_flow_geometry::algorithm::coords_iter::CoordsIter;
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

#[derive(Debug, Clone, Default)]
pub struct VertexCounterFactory;

impl ProcessorFactory for VertexCounterFactory {
    fn name(&self) -> &str {
        "VertexCounter"
    }

    fn description(&self) -> &str {
        "Count Geometry Vertices to Attribute"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(VertexCounterParam))
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
        let params: VertexCounterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::VertexCounterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::VertexCounterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::VertexCounterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(VertexCounter {
            output_attribute: params.output_attribute,
        }))
    }
}

/// # Vertex Counter Parameters
/// Configure where to store the count of vertices found in geometries
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VertexCounterParam {
    /// # Output Attribute
    /// Name of the attribute where the vertex count will be stored as a number
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct VertexCounter {
    output_attribute: Attribute,
}

impl Processor for VertexCounter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.coords_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.coords_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::CityGmlGeometry(geometry) => {
                let vertex_count: usize = geometry
                    .gml_geometries
                    .iter()
                    .map(|gml_feature| {
                        gml_feature
                            .polygons
                            .iter()
                            .map(|p| p.coords_count())
                            .sum::<usize>()
                            + gml_feature
                                .line_strings
                                .iter()
                                .map(|ls| ls.coords_count())
                                .sum::<usize>()
                    })
                    .sum();
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(vertex_count.into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "VertexCounter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use reearth_flow_geometry::types::{
        geometry::Geometry2D, line_string::LineString, polygon::Polygon,
    };
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{Attribute, Feature, Geometry, GeometryValue};

    #[test]
    fn test_vertex_counter_point_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        };

        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Default::default())),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            let result_feature = &noop.send_features.lock().unwrap()[0];
            assert_eq!(
                result_feature
                    .attributes
                    .get(&Attribute::new("vertexCount")),
                Some(&AttributeValue::Number(1.into()))
            );
        }
    }

    #[test]
    fn test_vertex_counter_linestring_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        };

        let line_string = LineString::from(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]);
        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(line_string)),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            let result_feature = &noop.send_features.lock().unwrap()[0];
            assert_eq!(
                result_feature
                    .attributes
                    .get(&Attribute::new("vertexCount")),
                Some(&AttributeValue::Number(3.into()))
            );
        }
    }

    #[test]
    fn test_vertex_counter_polygon_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        };

        // Create a polygon with 5 vertices in exterior ring (square with closing point)
        let exterior = LineString::from(vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (0.0, 0.0),
        ]);
        let polygon = Polygon::new(exterior, vec![]);
        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            let result_feature = &noop.send_features.lock().unwrap()[0];
            assert_eq!(
                result_feature
                    .attributes
                    .get(&Attribute::new("vertexCount")),
                Some(&AttributeValue::Number(5.into()))
            );
        }
    }

    #[test]
    fn test_vertex_counter_polygon_with_hole() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        };

        // Create a polygon with exterior and one interior ring (hole)
        let exterior = LineString::from(vec![
            (0.0, 0.0),
            (4.0, 0.0),
            (4.0, 4.0),
            (0.0, 4.0),
            (0.0, 0.0),
        ]);
        let interior = LineString::from(vec![
            (1.0, 1.0),
            (3.0, 1.0),
            (3.0, 3.0),
            (1.0, 3.0),
            (1.0, 1.0),
        ]);
        let polygon = Polygon::new(exterior, vec![interior]);
        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            let result_feature = &noop.send_features.lock().unwrap()[0];
            // 5 vertices in exterior + 5 vertices in hole = 10 total
            assert_eq!(
                result_feature
                    .attributes
                    .get(&Attribute::new("vertexCount")),
                Some(&AttributeValue::Number(10.into()))
            );
        }
    }

    #[test]
    fn test_vertex_counter_empty_geometry() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        };

        let feature = Feature {
            geometry: Geometry {
                value: GeometryValue::None,
                ..Default::default()
            },
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(noop.send_ports.lock().unwrap().len(), 1);
            let result_feature = &noop.send_features.lock().unwrap()[0];
            // Should pass through without adding attribute
            assert_eq!(
                result_feature
                    .attributes
                    .get(&Attribute::new("vertexCount")),
                None
            );
        }
    }
}

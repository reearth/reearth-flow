use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::coords_iter::CoordsIter;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::CountVertices;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct VertexCounterFactory;

impl ProcessorFactory for VertexCounterFactory {
    fn name(&self) -> &str {
        "Vertex Counter"
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
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
    /// Counts the coordinates the geometry stores, as stored: a ring keeps its
    /// closing vertex and a mesh counts its shared vertex pool once. A feature
    /// with no geometry passes through without the attribute, the way an empty
    /// geometry does, rather than claiming a count of zero.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let geometry = &ctx.feature.geometry;
        if matches!(**geometry, reearth_flow_geometry::Geometry::None) {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        }
        let count = geometry.count_vertices();
        let mut feature = ctx.feature.clone();
        feature.attributes_mut().insert(
            self.output_attribute.clone(),
            AttributeValue::Number(count.into()),
        );
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.coords_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                let mut feature = feature.clone();
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(geometry.coords_count().into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::Number(vertex_count.into()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Vertex Counter"
    }
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use reearth_flow_geometry::types::{
        geometry::Geometry2D, line_string::LineString, polygon::Polygon,
    };
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{feature::Attributes, Attribute, Feature, Geometry, GeometryValue};

    #[test]
    fn test_vertex_counter_point_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        };

        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Default::default())),
                ..Default::default()
            },
        );
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
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(line_string)),
                ..Default::default()
            },
        );
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
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
                ..Default::default()
            },
        );
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
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
                ..Default::default()
            },
        );
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

        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::None,
                ..Default::default()
            },
        );
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

#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    fn face(ring: Vec<[f64; 3]>) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    /// Run the processor over `feature`, returning the single feature it forwards.
    fn count(feature: Feature) -> Feature {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let ctx = create_default_execute_context(&feature);
        VertexCounter {
            output_attribute: Attribute::new("vertexCount"),
        }
        .process(ctx, &fw)
        .unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], *FEATURES_PORT);
        let features = noop.send_features.lock().unwrap();
        assert_eq!(features.len(), 1);
        features[0].clone()
    }

    fn vertex_count(feature: &Feature) -> Option<&AttributeValue> {
        feature.attributes.get(&Attribute::new("vertexCount"))
    }

    /// The closing vertex counts, matching the legacy path where a closed
    /// square reports five.
    #[test]
    fn a_closed_square_counts_its_repeated_first_vertex() {
        let feature = count(Feature::from(face(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ])));
        assert_eq!(
            vertex_count(&feature),
            Some(&AttributeValue::Number(5.into()))
        );
    }

    /// A triangle written without its closing vertex reports three, so a
    /// downstream check can tell it apart from the well-formed four.
    #[test]
    fn an_open_triangle_counts_three() {
        let feature = count(Feature::from(face(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ])));
        assert_eq!(
            vertex_count(&feature),
            Some(&AttributeValue::Number(3.into()))
        );
    }

    #[test]
    fn a_feature_without_geometry_passes_through_uncounted() {
        let feature = count(Feature::from(Geometry::None));
        assert_eq!(vertex_count(&feature), None);
    }
}

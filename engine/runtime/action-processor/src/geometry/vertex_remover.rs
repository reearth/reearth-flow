use std::collections::HashMap;

use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::utils::remove_redundant_vertices;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::Geometry;
use reearth_flow_types::{Feature, GeometryValue};
use serde_json::Value;

const EPSILON: f64 = 0.0001;

#[derive(Debug, Clone, Default)]
pub struct VertexRemoverFactory;

impl ProcessorFactory for VertexRemoverFactory {
    fn name(&self) -> &str {
        "VertexRemover"
    }

    fn description(&self) -> &str {
        "Remove Redundant Vertices from Geometry"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(VertexRemover))
    }
}

#[derive(Debug, Clone)]
pub struct VertexRemover;

impl Processor for VertexRemover {
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
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "VertexRemover"
    }
}

impl VertexRemover {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry2D::LineString(line_string) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let line_string: LineString2D<f64> = line_string.clone();
                geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::LineString(
                    remove_redundant_vertices(&line_string, EPSILON),
                ));
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry2D::MultiLineString(mline_string) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let line_strings: Vec<LineString2D<f64>> = mline_string.iter().cloned().collect();
                geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::MultiLineString(
                    line_strings
                        .iter()
                        .map(|line_string| remove_redundant_vertices(line_string, EPSILON))
                        .collect(),
                ));
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry3D::LineString(line_string) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let line_string: LineString3D<f64> = line_string.clone();
                geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::LineString(
                    remove_redundant_vertices(&line_string, EPSILON),
                ));
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            Geometry3D::MultiLineString(mline_string) => {
                let mut feature = feature.clone();
                let mut geometry = geometry.clone();
                let line_strings: Vec<LineString3D<f64>> = mline_string.iter().cloned().collect();
                geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::MultiLineString(
                    line_strings
                        .iter()
                        .map(|line_string| remove_redundant_vertices(line_string, EPSILON))
                        .collect(),
                ));
                feature.geometry = geometry;
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
    }
}

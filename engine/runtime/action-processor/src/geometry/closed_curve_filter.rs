use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D;
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Feature, GeometryValue};
use serde_json::Value;

static CLOSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("closed"));
static OPEN_PORT: Lazy<Port> = Lazy::new(|| Port::new("open"));

#[derive(Debug, Clone, Default)]
pub(super) struct ClosedCurveFilterFactory;

impl ProcessorFactory for ClosedCurveFilterFactory {
    fn name(&self) -> &str {
        "ClosedCurveFilter"
    }

    fn description(&self) -> &str {
        "Filter LineString Features by Closed/Open Status"
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
        vec![
            CLOSED_PORT.clone(),
            OPEN_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(ClosedCurveFilter))
    }
}

#[derive(Debug, Clone)]
struct ClosedCurveFilter;

impl Processor for ClosedCurveFilter {
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
                self.handle_2d_geometry(geos, feature, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, &ctx, fw);
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
        "ClosedCurveFilter"
    }
}

impl ClosedCurveFilter {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry2D::MultiLineString(line_strings) => {
                self.handle_2d_line_strings(
                    feature,
                    line_strings.into_iter().cloned().collect(),
                    ctx,
                    fw,
                );
            }
            Geometry2D::LineString(line_string) => {
                self.handle_2d_line_strings(feature, vec![line_string.clone()], ctx, fw);
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
    }

    fn handle_2d_line_strings(
        &self,
        feature: &Feature,
        lines: Vec<LineString2D<f64>>,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let feature = feature.clone();
        if lines.iter().all(|line| line.is_closed()) {
            fw.send(ctx.new_with_feature_and_port(feature, CLOSED_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(feature, OPEN_PORT.clone()));
        }
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match geos {
            Geometry3D::MultiLineString(line_strings) => {
                self.handle_3d_line_strings(
                    feature,
                    line_strings.into_iter().cloned().collect(),
                    ctx,
                    fw,
                );
            }
            Geometry3D::LineString(line_string) => {
                self.handle_3d_line_strings(feature, vec![line_string.clone()], ctx, fw);
            }
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
        }
    }

    fn handle_3d_line_strings(
        &self,
        feature: &Feature,
        lines: Vec<LineString3D<f64>>,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let feature = feature.clone();
        if lines.iter().all(|line| line.is_closed()) {
            fw.send(ctx.new_with_feature_and_port(feature, CLOSED_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(feature, OPEN_PORT.clone()));
        }
    }
}

use std::collections::HashMap;
#[cfg(feature = "new-geometry")]
use std::num::NonZeroUsize;
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{SelectVertices, VertexRange, VertexSelection};
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
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::Geometry;
use reearth_flow_types::{Feature, GeometryValue};
#[cfg(feature = "new-geometry")]
use schemars::JsonSchema;
#[cfg(feature = "new-geometry")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "new-geometry")]
use super::errors::GeometryProcessorError;

const EPSILON: f64 = 0.0001;

#[derive(Debug, Clone, Default)]
pub struct VertexRemoverFactory;

impl ProcessorFactory for VertexRemoverFactory {
    fn name(&self) -> &str {
        "Vertex Remover"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn description(&self) -> &str {
        "Remove Redundant Vertices from Geometry"
    }

    #[cfg(feature = "new-geometry")]
    fn description(&self) -> &str {
        "Keeps or removes a range of vertices, selected by position, from a point, line string or \
         polygon, leaving the remaining vertices as a point or a line string."
    }

    #[cfg(not(feature = "new-geometry"))]
    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    #[cfg(feature = "new-geometry")]
    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(VertexRemoverParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    #[cfg(not(feature = "new-geometry"))]
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(VertexRemover))
    }

    #[cfg(feature = "new-geometry")]
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: VertexRemoverParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::VertexRemoverFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::VertexRemoverFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::VertexRemoverFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(VertexRemover { params }))
    }
}

/// Whether the vertices in the range are the ones kept or the ones removed.
#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum SelectedVertices {
    /// # Keep
    /// Keeps only the vertices in the range and removes all others.
    Keep,
    /// # Remove
    /// Removes the vertices in the range and keeps all others. Removing every vertex leaves the
    /// feature with no geometry.
    Remove,
}

/// # Vertex Remover Parameters
/// Selects a range of vertices by position and whether to keep or remove them.
#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct VertexRemoverParam {
    /// # Selected Vertices
    /// Whether the vertices in the range are kept or removed.
    selected_vertices: SelectedVertices,
    /// # Start Index
    /// Zero-based position of the first vertex in the range; a negative index counts back from
    /// the end, so -1 is the last vertex. A polygon numbers its exterior ring and then each
    /// interior ring, counting each ring's closing vertex.
    start_index: i64,
    /// # Count
    /// Number of vertices in the range, counted from the start index toward the end. A range
    /// that runs past the last vertex stops there.
    count: NonZeroUsize,
}

#[cfg(not(feature = "new-geometry"))]
#[derive(Debug, Clone)]
pub struct VertexRemover;

#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
pub struct VertexRemover {
    params: VertexRemoverParam,
}

impl Processor for VertexRemover {
    fn num_threads(&self) -> usize {
        2
    }

    /// A geometry other than a point, line string or polygon, an absent
    /// geometry, or a start index outside the vertices leaves via `rejected`.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let selection = match self.params.selected_vertices {
            SelectedVertices::Keep => VertexSelection::Keep,
            SelectedVertices::Remove => VertexSelection::Remove,
        };
        let range = VertexRange {
            start: self.params.start_index,
            count: self.params.count,
        };
        match ctx.feature.geometry.select_vertices(selection, range) {
            Ok(geometry) => {
                let mut feature = ctx.feature.clone();
                feature.set_geometry(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Err(reason) => {
                ctx.event_hub.debug_log(
                    Some(ctx.error_span()),
                    format!("vertex removal rejected: {reason}"),
                );
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
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

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Vertex Remover"
    }
}

impl VertexRemover {
    #[cfg(not(feature = "new-geometry"))]
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
                feature.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
                feature.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
    }

    #[cfg(not(feature = "new-geometry"))]
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
                feature.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
                feature.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
    }
}

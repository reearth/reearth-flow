use std::collections::HashMap;

use itertools::Itertools;
use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::clipper::{Clipper2D, Clipper3D};
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory},
};
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

static CLIPPER_PORT: Lazy<Port> = Lazy::new(|| Port::new("clipper"));
static CANDIDATE_PORT: Lazy<Port> = Lazy::new(|| Port::new("candidate"));
static INSIDE_PORT: Lazy<Port> = Lazy::new(|| Port::new("inside"));
static OUTSIDE_PORT: Lazy<Port> = Lazy::new(|| Port::new("outside"));

#[derive(Debug, Clone, Default)]
pub(super) struct ClipperFactory;

impl ProcessorFactory for ClipperFactory {
    fn name(&self) -> &str {
        "Clipper"
    }

    fn description(&self) -> &str {
        "Divides Candidate features using Clipper features, so that Candidates and parts of Candidates that are inside or outside of the Clipper features are output separately"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![CLIPPER_PORT.clone(), CANDIDATE_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            INSIDE_PORT.clone(),
            OUTSIDE_PORT.clone(),
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
        Ok(Box::new(Clipper {
            clippers: Vec::new(),
            candidates: Vec::new(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Clipper {
    clippers: Vec<Feature>,
    candidates: Vec<Feature>,
}

impl Processor for Clipper {
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
            GeometryValue::FlowGeometry2D(_) | GeometryValue::FlowGeometry3D(_) => {
                match &ctx.port {
                    port if port == &*CLIPPER_PORT => self.clippers.push(feature.clone()),
                    port if port == &*CANDIDATE_PORT => self.candidates.push(feature.clone()),
                    _ => {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
                    }
                }
            }
            GeometryValue::CityGmlGeometry(_) => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()))
            }
        }
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let clip_regions2d = self
            .clippers
            .iter()
            .filter_map(|g| match &g.geometry.value {
                GeometryValue::FlowGeometry2D(geos) => Some(geos),
                _ => None,
            })
            .collect_vec();
        let clip_regions2d = clip_regions2d
            .iter()
            .flat_map(|g| match g {
                Geometry2D::Polygon(poly) => Some(poly.clone()),
                _ => None,
            })
            .collect_vec();
        let clip_regions3d = self
            .clippers
            .iter()
            .filter_map(|g| match &g.geometry.value {
                GeometryValue::FlowGeometry3D(geos) => Some(geos),
                _ => None,
            })
            .collect_vec();
        let clip_regions3d = clip_regions3d
            .iter()
            .flat_map(|g| match g {
                Geometry3D::Polygon(poly) => Some(poly.clone()),
                _ => None,
            })
            .collect_vec();
        if clip_regions2d.is_empty() && clip_regions3d.is_empty() {
            for candidate in &self.candidates {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    candidate.clone(),
                    REJECTED_PORT.clone(),
                ));
            }
            for clip in &self.clippers {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    clip.clone(),
                    REJECTED_PORT.clone(),
                ));
            }
            return Ok(());
        }
        for candidate in &self.candidates {
            let geometry = candidate.geometry.value.clone();
            match geometry {
                GeometryValue::FlowGeometry2D(geos) => {
                    handle_2d_geometry(
                        &geos,
                        &clip_regions2d,
                        candidate,
                        &candidate.geometry,
                        &ctx,
                        fw,
                    );
                }
                GeometryValue::FlowGeometry3D(geos) => {
                    handle_3d_geometry(
                        &geos,
                        &clip_regions3d,
                        candidate,
                        &candidate.geometry,
                        &ctx,
                        fw,
                    );
                }
                _ => {
                    fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                        &ctx,
                        candidate.clone(),
                        REJECTED_PORT.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Clipper"
    }
}

fn handle_2d_geometry(
    geos: &Geometry2D,
    clip_regions: &[Polygon2D<f64>],
    feature: &Feature,
    geometry: &Geometry,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    match geos {
        Geometry2D::Polygon(poly) => {
            let (insides, outsides) = clip_polygon2d(poly, clip_regions);
            forward_polygon2d(&insides, &outsides, feature, geometry, ctx, fw);
        }
        Geometry2D::MultiPolygon(mpoly) => {
            let (insides, outsides) = clip_mpolygon2d(mpoly, clip_regions);
            forward_polygon2d(&insides, &outsides, feature, geometry, ctx, fw);
        }
        Geometry2D::GeometryCollection(collection) => {
            for single in collection {
                handle_2d_geometry(single, clip_regions, feature, geometry, ctx, fw)
            }
        }
        _ => {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            ));
        }
    }
}

fn forward_polygon2d(
    insides: &[Polygon2D<f64>],
    outsides: &[Polygon2D<f64>],
    feature: &Feature,
    geometry: &Geometry,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    for inside in insides {
        let mut feature = feature.clone();
        let mut geometry = geometry.clone();
        geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(inside.clone()));
        feature.geometry = geometry;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            feature,
            INSIDE_PORT.clone(),
        ));
    }
    for outside in outsides {
        let mut feature = feature.clone();
        let mut geometry = geometry.clone();
        geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(outside.clone()));
        feature.geometry = geometry;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            feature,
            OUTSIDE_PORT.clone(),
        ));
    }
}

fn handle_3d_geometry(
    geos: &Geometry3D,
    clip_regions: &[Polygon3D<f64>],
    feature: &Feature,
    geometry: &Geometry,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    match geos {
        Geometry3D::Polygon(poly) => {
            let (insides, outsides) = clip_polygon3d(poly, clip_regions);
            forward_polygon3d(&insides, &outsides, feature, geometry, ctx, fw);
        }
        Geometry3D::MultiPolygon(mpoly) => {
            let (insides, outsides) = clip_mpolygon3d(mpoly, clip_regions);
            forward_polygon3d(&insides, &outsides, feature, geometry, ctx, fw);
        }
        Geometry3D::GeometryCollection(collection) => {
            for single in collection {
                handle_3d_geometry(single, clip_regions, feature, geometry, ctx, fw)
            }
        }
        _ => {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            ));
        }
    }
}

fn forward_polygon3d(
    insides: &[Polygon3D<f64>],
    outsides: &[Polygon3D<f64>],
    feature: &Feature,
    geometry: &Geometry,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    for inside in insides {
        let mut feature = feature.clone();
        let mut geometry = geometry.clone();
        geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::Polygon(inside.clone()));
        feature.geometry = geometry;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            feature,
            INSIDE_PORT.clone(),
        ));
    }
    for outside in outsides {
        let mut feature = feature.clone();
        let mut geometry = geometry.clone();
        geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::Polygon(outside.clone()));
        feature.geometry = geometry;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            feature,
            OUTSIDE_PORT.clone(),
        ));
    }
}

fn clip_polygon2d(
    polygon: &Polygon2D<f64>,
    clip_regions: &[Polygon2D<f64>],
) -> (Vec<Polygon2D<f64>>, Vec<Polygon2D<f64>>) {
    let mut inside = MultiPolygon2D::new(vec![polygon.clone()]);
    let mut outside = MultiPolygon2D::new(vec![polygon.clone()]);
    for clip in clip_regions {
        inside = inside.intersection2d(clip, 1.0);
        outside = outside.difference2d(clip, 1.0);
    }
    (
        inside.iter().cloned().collect(),
        outside.iter().cloned().collect(),
    )
}

fn clip_mpolygon2d(
    mpolygon: &MultiPolygon2D<f64>,
    clip_regions: &[Polygon2D<f64>],
) -> (Vec<Polygon2D<f64>>, Vec<Polygon2D<f64>>) {
    let mut inside = mpolygon.clone();
    let mut outside = mpolygon.clone();
    for clip in clip_regions {
        inside = inside.intersection2d(clip, 1.0);
        outside = outside.difference2d(clip, 1.0);
    }
    (
        inside.iter().cloned().collect(),
        outside.iter().cloned().collect(),
    )
}

fn clip_polygon3d(
    polygon: &Polygon3D<f64>,
    clip_regions: &[Polygon3D<f64>],
) -> (Vec<Polygon3D<f64>>, Vec<Polygon3D<f64>>) {
    let mut inside = MultiPolygon3D::new(vec![polygon.clone()]);
    let mut outside = MultiPolygon3D::new(vec![polygon.clone()]);
    for clip in clip_regions {
        inside = inside.intersection3d(clip, 1.0);
        outside = outside.difference3d(clip, 1.0);
    }
    (
        inside.iter().cloned().collect(),
        outside.iter().cloned().collect(),
    )
}

fn clip_mpolygon3d(
    mpolygon: &MultiPolygon3D<f64>,
    clip_regions: &[Polygon3D<f64>],
) -> (Vec<Polygon3D<f64>>, Vec<Polygon3D<f64>>) {
    let mut inside = mpolygon.clone();
    let mut outside = mpolygon.clone();
    for clip in clip_regions {
        inside = inside.intersection3d(clip, 1.0);
        outside = outside.difference3d(clip, 1.0);
    }
    (
        inside.iter().cloned().collect(),
        outside.iter().cloned().collect(),
    )
}

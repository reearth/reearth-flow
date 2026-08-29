use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

use once_cell::sync::Lazy;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{ExtractHoles, ExtractedPart};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    polygon::{Polygon2D, Polygon3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Feature, GeometryValue};
use serde_json::Value;

pub static EXTERIOR_PORT: Lazy<Port> = Lazy::new(|| Port::new("exterior"));
pub static HOLE_PORT: Lazy<Port> = Lazy::new(|| Port::new("hole"));

#[derive(Debug, Clone, Default)]
pub struct HoleExtractorFactory;

impl ProcessorFactory for HoleExtractorFactory {
    fn name(&self) -> &str {
        "Hole Extractor"
    }

    fn description(&self) -> &str {
        "Splits each face of a geometry into its rings, emitting the exterior ring and \
         every interior ring (hole) as a feature of its own."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            EXTERIOR_PORT.clone(),
            HOLE_PORT.clone(),
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
        Ok(Box::new(HoleExtractor))
    }
}

#[derive(Debug, Clone)]
pub struct HoleExtractor;

impl Processor for HoleExtractor {
    /// Take every face of the geometry apart, sending its exterior ring (holes
    /// removed) to `exterior` and each hole, as an area of its own, to `hole`.
    /// A face without holes still leaves via `exterior`.
    ///
    /// Geometry that bounds no area has nothing to take apart and leaves via
    /// `rejected`, as does a feature with no geometry. A multi-part geometry is
    /// deaggregated, so a member that bounds no area is rejected on its own
    /// rather than discarding the areas beside it.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let context = ctx.as_context();
        // Parts stream out as they are produced, so a large surface is never
        // decomposed into memory all at once.
        let result = ctx.feature.geometry.extract_holes(&mut |geometry, part| {
            let port = match part {
                ExtractedPart::Outershell => EXTERIOR_PORT.clone(),
                ExtractedPart::Hole => HOLE_PORT.clone(),
                ExtractedPart::Rejected => REJECTED_PORT.clone(),
            };
            let mut feature = ctx.feature.clone();
            // One input yields many features, so each needs an id of its own.
            feature.refresh_id();
            feature.set_geometry(geometry);
            fw.send(ExecutorContext::new_with_context_feature_and_port(
                &context, feature, port,
            ));
        });
        if let Err(e) = result {
            // Rejecting geometry that bounds no area is this port's normal
            // business, so it is not worth a warning per feature.
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                format!("hole extraction rejected: {e}"),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
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
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()))
            }
            GeometryValue::FlowGeometry2D(geometry) => match geometry {
                Geometry2D::Polygon(polygon) => {
                    handle_polygon2d(polygon, feature, &ctx, fw);
                }
                Geometry2D::MultiPolygon(mpolygon) => {
                    for polygon in mpolygon.iter() {
                        handle_polygon2d(polygon, feature, &ctx, fw);
                    }
                }
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
            GeometryValue::FlowGeometry3D(geometry) => match geometry {
                Geometry3D::Polygon(polygon) => {
                    handle_polygon3d(polygon, feature, &ctx, fw);
                }
                Geometry3D::MultiPolygon(mpolygon) => {
                    for polygon in mpolygon.iter() {
                        handle_polygon3d(polygon, feature, &ctx, fw);
                    }
                }
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
            GeometryValue::CityGmlGeometry(geometry) => {
                for geo_feature in geometry.gml_geometries.iter() {
                    for polygon in &geo_feature.polygons {
                        handle_polygon3d(polygon, feature, &ctx, fw);
                    }
                }
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
        "Hole Extractor"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::collection::Collection3D;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};

    /// A closed 4x4 square, as an exterior ring.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// A closed square hole ring of side 1, with its lower-left corner at `(x, y)`.
    fn hole_ring(x: f64, y: f64) -> Vec<[f64; 3]> {
        vec![
            [x, y, 0.0],
            [x, y + 1.0, 0.0],
            [x + 1.0, y + 1.0, 0.0],
            [x + 1.0, y, 0.0],
            [x, y, 0.0],
        ]
    }

    fn face_with_holes(n: usize) -> Geometry {
        let holes: Vec<_> = (0..n)
            .map(|i| hole_ring(1.0 + i as f64 * 1.5, 1.0))
            .collect();
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(CoordinateFrame::Euclidean, SQUARE, holes),
        )))
    }

    fn point() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]))
    }

    /// A feature carrying `geometry` and one attribute to trace through.
    fn feature(geometry: Geometry) -> Feature {
        let mut feature = Feature::from(geometry);
        feature.insert("surfaceId", AttributeValue::Number(7.into()));
        feature
    }

    /// Run the processor over `feature`, returning what it sent, port by port.
    fn extract(feature: &Feature) -> Vec<(Port, Feature)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        HoleExtractor
            .process(create_default_execute_context(feature), &fw)
            .unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    fn ports(sent: &[(Port, Feature)]) -> Vec<String> {
        sent.iter().map(|(port, _)| port.to_string()).collect()
    }

    fn exterior(feature: &Feature) -> Vec<[f64; 3]> {
        match &*feature.geometry {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => p.exterior().to_vec(),
            other => panic!("expected a 3D polygon, got {other:?}"),
        }
    }

    #[test]
    fn a_donut_leaves_as_an_exterior_ring_and_one_feature_per_hole() {
        let input = feature(face_with_holes(2));
        let sent = extract(&input);

        assert_eq!(ports(&sent), ["exterior", "hole", "hole"]);
        assert_eq!(exterior(&sent[0].1), SQUARE);
        assert_eq!(exterior(&sent[1].1), hole_ring(1.0, 1.0));
        assert_eq!(exterior(&sent[2].1), hole_ring(2.5, 1.0));
    }

    #[test]
    fn a_non_donut_area_leaves_untouched_via_its_exterior_ring() {
        let sent = extract(&feature(face_with_holes(0)));
        assert_eq!(ports(&sent), ["exterior"]);
        assert_eq!(exterior(&sent[0].1), SQUARE);
    }

    #[test]
    fn every_part_keeps_the_attributes_and_takes_an_id_of_its_own() {
        let input = feature(face_with_holes(1));
        let sent = extract(&input);

        for (_, part) in &sent {
            assert_eq!(
                part.attributes.get(&Attribute::new("surfaceId")),
                Some(&AttributeValue::Number(7.into()))
            );
            assert_ne!(part.id, input.id, "a part needs an id of its own");
        }
        assert_ne!(sent[0].1.id, sent[1].1.id);
    }

    #[test]
    fn geometry_that_bounds_no_area_is_rejected_whole() {
        let input = feature(Geometry::Euclidean3D(point()));
        let sent = extract(&input);
        assert_eq!(ports(&sent), ["rejected"]);
        // One in, one out: the feature is passed through as it arrived.
        assert_eq!(sent[0].1.id, input.id);
        assert_eq!(&*sent[0].1.geometry, &*input.geometry);
    }

    #[test]
    fn a_feature_without_geometry_is_rejected() {
        let sent = extract(&feature(Geometry::None));
        assert_eq!(ports(&sent), ["rejected"]);
    }

    // Deaggregating must not let one point discard the area beside it.
    #[test]
    fn a_multi_part_geometry_rejects_only_the_parts_that_bound_no_area() {
        let Geometry::Euclidean3D(area) = face_with_holes(1) else {
            unreachable!("built as a 3D polygon");
        };
        let collection =
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                area,
                point(),
            ])));
        let sent = extract(&feature(collection));

        assert_eq!(ports(&sent), ["exterior", "hole", "rejected"]);
        assert_eq!(
            &*sent[2].1.geometry,
            &Geometry::Euclidean3D(point()),
            "the rejected part carries the member that was rejected"
        );
    }
}

#[cfg(not(feature = "new-geometry"))]
fn handle_polygon2d(
    polygon: &Polygon2D<f64>,
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    let exterior = polygon.exterior();
    let exterior_polygon = Polygon2D::new(exterior.clone(), vec![]);
    let mut exterior_feature = feature.clone();
    exterior_feature.refresh_id();
    let mut exterior_geometry = (*feature.geometry).clone();
    exterior_geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(exterior_polygon));
    exterior_feature.geometry = Arc::new(exterior_geometry);
    fw.send(ctx.new_with_feature_and_port(exterior_feature, EXTERIOR_PORT.clone()));
    for interior in polygon.interiors().iter() {
        let interior_polygon = Polygon2D::new(interior.clone(), vec![]);
        let mut interior_feature = feature.clone();
        interior_feature.refresh_id();
        let mut interior_geometry = (*feature.geometry).clone();
        interior_geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::Polygon(interior_polygon));
        interior_feature.geometry = Arc::new(interior_geometry);
        fw.send(ctx.new_with_feature_and_port(interior_feature, HOLE_PORT.clone()));
    }
}

#[cfg(not(feature = "new-geometry"))]
fn handle_polygon3d(
    polygon: &Polygon3D<f64>,
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    let exterior = polygon.exterior();
    let exterior_polygon = Polygon3D::new(exterior.clone(), vec![]);
    let mut exterior_feature = feature.clone();
    exterior_feature.refresh_id();
    let mut exterior_geometry = (*feature.geometry).clone();
    exterior_geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::Polygon(exterior_polygon));
    exterior_feature.geometry = Arc::new(exterior_geometry);
    fw.send(ctx.new_with_feature_and_port(exterior_feature, EXTERIOR_PORT.clone()));
    for interior in polygon.interiors().iter() {
        let interior_polygon = Polygon3D::new(interior.clone(), vec![]);
        let mut interior_feature = feature.clone();
        interior_feature.refresh_id();
        let mut interior_geometry = (*feature.geometry).clone();
        interior_geometry.value =
            GeometryValue::FlowGeometry3D(Geometry3D::Polygon(interior_polygon));
        interior_feature.geometry = Arc::new(interior_geometry);
        fw.send(ctx.new_with_feature_and_port(interior_feature, HOLE_PORT.clone()));
    }
}

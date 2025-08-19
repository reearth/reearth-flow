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
use reearth_flow_types::{CityGmlGeometry, Feature, Geometry, GeometryValue};
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
        "Clip Features Using Boundary Shapes"
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
            GeometryValue::CityGmlGeometry(_) => match &ctx.port {
                port if port == &*CLIPPER_PORT => self.clippers.push(feature.clone()),
                port if port == &*CANDIDATE_PORT => self.candidates.push(feature.clone()),
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            },
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

        // Extract 3D clip regions from both FlowGeometry3D and CityGmlGeometry
        let clip_regions3d = self
            .clippers
            .iter()
            .filter_map(|g| match &g.geometry.value {
                GeometryValue::FlowGeometry3D(geos) => Some(geos),
                _ => None,
            })
            .collect_vec();
        let clip_regions3d_from_flow = clip_regions3d
            .iter()
            .flat_map(|g| match g {
                Geometry3D::Polygon(poly) => Some(poly.clone()),
                _ => None,
            })
            .collect_vec();

        // Extract polygons from CityGML geometries for clipping
        let clip_regions3d_from_citygml = self
            .clippers
            .iter()
            .filter_map(|g| match &g.geometry.value {
                GeometryValue::CityGmlGeometry(citygml) => Some(citygml),
                _ => None,
            })
            .flat_map(|citygml| {
                citygml
                    .gml_geometries
                    .iter()
                    .flat_map(|gml| gml.polygons.clone())
            })
            .collect_vec();

        let clip_regions3d: Vec<Polygon3D<f64>> = clip_regions3d_from_flow
            .into_iter()
            .chain(clip_regions3d_from_citygml)
            .collect();
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
                GeometryValue::CityGmlGeometry(citygml) => {
                    handle_citygml_geometry(
                        &citygml,
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

/// Clips a 3D polygon against multiple clip regions.
///
/// IMPORTANT: This implementation performs surface-level clipping on 3D polygons,
/// not true volumetric boolean operations. The underlying Clipper2 library
/// processes polygons in 2D space while preserving Z-coordinates.
///
/// Limitations:
/// - This is essentially 2.5D clipping (2D operations with Z preservation)
/// - Does NOT perform true 3D solid/volume intersections
/// - Suitable for clipping polygon surfaces in 3D space
/// - Not suitable for CSG (Constructive Solid Geometry) operations on 3D solids
///
/// Use cases:
/// - Clipping building facades against boundary polygons
/// - Extracting portions of 3D surfaces within a region
/// - Filtering 3D polygons by spatial boundaries
///
/// For true 3D volumetric operations, a different library would be required.
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

/// Clips a 3D multi-polygon against multiple clip regions.
///
/// See `clip_polygon3d` for important limitations regarding 3D clipping.
/// This function applies the same 2.5D clipping approach to multiple polygons.
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

fn process_gml_geometry(
    gml_geometry: &reearth_flow_types::GmlGeometry,
    clip_regions: &[Polygon3D<f64>],
    inside_gml_geometries: &mut Vec<reearth_flow_types::GmlGeometry>,
    outside_gml_geometries: &mut Vec<reearth_flow_types::GmlGeometry>,
) {
    let mut inside_polygons = Vec::new();
    let mut outside_polygons = Vec::new();

    // Clip each polygon in the GML geometry
    for polygon in &gml_geometry.polygons {
        let (insides, outsides) = clip_polygon3d(polygon, clip_regions);
        inside_polygons.extend(insides);
        outside_polygons.extend(outsides);
    }

    // Process line_strings if they exist (for Curve type geometries)
    // Note: Line strings cannot be clipped in the same way as polygons,
    // so we keep them as-is
    let line_strings = gml_geometry.line_strings.clone();

    // Process composite surfaces recursively
    let mut inside_composite_surfaces = Vec::new();
    let mut outside_composite_surfaces = Vec::new();

    for composite_surface in &gml_geometry.composite_surfaces {
        let mut temp_inside = Vec::new();
        let mut temp_outside = Vec::new();
        process_gml_geometry(
            composite_surface,
            clip_regions,
            &mut temp_inside,
            &mut temp_outside,
        );
        inside_composite_surfaces.extend(temp_inside);
        outside_composite_surfaces.extend(temp_outside);
    }

    // Check if we have Curve type with line_strings but no polygons
    let is_curve_without_polygons = gml_geometry.ty == reearth_flow_types::GeometryType::Curve
        && inside_polygons.is_empty()
        && outside_polygons.is_empty()
        && !line_strings.is_empty();

    // Create new GML geometries for inside results
    if !inside_polygons.is_empty()
        || !inside_composite_surfaces.is_empty()
        || is_curve_without_polygons
    {
        let mut inside_gml = gml_geometry.clone();
        inside_gml.polygons = inside_polygons;
        inside_gml.composite_surfaces = inside_composite_surfaces;
        // Keep line_strings as-is for Curve type geometries
        if gml_geometry.ty == reearth_flow_types::GeometryType::Curve {
            inside_gml.line_strings = line_strings.clone();
        }
        inside_gml_geometries.push(inside_gml);
    }

    // Create new GML geometries for outside results
    if !outside_polygons.is_empty() || !outside_composite_surfaces.is_empty() {
        let mut outside_gml = gml_geometry.clone();
        outside_gml.polygons = outside_polygons;
        outside_gml.composite_surfaces = outside_composite_surfaces;
        // Line strings are not included in outside results for non-Curve types
        if is_curve_without_polygons {
            outside_gml.line_strings = line_strings;
        } else {
            outside_gml.line_strings = vec![];
        }
        outside_gml_geometries.push(outside_gml);
    }
}

fn handle_citygml_geometry(
    citygml: &CityGmlGeometry,
    clip_regions: &[Polygon3D<f64>],
    feature: &Feature,
    geometry: &Geometry,
    ctx: &NodeContext,
    fw: &ProcessorChannelForwarder,
) {
    // Process each GML geometry in the CityGML
    let mut inside_gml_geometries = Vec::new();
    let mut outside_gml_geometries = Vec::new();

    for gml_geometry in &citygml.gml_geometries {
        process_gml_geometry(
            gml_geometry,
            clip_regions,
            &mut inside_gml_geometries,
            &mut outside_gml_geometries,
        );
    }

    // Send inside CityGML features
    if !inside_gml_geometries.is_empty() {
        let mut feature = feature.clone();
        let mut geometry = geometry.clone();
        let inside_citygml = CityGmlGeometry {
            gml_geometries: inside_gml_geometries,
            materials: citygml.materials.clone(),
            textures: citygml.textures.clone(),
            polygon_materials: citygml.polygon_materials.clone(),
            polygon_textures: citygml.polygon_textures.clone(),
            polygon_uvs: citygml.polygon_uvs.clone(),
        };
        geometry.value = GeometryValue::CityGmlGeometry(inside_citygml);
        feature.geometry = geometry;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            feature,
            INSIDE_PORT.clone(),
        ));
    }

    // Send outside CityGML features
    if !outside_gml_geometries.is_empty() {
        let mut feature = feature.clone();
        let mut geometry = geometry.clone();
        let outside_citygml = CityGmlGeometry {
            gml_geometries: outside_gml_geometries,
            materials: citygml.materials.clone(),
            textures: citygml.textures.clone(),
            polygon_materials: citygml.polygon_materials.clone(),
            polygon_textures: citygml.polygon_textures.clone(),
            polygon_uvs: citygml.polygon_uvs.clone(),
        };
        geometry.value = GeometryValue::CityGmlGeometry(outside_citygml);
        feature.geometry = geometry;
        fw.send(ExecutorContext::new_with_node_context_feature_and_port(
            ctx,
            feature,
            OUTSIDE_PORT.clone(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::Coordinate2D;
    use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::types::no_value::NoValue;
    use reearth_flow_geometry::types::point::Point2D;
    use reearth_flow_runtime::executor_operation::NodeContext;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;

    use crate::tests::utils::create_default_execute_context;

    fn create_test_polygon_2d() -> Polygon2D<f64> {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(10.0, 0.0),
            Coordinate2D::new_(10.0, 10.0),
            Coordinate2D::new_(0.0, 10.0),
            Coordinate2D::new_(0.0, 0.0),
        ]);
        Polygon2D::new(exterior, vec![])
    }

    fn create_clipper_polygon_2d() -> Polygon2D<f64> {
        let exterior = LineString2D::new(vec![
            Coordinate2D::new_(5.0, 5.0),
            Coordinate2D::new_(15.0, 5.0),
            Coordinate2D::new_(15.0, 15.0),
            Coordinate2D::new_(5.0, 15.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        Polygon2D::new(exterior, vec![])
    }

    fn create_test_polygon_3d() -> Polygon3D<f64> {
        let exterior = LineString3D::new(vec![
            (0.0, 0.0, 0.0).into(),
            (10.0, 0.0, 0.0).into(),
            (10.0, 10.0, 0.0).into(),
            (0.0, 10.0, 0.0).into(),
            (0.0, 0.0, 0.0).into(),
        ]);
        Polygon3D::new(exterior, vec![])
    }

    fn create_clipper_polygon_3d() -> Polygon3D<f64> {
        let exterior = LineString3D::new(vec![
            (5.0, 5.0, 0.0).into(),
            (15.0, 5.0, 0.0).into(),
            (15.0, 15.0, 0.0).into(),
            (5.0, 15.0, 0.0).into(),
            (5.0, 5.0, 0.0).into(),
        ]);
        Polygon3D::new(exterior, vec![])
    }

    #[test]
    fn test_clipper_with_empty_geometry() {
        let mut clipper = Clipper {
            clippers: Vec::new(),
            candidates: Vec::new(),
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);

        let feature = Feature::default();
        let ctx = create_default_execute_context(&feature);

        let result = clipper.process(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *REJECTED_PORT);
        }
    }

    #[test]
    fn test_clipper_adds_features_to_correct_lists() {
        let mut clipper = Clipper {
            clippers: Vec::new(),
            candidates: Vec::new(),
        };

        let polygon = create_test_polygon_2d();
        let clipper_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon.clone())),
                ..Default::default()
            },
            ..Default::default()
        };

        let candidate_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon.clone())),
                ..Default::default()
            },
            ..Default::default()
        };

        // Process clipper feature
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&clipper_feature);
        ctx.port = CLIPPER_PORT.clone();

        let result = clipper.process(ctx, &fw);
        assert!(result.is_ok());
        assert_eq!(clipper.clippers.len(), 1);
        assert_eq!(clipper.candidates.len(), 0);

        // Process candidate feature
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&candidate_feature);
        ctx.port = CANDIDATE_PORT.clone();

        let result = clipper.process(ctx, &fw);
        assert!(result.is_ok());
        assert_eq!(clipper.clippers.len(), 1);
        assert_eq!(clipper.candidates.len(), 1);
    }

    #[test]
    fn test_clipper_finish_with_no_clippers() {
        let clipper = Clipper {
            clippers: Vec::new(),
            candidates: vec![Feature::default()],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0], *REJECTED_PORT);
        }
    }

    #[test]
    fn test_clip_polygon2d_basic() {
        let polygon = create_test_polygon_2d();
        let clip_region = create_clipper_polygon_2d();

        let (insides, outsides) = clip_polygon2d(&polygon, &[clip_region]);

        // Should have both inside and outside results
        assert!(!insides.is_empty(), "Should have inside polygons");
        assert!(!outsides.is_empty(), "Should have outside polygons");
    }

    #[test]
    fn test_clip_polygon3d_basic() {
        let polygon = create_test_polygon_3d();
        let clip_region = create_clipper_polygon_3d();

        let (insides, outsides) = clip_polygon3d(&polygon, &[clip_region]);

        // Should have both inside and outside results
        assert!(!insides.is_empty(), "Should have inside polygons");
        assert!(!outsides.is_empty(), "Should have outside polygons");
    }

    #[test]
    fn test_clip_mpolygon2d() {
        let polygon = create_test_polygon_2d();
        let mpolygon = MultiPolygon2D::new(vec![polygon]);
        let clip_region = create_clipper_polygon_2d();

        let (insides, outsides) = clip_mpolygon2d(&mpolygon, &[clip_region]);

        assert!(!insides.is_empty(), "Should have inside polygons");
        assert!(!outsides.is_empty(), "Should have outside polygons");
    }

    #[test]
    fn test_clip_mpolygon3d() {
        let polygon = create_test_polygon_3d();
        let mpolygon = MultiPolygon3D::new(vec![polygon]);
        let clip_region = create_clipper_polygon_3d();

        let (insides, outsides) = clip_mpolygon3d(&mpolygon, &[clip_region]);

        assert!(!insides.is_empty(), "Should have inside polygons");
        assert!(!outsides.is_empty(), "Should have outside polygons");
    }

    #[test]
    fn test_clipper_finish_with_2d_geometries() {
        let polygon = create_test_polygon_2d();
        let clip_polygon = create_clipper_polygon_2d();

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should have sent features to both inside and outside ports
            assert!(ports.contains(&*INSIDE_PORT));
            assert!(ports.contains(&*OUTSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_finish_with_3d_geometries() {
        let polygon = create_test_polygon_3d();
        let clip_polygon = create_clipper_polygon_3d();

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should have sent features to both inside and outside ports
            assert!(ports.contains(&*INSIDE_PORT));
            assert!(ports.contains(&*OUTSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_with_multiple_clip_regions() {
        let polygon = create_test_polygon_2d();

        // Create two overlapping clip regions
        let clip_region1 = create_clipper_polygon_2d();
        let exterior2 = LineString2D::new(vec![
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(8.0, 2.0),
            Coordinate2D::new_(8.0, 8.0),
            Coordinate2D::new_(2.0, 8.0),
            Coordinate2D::new_(2.0, 2.0),
        ]);
        let clip_region2 = Polygon2D::new(exterior2, vec![]);

        let (insides, outsides) = clip_polygon2d(&polygon, &[clip_region1, clip_region2]);

        // With multiple clip regions, the result should be their intersection
        assert!(
            !insides.is_empty() || !outsides.is_empty(),
            "Should have some results"
        );
    }

    #[test]
    fn test_clipper_with_non_polygon_geometry() {
        let mut clipper = Clipper {
            clippers: Vec::new(),
            candidates: Vec::new(),
        };

        // Create a point geometry (non-polygon)
        let point_feature = Feature {
            geometry: Geometry {
                value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::new_(
                    5.0, 5.0, NoValue,
                ))),
                ..Default::default()
            },
            ..Default::default()
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut ctx = create_default_execute_context(&point_feature);
        ctx.port = CANDIDATE_PORT.clone();

        let result = clipper.process(ctx, &fw);
        assert!(result.is_ok());
        // Non-polygon geometries should still be added to the list
        assert_eq!(clipper.candidates.len(), 1);
    }

    #[test]
    fn test_clipper_finish_with_non_polygon_candidate() {
        let clip_polygon = create_clipper_polygon_2d();

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::new_(
                        5.0, 5.0, NoValue,
                    ))),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Non-polygon candidates should be rejected
            assert!(ports.contains(&*REJECTED_PORT));
        }
    }

    #[test]
    fn test_clipper_with_citygml_geometry() {
        use reearth_flow_types::{GeometryType, GmlGeometry};

        let polygon = create_test_polygon_3d();
        let clip_polygon = create_clipper_polygon_3d();

        // Create a CityGML geometry with a polygon
        let gml_geometry = GmlGeometry {
            id: Some("test_gml".to_string()),
            ty: GeometryType::Surface,
            lod: Some(2),
            pos: 0,
            len: 1,
            polygons: vec![polygon.clone()],
            line_strings: vec![],
            feature_id: Some("feature1".to_string()),
            feature_type: Some("Building".to_string()),
            composite_surfaces: vec![],
        };

        let citygml = CityGmlGeometry {
            gml_geometries: vec![gml_geometry],
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: Default::default(),
        };

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::CityGmlGeometry(citygml),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should have sent features to both inside and outside ports
            assert!(ports.contains(&*INSIDE_PORT));
            assert!(ports.contains(&*OUTSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_with_citygml_as_clipper() {
        use reearth_flow_types::{GeometryType, GmlGeometry};

        let polygon = create_test_polygon_3d();
        let clip_polygon = create_clipper_polygon_3d();

        // Create a CityGML geometry as the clipper
        let gml_clipper = GmlGeometry {
            id: Some("clipper_gml".to_string()),
            ty: GeometryType::Surface,
            lod: Some(2),
            pos: 0,
            len: 1,
            polygons: vec![clip_polygon.clone()],
            line_strings: vec![],
            feature_id: Some("clipper1".to_string()),
            feature_type: Some("Building".to_string()),
            composite_surfaces: vec![],
        };

        let citygml_clipper = CityGmlGeometry {
            gml_geometries: vec![gml_clipper],
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: Default::default(),
        };

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::CityGmlGeometry(citygml_clipper),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should have sent features to both inside and outside ports
            assert!(ports.contains(&*INSIDE_PORT));
            assert!(ports.contains(&*OUTSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_with_citygml_composite_surfaces() {
        use reearth_flow_types::{GeometryType, GmlGeometry};

        let polygon1 = create_test_polygon_3d();
        let polygon2 = create_clipper_polygon_3d();
        let clip_polygon = create_clipper_polygon_3d();

        // Create a nested GML geometry (composite surface)
        let nested_gml = GmlGeometry {
            id: Some("nested_surface".to_string()),
            ty: GeometryType::Surface,
            lod: Some(2),
            pos: 0,
            len: 1,
            polygons: vec![polygon1.clone()],
            line_strings: vec![],
            feature_id: Some("nested_feature".to_string()),
            feature_type: Some("Wall".to_string()),
            composite_surfaces: vec![],
        };

        // Create a parent GML geometry with composite surfaces
        let parent_gml = GmlGeometry {
            id: Some("parent_solid".to_string()),
            ty: GeometryType::Solid,
            lod: Some(2),
            pos: 0,
            len: 2,
            polygons: vec![polygon2.clone()],
            line_strings: vec![],
            feature_id: Some("parent_feature".to_string()),
            feature_type: Some("Building".to_string()),
            composite_surfaces: vec![nested_gml],
        };

        let citygml = CityGmlGeometry {
            gml_geometries: vec![parent_gml],
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: Default::default(),
        };

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::CityGmlGeometry(citygml),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should have sent features to both inside and outside ports
            assert!(ports.contains(&*INSIDE_PORT));
            assert!(ports.contains(&*OUTSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_with_citygml_curve_geometry() {
        use reearth_flow_geometry::types::line_string::LineString3D;
        use reearth_flow_types::{GeometryType, GmlGeometry};

        let clip_polygon = create_clipper_polygon_3d();

        // Create a line string for Curve geometry
        let line_string = LineString3D::new(vec![
            (0.0, 0.0, 0.0).into(),
            (10.0, 10.0, 0.0).into(),
            (20.0, 20.0, 0.0).into(),
        ]);

        // Create a Curve type GML geometry
        let curve_gml = GmlGeometry {
            id: Some("curve_gml".to_string()),
            ty: GeometryType::Curve,
            lod: Some(2),
            pos: 0,
            len: 1,
            polygons: vec![], // Curves don't have polygons
            line_strings: vec![line_string],
            feature_id: Some("curve_feature".to_string()),
            feature_type: Some("Road".to_string()),
            composite_surfaces: vec![],
        };

        let citygml = CityGmlGeometry {
            gml_geometries: vec![curve_gml],
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: Default::default(),
        };

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::CityGmlGeometry(citygml),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Curve geometry with line_strings should be sent to inside port as-is
            // since we can't clip line strings the same way as polygons
            assert!(ports.contains(&*INSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_with_citygml_solid_with_nested_surfaces() {
        use reearth_flow_types::{GeometryType, GmlGeometry};

        let polygon1 = create_test_polygon_3d();
        let polygon2 = create_clipper_polygon_3d();

        // Create nested surface geometries (representing walls, roof, etc.)
        let wall1 = GmlGeometry {
            id: Some("wall1".to_string()),
            ty: GeometryType::Surface,
            lod: Some(2),
            pos: 0,
            len: 1,
            polygons: vec![polygon1.clone()],
            line_strings: vec![],
            feature_id: Some("wall1_feature".to_string()),
            feature_type: Some("WallSurface".to_string()),
            composite_surfaces: vec![],
        };

        let wall2 = GmlGeometry {
            id: Some("wall2".to_string()),
            ty: GeometryType::Surface,
            lod: Some(2),
            pos: 1,
            len: 1,
            polygons: vec![polygon2.clone()],
            line_strings: vec![],
            feature_id: Some("wall2_feature".to_string()),
            feature_type: Some("WallSurface".to_string()),
            composite_surfaces: vec![],
        };

        // Create a Solid with composite surfaces
        let solid = GmlGeometry {
            id: Some("building_solid".to_string()),
            ty: GeometryType::Solid,
            lod: Some(2),
            pos: 0,
            len: 2,
            polygons: vec![], // Solid might not have direct polygons
            line_strings: vec![],
            feature_id: Some("building".to_string()),
            feature_type: Some("Building".to_string()),
            composite_surfaces: vec![wall1, wall2],
        };

        let citygml = CityGmlGeometry {
            gml_geometries: vec![solid],
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: Default::default(),
        };

        let clip_polygon = create_clipper_polygon_3d();

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::CityGmlGeometry(citygml),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // Should process nested surfaces and send results
            assert!(ports.contains(&*INSIDE_PORT));
            assert!(ports.contains(&*OUTSIDE_PORT));
        }
    }

    #[test]
    fn test_clipper_with_geometry_collection() {
        let polygon1 = create_test_polygon_2d();
        let polygon2 = create_clipper_polygon_2d();
        let collection = Geometry2D::GeometryCollection(vec![
            Geometry2D::Polygon(polygon1.clone()),
            Geometry2D::Polygon(polygon2.clone()),
        ]);

        let clip_polygon = create_clipper_polygon_2d();

        let clipper = Clipper {
            clippers: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(clip_polygon)),
                    ..Default::default()
                },
                ..Default::default()
            }],
            candidates: vec![Feature {
                geometry: Geometry {
                    value: GeometryValue::FlowGeometry2D(collection),
                    ..Default::default()
                },
                ..Default::default()
            }],
        };

        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = NodeContext::default();

        let result = clipper.finish(ctx, &fw);
        assert!(result.is_ok());

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            // GeometryCollection should process each geometry individually
            assert!(!ports.is_empty());
        }
    }
}

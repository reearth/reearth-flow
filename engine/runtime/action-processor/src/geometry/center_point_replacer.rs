use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{CityGmlGeometry, Feature, Geometry, GeometryValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));

#[derive(Debug, Clone, Default)]
pub(super) struct CenterPointReplacerFactory;

impl ProcessorFactory for CenterPointReplacerFactory {
    fn name(&self) -> &str {
        "CenterPointReplacer"
    }

    fn description(&self) -> &str {
        "Replace Feature Geometry with Center Point"
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
        vec![POINT_PORT.clone(), REJECTED_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(CenterPointReplacer))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CenterPointReplacer;

impl Processor for CenterPointReplacer {
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
            GeometryValue::CityGmlGeometry(city_gml) => {
                self.handle_citygml_geometry(city_gml, feature, geometry, &ctx, fw);
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
        "CenterPointReplacer"
    }
}

impl CenterPointReplacer {
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let Some(centroid) = geos.centroid() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        let mut feature = feature.clone();
        feature.geometry = Geometry {
            epsg: geometry.epsg,
            value: GeometryValue::FlowGeometry2D(centroid.into()),
        }
        .into();
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }

    fn handle_citygml_geometry(
        &self,
        city_gml: &CityGmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let mut polygons = city_gml
            .gml_geometries
            .iter()
            .flat_map(|g| g.polygons.iter());
        let Some(polygon) = polygons.next() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        if polygons.next().is_some() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        }
        let Some(centroid) = polygon.centroid() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        let mut feature = feature.clone();
        feature.geometry = Geometry {
            epsg: geometry.epsg,
            value: GeometryValue::FlowGeometry3D(centroid.into()),
        }
        .into();
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let Some(centroid) = geos.centroid() else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return;
        };
        let mut feature = feature.clone();
        feature.geometry = Geometry {
            epsg: geometry.epsg,
            value: GeometryValue::FlowGeometry3D(centroid.into()),
        }
        .into();
        fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::coordinate::Coordinate;
    use reearth_flow_geometry::types::line_string::LineString3D;
    use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
    use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::feature::Attributes;
    use reearth_flow_types::{GeometryType, GmlGeometry};

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    fn make_polygon_3d(coords: &[(f64, f64, f64)]) -> Polygon3D<f64> {
        let line: LineString3D<f64> =
            LineString3D::new(coords.iter().map(|&c| Coordinate::from(c)).collect());
        Polygon3D::new(line, vec![])
    }

    fn make_gml_geometry(polygons: Vec<Polygon3D<f64>>) -> GmlGeometry {
        GmlGeometry {
            len: polygons.len() as u32,
            polygons,
            ..GmlGeometry::new(GeometryType::Surface, Some(1))
        }
    }

    fn make_citygml_feature(gml_geometries: Vec<GmlGeometry>) -> Feature {
        let city_gml = CityGmlGeometry {
            gml_geometries,
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: MultiPolygon2D::default(),
        };
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::CityGmlGeometry(city_gml),
        };
        Feature::new_with_attributes_and_geometry(Attributes::new(), geometry, Default::default())
    }

    fn run_processor(feature: &Feature) -> (Vec<Feature>, Vec<Port>) {
        let mut processor = CenterPointReplacer;
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = create_default_execute_context(feature);
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap().clone();
            let ports = noop.send_ports.lock().unwrap().clone();
            (features, ports)
        } else {
            unreachable!()
        }
    }

    #[test]
    fn test_2d_polygon_centroid() {
        let coords: Vec<Coordinate<f64, _>> = vec![
            (0.0, 0.0).into(),
            (4.0, 0.0).into(),
            (4.0, 4.0).into(),
            (0.0, 4.0).into(),
            (0.0, 0.0).into(),
        ];
        let polygon = Polygon2D::new(coords.into(), vec![]);
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => {
                assert!((p.x() - 2.0).abs() < 1e-10);
                assert!((p.y() - 2.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry2D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_3d_polygon_centroid() {
        let polygon = make_polygon_3d(&[
            (0.0, 0.0, 0.0),
            (4.0, 0.0, 0.0),
            (4.0, 4.0, 0.0),
            (0.0, 4.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)),
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                assert!((p.x() - 2.0).abs() < 1e-10);
                assert!((p.y() - 2.0).abs() < 1e-10);
                assert!((p.z() - 0.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_citygml_single_polygon_centroid() {
        let polygon = make_polygon_3d(&[
            (0.0, 0.0, 10.0),
            (6.0, 0.0, 10.0),
            (6.0, 6.0, 10.0),
            (0.0, 6.0, 10.0),
            (0.0, 0.0, 10.0),
        ]);
        let gml = make_gml_geometry(vec![polygon]);
        let feature = make_citygml_feature(vec![gml]);

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                assert!((p.x() - 3.0).abs() < 1e-10);
                assert!((p.y() - 3.0).abs() < 1e-10);
                assert!((p.z() - 10.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_citygml_multiple_polygons_rejected() {
        let poly1 = make_polygon_3d(&[
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 1.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let poly2 = make_polygon_3d(&[
            (2.0, 2.0, 0.0),
            (3.0, 2.0, 0.0),
            (3.0, 3.0, 0.0),
            (2.0, 2.0, 0.0),
        ]);
        let gml = make_gml_geometry(vec![poly1, poly2]);
        let feature = make_citygml_feature(vec![gml]);

        let (_, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
    }
}

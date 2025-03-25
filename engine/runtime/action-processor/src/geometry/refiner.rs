use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::LineString;
use reearth_flow_geometry::types::point::Point;
use reearth_flow_geometry::types::polygon::Polygon;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REMAIN_PORT},
};
use reearth_flow_types::geometry::Geometry as TypeGeometry;
use reearth_flow_types::Feature;
use reearth_flow_types::{CityGmlGeometry, GeometryValue};

use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::vec;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct RefinerFactory;

impl ProcessorFactory for RefinerFactory {
    fn name(&self) -> &str {
        "Refiner"
    }

    fn description(&self) -> &str {
        "Geometry Refiner"
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
        vec![DEFAULT_PORT.clone(), REMAIN_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = Refiner {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct Refiner;

impl Processor for Refiner {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = feature.geometry.clone();
        let geometry_value = geometry.value.clone();

        let gc = geometry.clone();
        let geom_epsg = gc.epsg;
        let attributes = feature.attributes.clone();
        let metadata = feature.metadata.clone();

        fw.send(ctx.new_with_feature_and_port(feature.clone(), REMAIN_PORT.clone()));

        match geometry_value {
            GeometryValue::None => {}
            GeometryValue::CityGmlGeometry(city_gml) => {
                let _geometries = Self::refine_city_gml(&city_gml);
            }
            GeometryValue::FlowGeometry2D(flow_2d) => {
                let geometries = Self::refine_2d(&flow_2d);
                for geo in geometries {
                    let feature = Feature {
                        id: Uuid::new_v4(),
                        geometry: TypeGeometry {
                            epsg: geom_epsg,
                            value: GeometryValue::FlowGeometry2D(geo),
                        },
                        metadata: metadata.clone(),
                        attributes: attributes.clone(),
                    };
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
            }
            GeometryValue::FlowGeometry3D(flow_3d) => {
                let geometries = Self::refine_3d(&flow_3d);
                for geo in geometries {
                    let feature = Feature {
                        id: Uuid::new_v4(),
                        geometry: TypeGeometry {
                            epsg: geom_epsg,
                            value: GeometryValue::FlowGeometry3D(geo),
                        },
                        metadata: metadata.clone(),
                        attributes: attributes.clone(),
                    };
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
            }
        };

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Refiner"
    }
}

type RefinedGeometry<T, Z> = (Vec<Point<T, Z>>, Vec<LineString<T, Z>>, Vec<Polygon<T, Z>>);

impl Refiner {
    fn refine_city_gml(_geos: &CityGmlGeometry) -> Vec<CityGmlGeometry> {
        // not implemented
        Vec::new()
    }

    fn refine_2d(geos: &Geometry2D) -> Vec<Geometry2D<f64>> {
        let (points, lines, polygons) = Self::refine_geometry(geos.clone());
        let mut geometories: Vec<Geometry2D<f64>> = Vec::new();
        for point in points {
            geometories.push(Geometry2D::Point(point));
        }
        for line in lines {
            geometories.push(Geometry2D::LineString(line));
        }
        for polygon in polygons {
            geometories.push(Geometry2D::Polygon(polygon));
        }
        geometories
    }

    fn refine_3d(geos: &Geometry3D) -> Vec<Geometry3D<f64>> {
        let (points, lines, polygons) = Self::refine_geometry(geos.clone());
        let mut geometories: Vec<Geometry3D<f64>> = Vec::new();
        for point in points {
            geometories.push(Geometry3D::Point(point));
        }
        for line in lines {
            geometories.push(Geometry3D::LineString(line));
        }
        for polygon in polygons {
            geometories.push(Geometry3D::Polygon(polygon));
        }
        geometories
    }

    fn refine_geometry<T: CoordNum, Z: CoordNum>(
        geometry: Geometry<T, Z>,
    ) -> RefinedGeometry<T, Z> {
        let mut points: Vec<Point<T, Z>> = Vec::new();
        let mut lines: Vec<LineString<T, Z>> = Vec::new();
        let mut polygons: Vec<Polygon<T, Z>> = Vec::new();

        if let Geometry::GeometryCollection(gc) = geometry {
            for g in gc {
                match g {
                    Geometry::MultiPoint(mp) => {
                        for p in mp {
                            points.push(p);
                        }
                    }
                    Geometry::MultiLineString(mls) => {
                        for ls in mls {
                            lines.push(ls);
                        }
                    }
                    Geometry::MultiPolygon(mp) => {
                        for p in mp {
                            polygons.push(p);
                        }
                    }
                    _ => {}
                }
            }
        }
        (points, lines, polygons)
    }
}

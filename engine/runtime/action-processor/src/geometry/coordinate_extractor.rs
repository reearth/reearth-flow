use std::collections::HashMap;

use num_traits::NumCast;
use reearth_flow_geometry::algorithm::coords_iter::CoordsIter;
use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, CityGmlGeometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use super::utils::finite_z;

/// # Coordinate Extractor Parameters
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CoordinateExtractorParam {
    /// # Extraction Mode
    /// How to extract coordinates from geometry vertices.
    mode: CoordinateExtractionMode,
    /// # Default Z Value
    /// Z value to use for 2D geometries that have no Z coordinate.
    default_z_value: Option<f64>,
}

/// Extraction mode: determines how coordinates are output.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum CoordinateExtractionMode {
    /// Extract all vertices into a list attribute.
    #[serde(rename_all = "camelCase")]
    AllCoordinates {
        /// # Coordinates List Name
        /// Name of the list attribute that will store coordinate objects (default: "_indices")
        #[serde(default = "default_coordinates_list_name")]
        coordinates_list_name: Attribute,
    },
    /// Extract a single vertex by index.
    #[serde(rename_all = "camelCase")]
    SpecifyCoordinate {
        /// # Coordinate Index
        /// Index of the coordinate to extract.
        /// 0 = first vertex, negative values count from end (-1 = last).
        coordinate_index: i64,
        /// # X Attribute Name
        /// Name of the X coordinate attribute (default: "_x")
        #[serde(default = "default_x_attribute")]
        x_attribute: Attribute,
        /// # Y Attribute Name
        /// Name of the Y coordinate attribute (default: "_y")
        #[serde(default = "default_y_attribute")]
        y_attribute: Attribute,
        /// # Z Attribute Name
        /// Name of the Z coordinate attribute (default: "_z")
        #[serde(default = "default_z_attribute")]
        z_attribute: Attribute,
    },
}

impl Default for CoordinateExtractionMode {
    fn default() -> Self {
        Self::AllCoordinates {
            coordinates_list_name: default_coordinates_list_name(),
        }
    }
}

fn default_coordinates_list_name() -> Attribute {
    Attribute::new("_indices")
}

fn default_x_attribute() -> Attribute {
    Attribute::new("_x")
}

fn default_y_attribute() -> Attribute {
    Attribute::new("_y")
}

fn default_z_attribute() -> Attribute {
    Attribute::new("_z")
}

#[derive(Debug, Clone, Default)]
pub(super) struct CoordinateExtractorFactory;

impl ProcessorFactory for CoordinateExtractorFactory {
    fn name(&self) -> &str {
        "CoordinateExtractor"
    }

    fn description(&self) -> &str {
        "Extracts coordinates from geometry vertices into feature attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CoordinateExtractorParam))
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: CoordinateExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CoordinateExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CoordinateExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::CoordinateExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        Ok(Box::new(CoordinateExtractor { params }))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CoordinateExtractor {
    params: CoordinateExtractorParam,
}

impl Processor for CoordinateExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry_value = &feature.geometry.value;

        let coords = match geometry_value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
            GeometryValue::FlowGeometry2D(geom) => {
                collect_coords_2d(geom, self.params.default_z_value)
            }
            GeometryValue::FlowGeometry3D(geom) => collect_coords_3d(geom),
            GeometryValue::CityGmlGeometry(geom) => collect_coords_citygml(geom),
        };

        if coords.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let mut new_feature = feature.clone();

        match &self.params.mode {
            CoordinateExtractionMode::AllCoordinates {
                coordinates_list_name,
            } => {
                let list_name = coordinates_list_name.clone();
                let array: Vec<AttributeValue> = coords
                    .iter()
                    .map(|&(x, y, z)| {
                        let mut map = HashMap::new();
                        map.insert(
                            "x".to_string(),
                            AttributeValue::try_from(x).unwrap_or(AttributeValue::Null),
                        );
                        map.insert(
                            "y".to_string(),
                            AttributeValue::try_from(y).unwrap_or(AttributeValue::Null),
                        );
                        map.insert(
                            "z".to_string(),
                            z.and_then(|v| AttributeValue::try_from(v).ok())
                                .unwrap_or(AttributeValue::Null),
                        );
                        AttributeValue::Map(map)
                    })
                    .collect();
                new_feature.insert(list_name, AttributeValue::Array(array));
                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
            CoordinateExtractionMode::SpecifyCoordinate {
                coordinate_index,
                x_attribute,
                y_attribute,
                z_attribute,
            } => {
                let x_attr = x_attribute.clone();
                let y_attr = y_attribute.clone();
                let z_attr = z_attribute.clone();

                let index = *coordinate_index;
                let resolved = if index < 0 {
                    let positive = index.checked_abs().and_then(|v| usize::try_from(v).ok());
                    match positive {
                        Some(p) if p <= coords.len() => Some(coords.len() - p),
                        _ => None,
                    }
                } else {
                    let idx = usize::try_from(index).ok();
                    match idx {
                        Some(i) if i < coords.len() => Some(i),
                        _ => None,
                    }
                };

                match resolved {
                    Some(idx) => {
                        let (x, y, z) = coords[idx];
                        new_feature.insert(
                            x_attr,
                            AttributeValue::try_from(x).unwrap_or(AttributeValue::Null),
                        );
                        new_feature.insert(
                            y_attr,
                            AttributeValue::try_from(y).unwrap_or(AttributeValue::Null),
                        );
                        new_feature.insert(
                            z_attr,
                            z.and_then(|v| AttributeValue::try_from(v).ok())
                                .unwrap_or(AttributeValue::Null),
                        );
                        fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                    }
                    None => {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        );
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
        "CoordinateExtractor"
    }
}

fn coord_to_tuple<T: CoordNum, Z: CoordNum>(
    coord: &Coordinate<T, Z>,
    default_z: Option<f64>,
) -> (f64, f64, Option<f64>) {
    let x: f64 = NumCast::from(coord.x).unwrap_or(0.0);
    let y: f64 = NumCast::from(coord.y).unwrap_or(0.0);
    let z = finite_z(coord.z).or(default_z);
    (x, y, z)
}

fn collect_coords_2d(
    geom: &Geometry2D<f64>,
    default_z: Option<f64>,
) -> Vec<(f64, f64, Option<f64>)> {
    geom.coords_iter()
        .map(|c| coord_to_tuple(&c, default_z))
        .collect()
}

fn collect_coords_3d(geom: &Geometry3D<f64>) -> Vec<(f64, f64, Option<f64>)> {
    geom.coords_iter()
        .map(|c| coord_to_tuple(&c, None))
        .collect()
}

fn collect_coords_citygml(geom: &CityGmlGeometry) -> Vec<(f64, f64, Option<f64>)> {
    let mut coords = Vec::new();
    for gml_feature in &geom.gml_geometries {
        for polygon in &gml_feature.polygons {
            for c in polygon.coords_iter() {
                coords.push(coord_to_tuple(&c, None));
            }
        }
        for line_string in &gml_feature.line_strings {
            for c in line_string.coords_iter() {
                coords.push(coord_to_tuple(&c, None));
            }
        }
        for point in &gml_feature.points {
            let x: f64 = point.x;
            let y: f64 = point.y;
            let z = if point.z.is_finite() {
                Some(point.z)
            } else {
                None
            };
            coords.push((x, y, z));
        }
    }
    coords
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use reearth_flow_geometry::types::coordinate::Coordinate3D;
    use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
    use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
    use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::types::no_value::NoValue;
    use reearth_flow_geometry::types::point::{Point2D, Point3D};
    use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{feature::Attributes, Feature, Geometry};

    #[test]
    fn test_all_coords_point_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_all_coords_processor(None);

        let point = Point2D::new(1.0, 2.0, NoValue);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::Point(point)));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(features.len(), 1);
            assert_eq!(ports[0], DEFAULT_PORT.clone());

            let arr = features[0]
                .attributes
                .get(&Attribute::new("_indices"))
                .unwrap();
            if let AttributeValue::Array(items) = arr {
                assert_eq!(items.len(), 1);
                if let AttributeValue::Map(map) = &items[0] {
                    assert_eq!(map.get("x"), Some(&AttributeValue::try_from(1.0).unwrap()));
                    assert_eq!(map.get("y"), Some(&AttributeValue::try_from(2.0).unwrap()));
                    assert_eq!(map.get("z"), Some(&AttributeValue::Null));
                } else {
                    panic!("Expected Map");
                }
            } else {
                panic!("Expected Array");
            }
        }
    }

    #[test]
    fn test_all_coords_point_3d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_all_coords_processor(None);

        let point = Point3D::new(1.0, 2.0, 3.0);
        let feature = make_feature(GeometryValue::FlowGeometry3D(FlowGeometry3D::Point(point)));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            assert_eq!(features.len(), 1);

            let arr = features[0]
                .attributes
                .get(&Attribute::new("_indices"))
                .unwrap();
            if let AttributeValue::Array(items) = arr {
                assert_eq!(items.len(), 1);
                if let AttributeValue::Map(map) = &items[0] {
                    assert_eq!(map.get("x"), Some(&AttributeValue::try_from(1.0).unwrap()));
                    assert_eq!(map.get("y"), Some(&AttributeValue::try_from(2.0).unwrap()));
                    assert_eq!(map.get("z"), Some(&AttributeValue::try_from(3.0).unwrap()));
                } else {
                    panic!("Expected Map");
                }
            } else {
                panic!("Expected Array");
            }
        }
    }

    #[test]
    fn test_all_coords_linestring_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_all_coords_processor(None);

        let ls = LineString2D::from(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::LineString(
            ls,
        )));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let arr = features[0]
                .attributes
                .get(&Attribute::new("_indices"))
                .unwrap();
            if let AttributeValue::Array(items) = arr {
                assert_eq!(items.len(), 3);
            } else {
                panic!("Expected Array");
            }
        }
    }

    #[test]
    fn test_all_coords_polygon_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_all_coords_processor(None);

        let exterior = LineString2D::from(vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (0.0, 0.0),
        ]);
        let polygon = Polygon2D::new(exterior, vec![]);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::Polygon(
            polygon,
        )));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let arr = features[0]
                .attributes
                .get(&Attribute::new("_indices"))
                .unwrap();
            if let AttributeValue::Array(items) = arr {
                assert_eq!(items.len(), 5);
            } else {
                panic!("Expected Array");
            }
        }
    }

    #[test]
    fn test_specify_coord_positive_index() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_specify_processor(1, None);

        let ls = LineString2D::from(vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)]);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::LineString(
            ls,
        )));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], DEFAULT_PORT.clone());
            assert_eq!(
                features[0].attributes.get(&Attribute::new("_x")),
                Some(&AttributeValue::try_from(30.0).unwrap())
            );
            assert_eq!(
                features[0].attributes.get(&Attribute::new("_y")),
                Some(&AttributeValue::try_from(40.0).unwrap())
            );
        }
    }

    #[test]
    fn test_specify_coord_negative_index() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_specify_processor(-1, None);

        let ls = LineString2D::from(vec![(10.0, 20.0), (30.0, 40.0), (50.0, 60.0)]);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::LineString(
            ls,
        )));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], DEFAULT_PORT.clone());
            assert_eq!(
                features[0].attributes.get(&Attribute::new("_x")),
                Some(&AttributeValue::try_from(50.0).unwrap())
            );
            assert_eq!(
                features[0].attributes.get(&Attribute::new("_y")),
                Some(&AttributeValue::try_from(60.0).unwrap())
            );
        }
    }

    #[test]
    fn test_specify_coord_out_of_range() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_specify_processor(100, None);

        let ls = LineString2D::from(vec![(10.0, 20.0), (30.0, 40.0)]);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::LineString(
            ls,
        )));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], REJECTED_PORT.clone());
        }
    }

    #[test]
    fn test_no_geometry_rejected() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_all_coords_processor(None);

        let feature = make_feature(GeometryValue::None);
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], REJECTED_PORT.clone());
        }
    }

    #[test]
    fn test_default_z_value_2d() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_specify_processor(0, Some(99.0));

        let point = Point2D::new(5.0, 6.0, NoValue);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::Point(point)));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], DEFAULT_PORT.clone());
            assert_eq!(
                features[0].attributes.get(&Attribute::new("_z")),
                Some(&AttributeValue::try_from(99.0).unwrap())
            );
        }
    }

    #[test]
    fn test_specify_coord_negative_out_of_range() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_specify_processor(-5, None);

        let ls = LineString2D::from(vec![(10.0, 20.0), (30.0, 40.0)]);
        let feature = make_feature(GeometryValue::FlowGeometry2D(FlowGeometry2D::LineString(
            ls,
        )));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], REJECTED_PORT.clone());
        }
    }

    #[test]
    fn test_all_coords_citygml() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let mut processor = make_all_coords_processor(None);

        let polygon = Polygon3D::new(
            LineString3D::new(vec![
                Coordinate3D::new__(1.0, 2.0, 3.0),
                Coordinate3D::new__(4.0, 5.0, 6.0),
                Coordinate3D::new__(7.0, 8.0, 9.0),
            ]),
            vec![],
        );
        let gml_geom = reearth_flow_types::GmlGeometry {
            polygons: vec![polygon],
            ..reearth_flow_types::GmlGeometry::new(
                reearth_flow_types::geometry::GeometryType::Surface,
                Some(2),
            )
        };
        let citygml = CityGmlGeometry {
            gml_geometries: vec![gml_geom],
            ..Default::default()
        };
        let feature = make_feature(GeometryValue::CityGmlGeometry(citygml));
        let ctx = create_default_execute_context(&feature);

        processor.process(ctx, &fw).unwrap();

        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap();
            let ports = noop.send_ports.lock().unwrap();
            assert_eq!(ports[0], DEFAULT_PORT.clone());

            let arr = features[0]
                .attributes
                .get(&Attribute::new("_indices"))
                .unwrap();
            if let AttributeValue::Array(items) = arr {
                // 3 distinct points + 1 closing point auto-added by Polygon::new
                assert_eq!(items.len(), 4);
                if let AttributeValue::Map(map) = &items[0] {
                    assert_eq!(map.get("x"), Some(&AttributeValue::try_from(1.0).unwrap()));
                    assert_eq!(map.get("y"), Some(&AttributeValue::try_from(2.0).unwrap()));
                    assert_eq!(map.get("z"), Some(&AttributeValue::try_from(3.0).unwrap()));
                } else {
                    panic!("Expected Map");
                }
            } else {
                panic!("Expected Array");
            }
        }
    }

    //
    // helper functions
    //

    fn make_feature(value: GeometryValue) -> Feature {
        Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value,
                ..Default::default()
            },
        )
    }

    fn make_all_coords_processor(default_z_value: Option<f64>) -> CoordinateExtractor {
        CoordinateExtractor {
            params: CoordinateExtractorParam {
                default_z_value,
                ..Default::default()
            },
        }
    }

    fn make_specify_processor(
        coordinate_index: i64,
        default_z_value: Option<f64>,
    ) -> CoordinateExtractor {
        CoordinateExtractor {
            params: CoordinateExtractorParam {
                mode: CoordinateExtractionMode::SpecifyCoordinate {
                    coordinate_index,
                    x_attribute: default_x_attribute(),
                    y_attribute: default_y_attribute(),
                    z_attribute: default_z_attribute(),
                },
                default_z_value,
            },
        }
    }
}

use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D, Geometry3D};
use reearth_flow_geometry::types::point::Point;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use reearth_flow_types::{CityGmlGeometry, GeometryValue};

use num_traits::NumCast;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::fmt::Debug;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone)]
pub struct Bounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl Bounds {
    fn min_x_value(&self) -> AttributeValue {
        AttributeValue::Number(Number::from_f64(self.min_x).unwrap_or_else(|| Number::from(0)))
    }
    fn max_x_value(&self) -> AttributeValue {
        AttributeValue::Number(Number::from_f64(self.max_x).unwrap_or_else(|| Number::from(0)))
    }
    fn min_y_value(&self) -> AttributeValue {
        AttributeValue::Number(Number::from_f64(self.min_y).unwrap_or_else(|| Number::from(0)))
    }
    fn max_y_value(&self) -> AttributeValue {
        AttributeValue::Number(Number::from_f64(self.max_y).unwrap_or_else(|| Number::from(0)))
    }
    fn min_z_value(&self) -> AttributeValue {
        AttributeValue::Number(Number::from_f64(self.min_z).unwrap_or_else(|| Number::from(0)))
    }
    fn max_z_value(&self) -> AttributeValue {
        AttributeValue::Number(Number::from_f64(self.max_z).unwrap_or_else(|| Number::from(0)))
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct BoundsExtractorFactory;

impl ProcessorFactory for BoundsExtractorFactory {
    fn name(&self) -> &str {
        "BoundsExtractor"
    }

    fn description(&self) -> &str {
        "Bounds Extractor"
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: BoundsExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::BoundsExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::BoundsExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::BoundsExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = BoundsExtractor { params };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundsExtractorParam {
    xmin: Option<Attribute>,
    xmax: Option<Attribute>,
    ymin: Option<Attribute>,
    ymax: Option<Attribute>,
    zmin: Option<Attribute>,
    zmax: Option<Attribute>,
}

#[derive(Debug, Clone)]
pub struct BoundsExtractor {
    params: BoundsExtractorParam,
}

impl Processor for BoundsExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = feature.geometry.clone();
        let geometry_value = geometry.value.clone();
        let bounds = match geometry_value {
            GeometryValue::None => None,
            GeometryValue::CityGmlGeometry(city_gml) => Self::calc_city_gml(&city_gml),
            GeometryValue::FlowGeometry2D(flow_2d) => Self::calc_2d(&flow_2d),
            GeometryValue::FlowGeometry3D(flow_3d) => Self::calc_3d(&flow_3d),
        };
        if let Some(bounds) = bounds {
            let mut attributes = feature.attributes.clone();

            attributes.insert(
                self.params.xmin.clone().unwrap_or(Attribute::new("xmin")),
                bounds.min_x_value(),
            );
            attributes.insert(
                self.params.xmax.clone().unwrap_or(Attribute::new("xmax")),
                bounds.max_x_value(),
            );
            attributes.insert(
                self.params.ymin.clone().unwrap_or(Attribute::new("ymin")),
                bounds.min_y_value(),
            );
            attributes.insert(
                self.params.ymax.clone().unwrap_or(Attribute::new("ymax")),
                bounds.max_y_value(),
            );
            attributes.insert(
                self.params.zmin.clone().unwrap_or(Attribute::new("zmin")),
                bounds.min_z_value(),
            );
            attributes.insert(
                self.params.zmax.clone().unwrap_or(Attribute::new("zmax")),
                bounds.max_z_value(),
            );

            let feature = Feature {
                attributes,
                ..feature.clone()
            };
            fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
        };
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BoundsExtractor"
    }
}

impl BoundsExtractor {
    fn update_bounds(current_bounds: Option<Bounds>, new_bounds: Option<Bounds>) -> Option<Bounds> {
        match (current_bounds, new_bounds) {
            (Some(mut cb), Some(nb)) => {
                cb.min_x = cb.min_x.min(nb.min_x);
                cb.max_x = cb.max_x.max(nb.max_x);
                cb.min_y = cb.min_y.min(nb.min_y);
                cb.max_y = cb.max_y.max(nb.max_y);
                cb.min_z = cb.min_z.min(nb.min_z);
                cb.max_z = cb.max_z.max(nb.max_z);
                Some(cb)
            }
            (None, Some(nb)) => Some(nb),
            (Some(cb), None) => Some(cb),
            (None, None) => None,
        }
    }

    fn update_bounds_for_coord<T, Z>(
        bounds: Option<Bounds>,
        coord: &Coordinate<T, Z>,
    ) -> Option<Bounds>
    where
        T: CoordNum + NumCast + PartialOrd + Debug + Copy,
        Z: CoordNum,
    {
        Self::update_bounds(
            bounds,
            Some(Bounds {
                min_x: NumCast::from(coord.x).unwrap(),
                max_x: NumCast::from(coord.x).unwrap(),
                min_y: NumCast::from(coord.y).unwrap(),
                max_y: NumCast::from(coord.y).unwrap(),
                min_z: NumCast::from(coord.z).unwrap(),
                max_z: NumCast::from(coord.z).unwrap(),
            }),
        )
    }

    fn update_bounds_for_point<T, Z>(point: &Point<T, Z>) -> Option<Bounds>
    where
        T: CoordNum + NumCast + PartialOrd + Debug + Copy,
        Z: CoordNum,
    {
        Some(Bounds {
            min_x: NumCast::from(point.x()).unwrap(),
            max_x: NumCast::from(point.x()).unwrap(),
            min_y: NumCast::from(point.y()).unwrap(),
            max_y: NumCast::from(point.y()).unwrap(),
            min_z: NumCast::from(point.z()).unwrap(),
            max_z: NumCast::from(point.z()).unwrap(),
        })
    }

    fn calc_city_gml(geos: &CityGmlGeometry) -> Option<Bounds> {
        let mut out_bounds: Option<Bounds> = None;

        geos.gml_geometries.iter().for_each(|geo_feature| {
            let mut bounds: Option<Bounds> = None;
            for polygon in &geo_feature.polygons {
                let p = Geometry::Polygon(polygon.clone());
                match p {
                    Geometry::Point(point) => {
                        bounds = Self::update_bounds_for_point(&point);
                    }
                    Geometry::Line(line) => {
                        for coord in &[line.start, line.end] {
                            bounds = Self::update_bounds_for_coord(bounds, coord);
                        }
                    }
                    Geometry::LineString(line_string) => {
                        for coord in line_string.0.iter() {
                            bounds = Self::update_bounds_for_coord(bounds, coord);
                        }
                    }
                    Geometry::Polygon(polygon) => {
                        for coord in polygon.exterior().0.iter() {
                            bounds = Self::update_bounds_for_coord(bounds, coord);
                        }
                    }
                    _ => {}
                }
            }
            if let Some(ref mut out_bounds) = out_bounds {
                *out_bounds = Self::update_bounds(Some(out_bounds.clone()), bounds).unwrap();
            } else {
                out_bounds = bounds;
            }
        });
        out_bounds
    }

    fn calc_2d(geos: &Geometry2D) -> Option<Bounds> {
        let mut bounds: Option<Bounds> = None;
        match geos {
            Geometry2D::Point(point) => {
                bounds = Self::update_bounds_for_point(point);
            }
            Geometry2D::Line(line) => {
                for coord in &[line.start, line.end] {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry2D::LineString(line_string) => {
                for coord in line_string.0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry2D::Polygon(polygon) => {
                for coord in polygon.exterior().0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            _ => {}
        }
        bounds
    }

    fn calc_3d(geos: &Geometry3D) -> Option<Bounds> {
        let mut bounds: Option<Bounds> = None;
        match geos {
            Geometry3D::Point(point) => {
                bounds = Self::update_bounds_for_point(point);
            }
            Geometry3D::Line(line) => {
                for coord in &[line.start, line.end] {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry3D::LineString(line_string) => {
                for coord in line_string.0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry3D::Polygon(polygon) => {
                for coord in polygon.exterior().0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            _ => {}
        }
        bounds
    }
}

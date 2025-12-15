use std::collections::HashMap;

use reearth_flow_geometry::algorithm::{area2d::Area2D, area3d::Area3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AreaCalculatorFactory;

impl ProcessorFactory for AreaCalculatorFactory {
    fn name(&self) -> &str {
        "AreaCalculator"
    }

    fn description(&self) -> &str {
        "Calculates the planar or sloped area of polygon geometries and adds the results as attributes"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaCalculator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let calculator: AreaCalculator = if let Some(with) = with {
            // using a serde_json roundtrip (converting to Value and then back from Value) as
            // a way to deserialize the HashMap<String, Value> parameter into an AreaCalculator struct.
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaCalculatorFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaCalculatorFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            // Use defaults if no parameters provided
            AreaCalculator {
                area_type: AreaType::PlaneArea,
                output_attribute: Attribute::new("area".to_string()),
                multiplier: 1.0,
            }
        };
        Ok(Box::new(calculator))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum AreaType {
    #[serde(alias = "plane_area")]
    #[serde(alias = "planeArea")]
    PlaneArea,
    #[serde(alias = "sloped_area")]
    #[serde(alias = "slopedArea")]
    SlopedArea,
}

impl Default for AreaType {
    fn default() -> Self {
        AreaType::PlaneArea
    }
}

/// # AreaCalculator Parameters
///
/// Configuration for calculating areas of geometries.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaCalculator {
    /// Type of area calculation to perform (PlaneArea or SlopedArea)
    #[serde(default)]
    area_type: AreaType,

    /// Name of the attribute to store the calculated area (default: "area")
    #[serde(default = "default_output_attribute")]
    output_attribute: Attribute,

    /// Multiplier to scale the area values (default: 1.0)
    #[serde(default = "default_multiplier")]
    multiplier: f64,
}

fn default_output_attribute() -> Attribute {
    Attribute::new("area".to_string())
}

fn default_multiplier() -> f64 {
    1.0
}

impl Processor for AreaCalculator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }

        // Calculate area based on the geometry type
        let area = match &geometry.value {
            GeometryValue::None => 0.0,
            GeometryValue::FlowGeometry2D(geom_2d) => geom_2d.unsigned_area2d() * self.multiplier,
            GeometryValue::FlowGeometry3D(geom_3d) => {
                // For 3D geometries, the behavior depends on the area type
                match self.area_type {
                    AreaType::PlaneArea => {
                        // For plane area, we convert the 3D geometry to 2D (dropping Z coordinates)
                        // and then calculate the area
                        let projected_2d: reearth_flow_geometry::types::geometry::Geometry2D<_> =
                            geom_3d.clone().into();
                        projected_2d.unsigned_area2d() * self.multiplier
                    }
                    AreaType::SlopedArea => {
                        // Calculate the true 3D area including Z coordinates
                        geom_3d.unsigned_area3d() * self.multiplier
                    }
                }
            }
            GeometryValue::CityGmlGeometry(city_gml_geom) => {
                // For CityGML geometry, we calculate area for each polygon
                let mut total_area = 0.0;
                for gml_feature in &city_gml_geom.gml_geometries {
                    for polygon in &gml_feature.polygons {
                        match self.area_type {
                            AreaType::PlaneArea => {
                                // Convert 3D polygon to 2D for plane area calculation
                                let projected_2d: reearth_flow_geometry::types::polygon::Polygon2D<
                                    _,
                                > = polygon.clone().into();
                                total_area += projected_2d.unsigned_area2d();
                            }
                            AreaType::SlopedArea => {
                                total_area += polygon.unsigned_area3d();
                            }
                        }
                    }
                }
                total_area * self.multiplier
            }
        };

        // Create a new feature with the calculated area attribute
        let mut new_feature = feature.clone();
        new_feature.attributes.insert(
            self.output_attribute.clone(),
            AttributeValue::Number(
                serde_json::Number::from_f64(area).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );

        fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AreaCalculator"
    }
}

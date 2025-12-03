use std::collections::HashMap;

use super::errors::GeometryProcessorError;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::polygon::Polygon3D;
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
use serde_json::{Number, Value};

#[derive(Debug, Clone, Default)]
pub(super) struct PolygonNormalExtractorFactory;

impl ProcessorFactory for PolygonNormalExtractorFactory {
    fn name(&self) -> &str {
        "PolygonNormalExtractor"
    }

    fn description(&self) -> &str {
        "Extract normal vectors and other properties for polygon features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(PolygonNormalExtractor))
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
        let normal_polygon: PolygonNormalExtractor = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::PolygonNormalExtractorFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::PolygonNormalExtractorFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::PolygonNormalExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(normal_polygon))
    }
}

/// # PolygonNormalExtractor Parameters
///
/// Configuration for calculating normal vectors from polygon features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PolygonNormalExtractor {
    /// Whether to add normalX attribute (default: true)
    #[serde(default = "default_true")]
    add_normal_x: bool,

    /// Whether to add normalY attribute (default: true)
    #[serde(default = "default_true")]
    add_normal_y: bool,

    /// Whether to add normalZ attribute (default: true)
    #[serde(default = "default_true")]
    add_normal_z: bool,

    /// Whether to add signedArea2D attribute (default: true)
    #[serde(default = "default_true")]
    add_signed_area_2d: bool,

    /// Whether to add Slope attribute (default: true)
    #[serde(default = "default_true")]
    add_slope: bool,

    /// Whether to add Azimuth attribute (default: true)
    #[serde(default = "default_true")]
    add_azimuth: bool,
}
#[allow(unused)]
fn default_true() -> bool {
    true
}

impl Processor for PolygonNormalExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }

        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                match geos {
                    Geometry2D::Polygon(_polygon) => {
                        // Calculate normal properties for 2D polygons
                        // Just pass through since 2D polygons don't have meaningful 3D normals
                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                    Geometry2D::MultiPolygon(_) => {
                        // For MultiPolygon, just pass through
                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                    _ => {
                        // For non-polygon geometries, pass through unchanged
                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                }
            }
            GeometryValue::FlowGeometry3D(geos) => {
                match geos {
                    Geometry3D::Polygon(polygon) => {
                        // Calculate normal properties for 3D polygons
                        let normal_result =
                            PolygonNormalExtractor::calculate_normal_properties_3d(polygon);

                        // Add calculated attributes to the feature
                        if self.add_normal_x {
                            feature.attributes.insert(
                                Attribute::new("normalX".to_string()),
                                AttributeValue::Number(
                                    Number::from_f64(normal_result.normal_x).ok_or_else(|| {
                                        GeometryProcessorError::PolygonNormalExtractor(
                                            "Failed to convert normalX to JSON number".to_string(),
                                        )
                                    })?,
                                ),
                            );
                        }
                        if self.add_normal_y {
                            feature.attributes.insert(
                                Attribute::new("normalY".to_string()),
                                AttributeValue::Number(
                                    Number::from_f64(normal_result.normal_y).ok_or_else(|| {
                                        GeometryProcessorError::PolygonNormalExtractor(
                                            "Failed to convert normalY to JSON number".to_string(),
                                        )
                                    })?,
                                ),
                            );
                        }
                        if self.add_normal_z {
                            feature.attributes.insert(
                                Attribute::new("normalZ".to_string()),
                                AttributeValue::Number(
                                    Number::from_f64(normal_result.normal_z).ok_or_else(|| {
                                        GeometryProcessorError::PolygonNormalExtractor(
                                            "Failed to convert normalZ to JSON number".to_string(),
                                        )
                                    })?,
                                ),
                            );
                        }
                        if self.add_signed_area_2d {
                            feature.attributes.insert(
                                Attribute::new("signedArea2D".to_string()),
                                AttributeValue::Number(
                                    Number::from_f64(normal_result.signed_area_2d).ok_or_else(
                                        || {
                                            GeometryProcessorError::PolygonNormalExtractor(
                                                "Failed to convert signedArea2D to JSON number"
                                                    .to_string(),
                                            )
                                        },
                                    )?,
                                ),
                            );
                        }
                        if self.add_slope {
                            feature.attributes.insert(
                                Attribute::new("Slope".to_string()),
                                AttributeValue::Number(
                                    Number::from_f64(normal_result.slope).ok_or_else(|| {
                                        GeometryProcessorError::PolygonNormalExtractor(
                                            "Failed to convert slope to JSON number".to_string(),
                                        )
                                    })?,
                                ),
                            );
                        }
                        if self.add_azimuth {
                            feature.attributes.insert(
                                Attribute::new("Azimuth".to_string()),
                                AttributeValue::Number(
                                    Number::from_f64(normal_result.azimuth).ok_or_else(|| {
                                        GeometryProcessorError::PolygonNormalExtractor(
                                            "Failed to convert azimuth to JSON number".to_string(),
                                        )
                                    })?,
                                ),
                            );
                        }

                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                    Geometry3D::MultiPolygon(multi_polygon) => {
                        // Calculate normal for the first polygon in the multipolygon
                        if let Some(first_polygon) = multi_polygon.iter().next() {
                            let normal_result =
                                PolygonNormalExtractor::calculate_normal_properties_3d(
                                    first_polygon,
                                );

                            // Add calculated attributes to the feature
                            if self.add_normal_x {
                                feature.attributes.insert(
                                    Attribute::new("normalX".to_string()),
                                    AttributeValue::Number(
                                        Number::from_f64(normal_result.normal_x).ok_or_else(
                                            || {
                                                GeometryProcessorError::PolygonNormalExtractor(
                                                    "Failed to convert normalX to JSON number"
                                                        .to_string(),
                                                )
                                            },
                                        )?,
                                    ),
                                );
                            }
                            if self.add_normal_y {
                                feature.attributes.insert(
                                    Attribute::new("normalY".to_string()),
                                    AttributeValue::Number(
                                        Number::from_f64(normal_result.normal_y).ok_or_else(
                                            || {
                                                GeometryProcessorError::PolygonNormalExtractor(
                                                    "Failed to convert normalY to JSON number"
                                                        .to_string(),
                                                )
                                            },
                                        )?,
                                    ),
                                );
                            }
                            if self.add_normal_z {
                                feature.attributes.insert(
                                    Attribute::new("normalZ".to_string()),
                                    AttributeValue::Number(
                                        Number::from_f64(normal_result.normal_z).ok_or_else(
                                            || {
                                                GeometryProcessorError::PolygonNormalExtractor(
                                                    "Failed to convert normalZ to JSON number"
                                                        .to_string(),
                                                )
                                            },
                                        )?,
                                    ),
                                );
                            }
                            if self.add_signed_area_2d {
                                feature.attributes.insert(
                                    Attribute::new("signedArea2D".to_string()),
                                    AttributeValue::Number(
                                        Number::from_f64(normal_result.signed_area_2d).ok_or_else(
                                            || {
                                                GeometryProcessorError::PolygonNormalExtractor(
                                                    "Failed to convert signedArea2D to JSON number"
                                                        .to_string(),
                                                )
                                            },
                                        )?,
                                    ),
                                );
                            }
                            if self.add_slope {
                                feature.attributes.insert(
                                    Attribute::new("Slope".to_string()),
                                    AttributeValue::Number(
                                        Number::from_f64(normal_result.slope).ok_or_else(|| {
                                            GeometryProcessorError::PolygonNormalExtractor(
                                                "Failed to convert slope to JSON number"
                                                    .to_string(),
                                            )
                                        })?,
                                    ),
                                );
                            }
                            if self.add_azimuth {
                                feature.attributes.insert(
                                    Attribute::new("Azimuth".to_string()),
                                    AttributeValue::Number(
                                        Number::from_f64(normal_result.azimuth).ok_or_else(
                                            || {
                                                GeometryProcessorError::PolygonNormalExtractor(
                                                    "Failed to convert azimuth to JSON number"
                                                        .to_string(),
                                                )
                                            },
                                        )?,
                                    ),
                                );
                            }
                        }

                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                    _ => {
                        // For non-polygon geometries, pass through unchanged
                        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                    }
                }
            }
            GeometryValue::CityGmlGeometry(_) => {
                // For now, just pass CityGML geometries through unchanged
                // Normal calculation for CityGML would require more complex handling
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "PolygonNormalExtractor"
    }
}

// Structure to hold normal calculation results
#[derive(Debug, PartialEq)]
struct NormalResult {
    normal_x: f64,
    normal_y: f64,
    normal_z: f64,
    signed_area_2d: f64,
    slope: f64,
    azimuth: f64,
}

impl PolygonNormalExtractor {
    // TODO: write unit test for this function for Polygon3D
    fn calculate_normal_properties_3d(polygon: &Polygon3D<f64>) -> NormalResult {
        // Get the exterior ring of the polygon
        let exterior = polygon.exterior();
        let coords: Vec<_> = exterior.coords().collect();

        if coords.len() < 3 {
            // Not enough points to calculate a normal
            return NormalResult {
                normal_x: 0.0,
                normal_y: 0.0,
                normal_z: 0.0,
                signed_area_2d: 0.0,
                slope: 0.0,
                azimuth: 0.0,
            };
        }

        // Calculate surface normal using Newell's method (based on the Python implementation)
        let mut normal_x = 0.0;
        let mut normal_y = 0.0;
        let mut normal_z = 0.0;
        let mut signed_area_2d = 0.0;

        let coord_count = coords.len() - 1; // Exclude the closing vertex which is the same as the first
        for i in 0..coord_count {
            let before = (i + coord_count - 1) % coord_count;
            let after = (i + 1) % coord_count;

            // coords0 is the point before
            let coords0 = &coords[before];
            // coords1 is the current point
            let coords1 = &coords[i];
            // coords2 is the point after
            let coords2 = &coords[after];

            let x1 = coords0.x;
            let x2 = coords1.x;
            let x3 = coords2.x;

            let y1 = coords0.y;
            let y2 = coords1.y;
            let y3 = coords2.y;

            let z1 = coords0.z;
            let z2 = coords1.z;
            let z3 = coords2.z;

            // Calculate normal
            normal_x += (y2 - y1) * (z3 - z1) - (y3 - y1) * (z2 - z1);
            normal_y += (z2 - z1) * (x3 - x1) - (z3 - z1) * (x2 - x1);
            normal_z += (x2 - x1) * (y3 - y1) - (x3 - x1) * (y2 - y1);

            // Polygon area (Gauss area formula)
            let area_tri = ((y2 + y3) * (x2 - x3)) / 2.0;
            signed_area_2d += area_tri;
        }

        // Normalize the vector
        let length = (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();

        let (normalized_normal_x, normalized_normal_y, normalized_normal_z, azimuth) =
            if length != 0.0 {
                let normalized_normal_x = normal_x / length;
                let normalized_normal_y = normal_y / length;
                let normalized_normal_z = normal_z / length;
                // Calculate azimuth (angle in horizontal plane)
                let azimuth = (-normalized_normal_x)
                    .atan2(-normalized_normal_y)
                    .to_degrees();
                // Ensure azimuth is in the range [0, 360)
                let azimuth = if azimuth < 0.0 {
                    azimuth + 360.0
                } else {
                    azimuth
                };
                (
                    normalized_normal_x,
                    normalized_normal_y,
                    normalized_normal_z,
                    azimuth,
                )
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

        // Calculate slope (angle from normal to Z axis) - matching Python implementation
        let slope = if normalized_normal_z != 0.0 {
            ((normalized_normal_x * normalized_normal_x
                + normalized_normal_y * normalized_normal_y)
                .sqrt()
                / normalized_normal_z)
                .atan()
                .to_degrees()
                .abs()
        } else {
            90.0
        };

        NormalResult {
            normal_x: normalized_normal_x,
            normal_y: normalized_normal_y,
            normal_z: normalized_normal_z,
            signed_area_2d,
            slope,
            azimuth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::Coordinate3D;
    use reearth_flow_geometry::types::line_string::LineString3D;

    #[test]
    fn case01_validate_polygon3d_normal() {
        // Using the coordinates from the Python implementation
        // Create the coordinate vector
        let coordinates = vec![
            Coordinate3D::new__(-310443.027642464, 42024.572164127916, 11.64245729), // First point
            Coordinate3D::new__(-310443.2722657761, 42017.31538718905, 10.06246068),
            Coordinate3D::new__(-310413.8126146904, 42016.32232996605, 10.06245526),
            Coordinate3D::new__(-310413.567998781, 42023.57910553894, 11.64245187),
            Coordinate3D::new__(-310443.027642464, 42024.572164127916, 11.64245729), // Last point (same as first to close the polygon)
        ];

        // Create the exterior ring as a LineString3D
        let exterior = LineString3D::new(coordinates);
        // Create the Polygon3D with the exterior ring (no interior rings)
        let polygon = Polygon3D::new(exterior, vec![]);

        let result = PolygonNormalExtractor::calculate_normal_properties_3d(&polygon);

        // Expected values from the Python implementation
        let expected_polygon_normal = NormalResult {
            normal_x: -0.0071632345554907256,
            normal_y: -0.21250690309836642,
            normal_z: 0.9771333093320709,
            slope: 12.282607611747856,
            signed_area_2d: 214.02499108342454,
            azimuth: 1.9306091257916358,
        };

        assert_eq!(expected_polygon_normal.normal_x, result.normal_x);
        assert_eq!(expected_polygon_normal.normal_y, result.normal_y);
        assert_eq!(expected_polygon_normal.normal_z, result.normal_z);
        assert_eq!(
            expected_polygon_normal.signed_area_2d,
            result.signed_area_2d
        );
        assert_eq!(expected_polygon_normal.azimuth, result.azimuth);
        assert!((result.slope - expected_polygon_normal.slope).abs() < 1e-2);
    }
}

use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::{area2d::Area2D, area3d::Area3D};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::Area;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AreaCalculatorFactory;

impl ProcessorFactory for AreaCalculatorFactory {
    fn name(&self) -> &str {
        "Area Calculator"
    }

    fn description(&self) -> &str {
        "Calculates the planar or sloped area of a feature's geometry and stores it in an attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaCalculator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
            AreaCalculator::default()
        };
        Ok(Box::new(calculator))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
enum AreaType {
    /// # Planar Area
    /// Calculates the flat area of the geometry projected onto the XY plane.
    #[serde(alias = "plane_area")]
    #[serde(alias = "planeArea")]
    #[default]
    PlaneArea,
    /// # Sloped Area
    /// Calculates the true surface area, accounting for the slope of each face.
    #[serde(alias = "sloped_area")]
    #[serde(alias = "slopedArea")]
    SlopedArea,
}

/// # Area Calculator Parameters
///
/// Configure how the area of each feature's geometry is measured and stored.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaCalculator {
    /// # Area Type
    /// Whether to measure the flat projected area or the true sloped surface
    /// area. Has no effect on a geometry with no elevation, which is always flat.
    #[serde(default)]
    area_type: AreaType,

    /// # Output Attribute
    /// Attribute to store the calculated area in. Defaults to `area`.
    #[serde(default = "default_output_attribute")]
    output_attribute: Attribute,

    /// # Multiplier
    /// Factor applied to the calculated area, for converting to another unit.
    /// Defaults to 1.0.
    #[serde(default = "default_multiplier")]
    multiplier: f64,
}

impl Default for AreaCalculator {
    fn default() -> Self {
        Self {
            area_type: AreaType::default(),
            output_attribute: default_output_attribute(),
            multiplier: default_multiplier(),
        }
    }
}

fn default_output_attribute() -> Attribute {
    Attribute::new("area".to_string())
}

fn default_multiplier() -> f64 {
    1.0
}

impl Processor for AreaCalculator {
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let geometry = &*ctx.feature.geometry;
        let measured = match self.area_type {
            AreaType::PlaneArea => geometry.projected_area(),
            AreaType::SlopedArea => geometry.surface_area(),
        };
        // The attribute is always written, so an unmeasurable geometry records
        // zero — but says so, rather than passing silently.
        let area = match measured {
            Ok(area) => area * self.multiplier,
            Err(why) => {
                ctx.event_hub.warn_log(
                    Some(ctx.error_span()),
                    format!("area not measurable, writing 0: {why}"),
                );
                0.0
            }
        };

        let mut feature = ctx.feature.clone();
        feature.attributes_mut().insert(
            self.output_attribute.clone(),
            AttributeValue::Number(
                serde_json::Number::from_f64(area).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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

        // A geometry with no area — a point, a curve, or no geometry at all —
        // measures zero. The attribute is written either way, so that downstream
        // steps never have to distinguish "no area" from "not measured".
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
        new_feature.attributes_mut().insert(
            self.output_attribute.clone(),
            AttributeValue::Number(
                serde_json::Number::from_f64(area).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );

        fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Area Calculator"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::csg::Csg;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::solid::Solid;
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3DData;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;
    use serde_json::json;

    /// The unit square tilted 45 degrees about the x axis: 1.0 of surface,
    /// 1/sqrt(2) of it projected. Any test that can tell the two area types
    /// apart has to use a sloped face.
    fn tilted_square() -> Geometry {
        let h = std::f64::consts::FRAC_1_SQRT_2;
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, h, h],
                    [0.0, h, h],
                    [0.0, 0.0, 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    fn build(with: Option<Value>) -> Box<dyn Processor> {
        let with = with.map(|value| serde_json::from_value(value).unwrap());
        AreaCalculatorFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "Area Calculator".to_string(),
                with,
            )
            .unwrap()
    }

    /// Run `processor` over one feature and return the single feature it sent.
    fn run(processor: &mut dyn Processor, feature: &Feature) -> Feature {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        processor
            .process(create_default_execute_context(feature), &fw)
            .unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let sent = noop.send_features.lock().unwrap().clone();
        assert_eq!(sent.len(), 1, "the action forwards exactly one feature");
        sent.into_iter().next().unwrap()
    }

    fn area_of(feature: &Feature, attribute: &str) -> f64 {
        match feature
            .attributes
            .get(&Attribute::new(attribute.to_string()))
        {
            Some(AttributeValue::Number(n)) => n.as_f64().unwrap(),
            other => panic!("expected a number in `{attribute}`, got {other:?}"),
        }
    }

    /// The two area types must reach the two different measures. A flat face
    /// would pass with either one wired to both.
    #[test]
    fn plane_area_and_sloped_area_measure_differently() {
        let feature = Feature::from(tilted_square());

        let plane = run(
            &mut *build(Some(json!({"areaType": "planeArea"}))),
            &feature,
        );
        assert!(
            (area_of(&plane, "area") - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12,
            "plane area was {}",
            area_of(&plane, "area")
        );

        let sloped = run(
            &mut *build(Some(json!({"areaType": "slopedArea"}))),
            &feature,
        );
        assert!((area_of(&sloped, "area") - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_multiplier_scales_the_result() {
        let feature = Feature::from(tilted_square());
        let out = run(
            &mut *build(Some(json!({"areaType": "slopedArea", "multiplier": 4.0}))),
            &feature,
        );
        assert!((area_of(&out, "area") - 4.0).abs() < 1e-12);
    }

    #[test]
    fn the_output_attribute_defaults_to_area_and_can_be_named() {
        let feature = Feature::from(tilted_square());

        let default = run(&mut *build(None), &feature);
        assert!(default
            .attributes
            .get(&Attribute::new("area".to_string()))
            .is_some());

        let named = run(
            &mut *build(Some(json!({"outputAttribute": "roofArea"}))),
            &feature,
        );
        assert!((area_of(&named, "roofArea") - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
    }

    /// The old code commented this explicitly and the promise is kept: a
    /// geometry with no area still gets the attribute, so downstream steps
    /// never have to tell "no area" from "not measured".
    #[test]
    fn a_feature_with_no_geometry_still_gets_the_attribute() {
        let out = run(&mut *build(None), &Feature::from(Geometry::None));
        assert_eq!(area_of(&out, "area"), 0.0);
    }

    /// So does an unmeasurable one — it writes zero rather than failing the
    /// feature or leaving the attribute off. This is the only path that reaches
    /// the action's `Err` arm, since `Csg` is the model's one unmeasurable type.
    #[test]
    fn an_unmeasurable_geometry_still_gets_the_attribute() {
        let solid = || {
            Solid::from_exterior(
                CoordinateFrame::Euclidean,
                TriangularMesh3DData::from_parts(
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    [0u32, 1, 2],
                )
                .unwrap(),
            )
        };
        let csg = Csg::Union(Box::new(solid().into()), Box::new(solid().into()));
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg));

        let out = run(&mut *build(None), &Feature::from(g));
        assert_eq!(area_of(&out, "area"), 0.0);
    }

    #[test]
    fn attributes_already_on_the_feature_survive() {
        let mut feature = Feature::from(tilted_square());
        feature.insert("buildingId", AttributeValue::Number(7.into()));
        let out = run(&mut *build(None), &feature);
        assert_eq!(
            out.attributes
                .get(&Attribute::new("buildingId".to_string())),
            Some(&AttributeValue::Number(7.into()))
        );
    }
}

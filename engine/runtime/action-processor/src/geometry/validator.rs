use std::collections::HashMap;

use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{GeoFloat, GeoNum},
    types::geometry::Geometry as FlowGeometry,
    validation::*,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static SUCCESS_PORT: Lazy<Port> = Lazy::new(|| Port::new("success"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

#[derive(Debug, Clone, Default)]
pub struct GeometryValidatorFactory;

impl ProcessorFactory for GeometryValidatorFactory {
    fn name(&self) -> &str {
        "GeometryValidator"
    }

    fn description(&self) -> &str {
        "Validates the geometry of a feature"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryValidator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            SUCCESS_PORT.clone(),
            FAILED_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let processor: GeometryValidator = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryValidatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(processor))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum ValidationType {
    #[serde(rename = "duplicatePoints")]
    DuplicatePoints,
    #[serde(rename = "corruptGeometry")]
    CorruptGeometry,
    #[serde(rename = "selfIntersection")]
    SelfIntersection,
}

impl From<ValidationType> for reearth_flow_geometry::validation::ValidationType {
    fn from(validation_type: ValidationType) -> Self {
        match validation_type {
            ValidationType::DuplicatePoints => {
                reearth_flow_geometry::validation::ValidationType::DuplicatePoints
            }
            ValidationType::CorruptGeometry => {
                reearth_flow_geometry::validation::ValidationType::CorruptGeometry
            }
            ValidationType::SelfIntersection => {
                reearth_flow_geometry::validation::ValidationType::SelfIntersection
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidationResult {
    error_count: usize,
    details: Vec<serde_json::Value>,
}

impl ValidationResult {
    fn merge(results: Vec<Self>) -> Self {
        let error_count = results.iter().map(|result| result.error_count).sum();
        let details = results
            .into_iter()
            .flat_map(|result| result.details)
            .collect();
        Self {
            error_count,
            details,
        }
    }
}

impl From<ValidationProblemReport> for ValidationResult {
    fn from(report: ValidationProblemReport) -> Self {
        Self {
            error_count: report.error_count(),
            details: report
                .reports()
                .into_iter()
                .map(|detail| serde_json::to_value(detail).unwrap())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryValidator {
    validation_types: Vec<ValidationType>,
}

impl Processor for GeometryValidator {
    fn num_threads(&self) -> usize {
        2
    }

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
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                self.process_flow_geometry(&ctx, fw, geometry)?;
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                self.process_flow_geometry(&ctx, fw, geometry)?;
            }
            GeometryValue::CityGmlGeometry(gml_geometry) => {
                let result = gml_geometry
                    .gml_geometries
                    .iter()
                    .flat_map(|feature| {
                        feature.polygons.iter().map(|polygon| {
                            let mut result = Vec::new();
                            for validation_type in &self.validation_types {
                                if let Some(report) =
                                    polygon.validate(validation_type.clone().into())
                                {
                                    result.push(ValidationResult::from(report));
                                }
                            }
                            result
                        })
                    })
                    .flatten()
                    .collect::<Vec<ValidationResult>>();
                if result.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
                } else {
                    let mut feature = feature.clone();
                    feature.insert(
                        "validationResult",
                        serde_json::to_value(ValidationResult::merge(result))?.into(),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                }
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometryValidator"
    }
}

impl GeometryValidator {
    fn process_flow_geometry<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64> + FromPrimitive + GeoFloat,
        Z: GeoNum + approx::AbsDiffEq<Epsilon = f64> + FromPrimitive + GeoFloat,
    >(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        geometry: &FlowGeometry<T, Z>,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let mut result = Vec::new();
        for validation_type in &self.validation_types {
            if let Some(report) = geometry.validate(validation_type.clone().into()) {
                result.push(ValidationResult::from(report));
            }
        }
        if result.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
        } else {
            let mut feature = feature.clone();
            feature.insert(
                "validationResult",
                serde_json::to_value(ValidationResult::merge(result))?.into(),
            );
            fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
        }
        Ok(())
    }
}

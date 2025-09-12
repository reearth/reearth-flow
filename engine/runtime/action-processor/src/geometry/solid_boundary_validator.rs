use std::collections::HashMap;

use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::{GeoFloat, GeoNum},
    types::{coordnum::CoordNum, geometry::Geometry as FlowGeometry},
    validation::*,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{geometry::CityGmlGeometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

static SUCCESS_PORT: Lazy<Port> = Lazy::new(|| Port::new("success"));
static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

#[derive(Debug, Clone, Default)]
pub struct SoilidBoundaryValidatorFactory;

impl ProcessorFactory for SoilidBoundaryValidatorFactory {
    fn name(&self) -> &str {
        "SolidBoundaryValidator"
    }

    fn description(&self) -> &str {
        "Validates the Solid Boundary Geometry"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SolidBoundaryValidator))
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
        let processor: SolidBoundaryValidator = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::SoilidBoundaryValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::SoilidBoundaryValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::SoilidBoundaryValidatorFactory(
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
    #[serde(rename = "duplicateConsecutivePoints")]
    DuplicateConsecutivePoints,
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
            ValidationType::DuplicateConsecutivePoints => {
                reearth_flow_geometry::validation::ValidationType::DuplicateConsecutivePoints
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

/// # Solid Boundary Validator Parameters
/// Configure which validation checks to perform on feature geometries
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SolidBoundaryValidator {
    /// # Validation Types
    /// List of validation checks to perform on the geometry (duplicate points, corrupt geometry, self-intersection)
    validation_types: Vec<ValidationType>,
}

impl Processor for SolidBoundaryValidator {
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
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "SolidBoundaryValidator"
    }
}

impl SolidBoundaryValidator {
    fn check_manifold_condition<T: GeoFloat + FromPrimitive>(
        &self,
        geometry: &FlowGeometry<T>,
    ) -> Result<ValidationResult, BoxedError> {
        let mut results = Vec::new();
        for vt in &self.validation_types {
        }
        Ok(ValidationResult::merge(results))
    }
}

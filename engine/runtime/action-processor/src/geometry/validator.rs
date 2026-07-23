use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
#[cfg(not(feature = "new-geometry"))]
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
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{geometry::CityGmlGeometry, GeometryValue};
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
        "Geometry Validator"
    }

    fn description(&self) -> &str {
        "Validate Feature Geometry Quality"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryValidator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["validate"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
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
    #[serde(rename = "duplicateConsecutivePoints")]
    DuplicateConsecutivePoints(f64),
    /// Corrupt geometry check with optional tolerance for interior/exterior ring intersection.
    #[serde(rename = "corruptGeometry")]
    CorruptGeometry(Option<f64>),
    /// Self-intersection check with optional tolerance.
    /// If tolerance is None or 0.0, exact intersection check is performed.
    /// If tolerance > 0.0, intersections within tolerance distance are ignored.
    #[serde(rename = "selfIntersection")]
    SelfIntersection(Option<f64>),
}

/// An advisory (optional) validation check that can be individually disabled.
/// A disabled check does not run and is treated as passing. Only checks that the
/// geometry crate classifies as optional are listed here; core validity checks
/// always run and cannot be disabled.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
pub enum OptionalCheck {
    #[serde(rename = "duplicatePoints")]
    DuplicatePoints,
    #[serde(rename = "orientable")]
    Orientable,
    #[serde(rename = "orientation")]
    Orientation,
    #[serde(rename = "shellOrientation")]
    ShellOrientation,
}

/// How the planarity check bounds a face's out-of-plane deviation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PlanarityThreshold {
    /// Dimensionless ratio of the face's convex-hull minimum height to its
    /// diameter; scale-invariant.
    Ratio(f64),
    /// Absolute maximum out-of-plane height, in the coordinate unit (metres).
    /// Applied only in a linear-unit frame, where the planarity check runs.
    MaxHeight(f64),
}

/// The smallest measure a geometry may have before the degeneracy check flags
/// it, per dimension. Each threshold applies to geometries of its dimension.
/// Values are in the coordinate unit (the frame's linear unit, e.g. metres). Each defaults
/// to zero, flagging only an exactly-zero measure.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DegenerateThresholds {
    /// Minimum length of a 1D geometry (line / ring edge).
    #[serde(default)]
    min_length: f64,
    /// Minimum area of a 2D geometry (face / ring).
    #[serde(default)]
    min_area: f64,
    /// Minimum volume of a 3D geometry (solid).
    #[serde(default)]
    min_volume: f64,
}

#[cfg(feature = "new-geometry")]
impl From<DegenerateThresholds> for reearth_flow_geometry::validation_next::DegenerateThresholds {
    fn from(thresholds: DegenerateThresholds) -> Self {
        Self {
            min_length: thresholds.min_length,
            min_area: thresholds.min_area,
            min_volume: thresholds.min_volume,
        }
    }
}

#[cfg(feature = "new-geometry")]
impl From<PlanarityThreshold> for reearth_flow_geometry::validation_next::PlanarityThreshold {
    fn from(threshold: PlanarityThreshold) -> Self {
        use reearth_flow_geometry::validation_next::PlanarityThreshold as Inner;
        match threshold {
            PlanarityThreshold::Ratio(r) => Inner::Ratio(r),
            PlanarityThreshold::MaxHeight(h) => Inner::MaxHeight(h),
        }
    }
}

#[cfg(feature = "new-geometry")]
impl From<OptionalCheck> for reearth_flow_geometry::validation_next::ValidationType {
    fn from(check: OptionalCheck) -> Self {
        use reearth_flow_geometry::validation_next::ValidationType;
        match check {
            OptionalCheck::DuplicatePoints => ValidationType::DuplicatePoints,
            OptionalCheck::Orientable => ValidationType::Orientable,
            OptionalCheck::Orientation => ValidationType::Orientation,
            OptionalCheck::ShellOrientation => ValidationType::ShellOrientation,
        }
    }
}

#[cfg(not(feature = "new-geometry"))]
impl From<ValidationType> for reearth_flow_geometry::validation::ValidationType {
    fn from(validation_type: ValidationType) -> Self {
        match validation_type {
            ValidationType::DuplicatePoints => {
                reearth_flow_geometry::validation::ValidationType::DuplicatePoints
            }
            ValidationType::DuplicateConsecutivePoints(tolerance) => {
                reearth_flow_geometry::validation::ValidationType::DuplicateConsecutivePoints(
                    tolerance,
                )
            }
            ValidationType::CorruptGeometry(tolerance) => {
                reearth_flow_geometry::validation::ValidationType::CorruptGeometry(tolerance)
            }
            ValidationType::SelfIntersection(tolerance) => {
                reearth_flow_geometry::validation::ValidationType::SelfIntersection(tolerance)
            }
        }
    }
}

#[cfg(not(feature = "new-geometry"))]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidationResult {
    error_count: usize,
    details: Vec<serde_json::Value>,
}

#[cfg(not(feature = "new-geometry"))]
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

#[cfg(not(feature = "new-geometry"))]
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

/// # Geometry Validator Parameters
/// Configure which validation checks to perform on feature geometries
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryValidator {
    /// # Validation Types
    #[serde(default)]
    #[cfg(not(feature = "new-geometry"))]
    validation_types: Vec<ValidationType>,

    /// # Disabled Optional Checks
    /// Advisory checks to disable. Disabled checks do not run and are treated as passing;
    /// core validity checks always run. Empty by default, so every optional check runs.
    #[serde(default)]
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    disabled_optional_checks: Vec<OptionalCheck>,

    /// # Planarity Threshold
    /// Optional override for how the planarity check bounds a face's out-of-plane deviation:
    /// a scale-invariant `ratio` (the default), or an absolute `maxHeight` in the frame's linear
    /// unit (linear-unit frames only).
    #[serde(default)]
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    planarity_threshold: Option<PlanarityThreshold>,

    /// # Duplicate Point Tolerance
    /// Optional distance within which two coordinates count as duplicates for the duplicate-points
    /// check. Omitted (the default) means exact-equality detection.
    #[serde(default)]
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    duplicate_tolerance: Option<f64>,

    /// # Degeneracy Thresholds
    /// Minimum length / area / volume below which the degeneracy check flags a geometry, per
    /// dimension. Each defaults to zero, flagging only an exactly-zero measure. Values are in the
    /// coordinate unit (the frame's linear unit, e.g. metres).
    #[serde(default)]
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    degenerate_thresholds: DegenerateThresholds,
}

impl Processor for GeometryValidator {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(not(feature = "new-geometry"))]
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
                self.process_citygml_geometry(&ctx, fw, gml_geometry)?;
            }
        }
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

    /// Validate the feature's geometry with the geometry crate's full validation
    /// matrix. Features with no geometry go to `rejected`, geometries that pass
    /// every applicable check go to `success`, and geometries with at least one
    /// failed check go to `failed` carrying a `validationResult` attribute (the
    /// total problem count and a per-check problem count).
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        use reearth_flow_geometry::validation_next::{
            frame_skips, validate_with, ValidationParams, ValidationResult,
        };
        use reearth_flow_geometry::Geometry;

        let feature = &ctx.feature;
        if matches!(feature.geometry.as_ref(), Geometry::None) {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        // Unit-sensitive checks (planarity, 3D surface self-intersection) are
        // skipped on a angular or unclassifiable CRS; tell the user why and
        // how to enable them.
        let skips = frame_skips(feature.geometry.as_ref());
        if skips.angular {
            ctx.event_hub.warn_log(
                Some(ctx.info_span()),
                format!(
                    "Feature {}: geometry is in an angular-unit (geographic) CRS; \
                     planarity and 3D surface self-intersection were skipped. \
                     Reproject to a linear-unit CRS to enable them.",
                    feature.id
                ),
            );
        }
        for reason in &skips.undeterminable {
            ctx.event_hub.warn_log(
                Some(ctx.info_span()),
                format!(
                    "Feature {}: geometry's CRS could not be classified by PROJ \
                     ({reason}); planarity and 3D surface self-intersection were \
                     skipped. Verify the CRS code and that PROJ data is available.",
                    feature.id
                ),
            );
        }

        let mut params = ValidationParams {
            duplicate_tolerance: self.duplicate_tolerance,
            degenerate: self.degenerate_thresholds.into(),
            ..ValidationParams::default()
        };
        for check in &self.disabled_optional_checks {
            params.disabled_checks.insert((*check).into());
        }
        if let Some(threshold) = self.planarity_threshold {
            params.planarity = threshold.into();
        }

        let mut checks = serde_json::Map::new();
        let mut error_count = 0usize;
        for (check, result) in validate_with(feature.geometry.as_ref(), &params) {
            if let ValidationResult::Failed(positions) = result {
                error_count += positions.len();
                checks.insert(check.to_string(), serde_json::json!(positions.len()));
            }
        }

        if checks.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
        } else {
            let mut feature = feature.clone();
            feature.insert(
                "validationResult",
                serde_json::json!({ "errorCount": error_count, "checks": checks }).into(),
            );
            fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
        }
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

    fn name(&self) -> &str {
        "Geometry Validator"
    }
}

#[cfg(not(feature = "new-geometry"))]
impl GeometryValidator {
    fn process_citygml_geometry(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        gml_geometry: &CityGmlGeometry,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let result = gml_geometry
            .gml_geometries
            .iter()
            .flat_map(|gml_feature| {
                gml_feature.polygons.iter().map(|polygon| {
                    let mut result = Vec::new();
                    for validation_type in &self.validation_types {
                        if let Some(report) = polygon.validate(validation_type.clone().into()) {
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
            let merged = ValidationResult::merge(result);
            let mut feature = feature.clone();
            feature.insert("validationResult", serde_json::to_value(merged)?.into());
            fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
        }
        Ok(())
    }

    fn process_flow_geometry<
        T: GeoNum + approx::AbsDiffEq<Epsilon = f64> + FromPrimitive + GeoFloat + From<Z>,
        Z: CoordNum + GeoNum + approx::AbsDiffEq<Epsilon = f64> + FromPrimitive + GeoFloat,
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

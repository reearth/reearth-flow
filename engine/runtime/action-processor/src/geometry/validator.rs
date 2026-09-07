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
#[cfg(feature = "new-geometry")]
static ISSUE_LOCATIONS_PORT: Lazy<Port> = Lazy::new(|| Port::new("issue-locations"));
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

#[derive(Debug, Clone, Default)]
pub struct GeometryValidatorFactory;

impl ProcessorFactory for GeometryValidatorFactory {
    fn name(&self) -> &str {
        "Geometry Validator"
    }

    fn description(&self) -> &str {
        "Validates feature geometry for issues such as duplicate points, corrupt geometry, or self-intersection."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryValidator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["validation"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            SUCCESS_PORT.clone(),
            FAILED_PORT.clone(),
            // `issue-locations` duplicates what `failed` already carries, one
            // feature per flagged position, so leaving it unwired loses nothing.
            #[cfg(feature = "new-geometry")]
            ISSUE_LOCATIONS_PORT.clone(),
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
    /// # Duplicate Points
    /// Flags coordinates that repeat anywhere in the geometry, using exact equality.
    #[serde(rename = "duplicatePoints")]
    DuplicatePoints,
    /// # Duplicate Consecutive Points
    /// Flags neighbouring coordinates closer together than the given tolerance.
    #[serde(rename = "duplicateConsecutivePoints")]
    DuplicateConsecutivePoints(f64),
    /// # Corrupt Geometry
    /// Flags structurally invalid geometry. Takes an optional tolerance for
    /// interior/exterior ring intersection.
    #[serde(rename = "corruptGeometry")]
    CorruptGeometry(Option<f64>),
    /// # Self-Intersection
    /// Flags geometry that crosses itself. Omitting the tolerance, or setting it
    /// to 0.0, checks for exact intersections; a larger value ignores
    /// intersections within that distance.
    #[serde(rename = "selfIntersection")]
    SelfIntersection(Option<f64>),
}

/// An advisory (optional) validation check that can be individually disabled.
/// A disabled check does not run and is treated as passing. Only checks that the
/// geometry crate classifies as optional are listed here; core validity checks
/// always run and cannot be disabled.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
pub enum OptionalCheck {
    /// # Duplicate Points
    /// Detection of coordinates that repeat within the geometry.
    #[serde(rename = "duplicatePoints")]
    DuplicatePoints,
    /// # Orientable
    /// Detection of surfaces whose faces cannot be given a consistent orientation.
    #[serde(rename = "orientable")]
    Orientable,
    /// # Orientation
    /// Detection of rings wound in the wrong direction.
    #[serde(rename = "orientation")]
    Orientation,
    /// # Shell Orientation
    /// Detection of solid shells whose faces point the wrong way.
    #[serde(rename = "shellOrientation")]
    ShellOrientation,
}

/// How the planarity check bounds a face's out-of-plane deviation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PlanarityThreshold {
    /// # Ratio
    /// Dimensionless ratio of the face's convex-hull minimum height to its
    /// diameter; scale-invariant.
    Ratio(f64),
    /// # Max Height
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
    /// # Minimum Length
    /// Shortest length a 1D geometry (line or ring edge) may have before it is flagged.
    #[serde(default)]
    min_length: f64,
    /// # Minimum Area
    /// Smallest area a 2D geometry (face or ring) may have before it is flagged.
    #[serde(default)]
    min_area: f64,
    /// # Minimum Volume
    /// Smallest volume a 3D geometry (solid) may have before it is flagged.
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
/// Configure which validation checks to perform on feature geometries.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryValidator {
    /// # Validation Types
    /// Checks to run against each feature's geometry. Empty by default, which
    /// passes every feature that has geometry.
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
    ///
    /// A failed geometry additionally goes to `issue-locations` once per flagged
    /// position: the same attributes plus `validationCheck` naming the check,
    /// with the geometry replaced by the position the check flagged.
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
        let mut issue_locations: Vec<(String, Geometry)> = Vec::new();
        for (check, result) in validate_with(feature.geometry.as_ref(), &params) {
            if let ValidationResult::Failed(positions) = result {
                error_count += positions.len();
                let check = check.to_string();
                checks.insert(check.clone(), serde_json::json!(positions.len()));
                issue_locations.extend(positions.into_iter().map(|p| (check.clone(), p)));
            }
        }

        if checks.is_empty() {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), SUCCESS_PORT.clone()));
        } else {
            let mut failed = feature.clone();
            failed.insert(
                "validationResult",
                serde_json::json!({ "errorCount": error_count, "checks": checks }).into(),
            );
            fw.send(ctx.new_with_feature_and_port(failed, FAILED_PORT.clone()));

            // `validate_with` returns an unordered map, so emission order would
            // otherwise vary between runs. Stable sort keeps each check's own
            // positions in the order the check found them.
            issue_locations.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (check, position) in issue_locations {
                let mut located = feature.clone();
                located.insert("validationCheck", serde_json::json!(check).into());
                located.set_geometry(position);
                fw.send(ctx.new_with_feature_and_port(located, ISSUE_LOCATIONS_PORT.clone()));
            }
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

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    /// A closed 4x4 square, flat and correctly wound.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// The same square with one corner lifted well out of the other three's plane.
    const NON_PLANAR: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 2.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// An attribute of the incoming feature, unrelated to validation.
    const SURFACE_ID_ATTRIBUTE: &str = "surfaceId";
    const SURFACE_ID: u64 = 7;

    fn polygon(ring: [[f64; 3]; 5]) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    fn feature(ring: [[f64; 3]; 5]) -> Feature {
        let mut feature = Feature::from(polygon(ring));
        feature.insert(
            SURFACE_ID_ATTRIBUTE,
            AttributeValue::Number(SURFACE_ID.into()),
        );
        feature
    }

    /// Run the validator over `feature`, returning what it sent, port by port.
    fn validate(feature: &Feature) -> Vec<(Port, Feature)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        serde_json::from_value::<GeometryValidator>(serde_json::json!({}))
            .expect("every parameter has a default")
            .process(create_default_execute_context(feature), &fw)
            .unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    fn on_port<'a>(sent: &'a [(Port, Feature)], port: &Port) -> Vec<&'a Feature> {
        sent.iter()
            .filter(|(p, _)| p == port)
            .map(|(_, f)| f)
            .collect()
    }

    #[test]
    fn valid_geometry_emits_no_issue_location() {
        let sent = validate(&feature(SQUARE));

        assert_eq!(
            sent.iter().map(|(p, _)| p.to_string()).collect::<Vec<_>>(),
            vec![SUCCESS_PORT.to_string()]
        );
    }

    #[test]
    fn each_flagged_position_becomes_one_issue_location() {
        let sent = validate(&feature(NON_PLANAR));

        let failed = on_port(&sent, &FAILED_PORT);
        assert_eq!(failed.len(), 1, "one feature on `failed`");
        let Some(AttributeValue::Map(result)) = failed[0].get(Attribute::new("validationResult"))
        else {
            panic!("`failed` should carry a validationResult map");
        };
        let Some(AttributeValue::Number(error_count)) = result.get("errorCount") else {
            panic!("validationResult should carry errorCount");
        };
        let error_count = error_count.as_u64().expect("errorCount is a count");

        let located = on_port(&sent, &ISSUE_LOCATIONS_PORT);
        assert_eq!(
            located.len() as u64,
            error_count,
            "one issue location per flagged position"
        );
        for feature in &located {
            assert!(
                matches!(
                    feature.get(Attribute::new("validationCheck")),
                    Some(AttributeValue::String(_))
                ),
                "each issue location names the check that flagged it"
            );
            assert_eq!(
                feature.get(Attribute::new(SURFACE_ID_ATTRIBUTE)),
                Some(&AttributeValue::Number(SURFACE_ID.into())),
                "attributes of the incoming feature are kept"
            );
            assert!(
                feature.get(Attribute::new("validationResult")).is_none(),
                "the per-geometry summary belongs to `failed`, not to a position"
            );
            assert_ne!(
                feature.geometry.as_ref(),
                &polygon(NON_PLANAR),
                "the geometry is replaced by the flagged position"
            );
        }
    }
}

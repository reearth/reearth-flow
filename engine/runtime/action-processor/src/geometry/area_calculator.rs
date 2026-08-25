use std::collections::HashMap;

#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::{area2d::Area2D, area3d::Area3D};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::coordinate::UnitKind;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{area_report, Area, AreaFrame};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::Geometry;
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
        "Calculates the true surface area of a feature's geometry and stores it in an attribute. A solid's area is the sum of its boundary surfaces, so a void's faces count toward it just like the exterior's."
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

/// # Area Calculator Parameters
///
/// Configure how the area of each feature's geometry is measured and stored.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AreaCalculator {
    /// # Output Attribute
    /// Attribute to store the calculated true surface area in. Defaults to
    /// `area`. A solid's area sums the areas of all of its boundary
    /// surfaces, so a void's faces count toward the total just like the
    /// exterior's do — a hollow body measures *more* surface than a solid one.
    #[serde(default = "default_output_attribute")]
    output_attribute: Attribute,
}

impl Default for AreaCalculator {
    fn default() -> Self {
        Self {
            output_attribute: default_output_attribute(),
        }
    }
}

fn default_output_attribute() -> Attribute {
    Attribute::new("area".to_string())
}

#[cfg(feature = "new-geometry")]
impl AreaCalculator {
    /// Say what the measurement is worth.
    ///
    /// None of this changes the number: an area in square degrees is still
    /// written, because refusing would need either a hard failure or a new
    /// rejected port, and a port change is a schema change affecting both
    /// worlds and the UI. The old model said nothing at all here; we have the
    /// information, so we say it.
    ///
    /// Each cause is reported per feature via `ctx.warn`, not deduplicated
    /// here: the runtime aggregates by error code, so a million-feature run
    /// still surfaces one entry per cause with a structural count, rather
    /// than one process-wide line that survives only as long as the worker
    /// does.
    ///
    /// None of these `ctx.warn` calls carry a `with_message`: in a real run
    /// `ExecutorContext::warn` records only `(code, kind, feature_id)` in the
    /// per-node diagnostics aggregator (`runtime/diagnostics/src/aggregator.rs`)
    /// and never stores the message, so a per-occurrence `DiagnosticDraft`
    /// message is silently thrown away — do not add one back. The user-facing
    /// text is the `message`/`help` declared per code in
    /// `schema/error-codes/geometry.toml`, which is what the run summary
    /// renders.
    fn warn_about_frames(&self, ctx: &ExecutorContext, geometry: &Geometry) {
        let report = area_report(geometry);

        if report.skipped > 0 {
            ctx.warn(DiagnosticDraft::new(ErrorCode::GeometryAreaSkippedParts));
        }

        match &report.frame {
            AreaFrame::Nothing => {}
            AreaFrame::Mixed => {
                ctx.warn(DiagnosticDraft::new(ErrorCode::GeometryAreaMixedFrames));
            }
            // Matched exhaustively via `unit_kind()` rather than picking the
            // EPSG code back out of `frame` with a wildcard fallback: a future
            // `CoordinateFrame` variant is classified automatically instead of
            // silently taking an early return and never being warned about.
            AreaFrame::One(frame) => match frame.unit_kind() {
                UnitKind::Linear => {}
                UnitKind::Angular => {
                    ctx.warn(DiagnosticDraft::new(ErrorCode::GeometryAreaAngularCrs));
                }
                UnitKind::Undeterminable(_) => {
                    ctx.warn(DiagnosticDraft::new(ErrorCode::GeometryAreaUnknownUnit));
                }
            },
        }
    }
}

impl Processor for AreaCalculator {
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let geometry = &*ctx.feature.geometry;
        let measured = geometry.surface_area();
        // The attribute is always written, so an unmeasurable geometry records
        // zero — but says so, rather than passing silently.
        let area = match measured {
            Ok(area) => {
                // Only worth saying what the unit is once there is a number to
                // qualify. On the `Err` path the warning below has already said
                // the whole story.
                self.warn_about_frames(&ctx, geometry);
                area
            }
            Err(_) => {
                // No `with_message` here either — see the comment on
                // `warn_about_frames`: a real run's aggregator never stores
                // it. The registry message/help for this code already says
                // a zero was written instead.
                ctx.warn(DiagnosticDraft::new(ErrorCode::GeometryAreaNotMeasurable));
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
        //
        // Always the true surface area, following the slope of each face —
        // matching `slopedArea`'s old behaviour now that `areaType` and
        // `multiplier` are gone (see the new-geometry world's `Area::surface_area`
        // and its module docs for why a closed body's faces are summed, not
        // unioned, and why a solid's voids add to its surface rather than
        // subtracting).
        let area = match &geometry.value {
            GeometryValue::None => 0.0,
            GeometryValue::FlowGeometry2D(geom_2d) => geom_2d.unsigned_area2d(),
            GeometryValue::FlowGeometry3D(geom_3d) => geom_3d.unsigned_area3d(),
            GeometryValue::CityGmlGeometry(city_gml_geom) => {
                // For CityGML geometry, we calculate area for each polygon
                let mut total_area = 0.0;
                for gml_feature in &city_gml_geom.gml_geometries {
                    for polygon in &gml_feature.polygons {
                        total_area += polygon.unsigned_area3d();
                    }
                }
                total_area
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

    // One body covers both worlds: the trait signature
    // (`runtime/runtime/src/node.rs`'s `Processor::finish`) is not itself
    // `#[cfg]`-gated, only its default implementation's error message differs
    // per world, so there is nothing here for a `#[cfg]` pair to gate.
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
    use reearth_flow_geometry::solid::{Shell, Solid};
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3DData;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;
    use serde_json::json;

    /// The unit square tilted 45 degrees about the x axis: 1.0 of true
    /// surface, 1/sqrt(2) of it projected onto the XY plane. A flat face
    /// would not tell surface area apart from projected area, so tests that
    /// need to pin the former use this instead.
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

    /// The action always measures the true surface area, following the slope
    /// of each face, not the flatter XY-projected area a tilted face would
    /// give if it dropped elevation.
    #[test]
    fn the_action_computes_true_surface_area() {
        let feature = Feature::from(tilted_square());
        let out = run(&mut *build(None), &feature);
        assert!(
            (area_of(&out, "area") - 1.0).abs() < 1e-12,
            "surface area was {}",
            area_of(&out, "area")
        );
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
        assert!((area_of(&named, "roofArea") - 1.0).abs() < 1e-12);
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

    /// A solid's area is the sum of its boundary surfaces: the exterior's
    /// faces and every void's, since a void's faces are real surfaces too and
    /// count toward the total rather than subtracting from it. This is the
    /// behaviour the parameter doc comment promises, and nothing else at the
    /// action level pins it.
    #[test]
    fn a_solids_area_sums_its_boundary_surfaces_including_voids() {
        // A right triangle with legs of length 1 has area 0.5. Used as both
        // the exterior shell and a void, so the solid's total surface is the
        // two added together rather than the void cancelling the exterior out.
        let triangle = || {
            TriangularMesh3DData::from_parts(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                [0u32, 1, 2],
            )
            .unwrap()
        };
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            triangle(),
            vec![Shell::from(triangle())],
        );
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid)));

        let out = run(&mut *build(None), &Feature::from(g));
        assert!((area_of(&out, "area") - 1.0).abs() < 1e-12);
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

    use std::sync::Arc;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_geometry::coordinate::EpsgCode;
    use reearth_flow_runtime::event::Event;
    use reearth_flow_runtime::kvs;
    use reearth_flow_storage::resolve::StorageResolver;

    /// A unit square in `frame`.
    fn square_in(frame: CoordinateFrame) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                frame,
                vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    /// Run every `geometry` through one processor and return the error code
    /// of every diagnostic emitted along the way.
    ///
    /// Diagnostics travel through `event_hub.diagnostic(..)`
    /// (`Event::Diagnostic`), not the log lane — `ctx.warn`/`ctx.warn_once`
    /// never call `warn_log`, so a `Level::WARN` filter over `Event::Log`
    /// would see nothing.
    ///
    /// Collected by `code`, not `message`: `warn_about_frames` sends no
    /// per-occurrence message (a real run's diagnostics aggregator would
    /// discard it anyway — see the comment there), so every diagnostic for a
    /// given cause renders the same registry-supplied text regardless of
    /// which EPSG code or how many parts triggered it. The code is the only
    /// thing left to tell one cause from another, which is exactly what a
    /// production run's summary is keyed on too.
    ///
    /// `create_default_execute_context` builds its own event hub, so this
    /// builds the contexts by hand in order to hold on to a receiver. A
    /// broadcast receiver only sees what is sent after it subscribes, hence
    /// the `resubscribe` before the first `process`.
    fn warnings_for(with: Option<Value>, geometries: Vec<Geometry>) -> Vec<ErrorCode> {
        let hub = EventHub::new(64);
        let mut rx = hub.receiver.resubscribe();
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = build(with);

        for geometry in geometries {
            let ctx = ExecutorContext::new(
                Feature::from(geometry),
                FEATURES_PORT.clone(),
                Arc::new(serde_json::Map::new()),
                Arc::new(StorageResolver::new()),
                Arc::new(kvs::create_kv_store()),
                hub.clone(),
                Uri::for_test("file:///"),
            );
            processor.process(ctx, &fw).unwrap();
        }

        let mut warnings = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let Event::Diagnostic(diagnostic) = event {
                warnings.push(diagnostic.code);
            }
        }
        warnings
    }

    /// An area on geographic coordinates is in square degrees and is not a
    /// real-world area. Say so, once per feature: `ctx.warn` records one
    /// diagnostic per call, and it is the runtime's per-run diagnostics
    /// aggregator (keyed by `ErrorCode`, not exercised by this
    /// `diagnostics: None` test harness) that collapses repeats of the same
    /// cause into a single structural count for a real, long-running worker.
    ///
    /// Uses EPSG:4269 (NAD83, degrees) for no reason beyond being a
    /// real-world geographic CRS distinct from the other angular-CRS test
    /// below.
    #[test]
    fn an_angular_crs_is_warned_about_once_per_feature() {
        let angular = CoordinateFrame::Crs(EpsgCode::from(4269));
        let warnings = warnings_for(
            None,
            vec![
                square_in(angular.clone()),
                square_in(angular.clone()),
                square_in(angular),
            ],
        );
        let about_units: Vec<_> = warnings
            .iter()
            .filter(|&&code| code == ErrorCode::GeometryAreaAngularCrs)
            .collect();
        assert_eq!(
            about_units.len(),
            3,
            "three features, one warning each; got {warnings:?}"
        );
    }

    /// Warning is not refusing: the number is still measured and written.
    ///
    /// Uses EPSG:4326 (WGS 84, degrees) as a second, distinct angular CRS.
    #[test]
    fn an_angular_crs_still_produces_a_number() {
        let g = square_in(CoordinateFrame::Crs(EpsgCode::from(4326)));
        let out = run(&mut *build(None), &Feature::from(g));
        assert_eq!(area_of(&out, "area"), 1.0);
    }

    /// A projected CRS is measuring in metres, so there is nothing to say.
    ///
    /// Uses EPSG:6677 (JGD2011 / Japan Plane Rectangular CS IX, metres).
    #[test]
    fn a_projected_crs_is_not_warned_about() {
        let warnings = warnings_for(
            None,
            vec![square_in(CoordinateFrame::Crs(EpsgCode::from(6677)))],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// Euclidean coordinates are plain lengths, so there is nothing to say
    /// about them either — and this is the common case, so it must stay quiet.
    #[test]
    fn euclidean_coordinates_are_not_warned_about() {
        let warnings = warnings_for(None, vec![square_in(CoordinateFrame::Euclidean)]);
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    /// A sum across frames adds square metres to square degrees. The area is
    /// still returned — refusing would need a new rejected port — but the user
    /// is told.
    ///
    /// This test sends a single feature, so one `ctx.warn` call is the whole
    /// story regardless of aggregation; it uses EPSG:3857 as an arbitrary
    /// projected CRS distinct from `Euclidean`.
    #[test]
    fn mixed_frames_are_warned_about() {
        let member = |frame| match square_in(frame) {
            Geometry::Euclidean3D(g) => g,
            _ => unreachable!("square_in builds a 3D polygon"),
        };
        let mixed = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            reearth_flow_geometry::collection::Collection3D::new(vec![
                member(CoordinateFrame::Euclidean),
                member(CoordinateFrame::Crs(EpsgCode::from(3857))),
            ]),
        ));

        let warnings = warnings_for(None, vec![mixed]);
        let about_mixing: Vec<_> = warnings
            .iter()
            .filter(|&&code| code == ErrorCode::GeometryAreaMixedFrames)
            .collect();
        assert_eq!(about_mixing.len(), 1, "{warnings:?}");
    }

    /// A container that skips an unmeasurable member is warned about.
    ///
    /// Both members sit in `CoordinateFrame::Euclidean`, not a CRS: that keeps
    /// the geometry's frame at `One(Euclidean)`, which is silent on its own,
    /// so the skip warning is the only one this assertion has to contend
    /// with. The measurement itself succeeds — `Collection3D`'s `Area` impl
    /// sums measurable members via `filter_map(.ok())` — so this reaches the
    /// `Ok` path where `warn_about_frames` runs, unlike a bare `Csg`, which
    /// takes the `Err` path instead (see
    /// `an_unmeasurable_geometry_still_gets_the_attribute`). One feature in,
    /// one `ctx.warn` call, one diagnostic out.
    #[test]
    fn a_skipped_member_is_warned_about() {
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
        let member = |frame| match square_in(frame) {
            Geometry::Euclidean3D(g) => g,
            _ => unreachable!("square_in builds a 3D polygon"),
        };
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(
            reearth_flow_geometry::collection::Collection3D::new(vec![
                member(CoordinateFrame::Euclidean),
                Euclidean3DGeometry::Csg(csg),
            ]),
        ));

        let warnings = warnings_for(None, vec![g]);
        let about_skip: Vec<_> = warnings
            .iter()
            .filter(|&&code| code == ErrorCode::GeometryAreaSkippedParts)
            .collect();
        assert_eq!(about_skip.len(), 1, "{warnings:?}");
    }
}

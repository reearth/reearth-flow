use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nusamai_projection::crs::{EPSG_JGD2011_GEOGRAPHIC_3D, EPSG_JGD2024_GEOGRAPHIC_3D};
use nusamai_projection::height_revision::Jgd2011ToJgd2024;
use nusamai_projection::vshift::{Jgd2011ToWgs84, Jgd2024ToWgs84, VerticalTransform};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct VerticalReprojectorFactory;

impl ProcessorFactory for VerticalReprojectorFactory {
    fn name(&self) -> &str {
        "VerticalReprojector"
    }

    fn description(&self) -> &str {
        "Reproject Vertical Coordinates Between Datums"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(VerticalReprojectorParam))
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
        let Some(with) = with else {
            return Err(GeometryProcessorError::VerticalReprojectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let params: VerticalReprojectorParam = {
            let value: Value = serde_json::to_value(&with).map_err(|e| {
                GeometryProcessorError::VerticalReprojectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::VerticalReprojectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        };
        let outside_coverage = params.outside_coverage.unwrap_or_default();
        let mode = match params.reprojector_type {
            VerticalReprojectorType::Jgd2011ToWgs84 => Mode::Jgd2011ToWgs84 {
                geoid2011: Arc::new(Jgd2011ToWgs84::new()),
                outside_coverage,
            },
            VerticalReprojectorType::Jgd2024ToWgs84 => Mode::Jgd2024ToWgs84 {
                revision: Arc::new(Jgd2011ToJgd2024::new()),
                geoid2024: Arc::new(Jgd2024ToWgs84::new()),
                outside_coverage,
            },
        };

        Ok(Box::new(VerticalReprojector {
            mode,
            fallback: 0,
            skipped: 0,
            unconverted: 0,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum VerticalReprojectorType {
    /// JGD2011 heights (EPSG:6697) to WGS84 ellipsoidal heights (EPSG:4979) using the GSIGEO2011 geoid. The EPSG code of the input is not checked.
    Jgd2011ToWgs84,
    /// Heights on either survey result to WGS84 ellipsoidal heights (EPSG:4979) on 測地成果2024. The input datum is taken from the geometry's EPSG code: 6697 (JGD2011 heights) gets the GSI height revision and then the JPGEO2024 geoid with the Hrefconv2024 correction, 11318 (JGD2024 heights) gets the geoid only. Features whose geometry has any other code, or none, fail, as do features with a vertex where the geoid has no value.
    Jgd2024ToWgs84,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum OutsideCoveragePolicy {
    /// For `jgd2011ToWgs84`, emit the feature with its heights unchanged. For `jgd2024ToWgs84`, skip the height revision and convert the unrevised heights with the JPGEO2024 geoid and the Hrefconv2024 correction.
    #[default]
    PassThrough,
    /// Fail the workflow.
    Error,
}

/// # Vertical Reprojector Parameters
/// Configure the type of vertical datum conversion to apply
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerticalReprojectorParam {
    /// # Reprojector Type
    /// The type of vertical coordinate transformation to apply
    reprojector_type: VerticalReprojectorType,
    /// # Outside Coverage
    /// What to do with a feature that has a vertex outside the coverage of the GSIGEO2011 geoid (`jgd2011ToWgs84`) or, for JGD2011 features, of the height revision parameters (`jgd2024ToWgs84`). Defaults to `passThrough`.
    outside_coverage: Option<OutsideCoveragePolicy>,
}

#[derive(Debug, Clone)]
enum Mode {
    Jgd2011ToWgs84 {
        geoid2011: Arc<Jgd2011ToWgs84>,
        outside_coverage: OutsideCoveragePolicy,
    },
    Jgd2024ToWgs84 {
        revision: Arc<Jgd2011ToJgd2024>,
        geoid2024: Arc<Jgd2024ToWgs84>,
        outside_coverage: OutsideCoveragePolicy,
    },
}

#[derive(Debug, Clone)]
pub struct VerticalReprojector {
    mode: Mode,
    /// JGD2011 features revised with the fallback parameter set.
    fallback: u64,
    /// JGD2011 features outside the parameter coverage, converted unrevised.
    skipped: u64,
    /// Features outside the GSIGEO2011 coverage, emitted with their heights unchanged.
    unconverted: u64,
}

/// Marker for a conversion that produced a non-finite height.
struct NoGeoidValue;

/// A vertical transform that records whether any converted height is not finite.
struct FiniteGuard<'a> {
    inner: &'a dyn VerticalTransform,
    non_finite: AtomicBool,
}

impl VerticalTransform for FiniteGuard<'_> {
    fn convert(&self, lng: f64, lat: f64, height: f64) -> (f64, f64, f64) {
        let converted = self.inner.convert(lng, lat, height);
        if !converted.2.is_finite() {
            self.non_finite.store(true, Ordering::Relaxed);
        }
        converted
    }
}

/// Applies a geoid model like [`transform_geometry`], but fails when the model has no value at any vertex.
fn apply_geoid(
    geometry: &Geometry,
    geoid: &dyn VerticalTransform,
) -> Result<Option<Geometry>, NoGeoidValue> {
    let guard = FiniteGuard {
        inner: geoid,
        non_finite: AtomicBool::new(false),
    };
    let converted = transform_geometry(geometry, &guard);
    if guard.non_finite.into_inner() {
        Err(NoGeoidValue)
    } else {
        Ok(converted)
    }
}

/// Applies a vertical transform to the geometries that carry heights. Returns
/// `None` for geometries without heights, which are left as they are.
fn transform_geometry(geometry: &Geometry, transform: &dyn VerticalTransform) -> Option<Geometry> {
    let epsg = geometry.epsg;
    match &geometry.value {
        GeometryValue::CityGmlGeometry(geos) => {
            let mut geos = geos.clone();
            geos.transform_inplace(transform);
            Some(Geometry {
                epsg,
                value: GeometryValue::CityGmlGeometry(geos),
            })
        }
        GeometryValue::FlowGeometry3D(geos) => {
            let mut geos = geos.clone();
            geos.transform_inplace(transform);
            Some(Geometry {
                epsg,
                value: GeometryValue::FlowGeometry3D(geos),
            })
        }
        GeometryValue::None | GeometryValue::FlowGeometry2D(..) => None,
    }
}

fn has_heights(geometry: &Geometry) -> bool {
    matches!(
        geometry.value,
        GeometryValue::CityGmlGeometry(..) | GeometryValue::FlowGeometry3D(..)
    )
}

impl Processor for VerticalReprojector {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let converted = match &self.mode {
            Mode::Jgd2011ToWgs84 {
                geoid2011,
                outside_coverage,
            } => match apply_geoid(&feature.geometry, geoid2011.as_ref()) {
                Ok(converted) => converted,
                Err(NoGeoidValue) => match outside_coverage {
                    OutsideCoveragePolicy::Error => {
                        return Err(GeometryProcessorError::VerticalReprojector(format!(
                            "Feature {} has a vertex outside the coverage of the GSIGEO2011 geoid",
                            feature.id
                        ))
                        .into());
                    }
                    OutsideCoveragePolicy::PassThrough => {
                        self.unconverted += 1;
                        None
                    }
                },
            },
            Mode::Jgd2024ToWgs84 {
                revision,
                geoid2024,
                outside_coverage,
            } => {
                let converted = if !has_heights(&feature.geometry) {
                    Ok(None)
                } else {
                    match feature.geometry.epsg {
                        Some(EPSG_JGD2024_GEOGRAPHIC_3D) => {
                            apply_geoid(&feature.geometry, geoid2024.as_ref())
                        }
                        Some(EPSG_JGD2011_GEOGRAPHIC_3D) => {
                            let revised = transform_geometry(&feature.geometry, revision.as_ref());
                            if revision.take_used_fallback() {
                                self.fallback += 1;
                            }
                            if revision.take_missed() {
                                match outside_coverage {
                                    OutsideCoveragePolicy::Error => {
                                        return Err(GeometryProcessorError::VerticalReprojector(format!(
                                            "Feature {} has a vertex outside the coverage of the height revision parameters",
                                            feature.id
                                        ))
                                        .into());
                                    }
                                    OutsideCoveragePolicy::PassThrough => {
                                        let converted =
                                            apply_geoid(&feature.geometry, geoid2024.as_ref());
                                        if converted.is_ok() {
                                            self.skipped += 1;
                                        }
                                        converted
                                    }
                                }
                            } else {
                                match revised {
                                    Some(g) => apply_geoid(&g, geoid2024.as_ref()),
                                    None => Ok(None),
                                }
                            }
                        }
                        Some(other) => {
                            return Err(GeometryProcessorError::VerticalReprojector(format!(
                                "Feature {} has EPSG:{other}, but `jgd2024ToWgs84` accepts only EPSG:{EPSG_JGD2011_GEOGRAPHIC_3D} (JGD2011 heights) or EPSG:{EPSG_JGD2024_GEOGRAPHIC_3D} (JGD2024 heights)",
                                feature.id
                            ))
                            .into());
                        }
                        None => {
                            return Err(GeometryProcessorError::VerticalReprojector(format!(
                                "Feature {} has no EPSG code (missing or unrecognised srsName), so its vertical datum is unknown",
                                feature.id
                            ))
                            .into());
                        }
                    }
                };
                converted.map_err(|NoGeoidValue| {
                    GeometryProcessorError::VerticalReprojector(format!(
                        "Feature {} has a vertex where the JPGEO2024 geoid has no value",
                        feature.id
                    ))
                })?
            }
        };
        if let Some(geometry) = converted {
            feature.geometry = Arc::new(geometry);
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if self.fallback > 0 {
            tracing::info!(
                "VerticalReprojector revised {} feature(s) with the fallback height revision parameter set",
                self.fallback
            );
        }
        if self.skipped > 0 {
            tracing::warn!(
                "VerticalReprojector converted {} feature(s) without height revision because they fall outside the height revision parameter coverage",
                self.skipped
            );
        }
        if self.unconverted > 0 {
            tracing::warn!(
                "VerticalReprojector emitted {} feature(s) with their heights unchanged because they fall outside the GSIGEO2011 coverage",
                self.unconverted
            );
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "VerticalReprojector"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::{geometry::Geometry3D, point::Point3D};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    // 沖ノ島 (Munakata), outside GSIGEO2011 and the height revision parameters,
    // inside JPGEO2024 + Hrefconv2024 (31.25 m).
    const OKINOSHIMA_LON: f64 = 130.1053;
    const OKINOSHIMA_LAT: f64 = 34.2442;

    fn reprojector(mode: Mode) -> VerticalReprojector {
        VerticalReprojector {
            mode,
            fallback: 0,
            skipped: 0,
            unconverted: 0,
        }
    }

    fn jgd2011_mode(outside_coverage: OutsideCoveragePolicy) -> Mode {
        Mode::Jgd2011ToWgs84 {
            geoid2011: Arc::new(Jgd2011ToWgs84::new()),
            outside_coverage,
        }
    }

    fn jgd2024_mode(outside_coverage: OutsideCoveragePolicy) -> Mode {
        Mode::Jgd2024ToWgs84 {
            revision: Arc::new(Jgd2011ToJgd2024::new()),
            geoid2024: Arc::new(Jgd2024ToWgs84::new()),
            outside_coverage,
        }
    }

    /// Runs one feature through the processor and returns the emitted height.
    fn run(reprojector: &mut VerticalReprojector, geometry: Geometry) -> Result<f64, BoxedError> {
        let feature = Feature::from(geometry);
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop.clone());
        reprojector.process(create_default_execute_context(&feature), &fw)?;
        let features = noop.send_features.lock().unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(noop.send_ports.lock().unwrap()[0], DEFAULT_PORT.clone());
        Ok(height(&features[0].geometry))
    }

    // Sendai station. The revision there is +0.1233 m and the geoid heights
    // are 41.8025 m (GSIGEO2011) and 41.9599 m (JPGEO2024 + Hrefconv2024).
    const LON: f64 = 140.8825;
    const LAT: f64 = 38.2592;

    fn point(epsg: Option<u16>, lon: f64, lat: f64, z: f64) -> Geometry {
        Geometry {
            epsg,
            value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::new(lon, lat, z))),
        }
    }

    fn height(geometry: &Geometry) -> f64 {
        match &geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => p.z(),
            other => panic!("unexpected geometry {other:?}"),
        }
    }

    #[test]
    fn jgd2011_input_is_revised_then_lifted_by_the_2024_geoid() {
        let revision = Jgd2011ToJgd2024::new();
        let g = point(Some(EPSG_JGD2011_GEOGRAPHIC_3D), LON, LAT, 10.0);
        let revised = transform_geometry(&g, &revision).unwrap();
        assert!(!revision.take_missed());
        // The benchmark correction here is -0.0205 m.
        assert!(
            (height(&revised) - 9.9795).abs() < 5e-4,
            "{}",
            height(&revised)
        );

        let new_path = transform_geometry(&revised, &Jgd2024ToWgs84::new()).unwrap();
        let old_path = transform_geometry(&g, &Jgd2011ToWgs84::new()).unwrap();
        assert!((height(&new_path) - (9.9795 + 41.9599)).abs() < 5e-3);
        let shift = height(&new_path) - height(&old_path);
        assert!((shift - 0.137).abs() < 2e-3, "shift {shift}");
    }

    #[test]
    fn jgd2024_input_gets_the_geoid_only() {
        let g = point(Some(EPSG_JGD2024_GEOGRAPHIC_3D), LON, LAT, 10.1233);
        let out = transform_geometry(&g, &Jgd2024ToWgs84::new()).unwrap();
        assert!((height(&out) - (10.1233 + 41.9599)).abs() < 5e-3);
        assert_eq!(out.epsg, Some(EPSG_JGD2024_GEOGRAPHIC_3D));
    }

    #[test]
    fn outside_coverage_is_flagged() {
        let revision = Jgd2011ToJgd2024::new();
        let sea = point(Some(EPSG_JGD2011_GEOGRAPHIC_3D), 135.0, 30.0, 10.0);
        let out = transform_geometry(&sea, &revision).unwrap();
        assert_eq!(height(&out), 10.0);
        assert!(revision.take_missed());
    }

    #[test]
    fn jgd2011_outside_the_geoid_passes_through_unchanged() {
        let mut r = reprojector(jgd2011_mode(OutsideCoveragePolicy::PassThrough));
        let g = point(None, OKINOSHIMA_LON, OKINOSHIMA_LAT, 4.0);
        assert_eq!(run(&mut r, g).unwrap(), 4.0);
        assert_eq!(r.unconverted, 1);
    }

    #[test]
    fn jgd2011_outside_the_geoid_can_fail() {
        let mut r = reprojector(jgd2011_mode(OutsideCoveragePolicy::Error));
        let g = point(None, OKINOSHIMA_LON, OKINOSHIMA_LAT, 4.0);
        assert!(run(&mut r, g).is_err());
    }

    #[test]
    fn jgd2024_outside_the_revision_gets_the_2024_geoid_unrevised() {
        let mut r = reprojector(jgd2024_mode(OutsideCoveragePolicy::PassThrough));
        let g = point(
            Some(EPSG_JGD2011_GEOGRAPHIC_3D),
            OKINOSHIMA_LON,
            OKINOSHIMA_LAT,
            4.0,
        );
        let h = run(&mut r, g).unwrap();
        assert!((h - (4.0 + 31.25)).abs() < 0.01, "{h}");
        assert_eq!(r.skipped, 1);
    }

    #[test]
    fn geometries_without_heights_are_left_alone() {
        let g = Geometry::with_value(GeometryValue::None);
        assert!(!has_heights(&g));
        assert!(transform_geometry(&g, &Jgd2011ToWgs84::new()).is_none());
        assert!(has_heights(&point(None, LON, LAT, 0.0)));
    }
}

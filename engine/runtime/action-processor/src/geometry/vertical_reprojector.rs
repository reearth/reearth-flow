use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use nusamai_projection::crs::{EPSG_JGD2011_GEOGRAPHIC_3D, EPSG_JGD2024_GEOGRAPHIC_3D};
use nusamai_projection::height_revision::{HeightRevisionGrid, Jgd2011ToJgd2024, ParameterSet};
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

/// The nationwide height revision parameters bundled with nusamai-projection,
/// shared by every node. GSI's own base map update applied the benchmark set
/// (水準点) and fell back to the triangulation set (三角点) where the
/// benchmark set has no coverage; this node does the same per feature.
static BENCHMARK_GRID: LazyLock<Arc<HeightRevisionGrid>> =
    LazyLock::new(|| load_embedded(ParameterSet::Benchmark));
static TRIANGULATION_GRID: LazyLock<Arc<HeightRevisionGrid>> =
    LazyLock::new(|| load_embedded(ParameterSet::Triangulation));

fn load_embedded(set: ParameterSet) -> Arc<HeightRevisionGrid> {
    let grid = HeightRevisionGrid::load_embedded(set);
    tracing::info!(
        "Loaded embedded height revision parameters {set:?} ({}) with {} grid nodes",
        grid.version(),
        grid.node_count()
    );
    Arc::new(grid)
}

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
        let mode = match params.reprojector_type {
            VerticalReprojectorType::Jgd2011ToWgs84 => Mode::Jgd2011ToWgs84 {
                geoid2011: Arc::new(Jgd2011ToWgs84::new()),
            },
            VerticalReprojectorType::Jgd2024ToWgs84 => Mode::Jgd2024ToWgs84 {
                revision_bm: Arc::new(Jgd2011ToJgd2024::new(Arc::clone(&BENCHMARK_GRID))),
                revision_tr: Arc::new(Jgd2011ToJgd2024::new(Arc::clone(&TRIANGULATION_GRID))),
                geoid2024: Arc::new(Jgd2024ToWgs84::new()),
                geoid2011: Arc::new(Jgd2011ToWgs84::new()),
                outside_coverage: params.outside_coverage.unwrap_or_default(),
            },
        };

        Ok(Box::new(VerticalReprojector {
            mode,
            fallback: 0,
            skipped: 0,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum VerticalReprojectorType {
    /// JGD2011 heights (EPSG:6697) to WGS84 ellipsoidal heights (EPSG:4979) using the GSIGEO2011 geoid. The EPSG code of the input is not checked.
    Jgd2011ToWgs84,
    /// Heights on either survey result to WGS84 ellipsoidal heights (EPSG:4979) on 測地成果2024. The input datum is taken from the geometry's EPSG code: 6697 (JGD2011 heights) gets the GSI height revision and then the JPGEO2024 geoid with the Hrefconv2024 correction, 11318 (JGD2024 heights) gets the geoid only. Features whose geometry has any other code, or none, fail. The revision uses the benchmark parameters (hyokorevBM) and, for a feature with a vertex outside their coverage, the triangulation parameters (hyokorevTR), as GSI did for its own base map.
    Jgd2024ToWgs84,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum OutsideCoveragePolicy {
    /// Convert the feature with the GSIGEO2011 geoid instead, which leaves it at the ellipsoidal height it had before 測地成果2024.
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
    /// What to do with a JGD2011 feature that has a vertex outside the coverage of both height revision parameter sets (`jgd2024ToWgs84` only). Defaults to `passThrough`.
    outside_coverage: Option<OutsideCoveragePolicy>,
}

#[derive(Debug, Clone)]
enum Mode {
    Jgd2011ToWgs84 {
        geoid2011: Arc<Jgd2011ToWgs84>,
    },
    Jgd2024ToWgs84 {
        revision_bm: Arc<Jgd2011ToJgd2024>,
        revision_tr: Arc<Jgd2011ToJgd2024>,
        geoid2024: Arc<Jgd2024ToWgs84>,
        geoid2011: Arc<Jgd2011ToWgs84>,
        outside_coverage: OutsideCoveragePolicy,
    },
}

#[derive(Debug, Clone)]
pub struct VerticalReprojector {
    mode: Mode,
    /// JGD2011 features revised with the triangulation parameters.
    fallback: u64,
    /// JGD2011 features outside both parameter sets, converted unrevised.
    skipped: u64,
}

/// Revises a JGD2011 geometry with the benchmark parameters, or with the
/// triangulation parameters when a vertex is outside the benchmark coverage.
/// Returns the revised geometry and which set was used, or `None` when both
/// sets miss.
fn revise(
    geometry: &Geometry,
    bm: &Jgd2011ToJgd2024,
    tr: &Jgd2011ToJgd2024,
) -> Option<(Option<Geometry>, ParameterSet)> {
    let revised = transform_geometry(geometry, bm);
    if !bm.take_missed() {
        return Some((revised, ParameterSet::Benchmark));
    }
    let revised = transform_geometry(geometry, tr);
    if !tr.take_missed() {
        return Some((revised, ParameterSet::Triangulation));
    }
    None
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
        match &self.mode {
            Mode::Jgd2011ToWgs84 { geoid2011 } => {
                if let Some(geometry) = transform_geometry(&feature.geometry, geoid2011.as_ref()) {
                    feature.geometry = Arc::new(geometry);
                }
            }
            Mode::Jgd2024ToWgs84 {
                revision_bm,
                revision_tr,
                geoid2024,
                geoid2011,
                outside_coverage,
            } => {
                if has_heights(&feature.geometry) {
                    let geometry = match feature.geometry.epsg {
                        Some(EPSG_JGD2024_GEOGRAPHIC_3D) => {
                            transform_geometry(&feature.geometry, geoid2024.as_ref())
                        }
                        Some(EPSG_JGD2011_GEOGRAPHIC_3D) => {
                            match revise(&feature.geometry, revision_bm, revision_tr) {
                                Some((revised, set)) => {
                                    if set == ParameterSet::Triangulation {
                                        self.fallback += 1;
                                    }
                                    revised.and_then(|g| transform_geometry(&g, geoid2024.as_ref()))
                                }
                                None => match outside_coverage {
                                    OutsideCoveragePolicy::Error => {
                                        return Err(GeometryProcessorError::VerticalReprojector(format!(
                                            "Feature {} has a vertex outside the coverage of both height revision parameter sets",
                                            feature.id
                                        ))
                                        .into());
                                    }
                                    OutsideCoveragePolicy::PassThrough => {
                                        self.skipped += 1;
                                        transform_geometry(&feature.geometry, geoid2011.as_ref())
                                    }
                                },
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
                    };
                    if let Some(geometry) = geometry {
                        feature.geometry = Arc::new(geometry);
                    }
                }
            }
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
                "VerticalReprojector revised {} feature(s) with the triangulation parameters because they fall outside the benchmark parameter coverage",
                self.fallback
            );
        }
        if self.skipped > 0 {
            tracing::warn!(
                "VerticalReprojector converted {} feature(s) with the GSIGEO2011 geoid because they fall outside both height revision parameter sets",
                self.skipped
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

    use super::*;

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

    fn revisions() -> (Jgd2011ToJgd2024, Jgd2011ToJgd2024) {
        (
            Jgd2011ToJgd2024::new(Arc::clone(&BENCHMARK_GRID)),
            Jgd2011ToJgd2024::new(Arc::clone(&TRIANGULATION_GRID)),
        )
    }

    #[test]
    fn jgd2011_input_is_revised_with_benchmarks_then_lifted_by_the_2024_geoid() {
        let (bm, tr) = revisions();
        let g = point(Some(EPSG_JGD2011_GEOGRAPHIC_3D), LON, LAT, 10.0);
        let (revised, set) = revise(&g, &bm, &tr).unwrap();
        let revised = revised.unwrap();
        assert_eq!(set, ParameterSet::Benchmark);
        // The benchmark correction here is -0.0205 m; the triangulation
        // correction would have been +0.1236 m.
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
    fn outside_both_sets_is_reported() {
        let (bm, tr) = revisions();
        let sea = point(Some(EPSG_JGD2011_GEOGRAPHIC_3D), 135.0, 30.0, 10.0);
        assert!(revise(&sea, &bm, &tr).is_none());
        assert!(!bm.take_missed() && !tr.take_missed());
    }

    #[test]
    fn geometries_without_heights_are_left_alone() {
        let g = Geometry::with_value(GeometryValue::None);
        assert!(!has_heights(&g));
        assert!(transform_geometry(&g, &Jgd2011ToWgs84::new()).is_none());
        assert!(has_heights(&point(None, LON, LAT, 0.0)));
    }
}

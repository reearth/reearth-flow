use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use nusamai_projection::height_revision::{HeightRevisionGrid, Jgd2011ToJgd2024};
use nusamai_projection::vshift::{Jgd2011ToWgs84, Jgd2024ToWgs84, VerticalTransform};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

/// Attribute set on features whose heights were left unrevised because they
/// fall outside the coverage of the height revision parameters. The
/// `jgd2024ToWgs84` mode reads it and falls back to the GSIGEO2011 geoid for
/// those features, so they end up at the same ellipsoidal height as before.
pub const HEIGHT_REVISION_SKIPPED_ATTRIBUTE: &str = "_heightRevisionSkipped";

/// The nationwide parameters bundled with nusamai-projection, shared by every
/// node.
static EMBEDDED_GRID: LazyLock<Arc<HeightRevisionGrid>> = LazyLock::new(|| {
    let grid = HeightRevisionGrid::load_embedded();
    tracing::info!(
        "Loaded embedded height revision parameters ({}) with {} grid nodes",
        grid.version(),
        grid.node_count()
    );
    Arc::new(grid)
});

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
                transform: Arc::new(Jgd2011ToWgs84::new()),
            },
            VerticalReprojectorType::Jgd2024ToWgs84 => Mode::Jgd2024ToWgs84 {
                transform: Arc::new(Jgd2024ToWgs84::new()),
                fallback: Arc::new(Jgd2011ToWgs84::new()),
            },
            VerticalReprojectorType::Jgd2011ToJgd2024 => Mode::Jgd2011ToJgd2024 {
                grid: Arc::clone(&EMBEDDED_GRID),
                outside_coverage: params.outside_coverage.unwrap_or_default(),
            },
        };

        Ok(Box::new(VerticalReprojector { mode, skipped: 0 }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum VerticalReprojectorType {
    /// JGD2011 heights (EPSG:6697) to WGS84 ellipsoidal heights (EPSG:4979) using the GSIGEO2011 geoid.
    Jgd2011ToWgs84,
    /// JGD2011 heights (測地成果2011) to JGD2024 heights (測地成果2024) using the GSI height revision parameters. Heights stay orthometric.
    Jgd2011ToJgd2024,
    /// JGD2024 heights (EPSG:6668 + EPSG:11317) to WGS84 ellipsoidal heights (EPSG:4979) using the JPGEO2024 geoid with the Hrefconv2024 correction.
    Jgd2024ToWgs84,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum OutsideCoveragePolicy {
    /// Keep the heights unchanged and mark the feature with the `_heightRevisionSkipped` attribute.
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
    /// What to do with a feature that has a vertex outside the coverage of the height revision parameters (`jgd2011ToJgd2024` only). Defaults to `passThrough`.
    outside_coverage: Option<OutsideCoveragePolicy>,
}

#[derive(Debug, Clone)]
enum Mode {
    Jgd2011ToWgs84 {
        transform: Arc<Jgd2011ToWgs84>,
    },
    Jgd2011ToJgd2024 {
        grid: Arc<HeightRevisionGrid>,
        outside_coverage: OutsideCoveragePolicy,
    },
    Jgd2024ToWgs84 {
        transform: Arc<Jgd2024ToWgs84>,
        fallback: Arc<Jgd2011ToWgs84>,
    },
}

#[derive(Debug, Clone)]
pub struct VerticalReprojector {
    mode: Mode,
    skipped: u64,
}

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

fn revision_skipped(feature: &reearth_flow_types::Feature) -> bool {
    matches!(
        feature.get(HEIGHT_REVISION_SKIPPED_ATTRIBUTE),
        Some(AttributeValue::Bool(true))
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
            Mode::Jgd2011ToWgs84 { transform } => {
                if let Some(geometry) = transform_geometry(&feature.geometry, transform.as_ref()) {
                    feature.geometry = Arc::new(geometry);
                }
            }
            Mode::Jgd2024ToWgs84 {
                transform,
                fallback,
            } => {
                let transform: &dyn VerticalTransform = if revision_skipped(&feature) {
                    fallback.as_ref()
                } else {
                    transform.as_ref()
                };
                if let Some(geometry) = transform_geometry(&feature.geometry, transform) {
                    feature.geometry = Arc::new(geometry);
                }
            }
            Mode::Jgd2011ToJgd2024 {
                grid,
                outside_coverage,
            } => {
                let transform = Jgd2011ToJgd2024::new(Arc::clone(grid));
                let geometry = transform_geometry(&feature.geometry, &transform);
                if transform.take_missed() {
                    match outside_coverage {
                        OutsideCoveragePolicy::Error => {
                            return Err(GeometryProcessorError::VerticalReprojector(format!(
                                "Feature {} has a vertex outside the coverage of the height revision parameters",
                                feature.id
                            ))
                            .into());
                        }
                        OutsideCoveragePolicy::PassThrough => {
                            self.skipped += 1;
                            let mut attributes = (*feature.attributes).clone();
                            attributes.insert(
                                Attribute::new(HEIGHT_REVISION_SKIPPED_ATTRIBUTE),
                                AttributeValue::Bool(true),
                            );
                            feature.attributes = Arc::new(attributes);
                        }
                    }
                } else if let Some(geometry) = geometry {
                    feature.geometry = Arc::new(geometry);
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
        if self.skipped > 0 {
            tracing::warn!(
                "VerticalReprojector left {} feature(s) unrevised because they fall outside the height revision parameter coverage",
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
    use indexmap::IndexMap;
    use reearth_flow_geometry::types::{geometry::Geometry3D, point::Point3D};
    use reearth_flow_types::Feature;

    use super::*;

    // Sendai station. The revision there is +0.1233 m and the geoid heights
    // are 41.8025 m (GSIGEO2011) and 41.9599 m (JPGEO2024 + Hrefconv2024).
    const LON: f64 = 140.8825;
    const LAT: f64 = 38.2592;

    fn point(z: f64) -> Geometry {
        Geometry::with_value(GeometryValue::FlowGeometry3D(Geometry3D::Point(
            Point3D::new(LON, LAT, z),
        )))
    }

    fn height(geometry: &Geometry) -> f64 {
        match &geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => p.z(),
            other => panic!("unexpected geometry {other:?}"),
        }
    }

    #[test]
    fn embedded_grid_revises_heights() {
        let revision = Jgd2011ToJgd2024::new(Arc::clone(&EMBEDDED_GRID));
        let revised = transform_geometry(&point(10.0), &revision).unwrap();
        assert!(!revision.take_missed());
        assert!(
            (height(&revised) - 10.1233).abs() < 5e-4,
            "{}",
            height(&revised)
        );

        let new_path = transform_geometry(&revised, &Jgd2024ToWgs84::new()).unwrap();
        let old_path = transform_geometry(&point(10.0), &Jgd2011ToWgs84::new()).unwrap();
        assert!((height(&new_path) - (10.1233 + 41.9599)).abs() < 5e-3);
        let shift = height(&new_path) - height(&old_path);
        assert!((shift - 0.281).abs() < 2e-3, "shift {shift}");
    }

    #[test]
    fn outside_coverage_is_flagged_and_passed_through() {
        let revision = Jgd2011ToJgd2024::new(Arc::clone(&EMBEDDED_GRID));
        let sea = Geometry::with_value(GeometryValue::FlowGeometry3D(Geometry3D::Point(
            Point3D::new(135.0, 30.0, 10.0),
        )));
        let out = transform_geometry(&sea, &revision).unwrap();
        assert_eq!(height(&out), 10.0);
        assert!(revision.take_missed());
    }

    #[test]
    fn skipped_attribute_selects_fallback() {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new(HEIGHT_REVISION_SKIPPED_ATTRIBUTE),
            AttributeValue::Bool(true),
        );
        assert!(revision_skipped(&Feature::new_with_attributes(attributes)));
        assert!(!revision_skipped(&Feature::new_with_attributes(
            IndexMap::new()
        )));
    }

    #[test]
    fn two_d_geometry_is_left_alone() {
        let g = Geometry::with_value(GeometryValue::None);
        assert!(transform_geometry(&g, &Jgd2011ToWgs84::new()).is_none());
    }
}

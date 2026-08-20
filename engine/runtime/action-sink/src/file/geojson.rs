use std::collections::HashMap;
use std::vec;

use bytes::Bytes;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
use reearth_flow_types::conversion::geojson::{write_feature, CrsCoverage};
use reearth_flow_types::{Attribute, AttributeValue, Code, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct GeoJsonWriterFactory;

impl SinkFactory for GeoJsonWriterFactory {
    fn name(&self) -> &str {
        "GeoJSON Writer"
    }

    fn description(&self) -> &str {
        "Writes features to GeoJSON files, optionally grouping them into separate files."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoJsonWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["geojson", "vector"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: GeoJsonWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SinkError::GeoJsonWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let output = params
            .output
            .compile()
            .map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!("Failed to compile `output`: {e:?}"))
            })?
            .eval_string_variables_only(ctx.variables.clone())
            .map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        let sink = GeoJsonWriter {
            output,
            group_by: params.group_by,
            write_crs: params.write_crs,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoJsonWriter {
    output: String,
    group_by: Option<Vec<Attribute>>,
    write_crs: bool,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # GeoJsonWriter Parameters
///
/// Configuration for writing features to GeoJSON files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoJsonWriterParam {
    /// # Output File
    /// Output path or expression for the GeoJSON file to create.
    pub(super) output: Code,
    /// # Group By
    /// Attributes to group features by, writing a separate file for each distinct group.
    pub(super) group_by: Option<Vec<Attribute>>,
    /// # Write CRS
    /// Whether to declare the coordinate reference system of the written coordinates in a
    /// legacy GeoJSON 2008 `crs` member. Defaults to false; enable it when the coordinates
    /// are not WGS84 longitude / latitude and the consumer reads that member.
    #[serde(default)]
    pub(super) write_crs: bool,
}

impl Sink for GeoJsonWriter {
    fn name(&self) -> &str {
        "GeoJSON Writer"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let key = if let Some(group_by) = &self.group_by {
            if group_by.is_empty() {
                AttributeValue::Null
            } else {
                let key = group_by
                    .iter()
                    .map(|k| feature.get(k).cloned().unwrap_or(AttributeValue::Null))
                    .collect::<Vec<_>>();
                AttributeValue::Array(key)
            }
        } else {
            AttributeValue::Null
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let path = self.output.as_str();
        for (key, features) in self.buffer.iter() {
            let out_path = if *key == AttributeValue::Null {
                path.to_string()
            } else {
                format!("{}/{}.geojson", path, to_hash(key.to_string().as_str()))
            };

            // Keep the sandbox gate at flush time via SinkOutput, then delegate
            // the actual serialization/write to the shared helper.
            let out = crate::SinkOutput::new(&ctx.sandbox_root, &out_path, &ctx.storage_resolver)
                .map_err(crate::errors::SinkError::geojson_writer)?;
            write_geojson_to_storage(&out, features, self.write_crs)?;
        }
        Ok(())
    }
}

/// Serialize `features` as a GeoJSON `FeatureCollection` and write it to `output`.
///
/// This is the single canonical implementation shared by both the `GeoJsonWriter`
/// sink and the `FeatureGeoJsonWriter` processor.
///
/// It takes a [`SinkOutput`](crate::SinkOutput) rather than a bare `Uri` so the
/// sandbox gate stays coupled to the write: callers must go through
/// `SinkOutput::new` (which validates the path against the sandbox root and
/// acquires the storage backend) before they can reach this helper.
///
/// `write_crs` requests the legacy `crs` member described on
/// [`crs_foreign_members`].
pub fn write_geojson_to_storage(
    output: &crate::SinkOutput,
    features: &[Feature],
    write_crs: bool,
) -> Result<(), SinkError> {
    let mut geojson_features: Vec<geojson::Feature> = Vec::with_capacity(features.len());
    // Accumulated as the features are written, so it covers exactly what reaches
    // the file: a feature the writer drops contributes no CRS.
    let mut crs = CrsCoverage::default();
    let mut failed = 0usize;

    for feature in features {
        match write_feature(feature) {
            Ok(written) => {
                crs = crs.and(written.crs);
                geojson_features.extend(written.features);
            }
            Err(e) => {
                failed += 1;
                tracing::warn!(feature_id = %feature.id, error = %e, "failed to convert feature to GeoJSON; omitting it");
            }
        }
    }

    let feature_collection = geojson::FeatureCollection {
        bbox: None,
        features: geojson_features,
        foreign_members: write_crs.then(|| crs_foreign_members(crs, output.uri())),
    };
    let buffer = serde_json::to_vec(&feature_collection)
        .map_err(|e| SinkError::GeoJsonWriter(format!("{e}")))?;
    output
        .write(Bytes::from(buffer))
        .map_err(SinkError::geojson_writer)?;

    if failed > 0 {
        tracing::warn!(
            failed,
            "{failed} feature(s) could not be converted to GeoJSON and were omitted from {}",
            output.uri()
        );
    }
    Ok(())
}

/// Build the legacy GeoJSON 2008 `crs` member, for outputs whose coordinates are
/// not in WGS84 longitude / latitude. RFC 7946 has no such member, so callers
/// write it only when a consumer needs it.
///
/// A named CRS when one EPSG code covers every written coordinate, `null`
/// otherwise — an absent member would instead claim the GeoJSON 2008 default,
/// WGS84 longitude / latitude.
fn crs_foreign_members(coverage: CrsCoverage, output: &Uri) -> geojson::JsonObject {
    let crs = match coverage {
        CrsCoverage::Single(epsg) => {
            let mut properties = geojson::JsonObject::new();
            properties.insert(
                "name".to_string(),
                Value::String(format!("urn:ogc:def:crs:EPSG::{epsg}")),
            );
            let mut crs = geojson::JsonObject::new();
            crs.insert("type".to_string(), Value::String("name".to_string()));
            crs.insert("properties".to_string(), Value::Object(properties));
            Value::Object(crs)
        }
        reason => {
            tracing::warn!(
                %reason,
                "no single CRS covers the coordinates written to {output}; \
                 writing `\"crs\": null` so they are not read as the GeoJSON \
                 default CRS (WGS84 longitude / latitude)"
            );
            Value::Null
        }
    };

    let mut foreign_members = geojson::JsonObject::new();
    foreign_members.insert("crs".to_string(), crs);
    foreign_members
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    /// Fixtures naming what a feature's coordinates are expressed in, so the
    /// same `crs` assertions read the same in both geometry worlds.
    #[cfg(not(feature = "new-geometry"))]
    mod fixture {
        use reearth_flow_geometry::types::{
            geometry::Geometry2D, no_value::NoValue, point::Point2D,
        };
        use reearth_flow_types::{Feature, Geometry, GeometryValue};

        fn point_in(epsg: Option<u16>) -> Feature {
            Feature::new_with_attributes_and_geometry(
                indexmap::IndexMap::new(),
                Geometry {
                    epsg,
                    value: GeometryValue::FlowGeometry2D(Geometry2D::Point(Point2D::new(
                        0.0, 0.0, NoValue,
                    ))),
                },
            )
        }

        pub(super) fn in_epsg(code: u16) -> Feature {
            point_in(Some(code))
        }

        pub(super) fn without_crs() -> Feature {
            point_in(None)
        }
    }

    #[cfg(feature = "new-geometry")]
    mod fixture {
        use reearth_flow_geometry::{
            coordinate::{BaseFrame, CoordinateFrame, EpsgCode, TangentPlane},
            point::Point2D,
            Euclidean2DGeometry, Geometry,
        };
        use reearth_flow_types::Feature;

        fn point_in(frame: CoordinateFrame) -> Feature {
            Feature::new_with_attributes_and_geometry(
                indexmap::IndexMap::new(),
                Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame, [0.0, 0.0]))),
            )
        }

        pub(super) fn in_epsg(code: u16) -> Feature {
            point_in(CoordinateFrame::Crs(EpsgCode::new(code)))
        }

        pub(super) fn without_crs() -> Feature {
            point_in(CoordinateFrame::Euclidean)
        }

        /// In-plane coordinates are metres, not the base CRS's, so the base EPSG
        /// must not be claimed as the collection's CRS.
        pub(super) fn in_tangent_plane() -> Feature {
            point_in(CoordinateFrame::Tangent(Box::new(TangentPlane {
                base: BaseFrame::Crs(EpsgCode::new(6675)),
                origin: [0.0, 0.0, 0.0],
                u: [1.0, 0.0, 0.0],
                v: [0.0, 1.0, 0.0],
            })))
        }

        pub(super) fn without_geometry() -> Feature {
            Feature::new_with_attributes_and_geometry(indexmap::IndexMap::new(), Geometry::None)
        }
    }

    fn output() -> Uri {
        Uri::from_str("file:///tmp/out.geojson").unwrap()
    }

    /// How far one CRS covers what writing `features` puts in the output, as the
    /// write itself accumulates it.
    fn coverage_of(features: &[Feature]) -> CrsCoverage {
        features
            .iter()
            .map(|feature| {
                write_feature(feature)
                    .expect("feature expected to convert")
                    .crs
            })
            .fold(CrsCoverage::default(), CrsCoverage::and)
    }

    /// The `crs` member written for `features` when one is asked for.
    fn crs_of(features: &[Feature]) -> Value {
        crs_foreign_members(coverage_of(features), &output())["crs"].clone()
    }

    /// An empty `FeatureCollection` serialized as it would be written, so the
    /// assertions on it are about the `crs` member alone.
    fn serialized_crs_member(write_crs: bool, coverage: CrsCoverage) -> String {
        let collection = geojson::FeatureCollection {
            bbox: None,
            features: Vec::new(),
            foreign_members: write_crs.then(|| crs_foreign_members(coverage, &output())),
        };
        serde_json::to_string(&collection).unwrap()
    }

    #[test]
    fn named_crs_from_a_single_epsg() {
        let crs = crs_of(&[fixture::in_epsg(6675), fixture::in_epsg(6675)]);
        assert_eq!(crs["type"], "name");
        assert_eq!(crs["properties"]["name"], "urn:ogc:def:crs:EPSG::6675");
    }

    #[test]
    fn null_crs_when_epsg_codes_differ() {
        let crs = crs_of(&[fixture::in_epsg(6675), fixture::in_epsg(6669)]);
        assert_eq!(crs, Value::Null);
    }

    #[test]
    fn null_crs_when_no_coordinate_carries_one() {
        let crs = crs_of(&[fixture::without_crs(), fixture::without_crs()]);
        assert_eq!(crs, Value::Null);
    }

    // A null `crs` must reach the file: an absent member would claim the GeoJSON
    // default CRS (WGS84 longitude / latitude) for coordinates that are not in it.
    #[test]
    fn null_crs_is_serialized_rather_than_omitted() {
        let json = serialized_crs_member(true, coverage_of(&[fixture::without_crs()]));
        assert!(json.contains("\"crs\":null"), "got: {json}");
    }

    // The `crs` member is a GeoJSON 2008 extension, so it is written only when
    // asked for — not even as `null`, which would still be an extension member.
    #[test]
    fn no_crs_member_unless_it_is_asked_for() {
        let json = serialized_crs_member(false, coverage_of(&[fixture::in_epsg(6675)]));
        assert!(!json.contains("crs"), "got: {json}");
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn null_crs_for_a_tangent_plane() {
        assert_eq!(crs_of(&[fixture::in_tangent_plane()]), Value::Null);
    }

    #[cfg(feature = "new-geometry")]
    #[test]
    fn null_crs_when_nothing_carries_coordinates() {
        assert_eq!(crs_of(&[fixture::without_geometry()]), Value::Null);
    }

    // One CRS-bearing feature does not name the collection's CRS while another
    // feature's coordinates are not in that CRS.
    #[cfg(feature = "new-geometry")]
    #[test]
    fn null_crs_when_only_some_coordinates_carry_one() {
        let crs = crs_of(&[fixture::in_epsg(6675), fixture::without_crs()]);
        assert_eq!(crs, Value::Null);
    }

    // What leaves the CRS unstated differs by geometry. Here a feature can carry
    // coordinates without an EPSG code, which says nothing about them rather than
    // placing them outside the code its neighbours name.
    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn named_crs_survives_a_feature_without_an_epsg() {
        let crs = crs_of(&[fixture::in_epsg(6675), fixture::without_crs()]);
        assert_eq!(crs["properties"]["name"], "urn:ogc:def:crs:EPSG::6675");
    }

    // Here every leaf states its frame, so the only feature saying nothing about
    // the CRS is one that writes no coordinates at all.
    #[cfg(feature = "new-geometry")]
    #[test]
    fn named_crs_survives_a_feature_without_coordinates() {
        let crs = crs_of(&[fixture::in_epsg(6675), fixture::without_geometry()]);
        assert_eq!(crs["properties"]["name"], "urn:ogc:def:crs:EPSG::6675");
    }
}

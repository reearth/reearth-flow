use std::collections::HashMap;
use std::vec;

use bytes::Bytes;
use reearth_flow_common::str::to_hash;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, FEATURES_PORT};
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
        "Writes geographic features to GeoJSON files with optional grouping"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeoJsonWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["geojson"]
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
            .eval_string_env_only(ctx.env_vars.clone())
            .map_err(|e| {
                SinkError::GeoJsonWriterFactory(format!("Failed to evaluate `output`: {e:?}"))
            })?;
        let sink = GeoJsonWriter {
            output,
            group_by: params.group_by,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct GeoJsonWriter {
    output: String,
    group_by: Option<Vec<Attribute>>,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # GeoJsonWriter Parameters
///
/// Configuration for writing features to GeoJSON files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct GeoJsonWriterParam {
    /// Output path or expression for the GeoJSON file to create
    pub(super) output: Code,
    /// Optional attributes to group features by, creating separate files for each group
    pub(super) group_by: Option<Vec<Attribute>>,
}

impl Sink for GeoJsonWriter {
    fn name(&self) -> &str {
        "GeoJSON Writer"
    }

    #[cfg(not(feature = "new-geometry"))]
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

    #[cfg(not(feature = "new-geometry"))]
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
            write_geojson_to_storage(&out, features)?;
        }
        Ok(())
    }
}

/// Serialize `features` as a GeoJSON `FeatureCollection` and write it to `output`.
///
/// This is the single canonical implementation shared by both the `GeoJsonWriter`
/// sink and the `FeatureGeoJsonWriter` processor. It is gated on
/// `not(new-geometry)` because it relies on `TryFrom<Feature> for
/// Vec<geojson::Feature>`, which is only provided in the current geometry world.
///
/// It takes a [`SinkOutput`](crate::SinkOutput) rather than a bare `Uri` so the
/// sandbox gate stays coupled to the write: callers must go through
/// `SinkOutput::new` (which validates the path against the sandbox root and
/// acquires the storage backend) before they can reach this helper.
#[cfg(not(feature = "new-geometry"))]
pub fn write_geojson_to_storage(
    output: &crate::SinkOutput,
    features: &[Feature],
) -> Result<(), SinkError> {
    let mut geojson_features: Vec<geojson::Feature> = Vec::with_capacity(features.len());
    let mut failed = 0usize;

    for feature in features {
        match TryInto::<Vec<geojson::Feature>>::try_into(feature.clone()) {
            Ok(mut converted) => geojson_features.append(&mut converted),
            Err(e) => {
                failed += 1;
                tracing::warn!(feature_id = %feature.id, error = %e, "failed to convert feature to GeoJSON; omitting it");
            }
        }
    }

    let feature_collection = geojson::FeatureCollection {
        bbox: None,
        features: geojson_features,
        foreign_members: crs_foreign_members(features),
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

/// Build the `crs` foreign member for a `FeatureCollection` from the features'
/// geometry EPSG codes.
///
/// This emits the legacy GeoJSON 2008 `crs` member so downstream desktop GIS
/// tools (QGIS/GDAL) interpret the projected coordinates in the correct CRS
/// instead of assuming WGS84. It is intentionally non-standard under RFC 7946,
/// which fixes coordinates to WGS84; the quality-check error outputs keep the
/// projected coordinates used during inspection and rely on this member.
///
/// Returns `None` (no `crs` member, preserving the previous behavior) when no
/// feature carries an EPSG code. When multiple distinct EPSG codes are present,
/// a warning is logged and the first one encountered is used.
#[cfg(not(feature = "new-geometry"))]
fn crs_foreign_members(features: &[Feature]) -> Option<geojson::JsonObject> {
    let mut epsg: Option<u16> = None;
    for feature in features {
        let Some(code) = feature.geometry.epsg else {
            continue;
        };
        match epsg {
            None => epsg = Some(code),
            Some(existing) if existing != code => {
                tracing::warn!(
                    first = existing,
                    other = code,
                    "GeoJSON features have mixed EPSG codes; using the first one for the `crs` member"
                );
            }
            Some(_) => {}
        }
    }

    let epsg = epsg?;
    let mut properties = geojson::JsonObject::new();
    properties.insert(
        "name".to_string(),
        Value::String(format!("urn:ogc:def:crs:EPSG::{epsg}")),
    );
    let mut crs = geojson::JsonObject::new();
    crs.insert("type".to_string(), Value::String("name".to_string()));
    crs.insert("properties".to_string(), Value::Object(properties));

    let mut foreign_members = geojson::JsonObject::new();
    foreign_members.insert("crs".to_string(), Value::Object(crs));
    Some(foreign_members)
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod tests {
    use super::*;
    use reearth_flow_types::{Geometry, GeometryValue};

    fn feature_with_epsg(epsg: Option<u16>) -> Feature {
        Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            Geometry {
                epsg,
                value: GeometryValue::None,
            },
        )
    }

    fn crs_name(members: &geojson::JsonObject) -> &str {
        members["crs"]["properties"]["name"].as_str().unwrap()
    }

    #[test]
    fn crs_member_from_single_epsg() {
        let features = vec![feature_with_epsg(Some(6675))];
        let members = crs_foreign_members(&features).expect("crs member expected");
        assert_eq!(members["crs"]["type"], "name");
        assert_eq!(crs_name(&members), "urn:ogc:def:crs:EPSG::6675");
    }

    #[test]
    fn no_crs_member_when_all_epsg_missing() {
        let features = vec![feature_with_epsg(None), feature_with_epsg(None)];
        assert!(crs_foreign_members(&features).is_none());
    }

    #[test]
    fn mixed_epsg_uses_first_encountered() {
        let features = vec![
            feature_with_epsg(None),
            feature_with_epsg(Some(6675)),
            feature_with_epsg(Some(6669)),
        ];
        let members = crs_foreign_members(&features).expect("crs member expected");
        assert_eq!(crs_name(&members), "urn:ogc:def:crs:EPSG::6675");
    }
}

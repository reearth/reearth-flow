#[cfg(feature = "new-geometry")]
mod next;

#[cfg(feature = "new-geometry")]
use next::coords::{FrameError, to_wgs84};
#[cfg(feature = "new-geometry")]
use next::extract::{ArealError, MAX_FACES_PER_FEATURE, faces_of};
#[cfg(feature = "new-geometry")]
use next::packet::{face_packet, polyline_packet};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::coordinate::CoordinateFrame;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::ReprojectionCache;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::first_vertex;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry as NewGeometry};

use std::collections::HashMap;
use std::io::Write;
use std::vec;

use bytes::Bytes;
use indexmap::IndexMap;
use itertools::Itertools;
use nusamai_czml::{
    CzmlBoolean, CzmlPolygon, Packet, PositionList, PositionListOfLists,
    PositionListOfListsProperties, PositionListProperties, StringProperties, StringValueType,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use reearth_flow_common::str::to_hash;
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{Context, ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{FEATURES_PORT, Port, Sink, SinkFactory};
use reearth_flow_types::{Attribute, AttributeValue, Code, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// currently czml writer is citygml data specific
use reearth_flow_types::CitygmlFeatureExt;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct CzmlWriterFactory;

impl SinkFactory for CzmlWriterFactory {
    fn name(&self) -> &str {
        "CZML Writer"
    }

    fn description(&self) -> &str {
        "Writes features to a CZML file for display in Cesium, or to one file per group when `groupBy` is set. With `timeField` and `groupTimeseriesBy` together, each group becomes one entity whose position is written as time-tagged samples, except for areal features, which keep a single footprint. With `timeField` alone, every feature becomes its own entity with a static position and its own availability window. Features with no time attribute are written as static entities."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CzmlWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Output"]
    }

    fn tags(&self) -> &[&'static str] {
        &["czml", "3d"]
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
        let mut params: CzmlWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::CzmlWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        // Before `sanitize`, so a folded-in attribute gets the same expression
        // unwrapping as one written under `polygonStyle`. Only when no
        // `polygonStyle` block was given: a params object carrying both
        // spellings is a half-finished migration, and the new one is what the
        // user wrote last.
        if params.polygon_style.is_none()
            && (params.legacy_color_attribute.is_some()
                || params.legacy_opacity.is_some()
                || params.legacy_height_attribute.is_some())
        {
            params.polygon_style = Some(PolygonStyleParam {
                color_attribute: params.legacy_color_attribute.take(),
                opacity: params.legacy_opacity.unwrap_or_else(default_opacity),
                height_attribute: params.legacy_height_attribute.take(),
            });
        }
        params.sanitize();
        // A degree of 0 is not an interpolation: CZML's own `interpolationDegree`
        // is the degree of the polynomial, so 0 would be written straight into the
        // output for a viewer to reject. Refuse it here, where the user can still
        // fix the workflow, rather than emitting an unusable file.
        if params.interpolation_degree < 1 {
            return Err(SinkError::CzmlWriterFactory(
                "`interpolationDegree` must be 1 or greater".to_string(),
            )
            .into());
        }
        // Refused here for the same reason as the degree above: `epoch` is
        // written into the file verbatim and every conversion that counts a
        // numeric time from it parses it as RFC 3339. A value like
        // `2024-01-01` therefore left `position.epoch` unreadable while
        // availability and the clock fell back to raw numbers — an unusable
        // file from a run that exited green.
        if let Some(epoch) = &params.epoch
            && chrono::DateTime::parse_from_rfc3339(epoch).is_err()
        {
            return Err(SinkError::CzmlWriterFactory(format!(
                "`epoch` must be an RFC 3339 timestamp such as \
                 `2024-01-01T00:00:00Z`, not {epoch:?}"
            ))
            .into());
        }
        let output = params
            .output
            .compile()
            .map_err(|e| SinkError::CzmlWriterFactory(format!("Failed to compile `output`: {e}")))?
            .eval_string_variables_only(ctx.variables.clone())
            .map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to evaluate `output`: {e}"))
            })?;
        let polygon_style_was_set = params.polygon_style.is_some();
        let style = params.polygon_style.unwrap_or_default();
        let sink = CzmlWriter {
            params: CzmlWriterCompiledParam {
                output,
                group_by: params.group_by,
                time_field: params.time_field,
                epoch: params.epoch,
                interpolation_algorithm: params.interpolation_algorithm,
                interpolation_degree: params.interpolation_degree,
                group_timeseries_by: params.group_timeseries_by,
                color_attribute: style.color_attribute,
                opacity: style.opacity,
                height_attribute: style.height_attribute,
                polygon_style_was_set,
            },
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(super) struct CzmlWriterCompiledParam {
    pub(super) output: String,
    pub(super) group_by: Option<Vec<Attribute>>,
    pub(super) time_field: Option<Attribute>,
    pub(super) epoch: Option<String>,
    pub(super) interpolation_algorithm: InterpolationAlgorithm,
    pub(super) interpolation_degree: u32,
    pub(super) group_timeseries_by: Option<Attribute>,
    pub(super) color_attribute: Option<Attribute>,
    pub(super) opacity: u8,
    pub(super) height_attribute: Option<Attribute>,
    /// Whether the user asked for styling at all. The three fields above are
    /// filled from `PolygonStyleParam::default()` when they did not, so they
    /// cannot answer this, and the geometry-only path needs the answer to say
    /// that the styling it was given will not reach the output.
    ///
    /// Only the new-geometry `finish` reads it; the legacy path it would warn
    /// about is the one being retired, so it is not wired there.
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    pub(super) polygon_style_was_set: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CzmlWriter {
    pub(super) params: CzmlWriterCompiledParam,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # CZML Writer Parameters
///
/// Configures where the CZML is written, how features are grouped into files
/// and entities, and how a derived polygon is styled.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CzmlWriterParam {
    /// # Output File Path
    /// Path of the CZML file to write. Evaluated once when the workflow starts,
    /// with workflow variables only, so it cannot depend on feature attributes.
    pub(super) output: Code,
    /// # Group By Attributes
    /// Attributes to group features by, writing one file per distinct group
    /// under the `output` path and naming it after a hash of the group's
    /// values. When omitted, every feature goes to the single file `output`
    /// names.
    pub(super) group_by: Option<Vec<Attribute>>,
    /// # Time Field
    /// Attribute holding each feature's time, either an ISO 8601 timestamp or a
    /// number of seconds counted from `epoch`. Set together with
    /// `groupTimeseriesBy`, a group merges into one entity whose position is
    /// written as time-tagged samples, except for areal features, which keep a
    /// single footprint. Set alone, each feature becomes its own entity,
    /// limited to the interval its time falls in, and supplies the timing only
    /// rather than something to draw: an areal feature is drawn as its polygon,
    /// while anything else needs a graphic of its own from a `czml.*`
    /// attribute, because a position by itself draws nothing.
    pub(super) time_field: Option<Attribute>,
    /// # Group Timeseries By
    /// Attribute whose value merges features into a single time-dynamic entity,
    /// one entity per distinct value. Has no effect unless `timeField` is set.
    pub(super) group_timeseries_by: Option<Attribute>,
    /// # Epoch
    /// Base time that numeric `timeField` values count from, as an RFC 3339
    /// timestamp carrying a date, a time and a zone, such as
    /// `2024-01-01T00:00:00Z`; a bare date like `2024-01-01` is refused. When
    /// omitted and every time value is numeric, the Unix epoch
    /// `1970-01-01T00:00:00Z` is used.
    pub(super) epoch: Option<String>,
    /// # Interpolation Algorithm
    /// How a viewer interpolates between the time-tagged samples. Written into
    /// the output as a hint; the engine itself never interpolates. Applies
    /// only to entities merged by `groupTimeseriesBy`.
    #[serde(default)]
    pub(super) interpolation_algorithm: InterpolationAlgorithm,
    /// # Interpolation Degree
    /// Degree of the interpolation polynomial, written alongside the algorithm.
    /// Linear interpolation takes 1; 5 is typical for Lagrange. Applies only to
    /// entities merged by `groupTimeseriesBy`.
    #[serde(default = "default_interpolation_degree")]
    #[schemars(range(min = 1))]
    pub(super) interpolation_degree: u32,
    /// # Polygon Style
    /// Fill colour and extrusion for the single polygon the writer derives from
    /// a feature's geometry. This applies to entities merged by
    /// `groupTimeseriesBy` and to features read back from CZML; a feature
    /// written straight from its geometry gets one packet per face instead,
    /// with no fill colour, extrusion or outline.
    #[serde(default)]
    pub(super) polygon_style: Option<PolygonStyleParam>,
    // The three styling parameters below sat at the top level until they were
    // grouped under `polygonStyle` to bring the count to §3.5's guideline.
    // `CzmlWriterParam` does not deny unknown fields, so a workflow still on
    // the flat spelling deserialized clean, left `polygon_style` as `None`,
    // and wrote an unstyled polygon with no error and no warning. These keep
    // it working, the same answer this action already gives for `force2d` and
    // for the uppercase interpolation values.
    //
    // `#[schemars(skip)]` keeps them out of the schema, and so out of the
    // palette and `action.md`: the advertised surface stays the eight
    // parameters the audit designed. `build()` folds them into `polygon_style`
    // and nothing downstream reads them, so this is the only place the old
    // spelling exists.
    #[serde(default, rename = "colorAttribute", skip_serializing)]
    #[schemars(skip)]
    pub(super) legacy_color_attribute: Option<Attribute>,
    #[serde(default, rename = "opacity", skip_serializing)]
    #[schemars(skip)]
    pub(super) legacy_opacity: Option<u8>,
    #[serde(default, rename = "heightAttribute", skip_serializing)]
    #[schemars(skip)]
    pub(super) legacy_height_attribute: Option<Attribute>,
}

/// # Polygon Style
///
/// Fill colour and extrusion for a derived polygon.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolygonStyleParam {
    /// # Color Attribute
    /// Attribute holding the fill colour as a hex string, such as `#ffd8c0`.
    /// When omitted, or when the attribute does not parse, the polygon is left
    /// unfilled by colour.
    pub(super) color_attribute: Option<Attribute>,
    /// # Opacity
    /// Alpha applied to the fill colour, from 0 for transparent to 255 for
    /// opaque. Has no effect without `colorAttribute`.
    #[serde(default = "default_opacity")]
    pub(super) opacity: u8,
    /// # Height Attribute
    /// Attribute holding the extrusion height in metres. When set, the polygon
    /// is extruded from the ground to that height and closed top and bottom.
    pub(super) height_attribute: Option<Attribute>,
}

/// Hand-written rather than derived. `#[serde(default = "default_opacity")]`
/// fills `opacity` only when a `polygonStyle` object is present and omits the
/// field; when the whole block is absent, `build()` reaches for `Default`
/// instead, and a derived one would hand it `opacity: 0` rather than the 180
/// the schema advertises.
impl Default for PolygonStyleParam {
    fn default() -> Self {
        Self {
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
        }
    }
}

fn default_interpolation_degree() -> u32 {
    1
}

fn default_opacity() -> u8 {
    180
}

/// Strip common expression wrappers from an attribute name.
///
/// The UI may wrap plain attribute names in expression syntax like
/// `env.get("__value").field_name`. This extracts the bare attribute name.
fn sanitize_attribute(attr: &Attribute) -> Attribute {
    let s = attr.inner();
    // env.get("__value").field_name → field_name
    if let Some(rest) = s.strip_prefix("env.get(\"__value\").") {
        return Attribute::new(rest);
    }
    Attribute::new(s)
}

/// Strip surrounding literal quote characters from a string.
///
/// The UI may double-quote epoch values, e.g. `"\"2024-01-01T00:00:00Z\""`.
/// After JSON deserialization this becomes `"2024-01-01T00:00:00Z"` (with
/// literal `"` chars). This helper removes them.
fn sanitize_epoch(epoch: &str) -> String {
    let s = epoch.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

impl CzmlWriterParam {
    /// Normalize fields that may contain expression syntax from the UI.
    fn sanitize(&mut self) {
        if let Some(ref attr) = self.time_field {
            self.time_field = Some(sanitize_attribute(attr));
        }
        if let Some(ref attr) = self.group_timeseries_by {
            self.group_timeseries_by = Some(sanitize_attribute(attr));
        }
        if let Some(style) = self.polygon_style.as_mut() {
            if let Some(ref attr) = style.color_attribute {
                style.color_attribute = Some(sanitize_attribute(attr));
            }
            if let Some(ref attr) = style.height_attribute {
                style.height_attribute = Some(sanitize_attribute(attr));
            }
        }
        if let Some(ref epoch) = self.epoch {
            self.epoch = Some(sanitize_epoch(epoch));
        }
    }
}

/// Interpolation algorithm for time-dynamic properties, written into the
/// output for the viewer to apply.
//
// Doc comments on this enum ship to users (§2), so the note that belongs to
// maintainers lives below the line: the wire values are camelCase per §3.4,
// while `Display` emits the uppercase spelling CZML itself requires. The two
// are deliberately different and neither may be derived from the other.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) enum InterpolationAlgorithm {
    /// # Linear
    /// Straight-line interpolation between neighbouring samples. Takes an
    /// interpolation degree of 1.
    #[default]
    #[serde(alias = "LINEAR")]
    Linear,
    /// # Lagrange
    /// Polynomial interpolation through the samples, giving a smooth path.
    /// Takes a higher interpolation degree, typically 5.
    #[serde(alias = "LAGRANGE")]
    Lagrange,
    /// # Hermite
    /// Spline interpolation that also reads tangent data where the samples
    /// carry it.
    #[serde(alias = "HERMITE")]
    Hermite,
}

impl std::fmt::Display for InterpolationAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => write!(f, "LINEAR"),
            Self::Lagrange => write!(f, "LAGRANGE"),
            Self::Hermite => write!(f, "HERMITE"),
        }
    }
}

/// Parameter-level tests that touch no geometry, so they run in both worlds.
#[cfg(test)]
mod param_tests {
    // Readable diffs on a failing packet or sample array.
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_interpolation_algorithms_old_uppercase_values_still_deserialize() {
        // The audit renamed the wire values to camelCase (§3.4). The uppercase
        // spellings stay as serde aliases so a workflow written before the
        // rename still loads; dropping them would fail the whole job at build
        // time rather than degrading.
        for (wire, expected) in [
            ("LINEAR", "LINEAR"),
            ("LAGRANGE", "LAGRANGE"),
            ("HERMITE", "HERMITE"),
            ("linear", "LINEAR"),
            ("lagrange", "LAGRANGE"),
            ("hermite", "HERMITE"),
        ] {
            let parsed: InterpolationAlgorithm = serde_json::from_value(serde_json::json!(wire))
                .unwrap_or_else(|e| {
                    panic!("`{wire}` must still deserialize: {e}");
                });
            // `Display` is what reaches the CZML file, and CZML wants the
            // uppercase spelling whichever way the parameter was written.
            assert_eq!(parsed.to_string(), expected);
        }
    }

    #[test]
    fn a_non_ascii_colour_is_refused_not_panicked_on() {
        // `len()` is a byte count: "aé123" is six bytes and five characters, so
        // the length check passed and `&hex[0..2]` split `é`, panicking on a
        // feature attribute's value. Exposing the writer and routing the
        // grouped-timeseries path through it widened who can reach this.
        assert_eq!(hex_to_rgba("#aé123", 180), None);
        assert_eq!(hex_to_rgba("aé123", 180), None);
        // Still refuses the merely wrong, and still accepts the right.
        assert_eq!(hex_to_rgba("#zzzzzz", 180), None);
        assert_eq!(hex_to_rgba("#fff", 180), None);
        assert_eq!(hex_to_rgba("#ffd8c0", 180), Some([255, 216, 192, 180]));
    }

    #[test]
    fn an_omitted_polygon_style_keeps_the_opacity_the_schema_advertises() {
        // `#[serde(default = "default_opacity")]` does not reach an absent
        // block, so `Default` has to carry 180 itself.
        assert_eq!(PolygonStyleParam::default().opacity, default_opacity());
        assert_eq!(PolygonStyleParam::default().opacity, 180);
    }

    /// Build the factory with `with`, returning the error message on refusal.
    fn build_with(with: serde_json::Value) -> Result<(), String> {
        let mut map: HashMap<String, serde_json::Value> =
            serde_json::from_value(with).expect("an object");
        map.insert(
            "output".to_string(),
            serde_json::json!({"type": "string", "value": "out.czml"}),
        );
        CzmlWriterFactory
            .build(
                NodeContext::default(),
                EventHub::new(30),
                "CZML Writer".to_string(),
                Some(map),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn an_epoch_czml_cannot_read_is_refused_where_the_user_can_still_fix_it() {
        // `epoch` is written into the file verbatim and parsed as RFC 3339 by
        // every conversion that counts a numeric time from it, so a value like
        // `2024-01-01` left `position.epoch` unreadable while availability and
        // the clock fell back to raw numbers: an unusable file from a green
        // run. Refused in `build()` for the same reason `interpolationDegree`
        // is.
        for bad in ["2024-01-01", "01/01/2024", "not a time"] {
            let err = build_with(serde_json::json!({ "epoch": bad }))
                .expect_err(&format!("{bad} must be refused"));
            assert!(
                err.contains("`epoch`"),
                "the message must name the parameter: {err}"
            );
        }

        // The accepted shapes, so the guard cannot be satisfied by refusing
        // everything.
        for good in ["2024-01-01T00:00:00Z", "2024-01-01T00:00:00+09:00"] {
            build_with(serde_json::json!({ "epoch": good }))
                .unwrap_or_else(|e| panic!("{good} must be accepted, got {e}"));
        }
        build_with(serde_json::json!({})).expect("no epoch at all is fine");
    }

    #[test]
    fn an_interpolation_degree_below_one_is_refused() {
        let err = build_with(serde_json::json!({ "interpolationDegree": 0 }))
            .expect_err("0 is not an interpolation");
        assert!(
            err.contains("`interpolationDegree`"),
            "the message must name the parameter: {err}"
        );
        build_with(serde_json::json!({ "interpolationDegree": 1 })).expect("1 is the minimum");
    }
}

impl Sink for CzmlWriter {
    fn name(&self) -> &str {
        "CZML Writer"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let key = if let Some(group_by) = &self.params.group_by {
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
        let path = self.params.output.clone();
        for (key, features) in self.buffer.iter() {
            let out_path = if *key == AttributeValue::Null {
                path.clone()
            } else {
                format!("{}/{}.json", path, to_hash(key.to_string().as_str()))
            };
            let out = crate::SinkOutput::new(&ctx.sandbox_root, &out_path, &ctx.storage_resolver)
                .map_err(crate::errors::SinkError::czml_writer)?;

            let is_grouped_timeseries =
                self.params.group_timeseries_by.is_some() && self.params.time_field.is_some();
            let has_citygml = features
                .iter()
                .any(|f| matches!(&f.geometry.value, GeometryValue::CityGmlGeometry(_)));

            if is_grouped_timeseries {
                let buffer = build_timeseries_czml(features, &self.params, &ctx)?;
                out.write(Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            } else if has_citygml {
                let (sender, receiver) = std::sync::mpsc::sync_channel(1000);
                let gctx = ctx.as_context();

                let (ra, rb) = rayon::join(
                    || {
                        features
                            .iter()
                            .par_bridge()
                            .try_for_each_with(sender, |sender, feature| {
                                let packets = feature_to_packets(&gctx, feature);
                                for packet in packets {
                                    let bytes = serde_json::to_vec(&packet).unwrap();
                                    if sender.send(bytes).is_err() {
                                        return Err(SinkError::czml_writer(
                                            "Failed to send packet".to_string(),
                                        ));
                                    };
                                }
                                Ok(())
                            })
                    },
                    || {
                        let doc = Packet {
                            id: Some("document".into()),
                            version: Some("1.0".into()),
                            ..Default::default()
                        };
                        let mut buffer =
                            Vec::from(format!(r#"[{},"#, serde_json::to_string(&doc).unwrap()));

                        let mut iter = receiver.into_iter().peekable();
                        while let Some(bytes) = iter.next() {
                            buffer
                                .write(&bytes)
                                .map_err(crate::errors::SinkError::czml_writer)?;
                            if iter.peek().is_some() {
                                buffer
                                    .write(b",")
                                    .map_err(crate::errors::SinkError::czml_writer)?;
                            };
                        }

                        buffer
                            .write(b"]\n")
                            .map_err(crate::errors::SinkError::czml_writer)?;
                        out.write(Bytes::from(buffer))
                            .map_err(crate::errors::SinkError::czml_writer)
                    },
                );
                ra?;
                rb?;
            } else {
                let buffer = build_embedded_czml(features, &self.params)?;
                out.write(Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            }
        }
        Ok(())
    }

    // `process` is the cheap half: it just buffers by `groupBy` key, exactly
    // like the old-world body above (no geometry access happens here).
    #[cfg(feature = "new-geometry")]
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let key = match &self.params.group_by {
            Some(group_by) if !group_by.is_empty() => AttributeValue::Array(
                group_by
                    .iter()
                    .map(|k| feature.get(k).cloned().unwrap_or(AttributeValue::Null))
                    .collect(),
            ),
            _ => AttributeValue::Null,
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }

    // Dispatch stays three-way, as in the old world:
    //
    //   1. grouped timeseries (`timeField` + `groupTimeseriesBy`)
    //   2. Embedded Mode (`czml.*`-bearing features, or `timeField` alone —
    //      see `is_embedded_mode`), the `preserveRaw` round-trip path
    //   3. otherwise, per-feature geometry packets
    //
    // Only the discriminator between (2) and (3) changed. Old-world `finish`
    // asked `has_citygml` and fell through to Embedded Mode; there is no
    // `CityGmlGeometry` in the new model, so that batch-level question (which
    // used to reject every non-CityGML feature in a batch containing one
    // CityGML sibling) has nothing left to trigger on. Embedded Mode is now
    // selected positively and (3) is the default, deciding every feature on
    // its own geometry in `feature_to_packets_next`: areal geometry writes
    // face packets, lines write polyline packets, anything else with a
    // position writes a position packet.
    //
    // `finish` takes `&self`, not `&mut self`, so the `ReprojectionCache`
    // `to_wgs84` needs can't live on `self`; it is created locally instead —
    // one per output group in grouped-timeseries mode, and one per rayon
    // worker (via `try_for_each_init`, not `try_for_each_with`) in the
    // per-feature fan-out, since `ReprojectionCache` holds raw PROJ handles
    // and is not `Send`.
    #[cfg(feature = "new-geometry")]
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let path = self.params.output.clone();
        for (key, features) in self.buffer.iter() {
            let out_path = if *key == AttributeValue::Null {
                path.clone()
            } else {
                format!("{}/{}.json", path, to_hash(key.to_string().as_str()))
            };
            let out = crate::SinkOutput::new(&ctx.sandbox_root, &out_path, &ctx.storage_resolver)
                .map_err(crate::errors::SinkError::czml_writer)?;

            let is_grouped_timeseries =
                self.params.group_timeseries_by.is_some() && self.params.time_field.is_some();

            if is_grouped_timeseries {
                let mut cache = ReprojectionCache::default();
                let buffer = build_timeseries_czml_next(
                    &ctx.as_context(),
                    features,
                    &self.params,
                    &mut cache,
                )?;
                out.write(Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            } else if is_embedded_mode(features, &self.params) {
                let mut cache = ReprojectionCache::default();
                let buffer = build_embedded_czml_next(
                    &ctx.as_context(),
                    features,
                    &self.params,
                    &mut cache,
                )?;
                out.write(Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::czml_writer)?;
            } else {
                let gctx = ctx.as_context();
                // This is the default path, and the one `polygonStyle` does
                // not reach: a feature written straight from its geometry
                // gets one packet per face, and those faces are unstyled. The
                // most ordinary configuration there is — features with
                // geometry, no `timeField` — therefore accepted the styling
                // block and ignored it in silence. Said once per node, since
                // it is a property of the configuration, not of any feature.
                if self.params.polygon_style_was_set {
                    gctx.as_executor_context(
                        Feature::new_with_attributes(Default::default()),
                        FEATURES_PORT.clone(),
                    )
                    .warn_once(DiagnosticDraft::new(ErrorCode::CzmlStyleNotApplied));
                }
                let (sender, receiver) = std::sync::mpsc::sync_channel(1000);

                let (ra, rb) = rayon::join(
                    || {
                        features.iter().par_bridge().try_for_each_init(
                            // `sender` is moved wholesale into this `init`, so once
                            // `try_for_each_init` returns, dropping `init` drops the
                            // last handle to it and the channel actually closes.
                            move || (sender.clone(), ReprojectionCache::default()),
                            |(sender, cache), feature| {
                                let packets = feature_to_packets_next(&gctx, feature, cache);
                                for packet in packets {
                                    let bytes = serde_json::to_vec(&packet).unwrap();
                                    if sender.send(bytes).is_err() {
                                        return Err(SinkError::czml_writer(
                                            "Failed to send packet".to_string(),
                                        ));
                                    };
                                }
                                Ok(())
                            },
                        )
                    },
                    || {
                        let doc = Packet {
                            id: Some("document".into()),
                            version: Some("1.0".into()),
                            ..Default::default()
                        };
                        // The separator is emitted *before* each subsequent
                        // packet, not seeded as a trailing comma after the
                        // document packet: with zero feature packets (every
                        // feature skipped -- an all-Euclidean-frame batch is
                        // the likely production case) a seeded trailing comma
                        // would produce `[{...},]`, which is invalid JSON.
                        // This way a zero-packet batch still closes as a
                        // valid, document-only array.
                        let mut buffer =
                            Vec::from(format!("[{}", serde_json::to_string(&doc).unwrap()));

                        for bytes in receiver.into_iter() {
                            buffer
                                .write(b",")
                                .map_err(crate::errors::SinkError::czml_writer)?;
                            buffer
                                .write(&bytes)
                                .map_err(crate::errors::SinkError::czml_writer)?;
                        }

                        buffer
                            .write(b"]\n")
                            .map_err(crate::errors::SinkError::czml_writer)?;
                        out.write(Bytes::from(buffer))
                            .map_err(crate::errors::SinkError::czml_writer)
                    },
                );
                ra?;
                rb?;
            }
        }
        Ok(())
    }
}

/// Turn a [`FrameError`] into the one diagnostic code the writer has for it,
/// regardless of whether it came from a face, a polyline or a bare position —
/// all three go through [`to_wgs84`] and can fail the same way.
#[cfg(feature = "new-geometry")]
fn frame_error_draft(err: FrameError, feature: &Feature) -> DiagnosticDraft {
    let detail = match err {
        FrameError::Unplaceable => "coordinate frame is not a concrete CRS".to_string(),
        FrameError::Transform(msg) => msg,
    };
    DiagnosticDraft::new(ErrorCode::CzmlUnplaceableFrame)
        .with_message(format!("{detail}: feature_id={}", feature.id))
}

/// A `description` value that references the properties packet's own
/// description. Mirrors `next::packet`'s private helper of the same shape,
/// which isn't reachable from here (this file may not modify that module).
#[cfg(feature = "new-geometry")]
fn description_reference_next(parent_id: &str) -> StringValueType {
    StringValueType::Object(StringProperties {
        reference: Some(format!("{parent_id}#description")),
        ..Default::default()
    })
}

/// Build a `{"position": {"cartographicDegrees": [...]}}` packet for a
/// feature whose geometry is neither areal nor a line — the last of the three
/// per-feature modes. Carries a minimal `point` graphic too, so the entity is
/// actually visible in Cesium rather than being data-only.
#[cfg(feature = "new-geometry")]
fn position_packet(
    cache: &mut ReprojectionCache,
    pos: [f64; 3],
    frame: &CoordinateFrame,
    parent_id: &str,
) -> Result<Packet, FrameError> {
    let mut coords = vec![pos];
    to_wgs84(cache, frame, &mut coords)?;
    // `to_wgs84` returns [lat, lon, height] (EPSG:4979 authority order, see
    // `next::coords`'s module docs); cartographicDegrees wants [lon, lat, height].
    let [lat, lon, height] = coords[0];

    let mut position: HashMap<String, Value> = HashMap::new();
    position.insert(
        "cartographicDegrees".to_string(),
        serde_json::json!([lon, lat, height]),
    );

    let mut point: HashMap<String, Value> = HashMap::new();
    point.insert("pixelSize".to_string(), serde_json::json!(10));
    point.insert("heightReference".to_string(), serde_json::json!("NONE"));

    Ok(Packet {
        position: Some(position),
        point: Some(point),
        description: Some(description_reference_next(parent_id)),
        parent: Some(parent_id.to_string()),
        ..Default::default()
    })
}

/// One line's vertex chain, paired with the frame it is expressed in.
#[cfg(feature = "new-geometry")]
type LineChain = (Vec<[f64; 3]>, CoordinateFrame);

/// **Every** line a feature carries, each with its own frame. Mirrors
/// `next::extract::faces_of`'s traversal shape rather than `first_vertex`'s:
/// the walk collects all members instead of stopping at the first match,
/// because a CityGML `MultiCurve` arrives as a `Collection` of `LineString`s
/// and writing only its first line is the exact linear analogue of the
/// MultiPolygon-writes-one-face defect this port exists to fix.
///
/// Each chain is returned in its own frame and its own coordinate order; the
/// reprojection and the lat/lon swap happen later, in `polyline_packet`.
#[cfg(feature = "new-geometry")]
fn lines_of(geometry: &NewGeometry) -> Vec<LineChain> {
    let mut out = Vec::new();
    collect_lines(geometry, &mut out);
    out
}

#[cfg(feature = "new-geometry")]
fn collect_lines(geometry: &NewGeometry, out: &mut Vec<LineChain>) {
    match geometry {
        NewGeometry::None => {}
        NewGeometry::Euclidean3D(g) => collect_lines_3d(g, out),
        NewGeometry::Euclidean2D(g) => collect_lines_2d(g, out),
        NewGeometry::GeometryCollection(c) => {
            for member in c.members() {
                collect_lines(member, out);
            }
        }
    }
}

/// The non-linear arms are spelled out one by one rather than swept up by a
/// `_ =>` wildcard: a wildcard is what let the "only the first line" defect
/// hide, and it would silently drop a linear leaf variant added later. An
/// exhaustive match forces that decision at compile time.
#[cfg(feature = "new-geometry")]
fn collect_lines_3d(geometry: &Euclidean3DGeometry, out: &mut Vec<LineChain>) {
    match geometry {
        Euclidean3DGeometry::LineString(l) => out.push((l.coords().to_vec(), l.frame().clone())),
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                collect_lines_3d(member, out);
            }
        }
        // Nothing linear: `Point`/`PointCloud` carry no chain, and the areal
        // and volumetric leaves are drawn by `faces_of` instead. A `Csg` tree
        // carries no coordinates of its own until evaluated.
        Euclidean3DGeometry::Point(_)
        | Euclidean3DGeometry::PointCloud(_)
        | Euclidean3DGeometry::Polygon(_)
        | Euclidean3DGeometry::PolygonMesh(_)
        | Euclidean3DGeometry::TriangularMesh(_)
        | Euclidean3DGeometry::Solid(_)
        | Euclidean3DGeometry::Csg(_) => {}
    }
}

/// As [`collect_lines_3d`], for a 2D-embedded geometry. Exhaustive for the
/// same reason.
#[cfg(feature = "new-geometry")]
fn collect_lines_2d(geometry: &Euclidean2DGeometry, out: &mut Vec<LineChain>) {
    match geometry {
        Euclidean2DGeometry::LineString(l) => {
            // Same 2D-leaf elevation fallback as `first_vertex`: an optional
            // elevation becomes the height when present, `0.0` when absent.
            let z = l.elevation().unwrap_or(0.0);
            out.push((
                l.coords().iter().map(|&[x, y]| [x, y, z]).collect(),
                l.frame().clone(),
            ));
        }
        Euclidean2DGeometry::Collection(c) => {
            for member in c.members() {
                collect_lines_2d(member, out);
            }
        }
        Euclidean2DGeometry::Point(_)
        | Euclidean2DGeometry::Polygon(_)
        | Euclidean2DGeometry::PolygonMesh(_)
        | Euclidean2DGeometry::TriangularMesh(_) => {}
    }
}

/// Why a feature could not be given a position. The two cases carry different
/// diagnostic codes, which is the whole reason this is not an `Option`: a
/// feature with nothing to place and a feature in a frame we cannot resolve
/// are different problems with different fixes.
#[cfg(feature = "new-geometry")]
enum PlacementError {
    /// The geometry holds no vertex to position the entity at.
    NothingToPlace,
    /// A vertex exists, but its frame could not be reprojected to WGS84.
    Frame(FrameError),
}

/// A feature's representative position, reprojected to WGS84 and swapped into
/// `cartographicDegrees` order (`[lon, lat, height]`).
///
/// Used by the two entity-building paths (grouped timeseries and Embedded
/// Mode). Both used to take an `Option` here and drop an unplaceable sample
/// without a word, so a batch could lose every position while the run stayed
/// green; the caller now reports through [`placement_draft`].
#[cfg(feature = "new-geometry")]
fn cartographic_position(
    cache: &mut ReprojectionCache,
    feature: &Feature,
) -> Result<(f64, f64, f64), PlacementError> {
    let (pos, frame) = first_vertex(&feature.geometry).ok_or(PlacementError::NothingToPlace)?;
    let mut coords = vec![pos];
    to_wgs84(cache, &frame, &mut coords).map_err(PlacementError::Frame)?;
    let [lat, lon, height] = coords[0];
    Ok((lon, lat, height))
}

/// The diagnostic for a [`PlacementError`], so that the entity paths report the
/// same two codes mode 3 reports for the same two causes.
#[cfg(feature = "new-geometry")]
fn placement_draft(err: PlacementError, feature: &Feature) -> DiagnosticDraft {
    match err {
        PlacementError::NothingToPlace => DiagnosticDraft::new(ErrorCode::CzmlEmptyGeometry)
            .with_message(format!(
                "feature has no drawable geometry: feature_id={}",
                feature.id
            )),
        PlacementError::Frame(err) => frame_error_draft(err, feature),
    }
}

/// Report a feature the entity paths could not place, then carry on.
///
/// `finish()` has no single feature of its own, so the per-feature
/// `ExecutorContext` that carries `report` is synthesized here exactly as
/// `feature_to_packets_next` does it. The `Result` is discarded for the same
/// reason it is there: a promoted `Fatal` has already reached the node's fatal
/// slot inside `report()`, and these builders return a document, not a
/// per-feature outcome.
#[cfg(feature = "new-geometry")]
fn report_unplaced(gctx: &Context, feature: &Feature, err: PlacementError) {
    let ectx = gctx.as_executor_context(feature.clone(), FEATURES_PORT.clone());
    let _ = ectx.report(placement_draft(err, feature));
}

/// The instant an ISO 8601 time names, or `None` when `s` names none.
///
/// Deliberately wider than RFC 3339, which is all `chrono` parses directly:
/// CZML follows ISO 8601, and a viewer reads a zone-less date-time as UTC, a
/// bare date as its midnight, and a `hh:mm` with no seconds as that minute.
/// Being stricter here drops values that reach the file and work — minute
/// precision in particular is ordinary ISO 8601 that `parse_from_rfc3339`
/// refuses, and refusing it here cost a group that had round-tripped before.
#[cfg(feature = "new-geometry")]
fn parse_iso_instant(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{NaiveDate, NaiveDateTime, TimeZone};

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&chrono::Utc));
    }
    // An explicit numeric zone at minute precision, in either spelling.
    for fmt in ["%Y-%m-%dT%H:%M%:z", "%Y-%m-%dT%H:%M%z"] {
        if let Ok(dt) = chrono::DateTime::parse_from_str(s, fmt) {
            return Some(dt.with_timezone(&chrono::Utc));
        }
    }
    // `Z` and no zone at all are both read as UTC, so they share these
    // formats; `parse_from_str` would otherwise need a `%z` for the `Z`.
    let naive = s
        .strip_suffix('Z')
        .or_else(|| s.strip_suffix('z'))
        .unwrap_or(s);
    for fmt in [
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(naive, fmt) {
            return Some(chrono::Utc.from_utc_datetime(&dt));
        }
    }
    NaiveDate::parse_from_str(naive, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| chrono::Utc.from_utc_datetime(&dt))
}

/// True when CZML can read `s` as a time.
#[cfg(feature = "new-geometry")]
fn is_writable_timestamp(s: &str) -> bool {
    parse_iso_instant(s).is_some()
}

/// The absolute instant a `timeField` value names, or `None` when it names
/// none that can be written.
///
/// Everything downstream of a time value has to agree about *when* it is: the
/// order the samples are written in, the entity's `availability`, and the
/// document `clock`. Comparing the strings cannot do that. Against an epoch of
/// midnight, `"3600"` is 01:00 and `"2024-01-01T02:00:00Z"` is 02:00, yet
/// `'2' < '3'` sorts the later one first, and an availability built from that
/// order runs backwards and covers none of the track. Resolving to one instant
/// before anything is compared is what keeps the answers consistent.
///
/// A number resolves only against an epoch, and only while the result is a
/// time that can be represented: `1e30` is finite, so the arithmetic has to
/// refuse it rather than saturate into a bound no viewer can read.
#[cfg(feature = "new-geometry")]
fn resolve_time_instant(
    time_str: &str,
    epoch: Option<&str>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    if let Some(dt) = parse_iso_instant(time_str) {
        return Some(dt);
    }
    let offset = match time_str.parse::<f64>() {
        Ok(n) if n.is_finite() => n,
        _ => parse_epoch_offset_timestamp(time_str)?,
    };
    let epoch_dt = chrono::DateTime::parse_from_rfc3339(epoch?).ok()?;
    let millis = offset * 1000.0;
    if !(millis >= i64::MIN as f64 && millis <= i64::MAX as f64) {
        return None;
    }
    chrono::Duration::try_milliseconds(millis as i64)
        .and_then(|delta| epoch_dt.checked_add_signed(delta))
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

/// An instant as CZML wants it written, keeping sub-second precision only
/// when the instant actually has some.
#[cfg(feature = "new-geometry")]
fn instant_to_czml(dt: chrono::DateTime<chrono::Utc>) -> String {
    let format = if dt.timestamp_subsec_nanos() == 0 {
        chrono::SecondsFormat::Secs
    } else {
        chrono::SecondsFormat::Millis
    };
    dt.to_rfc3339_opts(format, true)
}

/// What this writer will put in the file for a time value that is not part of
/// an all-numeric group, or `None` when the value names no time at all.
///
/// A number among timestamps still names one, so long as something says what
/// it counts from: converting it keeps a group that mixes the two spellings
/// loadable instead of writing half of it as a string no viewer can read.
#[cfg(feature = "new-geometry")]
fn writable_time_value(time_str: &str, epoch: Option<&str>) -> Option<Value> {
    if is_writable_timestamp(time_str) {
        return Some(serde_json::json!(time_str));
    }
    // Everything else is written as the instant it resolves to, through the
    // same function the document clock asks. Deciding acceptance twice let the
    // two disagree: this branch was gated on `parse::<f64>()`, which the
    // `"<iso>+<N>s"` spelling fails, so the entity dropped that sample while
    // the clock — which resolves it happily — still stretched its interval to
    // cover a time nothing had been written at. One rule, so they cannot drift.
    resolve_time_instant(time_str, epoch).map(|dt| serde_json::json!(instant_to_czml(dt)))
}

/// Report a feature whose time value names no time, so its position cannot be
/// placed on the entity's timeline.
#[cfg(feature = "new-geometry")]
fn report_unparsable_time(gctx: &Context, feature: &Feature, time_str: &str) {
    let ectx = gctx.as_executor_context(feature.clone(), FEATURES_PORT.clone());
    let _ = ectx.report(
        DiagnosticDraft::new(ErrorCode::CzmlUnparsableTime).with_message(format!(
            "dropped position whose time value {time_str:?} is neither a timestamp nor a \
             number: feature_id={}",
            feature.id
        )),
    );
}

/// Report a feature whose position could not be written because it carries no
/// time value. Both shapes of the problem are the same code: `where` names the
/// entity and says which of the two it was, since the draft message is the only
/// place that detail survives (the registry's help text covers both).
#[cfg(feature = "new-geometry")]
fn report_untimed_sample(gctx: &Context, feature: &Feature, where_: &str) {
    let ectx = gctx.as_executor_context(feature.clone(), FEATURES_PORT.clone());
    let _ = ectx.report(
        DiagnosticDraft::new(ErrorCode::CzmlUntimedSample).with_message(format!(
            "dropped position with no time value from {where_}: feature_id={}",
            feature.id
        )),
    );
}

/// Decide and build every CZML packet for one feature (mode 3: geometry-only
/// features that are neither grouped-timeseries nor `czml.*`-bearing). Every
/// face becomes its own polygon packet and every line its own polyline packet,
/// both from the same feature when it carries both; a feature with neither
/// falls back to a bare position. A feature that is none of these — or whose
/// frame can't be placed — is diagnosed and contributes nothing, per the
/// error-routing table:
///
/// | Condition | Code |
/// |---|---|
/// | `FrameError::Unplaceable` / `FrameError::Transform` | `CzmlUnplaceableFrame` |
/// | `ArealError::TooManyFaces(n)` | `CzmlTooManyFaces` |
/// | `ArealError::NothingDrawable` and no position either | `CzmlEmptyGeometry` |
///
/// Every condition routes through `report()`, never `warn()`, matching the
/// old-world path. `warn()` records `WarnContinue` and never calls
/// `handle.resolve(code)`, so a *dropped* feature would be summarized as
/// "warned about ... continue" and a `fatal` disposition override on the node
/// would be ignored outright — an entire batch could vanish with the run
/// still green. This is a sink: there is no downstream port, so `report()`'s
/// disposition cannot drop the feature a second time. Its `Result` is
/// discarded with `let _ =` (this fn returns `Vec<Packet>`, not a `Result`),
/// which is still not a silent drop: a promoted `Fatal` reaches the node's
/// fatal slot inside `report()` itself.
#[cfg(feature = "new-geometry")]
fn feature_to_packets_next(
    gctx: &Context,
    feature: &Feature,
    cache: &mut ReprojectionCache,
) -> Vec<Packet> {
    let ectx = gctx.as_executor_context(feature.clone(), FEATURES_PORT.clone());
    // The new model carries no CityGML `gml:id`-based `feature_id()`, and
    // requiring one would resurrect the old CityGML-only assumption this task
    // removes; `feature.id` (always present) parents every packet instead.
    let parent_id = feature.id.to_string();
    let properties_packet = Packet {
        id: Some(parent_id.clone()),
        description: Some(StringValueType::String(map_to_html_table(
            &feature.attributes,
        ))),
        ..Default::default()
    };

    match faces_of(&feature.geometry) {
        Ok(faces) => {
            // Areal geometry does not make a feature's lines disappear. A
            // collection holding a polygon and a line string reached only the
            // face branch, so the line was dropped with no packet and no
            // diagnostic — the same loss `a_multi_line_feature_writes_every_line`
            // exists to stop, one level up. Both are drawn.
            let lines = lines_of(&feature.geometry);
            let mut packets = Vec::with_capacity(faces.len() + lines.len() + 1);
            // A member that cannot be placed does not condemn its siblings.
            // Each face and each line carries its own frame, so a collection
            // can hold a WGS84 polygon beside a line in a local Euclidean
            // frame; returning early on the line discarded the polygon that
            // had already been built, which is data the writer used to emit.
            // Report the failure, keep what placed, and emit nothing only when
            // nothing placed at all.
            let mut unplaced = None;
            for face in &faces {
                match face_packet(cache, face, &parent_id) {
                    Ok(packet) => packets.push(packet),
                    Err(err) => unplaced = unplaced.or(Some(err)),
                }
            }
            for (vertices, frame) in &lines {
                match polyline_packet(cache, vertices, frame, &parent_id) {
                    Ok(packet) => packets.push(packet),
                    Err(err) => unplaced = unplaced.or(Some(err)),
                }
            }
            // Once per feature, like every other diagnostic on this path: a
            // collection of a thousand unplaceable faces is one problem.
            if let Some(err) = unplaced {
                let _ = ectx.report(frame_error_draft(err, feature));
            }
            if packets.is_empty() {
                return vec![];
            }
            packets.insert(0, properties_packet);
            packets
        }
        Err(ArealError::TooManyFaces(n)) => {
            let _ = ectx.report(
                DiagnosticDraft::new(ErrorCode::CzmlTooManyFaces).with_message(format!(
                    "feature has {n} faces, more than the {MAX_FACES_PER_FEATURE} the writer \
                     will emit: feature_id={}",
                    feature.id
                )),
            );
            vec![]
        }
        Err(ArealError::NothingDrawable) => {
            let lines = lines_of(&feature.geometry);
            if !lines.is_empty() {
                // Every line gets its own polyline packet, exactly as every
                // face gets its own polygon packet above — a `MultiCurve`
                // must not write only its first member.
                let mut packets = Vec::with_capacity(lines.len() + 1);
                let mut unplaced = None;
                for (vertices, frame) in &lines {
                    match polyline_packet(cache, vertices, frame, &parent_id) {
                        Ok(packet) => packets.push(packet),
                        Err(err) => unplaced = unplaced.or(Some(err)),
                    }
                }
                if let Some(err) = unplaced {
                    let _ = ectx.report(frame_error_draft(err, feature));
                }
                if packets.is_empty() {
                    return vec![];
                }
                packets.insert(0, properties_packet);
                packets
            } else if let Some((pos, frame)) = first_vertex(&feature.geometry) {
                match position_packet(cache, pos, &frame, &parent_id) {
                    Ok(packet) => vec![properties_packet, packet],
                    Err(err) => {
                        let _ = ectx.report(frame_error_draft(err, feature));
                        vec![]
                    }
                }
            } else {
                let _ = ectx.report(
                    DiagnosticDraft::new(ErrorCode::CzmlEmptyGeometry).with_message(format!(
                        "feature has no drawable geometry: feature_id={}",
                        feature.id
                    )),
                );
                vec![]
            }
        }
    }
}

/// The epoch numeric times are counted from: the one the user set, else the
/// Unix epoch when every time value is numeric, else none.
///
/// Shared so the document clock and the entity packets cannot disagree about
/// what a bare `60` means.
#[cfg(feature = "new-geometry")]
fn effective_epoch(explicit: Option<&str>, all_numeric: bool) -> Option<String> {
    match explicit {
        Some(epoch) => Some(epoch.to_string()),
        None if all_numeric => Some(UNIX_EPOCH_ISO.to_string()),
        None => None,
    }
}

/// The epoch CZML counts from when numeric times carry no explicit one.
#[cfg(feature = "new-geometry")]
const UNIX_EPOCH_ISO: &str = "1970-01-01T00:00:00Z";

/// Build a CZML document with time-dynamic entities grouped by attribute —
/// the new-geometry counterpart of `build_timeseries_czml`. Same shape;
/// position extraction goes through `first_vertex` + `to_wgs84`
/// (`cartographic_position`) instead of the old geometry-type match.
#[cfg(feature = "new-geometry")]
fn build_timeseries_czml_next(
    gctx: &Context,
    features: &[Feature],
    params: &CzmlWriterCompiledParam,
    cache: &mut ReprojectionCache,
) -> Result<Vec<u8>, BoxedError> {
    let time_field = params
        .time_field
        .as_ref()
        .ok_or_else(|| SinkError::czml_writer("time_field is required for timeseries output"))?;
    let group_attr = params.group_timeseries_by.as_ref().ok_or_else(|| {
        SinkError::czml_writer("group_timeseries_by is required for timeseries output")
    })?;

    let mut groups: IndexMap<String, Vec<&Feature>> = IndexMap::new();
    for feature in features {
        let key = feature
            .get(group_attr)
            .map(attribute_value_to_string)
            .unwrap_or_else(|| "unknown".to_string());
        groups.entry(key).or_default().push(feature);
    }

    let mut doc = serde_json::json!({
        "id": "document",
        "version": "1.0",
    });

    // The clock has to span the instants the entities actually wrote, which
    // means resolving each group the way that group resolves itself. Whether
    // times are numeric — and so which epoch a bare `60` counts from — is
    // decided per entity in `build_entity_packet_next`, so a document holding
    // one numeric group and one ISO group has two epochs in play. Reading the
    // whole document as a single population made `all_numeric` false, and the
    // numeric group's `0` and `60` were then sorted as strings and written
    // into `currentTime` verbatim, where CZML requires an ISO 8601 instant.
    //
    // A value that resolves to no instant is left out for the same reason the
    // entity drops it: a rejected `n/a` used to sort to the end of the strings
    // and become the interval's upper bound, after the entity had refused it.
    let mut instants: Vec<chrono::DateTime<chrono::Utc>> = Vec::new();
    for group_features in groups.values() {
        let times: Vec<String> = group_features
            .iter()
            .filter_map(|f| f.get(time_field).map(attribute_value_to_string))
            .filter(|ts| !ts.is_empty())
            .collect();
        // The same `any && all` the entity asks, over the same population.
        let all_numeric = !times.is_empty()
            && times
                .iter()
                .all(|ts| ts.parse::<f64>().is_ok_and(f64::is_finite));
        let group_epoch = effective_epoch(params.epoch.as_deref(), all_numeric);
        instants.extend(
            times
                .iter()
                .filter_map(|ts| resolve_time_instant(ts, group_epoch.as_deref())),
        );
    }
    instants.sort();

    if let (Some(start), Some(end)) = (instants.first(), instants.last())
        && instants.len() >= 2
    {
        let start_iso = instant_to_czml(*start);
        let end_iso = instant_to_czml(*end);
        doc["clock"] = serde_json::json!({
            "interval": format!("{start_iso}/{end_iso}"),
            "currentTime": start_iso,
            "multiplier": 1,
            "range": "LOOP_STOP",
            "step": "SYSTEM_CLOCK_MULTIPLIER",
        });
    }

    let mut output_buffer = Vec::new();
    output_buffer
        .write(format!("[{}", serde_json::to_string(&doc).unwrap()).as_bytes())
        .map_err(SinkError::czml_writer)?;

    for (entity_id, group_features) in &groups {
        let packet = build_entity_packet_next(gctx, entity_id, group_features, params, cache)?;
        output_buffer.write(b",").map_err(SinkError::czml_writer)?;
        output_buffer
            .write(&serde_json::to_vec(&packet).map_err(SinkError::czml_writer)?)
            .map_err(SinkError::czml_writer)?;
    }

    output_buffer
        .write(b"]\n")
        .map_err(SinkError::czml_writer)?;
    Ok(output_buffer)
}

/// The first face's polygon JSON for `feature`, if its geometry is areal —
/// restores `build_entity_packet`'s old "polygon for polygon features"
/// behavior for the new model. Only the first face is used: a CZML entity
/// carries exactly one graphic, unlike mode 2 (`feature_to_packets_next`)
/// where every feature gets its own entity and every face its own packet.
/// `None` on anything non-areal, on a too-many-faces feature, or on an
/// unplaceable frame — every case falls back to the plain `point` graphic in
/// the caller, silently, matching the old writer's non-polygon fallback.
#[cfg(feature = "new-geometry")]
fn polygon_value_for(
    feature: &Feature,
    entity_id: &str,
    cache: &mut ReprojectionCache,
) -> Option<Value> {
    let faces = faces_of(&feature.geometry).ok()?;
    let face = faces.first()?;
    let packet = face_packet(cache, face, entity_id).ok()?;
    serde_json::to_value(packet.polygon?).ok()
}

/// Build a single CZML packet for a time-dynamic entity from grouped
/// features — the new-geometry counterpart of `build_entity_packet`. Same
/// shape, including the polygon-vs-point graphic decision: position samples
/// come from `cartographic_position` instead of `extract_point_coords`, and
/// the polygon-or-point decision comes from `styled_polygon_value_for` (first
/// face of the group's first feature) instead of
/// `feature_geometry_to_polygon_json` (old-geometry-specific), but the
/// resulting shape is the same — a polygon feature still renders as a
/// footprint, with `position` removed, not a dot, and carrying the same
/// `polygonStyle` the old world applied here.
#[cfg(feature = "new-geometry")]
fn build_entity_packet_next(
    gctx: &Context,
    entity_id: &str,
    features: &[&Feature],
    params: &CzmlWriterCompiledParam,
    cache: &mut ReprojectionCache,
) -> Result<Value, BoxedError> {
    let time_field = params.time_field.as_ref().unwrap();

    // One pass to read every time value, so the three questions below —
    // "is this entity time-dynamic", "are its times numeric", "in what order"
    // — are all answered from the same population.
    //
    // That population is the features that actually carry a time value.
    // Deciding `all_numeric` over *every* feature in the group, as this did,
    // let a single untimed feature make the answer `false` while the document
    // clock (which skips untimed features) still said `true`. The clock then
    // wrote an ISO interval against the Unix epoch while the entity wrote its
    // times as raw JSON strings, which CZML does not accept: one document,
    // two readings of the same times, which is the gap `effective_epoch` was
    // extracted to close.
    let timed: Vec<(&Feature, String)> = features
        .iter()
        .map(|f| {
            let ts = f
                .get(time_field)
                .map(attribute_value_to_string)
                .unwrap_or_default();
            (*f, ts)
        })
        .collect();

    let all_numeric_times = {
        let mut any = false;
        let mut all = true;
        for (_, ts) in &timed {
            if ts.is_empty() {
                continue;
            }
            any = true;
            if !ts.parse::<f64>().is_ok_and(f64::is_finite) {
                all = false;
                break;
            }
        }
        any && all
    };

    // Decided before anything is emitted, because it governs the *shape* of
    // `cartographicDegrees`: a time-dynamic list is read in groups of four
    // (time, lon, lat, height), a static one in groups of three.
    let is_time_dynamic = timed.iter().any(|(_, ts)| !ts.is_empty());

    let epoch = effective_epoch(params.epoch.as_deref(), all_numeric_times);

    // Resolved once, here, so the order below and the availability bounds
    // further down are read off the same instant. Sorting the strings put
    // `2024-01-01T02:00:00Z` before `3600`, which against a midnight epoch is
    // 01:00 — the later sample first, and an availability window that ran
    // backwards and covered none of the track.
    let mut sorted: Vec<(&Feature, String, Option<chrono::DateTime<chrono::Utc>>)> = timed
        .into_iter()
        .map(|(f, ts)| {
            let instant = if ts.is_empty() {
                None
            } else {
                resolve_time_instant(&ts, epoch.as_deref())
            };
            (f, ts, instant)
        })
        .collect();
    sorted.sort_by(|(_, ta, ia), (_, tb, ib)| {
        // An untimed feature has no place on the timeline, so it sorts first
        // and is handled separately below. A value that names no instant sorts
        // last: it is reported and dropped, and sorting it to the front would
        // let it pick the entity's first sample on its way out.
        match (ta.is_empty(), tb.is_empty()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => match (ia, ib) {
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
        }
    });

    let mut cartographic_degrees: Vec<Value> = Vec::new();
    let mut first_instant: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_instant: Option<chrono::DateTime<chrono::Utc>> = None;
    // The feature behind the first sample actually written. Carrying a time
    // value is not enough to be the entity's representative: a timed feature
    // can still fail placement and contribute nothing.
    let mut first_sampled: Option<&Feature> = None;

    for (feature, time_str, instant) in &sorted {
        let time_str = time_str.as_str();
        // Reported once per feature, here, rather than at each of the two uses
        // below: a feature contributes at most one sample, so reporting at the
        // use sites would say nothing extra and would miss the feature whose
        // time value is unparseable *and* whose position is unplaceable.
        let placed = match cartographic_position(cache, feature) {
            Ok(pos) => Some(pos),
            Err(err) => {
                report_unplaced(gctx, feature, err);
                None
            }
        };

        if time_str.is_empty() {
            if is_time_dynamic {
                // Its siblings put this entity on a timeline, and CZML has no
                // way to say "this sample, at no time". Writing the position
                // anyway pushed a bare triple into a list Cesium reads in
                // fours, which knocked every later sample out of alignment and
                // sent the entity to garbage coordinates at garbage times.
                // Dropping the sample is the only valid output; saying so is
                // what keeps it from being a silent one.
                report_untimed_sample(gctx, feature, &format!("time-dynamic entity {entity_id}"));
                continue;
            }
            // Nothing in this group carries a time, so the entity is static
            // and its `cartographicDegrees` is a single [lon, lat, height].
            // Cesium chooses between the static and time-tagged readings on
            // length alone (`> 3` means time-tagged), so a second triple does
            // not add a second position: it reinterprets the whole list as
            // malformed quads, and the first longitude becomes a time. One
            // position is all a static entity has, so the first placeable
            // feature supplies it and the rest are dropped — reported, for
            // the same reason the time-dynamic branch reports.
            if let Some((lon, lat, height)) = placed {
                if first_sampled.is_none() {
                    first_sampled = Some(feature);
                    cartographic_degrees.push(serde_json::json!(lon));
                    cartographic_degrees.push(serde_json::json!(lat));
                    cartographic_degrees.push(serde_json::json!(height));
                } else {
                    report_untimed_sample(
                        gctx,
                        feature,
                        &format!("static entity {entity_id}, which holds one position"),
                    );
                }
            }
            continue;
        }

        // Checked before anything is emitted, because an unresolvable value
        // must not reach the samples, the availability window or the document
        // clock. `1e30` is finite, so the old `is_finite` check passed it
        // straight through to a bound that is not valid CZML.
        let Some(instant) = *instant else {
            report_unparsable_time(gctx, feature, time_str);
            continue;
        };

        let time_value: Value = if all_numeric_times {
            if let Ok(n) = time_str.parse::<f64>() {
                serde_json::json!(n)
            } else if let Some(offset) = parse_epoch_offset_timestamp(time_str) {
                serde_json::json!(offset)
            } else {
                report_unparsable_time(gctx, feature, time_str);
                continue;
            }
        } else {
            // The value used to be written unchanged here, so a `timeField`
            // holding `"n/a"` — or a group mixing `"60"` with a timestamp,
            // which also lands in this branch — reached the file as a time.
            // `JulianDate.fromIso8601` throws on it, so the whole document
            // failed to load in the viewer and nothing was reported.
            match writable_time_value(time_str, epoch.as_deref()) {
                Some(value) => value,
                None => {
                    report_unparsable_time(gctx, feature, time_str);
                    continue;
                }
            }
        };

        // Only now: a value that cannot be written must not reach the entity's
        // availability window or the document clock either, or the file is
        // unloadable for the sake of a sample that was never written.
        if first_instant.is_none() {
            first_instant = Some(instant);
        }
        last_instant = Some(instant);

        if let Some((lon, lat, height)) = placed {
            if first_sampled.is_none() {
                first_sampled = Some(feature);
            }
            cartographic_degrees.push(time_value);
            cartographic_degrees.push(serde_json::json!(lon));
            cartographic_degrees.push(serde_json::json!(lat));
            cartographic_degrees.push(serde_json::json!(height));
        }
    }

    let mut position = if is_time_dynamic {
        serde_json::json!({
            "cartographicDegrees": cartographic_degrees,
            "interpolationAlgorithm": params.interpolation_algorithm.to_string(),
            "interpolationDegree": params.interpolation_degree,
        })
    } else {
        serde_json::json!({
            "cartographicDegrees": cartographic_degrees,
        })
    };
    if is_time_dynamic && let Some(epoch) = &epoch {
        position["epoch"] = serde_json::json!(epoch);
    }

    let availability = match (first_instant, last_instant) {
        (Some(start), Some(end)) if start != end => Some(format!(
            "{}/{}",
            instant_to_czml(start),
            instant_to_czml(end)
        )),
        _ => None,
    };

    // `sorted` now carries each feature with the time string it was sorted by,
    // so the metadata below reads through the tuple. It is still the group's
    // earliest feature that names and describes the entity — but on a
    // time-dynamic entity it has to be the earliest feature that *contributed
    // a sample*. Untimed features sort to the front and are dropped by the
    // loop above, so taking `sorted.first()` blindly let a feature with no
    // place on the timeline choose the graphic: a dropped polygon would emit
    // `polygon` and remove `position`, throwing away every timed sample the
    // loop had just collected.
    let first_feature = if is_time_dynamic {
        // The feature that produced the first sample, not merely the first one
        // with a time value. `cartographic_position` places a feature from its
        // *first vertex*, while `styled_polygon_value_for` works face by face
        // in each face's own frame, so a feature can fail placement and still
        // yield a polygon — a collection whose first leaf is in an unplaceable
        // frame does exactly that. Letting it represent the entity emitted a
        // polygon and removed `position`, discarding the samples its siblings
        // had contributed. Falling back only matters when nothing was sampled
        // at all, where there is no position left to protect.
        first_sampled.or_else(|| {
            sorted
                .iter()
                .find(|(_, ts, _)| !ts.is_empty())
                .map(|(f, _, _)| *f)
        })
    } else {
        // Same rule on the static path: the feature whose position was
        // actually written is the one the entity stands for, so its graphic
        // and description are the ones that belong to that position. Falling
        // back to the first in the group only matters when nothing was
        // placeable, where there is no position to be consistent with.
        first_sampled.or_else(|| sorted.first().map(|(f, _, _)| *f))
    };

    let name = first_feature
        .and_then(|f| f.get(Attribute::new("name")))
        .map(attribute_value_to_string);

    let description = first_feature.map(|f| {
        let filtered = filter_description_attributes(&f.attributes);
        map_to_html_table(&filtered)
    });

    let mut packet = serde_json::json!({
        "id": entity_id,
        "position": position,
    });

    if let Some(avail) = availability {
        packet["availability"] = serde_json::json!(avail);
    }
    if let Some(n) = name {
        packet["name"] = serde_json::json!(n);
    }
    if let Some(d) = description {
        packet["description"] = serde_json::json!(d);
    }

    // Add graphic based on geometry type: polygon for areal features, point
    // for everything else — same decision `build_entity_packet` makes via
    // `feature_geometry_to_polygon_json`, styling included. The port originally
    // called the unstyled `polygon_value_for` here, which silently dropped
    // `colorAttribute`, `opacity` and `heightAttribute` from every grouped
    // timeseries entity; the old world applied them on this path.
    match first_feature.and_then(|f| styled_polygon_value_for(f, params, entity_id, cache)) {
        Some(polygon_val) => {
            packet["polygon"] = polygon_val;
            // A CZML polygon carries its own coordinates, so the entity's
            // `position` does not move it: leaving both would promise an
            // animation the viewer cannot draw. The samples collected above
            // are therefore dropped — and a group of N timestamped footprints
            // comes out as the first one, static. That is a real loss of
            // everything but one feature, so it is reported rather than left
            // for the user to notice in Cesium. Animating areal features
            // would mean one entity per feature, which is a different output,
            // not an audit fix.
            if let Some(obj) = packet.as_object_mut() {
                obj.remove("position");
            }
            if is_time_dynamic && let Some(f) = first_feature {
                let ectx = gctx.as_executor_context(f.clone(), FEATURES_PORT.clone());
                let _ = ectx.report(
                    DiagnosticDraft::new(ErrorCode::CzmlArealSamplesDropped).with_message(format!(
                        "wrote entity {entity_id} as one static footprint and dropped \
                             {} time sample(s)",
                        cartographic_degrees.len() / 4
                    )),
                );
            }
        }
        None => {
            packet["point"] = serde_json::json!({
                "pixelSize": 10,
                "heightReference": "NONE",
            });
        }
    }

    Ok(packet)
}

/// True when this batch should be written in Embedded Mode — the round-trip
/// path documented in `docs/czml-timeseries.md` as the recommended one.
///
/// Two independent triggers, matching the old world:
///
/// * any feature carries a `czml.*` attribute, i.e. it came through the CZML
///   Reader's `preserveRaw` strategy and is carrying a whole CZML packet
///   (`czml.timeseries`, `czml.epoch`, `czml.point`, `czml.label`,
///   `czml.path`, interpolation metadata, …) that only this mode writes back;
/// * `per_entity_mode` — `timeField` set *without* `groupTimeseriesBy`. This
///   is what makes `timeField` alone meaningful: it computes a per-entity
///   `availability` window plus the document `clock`. Without this branch,
///   `timeField` would be a no-op unless `groupTimeseriesBy` were set too.
///
/// Old-world `finish` reached Embedded Mode as the *default* (`else`) branch
/// after `has_citygml`; the new model has no `CityGmlGeometry` to dispatch
/// on, so the condition is stated positively here and the geometry-only
/// per-feature path becomes the default instead.
#[cfg(feature = "new-geometry")]
fn is_embedded_mode(features: &[Feature], params: &CzmlWriterCompiledParam) -> bool {
    let per_entity_mode = params.time_field.is_some() && params.group_timeseries_by.is_none();
    per_entity_mode
        || features.iter().any(|f| {
            f.attributes
                .keys()
                .any(|k| k.to_string().starts_with("czml."))
        })
}

/// A feature's first face as a *styled* CZML polygon JSON value — the
/// new-geometry counterpart of `feature_geometry_to_polygon_json`.
///
/// The geometry half is delegated wholesale to [`polygon_value_for`], so this
/// adds no new reprojection site and no new lat/lon swap: the coordinates
/// come out of the same `faces_of` → `face_packet` path every other
/// new-geometry writer branch uses. Only the `colorAttribute` / `opacity` /
/// `heightAttribute` styling and the default outline are applied here, byte
/// for byte as the old-world function applied them.
#[cfg(feature = "new-geometry")]
fn styled_polygon_value_for(
    feature: &Feature,
    params: &CzmlWriterCompiledParam,
    entity_id: &str,
    cache: &mut ReprojectionCache,
) -> Option<Value> {
    let mut polygon_val = polygon_value_for(feature, entity_id, cache)?;
    let obj = polygon_val.as_object_mut()?;

    // Apply color from attribute
    if let Some(color_attr) = &params.color_attribute
        && let Some(AttributeValue::String(hex)) = feature.get(color_attr)
        && let Some(rgba) = hex_to_rgba(hex, params.opacity)
    {
        obj.insert(
            "material".to_string(),
            serde_json::json!({
                "solidColor": {
                    "color": {
                        "rgba": rgba
                    }
                }
            }),
        );
    }

    // Apply extrusion height from attribute
    if let Some(height_attr) = &params.height_attribute
        && let Some(height_val) = feature.get(height_attr)
        && let Some(h) = height_val.as_f64()
    {
        obj.insert("extrudedHeight".to_string(), serde_json::json!(h));
        obj.insert("closeBottom".to_string(), serde_json::json!(true));
        obj.insert("closeTop".to_string(), serde_json::json!(true));
    }

    // Default outline
    obj.insert("fill".to_string(), serde_json::json!(true));
    obj.insert("outline".to_string(), serde_json::json!(true));
    obj.insert(
        "outlineColor".to_string(),
        serde_json::json!({ "rgba": [80, 80, 80, 220] }),
    );
    obj.insert("outlineWidth".to_string(), serde_json::json!(1.0));

    Some(polygon_val)
}

/// Build a CZML document from features with embedded `czml.*` attributes
/// (produced by the reader's `PreserveRaw` strategy) — the new-geometry
/// counterpart of [`build_embedded_czml`]. Identical attribute handling: the
/// only thing geometry contributes to this mode is the static entity's
/// position and the auto-converted polygon, so the port is confined to
/// `build_embedded_packet_next`.
#[cfg(feature = "new-geometry")]
fn build_embedded_czml_next(
    gctx: &Context,
    features: &[Feature],
    params: &CzmlWriterCompiledParam,
    cache: &mut ReprojectionCache,
) -> Result<Vec<u8>, BoxedError> {
    let per_entity_mode = params.time_field.is_some() && params.group_timeseries_by.is_none();

    // Auto-detect epoch for numeric time values in per-entity mode
    let effective_epoch: Option<String> = if per_entity_mode {
        let all_numeric = params.time_field.as_ref().is_some_and(|tf| {
            features.iter().all(|f| {
                f.get(tf)
                    .map(|v| {
                        let s = attribute_value_to_string(v);
                        s.parse::<f64>().is_ok()
                    })
                    .unwrap_or(true)
            })
        });
        effective_epoch(params.epoch.as_deref(), all_numeric)
    } else {
        params.epoch.clone()
    };

    let mut global_start: Option<String> = None;
    let mut global_end: Option<String> = None;

    // Pass 1: Collect time range
    if per_entity_mode && let Some(time_field) = &params.time_field {
        for feature in features {
            if let Some(time_val) = feature.get(time_field) {
                let time_str = attribute_value_to_string(time_val);
                let start_iso =
                    strip_epoch_offset_for_availability(&time_str, effective_epoch.as_deref());
                if global_start.is_none() || start_iso < *global_start.as_ref().unwrap() {
                    global_start = Some(start_iso.clone());
                }
                if global_end.is_none() || start_iso > *global_end.as_ref().unwrap() {
                    global_end = Some(start_iso);
                }
            }
        }
    }
    if !per_entity_mode {
        for feature in features {
            if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability"))
                && let Some((s, e)) = avail.split_once('/')
                && !s.is_empty()
                && !e.is_empty()
            {
                if global_start.is_none() || s < global_start.as_deref().unwrap_or("") {
                    global_start = Some(s.to_string());
                }
                if global_end.is_none() || e > global_end.as_deref().unwrap_or("") {
                    global_end = Some(e.to_string());
                }
            }
        }
    }

    let mut doc = serde_json::json!({
        "id": "document",
        "version": "1.0",
    });
    if let (Some(start), Some(end)) = (&global_start, &global_end) {
        let availability = format!("{start}/{end}");
        doc["clock"] = serde_json::json!({
            "interval": availability,
            "currentTime": start,
            "multiplier": 1,
            "range": "LOOP_STOP",
            "step": "SYSTEM_CLOCK_MULTIPLIER",
        });
    }

    let mut output_buffer = Vec::new();
    output_buffer
        .write(format!("[{}", serde_json::to_string(&doc).unwrap()).as_bytes())
        .map_err(SinkError::czml_writer)?;

    // Pass 2: Build packets
    for (idx, feature) in features.iter().enumerate() {
        let mut packet = build_embedded_packet_next(
            gctx,
            feature,
            params,
            global_end.as_deref(),
            effective_epoch.as_deref(),
            cache,
        )?;
        // Ensure every packet has an id (required by Cesium)
        if let Some(obj) = packet.as_object_mut()
            && !obj.contains_key("id")
        {
            obj.insert("id".to_string(), serde_json::json!(format!("entity_{idx}")));
        }
        output_buffer.write(b",").map_err(SinkError::czml_writer)?;
        output_buffer
            .write(&serde_json::to_vec(&packet).map_err(SinkError::czml_writer)?)
            .map_err(SinkError::czml_writer)?;
    }

    output_buffer
        .write(b"]\n")
        .map_err(SinkError::czml_writer)?;
    Ok(output_buffer)
}

/// Build a single CZML packet from a feature with embedded `czml.*`
/// attributes — the new-geometry counterpart of [`build_embedded_packet`].
///
/// Every `czml.*` attribute, `availability`, `parent`, `description` and the
/// `properties` bag are handled exactly as the old-world function handles
/// them; the two geometry-touching lines are the only difference:
/// `extract_point_coords` becomes `cartographic_position` (which reprojects
/// and swaps once, via the shared `to_wgs84` path), and
/// `feature_geometry_to_polygon_json` becomes `styled_polygon_value_for`.
#[cfg(feature = "new-geometry")]
fn build_embedded_packet_next(
    gctx: &Context,
    feature: &Feature,
    params: &CzmlWriterCompiledParam,
    global_end: Option<&str>,
    effective_epoch: Option<&str>,
    cache: &mut ReprojectionCache,
) -> Result<Value, BoxedError> {
    let mut packet = serde_json::Map::new();
    // Set when geometry could not place the entity, and resolved at the end of
    // this function once the `czml.*` graphics have had their chance.
    let mut unplaced: Option<PlacementError> = None;

    if let Some(AttributeValue::String(id)) = feature.get(Attribute::new("id")) {
        packet.insert("id".to_string(), serde_json::json!(id));
    } else if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        // Use name as fallback id
        packet.insert("id".to_string(), serde_json::json!(name));
    }
    if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        packet.insert("name".to_string(), serde_json::json!(name));
    }

    // Per-entity availability from time_field + global_end
    if let (Some(time_field), Some(end)) = (&params.time_field, global_end) {
        if let Some(time_val) = feature.get(time_field) {
            let time_str = attribute_value_to_string(time_val);
            let start_iso = strip_epoch_offset_for_availability(&time_str, effective_epoch);
            packet.insert(
                "availability".to_string(),
                serde_json::json!(format!("{start_iso}/{end}")),
            );
        }
    } else if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability"))
        && !avail.is_empty()
        && avail != "/"
    {
        packet.insert("availability".to_string(), serde_json::json!(avail));
    }

    if let Some(AttributeValue::String(parent)) = feature.get(Attribute::new("parent")) {
        packet.insert("parent".to_string(), serde_json::json!(parent));
    }

    let epoch = feature
        .get(Attribute::new("czml.epoch"))
        .and_then(|v| match v {
            AttributeValue::String(s) => Some(s.clone()),
            _ => None,
        });
    let interp_alg = feature
        .get(Attribute::new("czml.interpolationAlgorithm"))
        .and_then(|v| match v {
            AttributeValue::String(s) => Some(s.clone()),
            _ => None,
        });
    let interp_deg = feature
        .get(Attribute::new("czml.interpolationDegree"))
        .and_then(|v| match v {
            AttributeValue::Number(n) => n.as_f64(),
            _ => None,
        });

    if let Some(AttributeValue::String(ts_json)) = feature.get(Attribute::new("czml.timeseries")) {
        if let Ok(samples) = serde_json::from_str::<Vec<Value>>(ts_json) {
            let mut cartographic_degrees: Vec<Value> = Vec::new();

            for sample in &samples {
                let time_str = sample["time"].as_str().unwrap_or("");
                // These samples are already WGS84 lon/lat/height as the CZML
                // Reader parsed them out of `cartographicDegrees`; they are
                // not geometry and must not be reprojected or swapped.
                let lon = sample["lon"].as_f64().unwrap_or(0.0);
                let lat = sample["lat"].as_f64().unwrap_or(0.0);
                let height = sample["height"].as_f64().unwrap_or(0.0);

                let time_value: Value = if let Some(offset) = parse_epoch_offset_timestamp(time_str)
                {
                    serde_json::json!(offset)
                } else if let Ok(n) = time_str.parse::<f64>() {
                    serde_json::json!(n)
                } else if let Some(offset) = sample["timeOffset"].as_f64() {
                    if offset == 0.0 && !time_str.is_empty() && time_str.contains('T') {
                        serde_json::json!(time_str)
                    } else {
                        serde_json::json!(offset)
                    }
                } else {
                    serde_json::json!(time_str)
                };

                cartographic_degrees.push(time_value);
                cartographic_degrees.push(serde_json::json!(lon));
                cartographic_degrees.push(serde_json::json!(lat));
                cartographic_degrees.push(serde_json::json!(height));
            }

            let mut position = serde_json::json!({
                "cartographicDegrees": cartographic_degrees,
            });
            if let Some(ep) = &epoch {
                position["epoch"] = serde_json::json!(ep);
            }
            if let Some(alg) = &interp_alg {
                position["interpolationAlgorithm"] = serde_json::json!(alg);
            }
            if let Some(deg) = interp_deg {
                position["interpolationDegree"] = serde_json::json!(deg as u32);
            }

            packet.insert("position".to_string(), position);
        }
    } else {
        // Static entity: position from geometry. `cartographic_position`
        // already returns `(lon, lat, height)` in CZML order.
        match cartographic_position(cache, feature) {
            Ok((lon, lat, height)) => {
                packet.insert(
                    "position".to_string(),
                    serde_json::json!({
                        "cartographicDegrees": [lon, lat, height],
                    }),
                );
            }
            // Held, not reported yet. A feature read with `preserveRaw` can
            // carry `Geometry::None` and a perfectly good `czml.polygon`, whose
            // own `positions` place the graphic — the restoration loop below is
            // what puts it back. Reporting here would call that a placement
            // failure while the packet goes out fully drawn, which is the kind
            // of false positive that teaches users to ignore the diagnostic.
            Err(err) => unplaced = Some(err),
        }
    }

    let skip_czml_keys = [
        "czml.timeseries",
        "czml.epoch",
        "czml.interpolationAlgorithm",
        "czml.interpolationDegree",
    ];
    for (attr, value) in feature.attributes.iter() {
        let key = attr.to_string();
        if let Some(czml_key) = key.strip_prefix("czml.") {
            if skip_czml_keys.contains(&key.as_str()) {
                continue;
            }
            if let AttributeValue::String(json_str) = value
                && let Ok(parsed) = serde_json::from_str::<Value>(json_str.as_str())
            {
                packet.insert(czml_key.to_string(), parsed);
            }
        }
    }

    // Auto-convert polygon geometry when no graphic property was set via czml.* attributes
    if !packet.contains_key("point")
        && !packet.contains_key("billboard")
        && !packet.contains_key("model")
        && !packet.contains_key("label")
        && !packet.contains_key("polygon")
        && !packet.contains_key("polyline")
    {
        let entity_id = packet
            .get("id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| feature.id.to_string());
        if let Some(polygon_val) = styled_polygon_value_for(feature, params, &entity_id, cache) {
            packet.insert("polygon".to_string(), polygon_val);
            // Remove position for polygon entities (positions are in polygon.positions)
            packet.remove("position");
        }
        // No fallback point — if the geometry cannot be converted, rely on the
        // position that was already set (if any) or let the entity be data-only.
    }

    if !packet.contains_key("description") {
        if let Some(AttributeValue::String(desc)) = feature.get(Attribute::new("description")) {
            packet.insert("description".to_string(), serde_json::json!(desc));
        } else {
            // Auto-generate description from non-internal attributes
            let filtered = filter_description_attributes(&feature.attributes);
            let desc = map_to_html_table(&filtered);
            if !desc.is_empty() && desc != "<table></table>" {
                packet.insert("description".to_string(), serde_json::json!(desc));
            }
        }
    }

    // Add properties bag
    if !packet.contains_key("properties")
        && let Some(props) = build_properties_bag(feature)
    {
        packet.insert("properties".to_string(), props);
    }

    // Now that every source of a position or a graphic has had its turn —
    // `czml.*` restoration and the geometry auto-convert — a held placement
    // error is a real one only if the entity ended up with neither.
    //
    // "A graphic" has to mean one that places itself, which is a narrower set
    // than "a graphic this writer recognises". Only the polyline-family
    // graphics and `rectangle` carry their own coordinates; `point`,
    // `billboard`, `label`, `model` and the rest are drawn at the containing
    // entity's `position`. A packet holding one of those and no `position` is
    // not drawn at all, so suppressing the error there would restore exactly
    // the silence this diagnostic was added to end.
    let self_placed = SELF_PLACING_GRAPHICS.iter().any(|(graphic, placed_by)| {
        packet
            .get(*graphic)
            .and_then(|g| g.get(*placed_by))
            .is_some_and(|p| !p.is_null())
    });
    if let Some(err) = unplaced
        && !packet.contains_key("position")
        && !self_placed
    {
        report_unplaced(gctx, feature, err);
    }

    Ok(Value::Object(packet))
}

/// The CZML graphic properties that place themselves, each paired with the
/// property that does the placing.
///
/// Every other graphic takes its placement from the containing entity's
/// `position` — a `point`, `billboard`, `label` or `model` on a packet with no
/// `position` has nowhere to be drawn. So the presence of one of those says
/// nothing about whether the entity was placed, and only the pairs below can
/// stand in for a `position`.
#[cfg(feature = "new-geometry")]
const SELF_PLACING_GRAPHICS: [(&str, &str); 6] = [
    ("polygon", "positions"),
    ("polyline", "positions"),
    ("polylineVolume", "positions"),
    ("corridor", "positions"),
    ("wall", "positions"),
    ("rectangle", "coordinates"),
];

/// Build a CZML document from features with embedded `czml.*` attributes
/// (produced by the reader's `PreserveRaw` strategy).
#[cfg(not(feature = "new-geometry"))]
fn build_embedded_czml(
    features: &[Feature],
    params: &CzmlWriterCompiledParam,
) -> Result<Vec<u8>, BoxedError> {
    let per_entity_mode = params.time_field.is_some() && params.group_timeseries_by.is_none();

    // Auto-detect epoch for numeric time values in per-entity mode
    let effective_epoch: Option<String> = if per_entity_mode {
        if params.epoch.is_some() {
            params.epoch.clone()
        } else {
            let all_numeric = params.time_field.as_ref().is_some_and(|tf| {
                features.iter().all(|f| {
                    f.get(tf)
                        .map(|v| {
                            let s = attribute_value_to_string(v);
                            s.parse::<f64>().is_ok()
                        })
                        .unwrap_or(true)
                })
            });
            if all_numeric {
                Some("1970-01-01T00:00:00Z".to_string())
            } else {
                None
            }
        }
    } else {
        params.epoch.clone()
    };

    let mut global_start: Option<String> = None;
    let mut global_end: Option<String> = None;

    // Pass 1: Collect time range
    if per_entity_mode {
        if let Some(time_field) = &params.time_field {
            for feature in features {
                if let Some(time_val) = feature.get(time_field) {
                    let time_str = attribute_value_to_string(time_val);
                    let start_iso =
                        strip_epoch_offset_for_availability(&time_str, effective_epoch.as_deref());
                    if global_start.is_none() || start_iso < *global_start.as_ref().unwrap() {
                        global_start = Some(start_iso.clone());
                    }
                    if global_end.is_none() || start_iso > *global_end.as_ref().unwrap() {
                        global_end = Some(start_iso);
                    }
                }
            }
        }
    }
    if !per_entity_mode {
        for feature in features {
            if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability"))
            {
                if let Some((s, e)) = avail.split_once('/') {
                    if !s.is_empty() && !e.is_empty() {
                        if global_start.is_none() || s < global_start.as_deref().unwrap_or("") {
                            global_start = Some(s.to_string());
                        }
                        if global_end.is_none() || e > global_end.as_deref().unwrap_or("") {
                            global_end = Some(e.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut doc = serde_json::json!({
        "id": "document",
        "version": "1.0",
    });
    if let (Some(start), Some(end)) = (&global_start, &global_end) {
        let availability = format!("{start}/{end}");
        doc["clock"] = serde_json::json!({
            "interval": availability,
            "currentTime": start,
            "multiplier": 1,
            "range": "LOOP_STOP",
            "step": "SYSTEM_CLOCK_MULTIPLIER",
        });
    }

    let mut output_buffer = Vec::new();
    output_buffer
        .write(format!("[{}", serde_json::to_string(&doc).unwrap()).as_bytes())
        .map_err(SinkError::czml_writer)?;

    // Pass 2: Build packets
    for (idx, feature) in features.iter().enumerate() {
        let mut packet = build_embedded_packet(
            feature,
            params,
            global_end.as_deref(),
            effective_epoch.as_deref(),
        )?;
        // Ensure every packet has an id (required by Cesium)
        if let Some(obj) = packet.as_object_mut() {
            if !obj.contains_key("id") {
                obj.insert("id".to_string(), serde_json::json!(format!("entity_{idx}")));
            }
        }
        output_buffer.write(b",").map_err(SinkError::czml_writer)?;
        output_buffer
            .write(&serde_json::to_vec(&packet).map_err(SinkError::czml_writer)?)
            .map_err(SinkError::czml_writer)?;
    }

    output_buffer
        .write(b"]\n")
        .map_err(SinkError::czml_writer)?;
    Ok(output_buffer)
}

/// Build a single CZML packet from a feature with embedded `czml.*` attributes.
/// When `params` provides `time_field` and `global_end` is set, per-entity availability
/// is computed. Polygon geometry is auto-converted when no graphic property exists.
/// `effective_epoch` is the epoch to use for numeric time conversion (may be auto-detected).
#[cfg(not(feature = "new-geometry"))]
fn build_embedded_packet(
    feature: &Feature,
    params: &CzmlWriterCompiledParam,
    global_end: Option<&str>,
    effective_epoch: Option<&str>,
) -> Result<Value, BoxedError> {
    let mut packet = serde_json::Map::new();

    if let Some(AttributeValue::String(id)) = feature.get(Attribute::new("id")) {
        packet.insert("id".to_string(), serde_json::json!(id));
    } else if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        // Use name as fallback id
        packet.insert("id".to_string(), serde_json::json!(name));
    }
    if let Some(AttributeValue::String(name)) = feature.get(Attribute::new("name")) {
        packet.insert("name".to_string(), serde_json::json!(name));
    }

    // Per-entity availability from time_field + global_end
    if let (Some(time_field), Some(end)) = (&params.time_field, global_end) {
        if let Some(time_val) = feature.get(time_field) {
            let time_str = attribute_value_to_string(time_val);
            let start_iso = strip_epoch_offset_for_availability(&time_str, effective_epoch);
            packet.insert(
                "availability".to_string(),
                serde_json::json!(format!("{start_iso}/{end}")),
            );
        }
    } else if let Some(AttributeValue::String(avail)) = feature.get(Attribute::new("availability"))
    {
        if !avail.is_empty() && avail != "/" {
            packet.insert("availability".to_string(), serde_json::json!(avail));
        }
    }

    if let Some(AttributeValue::String(parent)) = feature.get(Attribute::new("parent")) {
        packet.insert("parent".to_string(), serde_json::json!(parent));
    }

    let epoch = feature
        .get(Attribute::new("czml.epoch"))
        .and_then(|v| match v {
            AttributeValue::String(s) => Some(s.clone()),
            _ => None,
        });
    let interp_alg = feature
        .get(Attribute::new("czml.interpolationAlgorithm"))
        .and_then(|v| match v {
            AttributeValue::String(s) => Some(s.clone()),
            _ => None,
        });
    let interp_deg = feature
        .get(Attribute::new("czml.interpolationDegree"))
        .and_then(|v| match v {
            AttributeValue::Number(n) => n.as_f64(),
            _ => None,
        });

    if let Some(AttributeValue::String(ts_json)) = feature.get(Attribute::new("czml.timeseries")) {
        if let Ok(samples) = serde_json::from_str::<Vec<Value>>(ts_json) {
            let mut cartographic_degrees: Vec<Value> = Vec::new();

            for sample in &samples {
                let time_str = sample["time"].as_str().unwrap_or("");
                let lon = sample["lon"].as_f64().unwrap_or(0.0);
                let lat = sample["lat"].as_f64().unwrap_or(0.0);
                let height = sample["height"].as_f64().unwrap_or(0.0);

                let time_value: Value = if let Some(offset) = parse_epoch_offset_timestamp(time_str)
                {
                    serde_json::json!(offset)
                } else if let Ok(n) = time_str.parse::<f64>() {
                    serde_json::json!(n)
                } else if let Some(offset) = sample["timeOffset"].as_f64() {
                    if offset == 0.0 && !time_str.is_empty() && time_str.contains('T') {
                        serde_json::json!(time_str)
                    } else {
                        serde_json::json!(offset)
                    }
                } else {
                    serde_json::json!(time_str)
                };

                cartographic_degrees.push(time_value);
                cartographic_degrees.push(serde_json::json!(lon));
                cartographic_degrees.push(serde_json::json!(lat));
                cartographic_degrees.push(serde_json::json!(height));
            }

            let mut position = serde_json::json!({
                "cartographicDegrees": cartographic_degrees,
            });
            if let Some(ep) = &epoch {
                position["epoch"] = serde_json::json!(ep);
            }
            if let Some(alg) = &interp_alg {
                position["interpolationAlgorithm"] = serde_json::json!(alg);
            }
            if let Some(deg) = interp_deg {
                position["interpolationDegree"] = serde_json::json!(deg as u32);
            }

            packet.insert("position".to_string(), position);
        }
    } else {
        // Static entity: position from geometry (only for point-like entities)
        if let Some((lon, lat, height)) = extract_point_coords(feature) {
            packet.insert(
                "position".to_string(),
                serde_json::json!({
                    "cartographicDegrees": [lon, lat, height],
                }),
            );
        }
    }

    let skip_czml_keys = [
        "czml.timeseries",
        "czml.epoch",
        "czml.interpolationAlgorithm",
        "czml.interpolationDegree",
    ];
    for (attr, value) in feature.attributes.iter() {
        let key = attr.to_string();
        if let Some(czml_key) = key.strip_prefix("czml.") {
            if skip_czml_keys.contains(&key.as_str()) {
                continue;
            }
            if let AttributeValue::String(json_str) = value {
                if let Ok(parsed) = serde_json::from_str::<Value>(json_str.as_str()) {
                    packet.insert(czml_key.to_string(), parsed);
                }
            }
        }
    }

    // Auto-convert polygon geometry when no graphic property was set via czml.* attributes
    if !packet.contains_key("point")
        && !packet.contains_key("billboard")
        && !packet.contains_key("model")
        && !packet.contains_key("label")
        && !packet.contains_key("polygon")
        && !packet.contains_key("polyline")
    {
        if let Some(polygon_val) = feature_geometry_to_polygon_json(feature, params) {
            packet.insert("polygon".to_string(), polygon_val);
            // Remove position for polygon entities (positions are in polygon.positions)
            packet.remove("position");
        }
        // No fallback point — if the geometry cannot be converted, rely on the
        // position that was already set (if any) or let the entity be data-only.
    }

    if !packet.contains_key("description") {
        if let Some(AttributeValue::String(desc)) = feature.get(Attribute::new("description")) {
            packet.insert("description".to_string(), serde_json::json!(desc));
        } else {
            // Auto-generate description from non-internal attributes
            let filtered = filter_description_attributes(&feature.attributes);
            let desc = map_to_html_table(&filtered);
            if !desc.is_empty() && desc != "<table></table>" {
                packet.insert("description".to_string(), serde_json::json!(desc));
            }
        }
    }

    // Add properties bag
    if !packet.contains_key("properties") {
        if let Some(props) = build_properties_bag(feature) {
            packet.insert("properties".to_string(), props);
        }
    }

    Ok(Value::Object(packet))
}

/// Build a CZML document with time-dynamic entities grouped by attribute.
#[cfg(not(feature = "new-geometry"))]
fn build_timeseries_czml(
    features: &[Feature],
    params: &CzmlWriterCompiledParam,
    _ctx: &NodeContext,
) -> Result<Vec<u8>, BoxedError> {
    let time_field = params
        .time_field
        .as_ref()
        .ok_or_else(|| SinkError::czml_writer("time_field is required for timeseries output"))?;
    let group_attr = params.group_timeseries_by.as_ref().ok_or_else(|| {
        SinkError::czml_writer("group_timeseries_by is required for timeseries output")
    })?;

    let mut groups: IndexMap<String, Vec<&Feature>> = IndexMap::new();
    for feature in features {
        let key = feature
            .get(group_attr)
            .map(attribute_value_to_string)
            .unwrap_or_else(|| "unknown".to_string());
        groups.entry(key).or_default().push(feature);
    }

    let mut doc = serde_json::json!({
        "id": "document",
        "version": "1.0",
    });

    let mut all_timestamps: Vec<String> = Vec::new();
    for feature in features {
        if let Some(ts) = feature.get(time_field) {
            all_timestamps.push(attribute_value_to_string(ts));
        }
    }
    if all_timestamps.len() >= 2 {
        // Check if all timestamps are numeric
        let all_numeric = all_timestamps.iter().all(|ts| {
            if let Ok(n) = ts.parse::<f64>() {
                n.is_finite()
            } else {
                false
            }
        });

        // Sort timestamps (numeric or string)
        if all_numeric {
            all_timestamps.sort_by(|a, b| {
                let na = a.parse::<f64>().unwrap();
                let nb = b.parse::<f64>().unwrap();
                na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            all_timestamps.sort();
        }

        let start = &all_timestamps[0];
        let end = &all_timestamps[all_timestamps.len() - 1];

        // Convert numeric times to ISO 8601 for clock if epoch is available
        let (start_iso, end_iso) = if all_numeric && params.epoch.is_some() {
            let start_iso = strip_epoch_offset_for_availability(start, params.epoch.as_deref());
            let end_iso = strip_epoch_offset_for_availability(end, params.epoch.as_deref());
            (start_iso, end_iso)
        } else {
            (start.clone(), end.clone())
        };

        let availability = format!("{start_iso}/{end_iso}");
        doc["clock"] = serde_json::json!({
            "interval": availability,
            "currentTime": start_iso,
            "multiplier": 1,
            "range": "LOOP_STOP",
            "step": "SYSTEM_CLOCK_MULTIPLIER",
        });
    }

    let mut output_buffer = Vec::new();
    output_buffer
        .write(format!("[{}", serde_json::to_string(&doc).unwrap()).as_bytes())
        .map_err(SinkError::czml_writer)?;

    for (entity_id, group_features) in &groups {
        let packet = build_entity_packet(entity_id, group_features, params)?;
        output_buffer.write(b",").map_err(SinkError::czml_writer)?;
        output_buffer
            .write(&serde_json::to_vec(&packet).map_err(SinkError::czml_writer)?)
            .map_err(SinkError::czml_writer)?;
    }

    output_buffer
        .write(b"]\n")
        .map_err(SinkError::czml_writer)?;
    Ok(output_buffer)
}

/// Build a single CZML packet for a time-dynamic entity from grouped features.
#[cfg(not(feature = "new-geometry"))]
fn build_entity_packet(
    entity_id: &str,
    features: &[&Feature],
    params: &CzmlWriterCompiledParam,
) -> Result<Value, BoxedError> {
    let time_field = params.time_field.as_ref().unwrap();

    // Check if all time values are numeric (finite numbers) and present
    let all_numeric_times = features.iter().all(|f| {
        if let Some(time_val) = f.get(time_field) {
            let time_str = attribute_value_to_string(time_val);
            if time_str.is_empty() {
                return false; // Empty times disqualify numeric mode
            }
            if let Ok(n) = time_str.parse::<f64>() {
                n.is_finite() // Reject NaN/Inf
            } else {
                false
            }
        } else {
            false // Missing time values disqualify numeric mode
        }
    });

    // Auto-generate epoch for numeric time values if not provided
    let epoch = if params.epoch.is_some() {
        params.epoch.clone()
    } else if all_numeric_times {
        // Use a default epoch for numeric time values
        // Using a fixed epoch makes the CZML time values consistent
        Some("1970-01-01T00:00:00Z".to_string())
    } else {
        None
    };

    let mut sorted: Vec<&Feature> = features.to_vec();
    sorted.sort_by(|a, b| {
        let ta = a
            .get(time_field)
            .map(attribute_value_to_string)
            .unwrap_or_default();
        let tb = b
            .get(time_field)
            .map(attribute_value_to_string)
            .unwrap_or_default();

        // For numeric comparison, parse and compare as numbers
        if all_numeric_times {
            // Safe to unwrap since all_numeric_times guarantees valid finite numbers
            let na = ta.parse::<f64>().unwrap();
            let nb = tb.parse::<f64>().unwrap();
            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            ta.cmp(&tb)
        }
    });

    let mut cartographic_degrees: Vec<Value> = Vec::new();
    let mut first_time: Option<String> = None;
    let mut last_time: Option<String> = None;

    for feature in &sorted {
        let time_str = feature
            .get(time_field)
            .map(attribute_value_to_string)
            .unwrap_or_default();

        if time_str.is_empty() {
            if let Some((lon, lat, height)) = extract_point_coords(feature) {
                cartographic_degrees.push(serde_json::json!(lon));
                cartographic_degrees.push(serde_json::json!(lat));
                cartographic_degrees.push(serde_json::json!(height));
            }
            continue;
        }

        if first_time.is_none() {
            first_time = Some(time_str.clone());
        }
        last_time = Some(time_str.clone());

        // Convert time values to proper format
        // Use numeric samples only if all times are numeric to avoid mixing types
        let time_value: Value = if all_numeric_times {
            // Emit numeric time samples
            if let Ok(n) = time_str.parse::<f64>() {
                if !n.is_finite() {
                    // Skip features with NaN/Inf in numeric mode
                    continue;
                }
                serde_json::json!(n)
            } else if let Some(offset) = parse_epoch_offset_timestamp(&time_str) {
                serde_json::json!(offset)
            } else {
                // Skip features with invalid numeric time
                continue;
            }
        } else {
            // Emit ISO 8601 string time samples
            serde_json::json!(time_str)
        };

        if let Some((lon, lat, height)) = extract_point_coords(feature) {
            cartographic_degrees.push(time_value);
            cartographic_degrees.push(serde_json::json!(lon));
            cartographic_degrees.push(serde_json::json!(lat));
            cartographic_degrees.push(serde_json::json!(height));
        }
    }

    let is_time_dynamic = first_time.is_some();
    let mut position = if is_time_dynamic {
        serde_json::json!({
            "cartographicDegrees": cartographic_degrees,
            "interpolationAlgorithm": params.interpolation_algorithm.to_string(),
            "interpolationDegree": params.interpolation_degree,
        })
    } else {
        serde_json::json!({
            "cartographicDegrees": cartographic_degrees,
        })
    };
    if is_time_dynamic {
        if let Some(epoch) = &epoch {
            position["epoch"] = serde_json::json!(epoch);
        }
    }

    let availability = match (&first_time, &last_time) {
        (Some(start), Some(end)) if start != end => {
            let start_iso = strip_epoch_offset_for_availability(start, epoch.as_deref());
            let end_iso = strip_epoch_offset_for_availability(end, epoch.as_deref());
            Some(format!("{start_iso}/{end_iso}"))
        }
        _ => None,
    };

    let name = sorted
        .first()
        .and_then(|f| f.get(Attribute::new("name")))
        .map(attribute_value_to_string);

    let description = sorted.first().map(|f| {
        let filtered = filter_description_attributes(&f.attributes);
        map_to_html_table(&filtered)
    });

    let mut packet = serde_json::json!({
        "id": entity_id,
        "position": position,
    });

    if let Some(avail) = availability {
        packet["availability"] = serde_json::json!(avail);
    }
    if let Some(n) = name {
        packet["name"] = serde_json::json!(n);
    }
    if let Some(d) = description {
        packet["description"] = serde_json::json!(d);
    }

    // Add graphic based on geometry type: polygon for polygon features, point for others
    if let Some(first_feature) = sorted.first() {
        if let Some(polygon_val) = feature_geometry_to_polygon_json(first_feature, params) {
            packet["polygon"] = polygon_val;
            // Polygon positions are self-contained; remove redundant position
            packet.as_object_mut().map(|m| m.remove("position"));
        } else {
            packet["point"] = serde_json::json!({
                "pixelSize": 10,
                "heightReference": "NONE",
            });
        }
    }

    Ok(packet)
}

#[cfg(not(feature = "new-geometry"))]
fn extract_point_coords(feature: &Feature) -> Option<(f64, f64, f64)> {
    match &feature.geometry.value {
        GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => Some((p.x(), p.y(), p.z())),
        GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => Some((p.x(), p.y(), 0.0)),
        GeometryValue::FlowGeometry3D(Geometry3D::LineString(ls)) => {
            ls.0.first().map(|c| (c.x, c.y, c.z))
        }
        GeometryValue::FlowGeometry2D(Geometry2D::LineString(ls)) => {
            ls.0.first().map(|c| (c.x, c.y, 0.0))
        }
        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(poly)) => {
            poly.exterior().0.first().map(|c| (c.x, c.y, c.z))
        }
        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)) => {
            poly.exterior().0.first().map(|c| (c.x, c.y, 0.0))
        }
        _ => None,
    }
}

/// Parse `"<iso>+<N>s"` and return the numeric offset `N`.
fn parse_epoch_offset_timestamp(s: &str) -> Option<f64> {
    if let Some(idx) = s.rfind('+') {
        let suffix = &s[idx + 1..];
        if let Some(num_str) = suffix.strip_suffix('s') {
            return num_str.parse::<f64>().ok();
        }
    }
    None
}

/// Convert an epoch-relative timestamp to ISO 8601 for CZML `availability`.
fn strip_epoch_offset_for_availability(ts: &str, epoch: Option<&str>) -> String {
    // Handle epoch offset format like "2024-01-01T00:00:00Z+120s"
    if let Some(offset) = parse_epoch_offset_timestamp(ts)
        && let Some(epoch_str) = epoch
        && let Ok(epoch_dt) = chrono::DateTime::parse_from_rfc3339(epoch_str)
    {
        // Checked, because `offset` comes from a feature attribute. `as i64`
        // saturates rather than wrapping, so a wild value arrives as
        // `i64::MAX`, and both `Duration::seconds` and `DateTime + Duration`
        // panic on their own bounds. A sink that panics takes the whole job
        // down; the raw string, returned below, is the honest fallback.
        if let Some(delta) = chrono::Duration::try_seconds(offset as i64)
            && let Some(result) = epoch_dt.checked_add_signed(delta)
        {
            let format = if result.timestamp_subsec_nanos() == 0 {
                chrono::SecondsFormat::Secs
            } else {
                chrono::SecondsFormat::Millis
            };
            return result.to_rfc3339_opts(format, true);
        }
        return ts.to_string();
    }

    // Handle pure numeric strings like "155" when epoch is provided
    if let Ok(offset) = ts.parse::<f64>() {
        // Reject NaN/Inf
        if !offset.is_finite() {
            return ts.to_string();
        }
        if let Some(epoch_str) = epoch
            && let Ok(epoch_dt) = chrono::DateTime::parse_from_rfc3339(epoch_str)
        {
            // Preserve fractional seconds by converting to milliseconds
            let millis = (offset * 1000.0) as i64;
            // Checked for the same reason as the offset branch above: a time
            // value far outside the representable range must degrade to the
            // raw string, not abort the run.
            if let Some(result) = chrono::Duration::try_milliseconds(millis)
                .and_then(|delta| epoch_dt.checked_add_signed(delta))
            {
                // The precision comes from the instant, not the offset. A
                // whole-second offset against an epoch that carries a
                // fraction still lands on a fractional instant: with epoch
                // `…T00:00:00.500Z`, time `0` is 00:00:00.500, and choosing
                // `Secs` from `offset.fract()` wrote `…T00:00:00Z` — a sample
                // 500 ms early, and one that can collide with a sample
                // genuinely written at the whole second.
                let format = if result.timestamp_subsec_nanos() == 0 {
                    chrono::SecondsFormat::Secs
                } else {
                    chrono::SecondsFormat::Millis
                };
                return result.to_rfc3339_opts(format, true);
            }
            return ts.to_string();
        }
    }

    ts.to_string()
}

fn attribute_value_to_string(value: &AttributeValue) -> String {
    match value {
        AttributeValue::String(s) => s.clone(),
        AttributeValue::Number(n) => n.to_string(),
        AttributeValue::Bool(b) => b.to_string(),
        _ => {
            let v: serde_json::Value = value.clone().into();
            v.to_string()
        }
    }
}

/// Filter attributes for description, removing internal and duplicate keys.
fn filter_description_attributes(
    attrs: &IndexMap<Attribute, AttributeValue>,
) -> IndexMap<Attribute, AttributeValue> {
    let skip_prefixes = ["czml.", "_", "http_"];
    let skip_keys = [
        "id",
        "name",
        "description",
        "availability",
        "parent",
        "match",
    ];

    attrs
        .iter()
        .filter(|(k, _)| {
            let key = k.to_string();
            !skip_prefixes.iter().any(|p| key.starts_with(p)) && !skip_keys.contains(&key.as_str())
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn map_to_html_table(map: &IndexMap<Attribute, AttributeValue>) -> String {
    let mut html = String::new();
    html.push_str("<table>");
    for (key, value) in map {
        let display = attribute_value_to_string(value);
        html.push_str(&format!("<tr><td>{key}</td><td>{display}</td></tr>"));
    }
    html.push_str("</table>");
    html
}

fn polygon_to_czml_polygon(poly: &Polygon3D<f64>) -> CzmlPolygon {
    let mut czml_polygon = CzmlPolygon::default();

    let exteriors = poly
        .exterior()
        .iter()
        .flat_map(|coord| vec![coord.x, coord.y, coord.z])
        .collect_vec();
    czml_polygon.positions = Some(PositionList::Object(PositionListProperties {
        cartographic_degrees: Some(exteriors),
        ..Default::default()
    }));

    let interiors = poly
        .interiors()
        .iter()
        .flat_map(|line| line.iter().map(|coord| vec![coord.x, coord.y, coord.z]))
        .collect_vec();
    czml_polygon.holes = Some(PositionListOfLists::Object(PositionListOfListsProperties {
        cartographic_degrees: Some(interiors),
        ..Default::default()
    }));

    czml_polygon
}

/// Parse a hex color string like "#ffd8c0" into [r, g, b, alpha].
fn hex_to_rgba(hex: &str, alpha: u8) -> Option<[u8; 4]> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    // `len()` counts bytes, so six bytes is not six hex digits: `#aé123` passes
    // a length check and then splits `é` on the first slice below, which panics
    // rather than returning `None`. The colour comes from a feature attribute,
    // so that is reachable from data. Require ASCII hex digits before slicing;
    // `from_str_radix` would reject them anyway, but only if we get that far.
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b, alpha])
}

/// Convert a Feature's polygon geometry to a styled CZML polygon JSON value.
#[cfg(not(feature = "new-geometry"))]
fn feature_geometry_to_polygon_json(
    feature: &Feature,
    params: &CzmlWriterCompiledParam,
) -> Option<Value> {
    let czml_polygon = match &feature.geometry.value {
        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(poly)) => polygon_to_czml_polygon(poly),
        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(poly)) => {
            let mut czml_poly = CzmlPolygon::default();
            let exteriors: Vec<f64> = poly
                .exterior()
                .iter()
                .flat_map(|coord| vec![coord.x, coord.y, 0.0])
                .collect();
            czml_poly.positions = Some(PositionList::Object(PositionListProperties {
                cartographic_degrees: Some(exteriors),
                ..Default::default()
            }));
            let interiors: Vec<Vec<f64>> = poly
                .interiors()
                .iter()
                .map(|line| {
                    line.iter()
                        .flat_map(|coord| vec![coord.x, coord.y, 0.0])
                        .collect()
                })
                .collect();
            if !interiors.is_empty() {
                czml_poly.holes =
                    Some(PositionListOfLists::Object(PositionListOfListsProperties {
                        cartographic_degrees: Some(interiors),
                        ..Default::default()
                    }));
            }
            czml_poly
        }
        GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(mp)) => {
            mp.0.first().map(polygon_to_czml_polygon)?
        }
        GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(mp)) => {
            let poly = mp.0.first()?;
            let mut czml_poly = CzmlPolygon::default();
            let exteriors: Vec<f64> = poly
                .exterior()
                .iter()
                .flat_map(|coord| vec![coord.x, coord.y, 0.0])
                .collect();
            czml_poly.positions = Some(PositionList::Object(PositionListProperties {
                cartographic_degrees: Some(exteriors),
                ..Default::default()
            }));
            czml_poly
        }
        _ => return None,
    };

    // Serialize base polygon to JSON, then apply styling
    let mut polygon_val = serde_json::to_value(&czml_polygon).ok()?;
    let obj = polygon_val.as_object_mut()?;

    // Apply color from attribute
    if let Some(color_attr) = &params.color_attribute {
        if let Some(AttributeValue::String(hex)) = feature.get(color_attr) {
            if let Some(rgba) = hex_to_rgba(hex, params.opacity) {
                obj.insert(
                    "material".to_string(),
                    serde_json::json!({
                        "solidColor": {
                            "color": {
                                "rgba": rgba
                            }
                        }
                    }),
                );
            }
        }
    }

    // Apply extrusion height from attribute
    if let Some(height_attr) = &params.height_attribute {
        if let Some(height_val) = feature.get(height_attr) {
            if let Some(h) = height_val.as_f64() {
                obj.insert("extrudedHeight".to_string(), serde_json::json!(h));
                obj.insert("closeBottom".to_string(), serde_json::json!(true));
                obj.insert("closeTop".to_string(), serde_json::json!(true));
            }
        }
    }

    // Default outline
    obj.insert("fill".to_string(), serde_json::json!(true));
    obj.insert("outline".to_string(), serde_json::json!(true));
    obj.insert(
        "outlineColor".to_string(),
        serde_json::json!({ "rgba": [80, 80, 80, 220] }),
    );
    obj.insert("outlineWidth".to_string(), serde_json::json!(1.0));

    Some(polygon_val)
}

/// Build a properties bag from feature attributes, excluding internal keys.
fn build_properties_bag(feature: &Feature) -> Option<Value> {
    let skip_prefixes = ["czml."];
    let skip_keys = ["id", "name", "description", "availability", "parent"];

    let mut properties = serde_json::Map::new();
    for (attr, value) in feature.attributes.iter() {
        let key = attr.to_string();
        if skip_prefixes.iter().any(|p| key.starts_with(p)) {
            continue;
        }
        if skip_keys.contains(&key.as_str()) {
            continue;
        }
        let json_val: serde_json::Value = value.clone().into();
        // Skip null values — Cesium's CzmlDataSource crashes on null property values
        if json_val.is_null() {
            continue;
        }
        properties.insert(key, json_val);
    }

    if properties.is_empty() {
        None
    } else {
        Some(Value::Object(properties))
    }
}

#[cfg(not(feature = "new-geometry"))]
fn feature_to_packets(ctx: &Context, feature: &Feature) -> Vec<Packet> {
    // `Context` (finish()-time, no single feature) has no report/warn API,
    // so synthesize the per-feature ExecutorContext those live on. This fn
    // returns Vec<Packet>, not a Result, so report()'s Result is discarded
    // below with `let _ =` — still not a silent drop, since a promoted
    // Fatal reaches the node's fatal slot inside report() itself.
    let ectx = ctx.as_executor_context(feature.clone(), FEATURES_PORT.clone());

    let Some(parent_id) = feature.feature_id() else {
        let _ = ectx.report(DiagnosticDraft::new(ErrorCode::CzmlMissingFeatureId));
        return vec![];
    };

    let properties = map_to_html_table(&feature.attributes);

    let GeometryValue::CityGmlGeometry(geometry) = &feature.geometry.value else {
        let _ = ectx.report(
            DiagnosticDraft::new(ErrorCode::CzmlNonCitygmlGeometry).with_message(format!(
                "Geometry is not a CityGML geometry with: feature_id={}",
                feature.id
            )),
        );
        return vec![];
    };

    let polygons = geometry
        .gml_geometries
        .iter()
        .filter(|geometry| geometry.lod.unwrap_or(0) > 0)
        .flat_map(|geometry| geometry.polygons.clone())
        .collect_vec();

    if polygons.is_empty() {
        let _ = ectx.report(
            DiagnosticDraft::new(ErrorCode::CzmlEmptyGeometry).with_message(format!(
                "Geometry does not contain any polygons: feature_id={}",
                feature.id
            )),
        );
        return vec![];
    }

    // Create a Packet that retains attributes and references it from child features
    let properties_packet = Packet {
        id: Some(parent_id.clone()),
        description: Some(StringValueType::String(properties)),
        ..Default::default()
    };
    let mut packets: Vec<Packet> = vec![properties_packet];

    for poly in polygons {
        let mut czml_polygon = polygon_to_czml_polygon(&poly);
        // In Cesium, if perPositionHeight is false, the polygon height is fixed
        czml_polygon.per_position_height = CzmlBoolean::Boolean(true);

        let packet = Packet {
            polygon: Some(czml_polygon),
            description: Some(StringValueType::Object(StringProperties {
                reference: Some(format!("{parent_id}#description")),
                ..Default::default()
            })),
            parent: Some(parent_id.clone()),
            ..Default::default()
        };
        packets.push(packet);
    }

    packets
}

#[cfg(test)]
mod tests {
    // Readable diffs on a failing packet or sample array.
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::types::coordinate::Coordinate;
    use reearth_flow_geometry::types::line_string::LineString;
    use reearth_flow_geometry::types::no_value::NoValue;
    use reearth_flow_geometry::types::point::Point3D;
    use reearth_flow_geometry::types::polygon::Polygon;
    use reearth_flow_types::Geometry;

    #[cfg(not(feature = "new-geometry"))]
    fn make_feature_3d(lon: f64, lat: f64, height: f64) -> Feature {
        Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            Geometry {
                epsg: Some(4326),
                value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Point3D::new(
                    lon, lat, height,
                ))),
            },
        )
    }

    fn make_timeseries_params() -> CzmlWriterCompiledParam {
        CzmlWriterCompiledParam {
            output: "/tmp/test.czml".to_string(),
            group_by: None,
            time_field: Some(Attribute::new("timestamp")),
            epoch: Some("2024-01-01T00:00:00Z".into()),
            interpolation_algorithm: InterpolationAlgorithm::Lagrange,
            interpolation_degree: 5,
            group_timeseries_by: Some(Attribute::new("entityId")),
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
            polygon_style_was_set: false,
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_build_entity_packet_basic() {
        let params = make_timeseries_params();
        let mut f1 = make_feature_3d(-75.0, 40.0, 0.0);
        f1.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("2024-01-01T00:00:00Z".into()),
        );
        f1.insert(
            Attribute::new("entityId"),
            AttributeValue::String("v1".into()),
        );
        f1.insert(
            Attribute::new("name"),
            AttributeValue::String("Vehicle".into()),
        );

        let mut f2 = make_feature_3d(-76.0, 41.0, 100.0);
        f2.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("2024-01-01T00:01:00Z".into()),
        );
        f2.insert(
            Attribute::new("entityId"),
            AttributeValue::String("v1".into()),
        );

        let features_ref: Vec<&Feature> = vec![&f1, &f2];
        let packet = build_entity_packet("v1", &features_ref, &params).unwrap();

        assert_eq!(packet["id"], "v1");
        assert_eq!(packet["name"], "Vehicle");
        assert_eq!(packet["position"]["interpolationAlgorithm"], "LAGRANGE");
        assert_eq!(packet["position"]["epoch"], "2024-01-01T00:00:00Z");

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 8);
        assert!(packet["availability"].as_str().is_some());
    }

    #[cfg(not(feature = "new-geometry"))]
    fn make_embedded_feature_with_timeseries() -> Feature {
        let mut f = make_feature_3d(139.6917, 35.6895, 50.0);
        f.insert(
            Attribute::new("id"),
            AttributeValue::String("vehicle-a".into()),
        );
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Vehicle Alpha".into()),
        );
        f.insert(
            Attribute::new("availability"),
            AttributeValue::String("2024-01-01T00:00:00Z/2024-01-01T00:10:00Z".into()),
        );
        f.insert(
            Attribute::new("czml.epoch"),
            AttributeValue::String("2024-01-01T00:00:00Z".into()),
        );
        f.insert(
            Attribute::new("czml.interpolationAlgorithm"),
            AttributeValue::String("LAGRANGE".into()),
        );
        f.insert(
            Attribute::new("czml.interpolationDegree"),
            AttributeValue::Number(serde_json::Number::from(5)),
        );
        let ts = serde_json::json!([
            {"time": "2024-01-01T00:00:00Z+0s", "timeOffset": 0.0, "lon": 139.6917, "lat": 35.6895, "height": 50.0},
            {"time": "2024-01-01T00:00:00Z+120s", "timeOffset": 120.0, "lon": 139.7003, "lat": 35.69, "height": 52.0},
        ]);
        f.insert(
            Attribute::new("czml.timeseries"),
            AttributeValue::String(serde_json::to_string(&ts).unwrap()),
        );
        f.insert(
            Attribute::new("czml.point"),
            AttributeValue::String(r#"{"pixelSize":12,"color":{"rgba":[255,0,0,255]}}"#.into()),
        );
        f.insert(
            Attribute::new("czml.path"),
            AttributeValue::String(
                r#"{"material":{"solidColor":{"color":{"rgba":[255,0,0,128]}}},"width":2}"#.into(),
            ),
        );
        f
    }

    #[cfg(not(feature = "new-geometry"))]
    fn make_embedded_static_feature() -> Feature {
        let mut f = make_feature_3d(139.7454, 35.6586, 333.0);
        f.insert(
            Attribute::new("id"),
            AttributeValue::String("static-poi".into()),
        );
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Tokyo Tower".into()),
        );
        f.insert(
            Attribute::new("czml.label"),
            AttributeValue::String(r#"{"text":"Tokyo Tower","font":"14pt sans-serif"}"#.into()),
        );
        f
    }

    fn make_default_params() -> CzmlWriterCompiledParam {
        CzmlWriterCompiledParam {
            output: "/tmp/test.czml".to_string(),
            group_by: None,
            time_field: None,
            epoch: None,
            interpolation_algorithm: InterpolationAlgorithm::default(),
            interpolation_degree: 1,
            group_timeseries_by: None,
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
            polygon_style_was_set: false,
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_build_embedded_packet_timeseries() {
        let f = make_embedded_feature_with_timeseries();
        let params = make_default_params();
        let packet = build_embedded_packet(&f, &params, None, None).unwrap();

        assert_eq!(packet["id"], "vehicle-a");
        assert_eq!(packet["name"], "Vehicle Alpha");

        assert_eq!(packet["position"]["epoch"], "2024-01-01T00:00:00Z");
        assert_eq!(packet["position"]["interpolationAlgorithm"], "LAGRANGE");
        assert_eq!(packet["position"]["interpolationDegree"], 5);

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 8);
        assert_eq!(coords[0].as_f64().unwrap(), 0.0);
        assert_eq!(coords[4].as_f64().unwrap(), 120.0);

        assert_eq!(packet["point"]["pixelSize"], 12);
        assert_eq!(packet["path"]["width"], 2);
        assert!(packet["availability"].as_str().unwrap().contains('/'));
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_build_embedded_packet_static() {
        let f = make_embedded_static_feature();
        let params = make_default_params();
        let packet = build_embedded_packet(&f, &params, None, None).unwrap();

        assert_eq!(packet["id"], "static-poi");
        assert_eq!(packet["name"], "Tokyo Tower");

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 3);
        assert!((coords[0].as_f64().unwrap() - 139.7454).abs() < 1e-4);

        assert_eq!(packet["label"]["text"], "Tokyo Tower");
        assert!(packet.get("point").is_none());
        assert!(packet.get("availability").is_none());
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_build_embedded_czml_document() {
        let f1 = make_embedded_feature_with_timeseries();
        let f2 = make_embedded_static_feature();
        let features = vec![f1, f2];

        let params = make_default_params();

        let buffer = build_embedded_czml(&features, &params).unwrap();
        let czml: Vec<Value> = serde_json::from_slice(&buffer).unwrap();

        assert_eq!(czml.len(), 3);
        assert_eq!(czml[0]["id"], "document");
        assert_eq!(czml[0]["version"], "1.0");

        assert!(
            czml[0]["clock"]["interval"]
                .as_str()
                .unwrap()
                .contains("2024-01-01T00:00:00Z")
        );

        assert_eq!(czml[1]["id"], "vehicle-a");
        assert_eq!(czml[2]["id"], "static-poi");
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_build_entity_packet_numeric_times() {
        // Test with numeric time values and no explicit epoch
        let params = CzmlWriterCompiledParam {
            output: "/tmp/test.czml".to_string(),
            group_by: None,
            time_field: Some(Attribute::new("timestamp")),
            epoch: None, // No explicit epoch - should auto-generate
            interpolation_algorithm: InterpolationAlgorithm::Linear,
            interpolation_degree: 1,
            group_timeseries_by: Some(Attribute::new("entityId")),
            color_attribute: None,
            opacity: default_opacity(),
            height_attribute: None,
            polygon_style_was_set: false,
        };

        let mut f1 = make_feature_3d(139.7, 35.7, 0.0);
        f1.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("0".into()), // Numeric string
        );
        f1.insert(
            Attribute::new("entityId"),
            AttributeValue::String("test1".into()),
        );

        let mut f2 = make_feature_3d(139.8, 35.8, 100.0);
        f2.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("60".into()), // Numeric string (60 seconds later)
        );
        f2.insert(
            Attribute::new("entityId"),
            AttributeValue::String("test1".into()),
        );

        let features_ref: Vec<&Feature> = vec![&f1, &f2];
        let packet = build_entity_packet("test1", &features_ref, &params).unwrap();

        assert_eq!(packet["id"], "test1");
        // Should have auto-generated epoch
        assert_eq!(packet["position"]["epoch"], "1970-01-01T00:00:00Z");

        let coords = packet["position"]["cartographicDegrees"]
            .as_array()
            .unwrap();
        assert_eq!(coords.len(), 8);

        // Time values should be numeric
        assert_eq!(coords[0].as_f64().unwrap(), 0.0);
        assert_eq!(coords[4].as_f64().unwrap(), 60.0);

        // Availability should be ISO 8601 format
        let avail = packet["availability"].as_str().unwrap();
        assert!(avail.contains("1970-01-01T00:00:00Z"));
        assert!(avail.contains("1970-01-01T00:01:00Z"));
    }

    #[test]
    fn test_hex_to_rgba() {
        assert_eq!(hex_to_rgba("#ffd8c0", 180), Some([255, 216, 192, 180]));
        assert_eq!(hex_to_rgba("ff0000", 255), Some([255, 0, 0, 255]));
        assert_eq!(hex_to_rgba("#000000", 0), Some([0, 0, 0, 0]));
        assert_eq!(hex_to_rgba("zzzzzz", 180), None);
        assert_eq!(hex_to_rgba("#fff", 180), None); // too short
    }

    #[cfg(not(feature = "new-geometry"))]
    fn make_polygon_2d_feature() -> Feature {
        let coords: Vec<Coordinate<f64, NoValue>> = vec![
            (139.75, 35.68).into(),
            (139.76, 35.68).into(),
            (139.76, 35.69).into(),
            (139.75, 35.69).into(),
            (139.75, 35.68).into(),
        ];
        let exterior = LineString(coords);
        let polygon: Polygon<f64, NoValue> = Polygon::new(exterior, vec![]);

        let mut f = Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            Geometry {
                epsg: Some(4326),
                value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
            },
        );
        f.insert(Attribute::new("id"), AttributeValue::String("poly1".into()));
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Test Polygon".into()),
        );
        f
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_feature_geometry_to_polygon_2d() {
        let f = make_polygon_2d_feature();
        let params = make_default_params();
        let polygon_val = feature_geometry_to_polygon_json(&f, &params);
        assert!(polygon_val.is_some());

        let pv = polygon_val.unwrap();
        let positions = &pv["positions"]["cartographicDegrees"];
        assert!(positions.is_array());
        let coords = positions.as_array().unwrap();
        // 5 coords * 3 values (lon, lat, 0.0) = 15
        assert_eq!(coords.len(), 15);
        // First coord: lon
        assert!((coords[0].as_f64().unwrap() - 139.75).abs() < 1e-4);
        // Z should be 0
        assert_eq!(coords[2].as_f64().unwrap(), 0.0);

        // Should have fill and outline
        assert_eq!(pv["fill"], true);
        assert_eq!(pv["outline"], true);
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_polygon_with_styling() {
        let mut f = make_polygon_2d_feature();
        f.insert(
            Attribute::new("fill_color"),
            AttributeValue::String("#ffd8c0".into()),
        );
        f.insert(
            Attribute::new("depth"),
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap()),
        );

        let params = CzmlWriterCompiledParam {
            color_attribute: Some(Attribute::new("fill_color")),
            opacity: 180,
            height_attribute: Some(Attribute::new("depth")),
            ..make_default_params()
        };

        let polygon_val = feature_geometry_to_polygon_json(&f, &params).unwrap();

        // Check material color
        let rgba = &polygon_val["material"]["solidColor"]["color"]["rgba"];
        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 216);
        assert_eq!(rgba[2], 192);
        assert_eq!(rgba[3], 180);

        // Check extrusion
        assert_eq!(polygon_val["extrudedHeight"], 1.5);
        assert_eq!(polygon_val["closeBottom"], true);
        assert_eq!(polygon_val["closeTop"], true);
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_per_entity_availability() {
        let mut f1 = make_polygon_2d_feature();
        f1.insert(
            Attribute::new("start_time"),
            AttributeValue::String("3600".into()), // 1 hour offset
        );

        let mut f2 = make_polygon_2d_feature();
        f2.insert(Attribute::new("id"), AttributeValue::String("poly2".into()));
        f2.insert(
            Attribute::new("start_time"),
            AttributeValue::String("7200".into()), // 2 hour offset
        );

        let params = CzmlWriterCompiledParam {
            time_field: Some(Attribute::new("start_time")),
            epoch: Some("2024-01-01T00:00:00Z".into()),
            ..make_default_params()
        };

        let features = vec![f1, f2];
        let buffer = build_embedded_czml(&features, &params).unwrap();
        let czml: Vec<Value> = serde_json::from_slice(&buffer).unwrap();

        // Document + 2 entities
        assert_eq!(czml.len(), 3);
        assert_eq!(czml[0]["id"], "document");
        assert!(czml[0]["clock"].is_object());

        // First entity should have availability starting at 1 hour
        let avail1 = czml[1]["availability"].as_str().unwrap();
        assert!(avail1.contains("2024-01-01T01:00:00Z"));

        // Second entity should have availability starting at 2 hours
        let avail2 = czml[2]["availability"].as_str().unwrap();
        assert!(avail2.contains("2024-01-01T02:00:00Z"));

        // Both should end at the same time (global end = max start = 2 hours)
        assert!(avail1.ends_with("2024-01-01T02:00:00Z"));
        assert!(avail2.ends_with("2024-01-01T02:00:00Z"));

        // Should have polygon geometry (not point)
        assert!(czml[1]["polygon"].is_object());
        assert!(czml[1].get("point").is_none());
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn test_build_properties_bag() {
        let mut f = make_feature_3d(139.7, 35.7, 0.0);
        f.insert(Attribute::new("id"), AttributeValue::String("test".into()));
        f.insert(
            Attribute::new("name"),
            AttributeValue::String("Test".into()),
        );
        f.insert(
            Attribute::new("depth"),
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap()),
        );
        f.insert(
            Attribute::new("czml.point"),
            AttributeValue::String("{}".into()),
        );
        f.insert(
            Attribute::new("custom_field"),
            AttributeValue::String("hello".into()),
        );

        let props = build_properties_bag(&f).unwrap();
        let obj = props.as_object().unwrap();

        // Should include custom fields but not internal ones
        assert!(obj.contains_key("depth"));
        assert!(obj.contains_key("custom_field"));
        assert!(!obj.contains_key("id"));
        assert!(!obj.contains_key("name"));
        assert!(!obj.contains_key("czml.point"));
    }
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod diagnostics_tests {
    // Readable diffs on a failing packet or sample array.
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    use indexmap::IndexMap;
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::node::NodeHandle;
    use reearth_flow_types::{CitygmlFeatureExt, Geometry};

    use super::*;

    fn context_with_handle(handle: &Arc<NodeDiagnosticsHandle>) -> Context {
        let node_ctx = NodeContext::default();
        let mut ctx: Context = node_ctx.into();
        ctx.diagnostics = Some(handle.clone());
        ctx
    }

    fn make_handle() -> Arc<NodeDiagnosticsHandle> {
        Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CzmlWriter".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            true,
        ))
    }

    #[test]
    fn missing_feature_id_is_reported_not_silently_dropped() {
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let feature = Feature::from(IndexMap::<String, AttributeValue>::new());

        let packets = feature_to_packets(&ctx, &feature);

        assert!(packets.is_empty());
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(summaries[0].message.contains("czml.missing_feature_id"));
    }

    #[test]
    fn non_citygml_geometry_is_reported_not_silently_dropped() {
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut feature = Feature::from(IndexMap::<String, AttributeValue>::new());
        feature.update_feature_id("gml-1".to_string());

        let packets = feature_to_packets(&ctx, &feature);

        assert!(packets.is_empty());
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(summaries[0].message.contains("czml.non_citygml_geometry"));
    }

    #[test]
    fn empty_geometry_is_reported_not_silently_dropped() {
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut feature = Feature::new_with_attributes_and_geometry(
            IndexMap::new(),
            Geometry {
                epsg: Some(4326),
                value: GeometryValue::CityGmlGeometry(Default::default()),
            },
        );
        feature.update_feature_id("gml-1".to_string());

        let packets = feature_to_packets(&ctx, &feature);

        assert!(packets.is_empty());
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(summaries[0].message.contains("czml.empty_geometry"));
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_tests {
    // Readable diffs on a failing packet or sample array.
    use super::*;
    use pretty_assertions::assert_eq;

    /// Parse written CZML as JSON, so a test tells "correctly skipped" apart
    /// from "wrote rubbish" — a substring check on the raw bytes cannot: it
    /// passes over a malformed document (e.g. a stray trailing comma) just as
    /// happily as over a genuinely valid one.
    fn parse(out: &str) -> Vec<Value> {
        serde_json::from_str(out).expect("written CZML must be valid JSON")
    }

    #[test]
    fn a_mesh_feature_and_a_point_feature_both_write() {
        // The old writer dispatched on `has_citygml` with an `.any()` over the
        // whole batch, so one CityGML feature caused every non-CityGML sibling
        // to be rejected. Deciding per feature is the fix.
        let doc = parse(&fixture::write_batch(vec![
            fixture::mesh_feature_in(6677),
            fixture::point_feature_in(6677),
        ]));
        assert!(
            doc.iter().any(|p| p.get("polygon").is_some()),
            "the mesh wrote a polygon"
        );
        assert!(
            doc.iter().any(|p| p.get("position").is_some()),
            "the point wrote a position"
        );
    }

    #[test]
    fn a_multi_face_feature_writes_every_face() {
        // Fixes the defect where only the first polygon was written.
        let doc = parse(&fixture::write_batch(vec![fixture::three_face_mesh_in(
            6677,
        )]));
        let polygon_count = doc.iter().filter(|p| p.get("polygon").is_some()).count();
        assert_eq!(polygon_count, 3);
    }

    #[test]
    fn a_line_feature_writes_a_polyline_not_a_point() {
        let doc = parse(&fixture::write_batch(vec![fixture::line_feature_in(6677)]));
        assert!(
            doc.iter().any(|p| p.get("polyline").is_some()),
            "a line must write a polyline"
        );
    }

    #[test]
    fn a_multi_line_feature_writes_every_line() {
        // The linear analogue of `a_multi_face_feature_writes_every_face`: a
        // CityGML `MultiCurve` arrives as a `Collection` of `LineString`s, and
        // the first implementation's `find_map` wrote only the first one.
        let doc = parse(&fixture::write_batch(vec![fixture::three_line_feature_in(
            6677,
        )]));
        let polyline_count = doc.iter().filter(|p| p.get("polyline").is_some()).count();
        assert_eq!(polyline_count, 3, "every line must write its own polyline");
    }

    #[test]
    fn an_embedded_czml_feature_round_trips_its_properties() {
        // Spec §8: Embedded Mode still round-trips its `czml.*` attributes
        // unchanged. Without the Embedded Mode branch this feature falls into
        // the per-feature geometry path, which writes a bare position plus a
        // dot and discards every `czml.*` attribute.
        let doc = parse(&fixture::write_batch(
            vec![fixture::embedded_czml_feature()],
        ));
        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("vehicle-1".to_string())))
            .expect("the vehicle-1 entity packet");

        // Raw `czml.*` graphics come back verbatim, key by key.
        assert_eq!(entity["point"], serde_json::json!({"pixelSize": 12}));
        assert_eq!(entity["label"], serde_json::json!({"text": "V1"}));
        assert_eq!(entity["path"], serde_json::json!({"width": 3}));

        // Timeseries samples, epoch and interpolation metadata rebuild the
        // time-dynamic position. The samples are already WGS84 lon/lat/height
        // as the reader parsed them, so they must appear unswapped.
        let position = &entity["position"];
        assert_eq!(position["epoch"], "2024-01-01T00:00:00Z");
        assert_eq!(position["interpolationAlgorithm"], "LAGRANGE");
        assert_eq!(position["interpolationDegree"], 5);
        assert_eq!(
            position["cartographicDegrees"],
            serde_json::json!([0.0, 139.69, 35.69, 50.0, 120.0, 139.70, 35.69, 52.0])
        );

        // Per-entity availability and the document clock survive too.
        assert_eq!(
            entity["availability"],
            "2024-01-01T00:00:00Z/2024-01-02T00:00:00Z"
        );
        assert_eq!(entity["name"], "Vehicle 1");
        assert_eq!(
            doc[0]["clock"]["interval"],
            "2024-01-01T00:00:00Z/2024-01-02T00:00:00Z"
        );

        // And the non-`czml.` attributes land in the properties bag.
        assert_eq!(entity["properties"]["fleet"], "north");
    }

    #[test]
    fn time_field_alone_takes_the_embedded_per_entity_path() {
        // `timeField` without `groupTimeseriesBy` is the old `per_entity_mode`:
        // it must still compute a per-entity availability window and a
        // document clock. The per-feature geometry path never emits either,
        // so `availability` is what tells the two apart.
        let doc = parse(&fixture::write_batch_with_time_field(
            vec![
                fixture::timed_point_feature_in(4979, "2024-03-01T00:00:00Z"),
                fixture::timed_point_feature_in(4979, "2024-03-02T00:00:00Z"),
            ],
            "timestamp",
        ));
        assert_eq!(
            doc[0]["clock"]["interval"], "2024-03-01T00:00:00Z/2024-03-02T00:00:00Z",
            "timeField alone must still produce a document clock: {:?}",
            doc[0]
        );
        let first = &doc[1];
        assert_eq!(
            first["availability"], "2024-03-01T00:00:00Z/2024-03-02T00:00:00Z",
            "timeField alone must still produce per-entity availability: {first:?}"
        );
        // The geometry still places the entity, reprojected and swapped
        // exactly once: [lat, lon, h] in 4979 becomes [lon, lat, h] in CZML.
        assert_eq!(
            first["position"]["cartographicDegrees"],
            serde_json::json!([139.76, 35.68, 10.0])
        );
    }

    #[test]
    fn an_unplaceable_feature_is_skipped_not_written_wrong() {
        // `parse` alone pins the regression this test exists for: seeding the
        // packet-array buffer with a trailing comma (`[{doc},`) produces
        // `[{"id":"document","version":"1.0"},]` when every feature is
        // skipped — exactly this fixture's case — which `serde_json` rejects.
        // A plain `!out.contains("\"polygon\"")` is trivially true of that
        // garbage too, so it can't catch the bug on its own.
        let doc = parse(&fixture::write_batch(vec![
            fixture::mesh_feature_euclidean(),
        ]));
        assert_eq!(doc.len(), 1, "only the document packet, nothing drawn");
        assert_eq!(doc[0]["id"], "document");
        assert!(
            doc.iter().all(|p| p.get("polygon").is_none()),
            "nothing should be drawn"
        );
    }

    #[test]
    fn a_grouped_timeseries_polygon_feature_still_renders_as_a_polygon() {
        // The old `build_entity_packet` rendered a polygon feature's entity
        // as a footprint (`polygon`, with `position` removed) rather than a
        // point. Restored via `polygon_value_for` (the group's first
        // feature's first face) rather than simplified away — losing this
        // silently would shrink every such entity to an undifferentiated dot.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![fixture::polygon_timeseries_feature_in(
                6677,
                "2024-01-01T00:00:00Z",
                "v1",
            )],
            "timestamp",
            "vehicleId",
        ));
        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity packet");
        assert!(
            entity.get("polygon").is_some(),
            "a polygon feature must render as a polygon, not a point: {entity:?}"
        );
        assert!(
            entity.get("position").is_none(),
            "position must be removed once polygon is set: {entity:?}"
        );
    }

    #[test]
    fn the_document_clock_reads_numeric_times_the_same_way_the_entities_do() {
        // With numeric times and no `epoch`, the clock used to take the raw
        // numbers — `"interval": "0/60"` — while the entities in the same
        // document defaulted to the Unix epoch. CZML wants ISO 8601 here, and
        // one document must not read the same times two ways.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                fixture::polygon_timeseries_feature_in(6677, "0", "v1"),
                fixture::polygon_timeseries_feature_in(6677, "60", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));
        let clock = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("document".to_string())))
            .and_then(|p| p.get("clock"))
            .expect("the document packet's clock");

        assert_eq!(
            clock["interval"],
            serde_json::json!("1970-01-01T00:00:00Z/1970-01-01T00:01:00Z"),
            "the clock must resolve numeric times against the same epoch the \
             entities infer: {clock:?}"
        );
        assert_eq!(
            clock["currentTime"],
            serde_json::json!("1970-01-01T00:00:00Z"),
            "{clock:?}"
        );
    }

    #[test]
    fn an_iso_timestamped_document_clock_is_left_alone() {
        // The counterpart: ISO times must pass through untouched, so the fix
        // above cannot quietly rewrite a document that was already correct.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                fixture::polygon_timeseries_feature_in(6677, "2024-01-01T00:00:00Z", "v1"),
                fixture::polygon_timeseries_feature_in(6677, "2024-01-01T00:01:00Z", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));
        let clock = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("document".to_string())))
            .and_then(|p| p.get("clock"))
            .expect("the document packet's clock");
        assert_eq!(
            clock["interval"],
            serde_json::json!("2024-01-01T00:00:00Z/2024-01-01T00:01:00Z"),
            "{clock:?}"
        );
    }

    #[test]
    fn a_grouped_timeseries_polygon_carries_its_polygon_style() {
        // The port called the *unstyled* `polygon_value_for` on this path, so
        // `colorAttribute`, `opacity` and `heightAttribute` were accepted by
        // the schema and then silently dropped from every grouped-timeseries
        // entity. The old world styled this path, so this is a regression the
        // port introduced, not a gap it inherited.
        let mut feature =
            fixture::polygon_timeseries_feature_in(6677, "2024-01-01T00:00:00Z", "v1");
        feature.insert(
            Attribute::new("fillColor"),
            AttributeValue::String("#ffd8c0".to_string()),
        );
        feature.insert(
            Attribute::new("buildingHeight"),
            AttributeValue::Number(serde_json::Number::from(12)),
        );

        let doc = parse(&fixture::write_styled_timeseries_batch(
            vec![feature],
            "timestamp",
            "vehicleId",
            "fillColor",
            "buildingHeight",
        ));
        let polygon = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .and_then(|p| p.get("polygon"))
            .expect("the vehicleId=v1 entity's polygon");

        assert_eq!(
            polygon["material"]["solidColor"]["color"]["rgba"],
            serde_json::json!([255, 216, 192, 180]),
            "colorAttribute and the default opacity must reach the polygon: {polygon:?}"
        );
        assert_eq!(
            polygon["extrudedHeight"],
            serde_json::json!(12.0),
            "heightAttribute must reach the polygon: {polygon:?}"
        );
    }

    /// Run one feature through the writer and return the diagnostic codes it
    /// emitted straight to the event hub (where `warn_once` goes, since it
    /// carries no feature to bucket under).
    fn codes_emitted_for(with: HashMap<String, serde_json::Value>) -> Vec<String> {
        use std::str::FromStr;

        use reearth_flow_common::uri::Uri;
        use reearth_flow_runtime::event::Event;

        let tmp = tempfile::tempdir().expect("tempdir");
        let root =
            Uri::from_str(&format!("file://{}", tmp.path().display())).expect("valid file uri");
        let node_ctx = NodeContext {
            sandbox_root: root,
            ..NodeContext::default()
        };
        let mut rx = node_ctx.event_hub.sender.subscribe();

        let mut with = with;
        with.insert(
            "output".to_string(),
            serde_json::json!({"type": "string", "value": "out.czml"}),
        );
        let mut sink = CzmlWriterFactory
            .build(
                node_ctx.clone(),
                EventHub::new(30),
                "CZML Writer".to_string(),
                Some(with),
            )
            .expect("factory build");

        let base_ctx = node_ctx.as_context();
        sink.process(
            base_ctx.as_executor_context(fixture::point_feature_in(4326), FEATURES_PORT.clone()),
        )
        .expect("process");
        sink.finish(node_ctx.clone()).expect("finish");

        let mut codes = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let Event::Diagnostic(d) = event {
                codes.push(d.code.as_str().to_string());
            }
        }
        codes
    }

    #[test]
    fn polygon_style_that_cannot_reach_the_output_is_reported() {
        // The geometry-only path is the default one, and the one
        // `polygonStyle` does not reach. The most ordinary configuration there
        // is — features with geometry, no `timeField` — accepted the styling
        // block and ignored it with no error, no warning and no diagnostic.
        let mut with = HashMap::new();
        with.insert(
            "polygonStyle".to_string(),
            serde_json::json!({ "colorAttribute": "fillColor" }),
        );
        let codes = codes_emitted_for(with);
        assert!(
            codes.iter().any(|c| c == "czml.style_not_applied"),
            "styling that cannot apply must say so: {codes:?}"
        );
    }

    #[test]
    fn no_polygon_style_means_nothing_to_report_about_it() {
        // The control: the notice is about styling the user actually asked
        // for, so an absent block must stay silent. Without this, the guard
        // could be satisfied by warning on every run.
        let codes = codes_emitted_for(HashMap::new());
        assert!(
            !codes.iter().any(|c| c == "czml.style_not_applied"),
            "nothing was asked for, so nothing is reported: {codes:?}"
        );
    }

    #[test]
    fn a_feature_carrying_both_a_polygon_and_a_line_writes_both() {
        // `faces_of` returns `Ok` whenever any face exists, so the line branch
        // was only ever reached when there was no areal geometry at all. A
        // collection holding both wrote the polygon and dropped the line, with
        // no packet and no diagnostic — the loss
        // `a_multi_line_feature_writes_every_line` exists to stop, one level up.
        let doc = parse(&fixture::write_batch(vec![
            fixture::polygon_and_line_feature_in(6677),
        ]));

        let polygons = doc.iter().filter(|p| p.get("polygon").is_some()).count();
        let polylines = doc.iter().filter(|p| p.get("polyline").is_some()).count();
        assert_eq!(polygons, 1, "the polygon is written: {doc:?}");
        assert_eq!(polylines, 1, "and so is the line it used to lose: {doc:?}");
    }

    #[test]
    fn the_styling_parameters_old_top_level_names_still_apply() {
        // `colorAttribute`, `opacity` and `heightAttribute` moved under
        // `polygonStyle` to bring the parameter count to §3.5's guideline.
        // `CzmlWriterParam` does not deny unknown fields, so a workflow still
        // on the flat spelling deserialized clean, left `polygon_style` as
        // `None`, and wrote an unstyled polygon with no error and no warning.
        // The same PR kept `force2d` and the uppercase interpolation values
        // working; these get the same treatment.
        let mut feature =
            fixture::polygon_timeseries_feature_in(6677, "2024-01-01T00:00:00Z", "v1");
        feature.insert(
            Attribute::new("fillColor"),
            AttributeValue::String("#ffd8c0".to_string()),
        );
        feature.insert(
            Attribute::new("buildingHeight"),
            AttributeValue::Number(serde_json::Number::from(12)),
        );

        let doc = parse(&fixture::write_legacy_styled_timeseries_batch(
            vec![feature],
            "timestamp",
            "vehicleId",
            "fillColor",
            "buildingHeight",
        ));
        let polygon = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .and_then(|p| p.get("polygon"))
            .expect("the vehicleId=v1 entity's polygon");

        assert_eq!(
            polygon["material"]["solidColor"]["color"]["rgba"],
            serde_json::json!([255, 216, 192, 120]),
            "the old top-level colorAttribute and opacity must still reach the \
             polygon: {polygon:?}"
        );
        assert_eq!(
            polygon["extrudedHeight"],
            serde_json::json!(12.0),
            "the old top-level heightAttribute must still reach the polygon: {polygon:?}"
        );
    }

    #[test]
    fn an_explicit_polygon_style_block_wins_over_the_old_top_level_names() {
        // The flat names are a fallback for workflows written before the
        // grouping, not a second way to say the same thing. A params object
        // carrying both is most likely a half-finished migration, and the new
        // spelling is the one the user last wrote.
        let mut feature =
            fixture::polygon_timeseries_feature_in(6677, "2024-01-01T00:00:00Z", "v1");
        feature.insert(
            Attribute::new("fillColor"),
            AttributeValue::String("#ffd8c0".to_string()),
        );

        let mut extra = HashMap::new();
        extra.insert("timeField".to_string(), serde_json::json!("timestamp"));
        extra.insert(
            "groupTimeseriesBy".to_string(),
            serde_json::json!("vehicleId"),
        );
        extra.insert("colorAttribute".to_string(), serde_json::json!("fillColor"));
        extra.insert("opacity".to_string(), serde_json::json!(10));
        extra.insert(
            "polygonStyle".to_string(),
            serde_json::json!({ "colorAttribute": "fillColor", "opacity": 200 }),
        );

        let doc = parse(&fixture::write_batch_with_params(vec![feature], extra));
        let polygon = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .and_then(|p| p.get("polygon"))
            .expect("the vehicleId=v1 entity's polygon");

        assert_eq!(
            polygon["material"]["solidColor"]["color"]["rgba"],
            serde_json::json!([255, 216, 192, 200]),
            "the polygonStyle block's opacity must win: {polygon:?}"
        );
    }

    /// One grouped-timeseries point feature: a placeable point, a group key,
    /// and `time` as its timestamp unless `time` is empty, in which case the
    /// attribute is left off entirely.
    fn grouped_point(time: &str, group: &str) -> Feature {
        grouped_point_inner(time, group, false)
    }

    /// As [`grouped_point`], but the timestamp attribute is always present,
    /// holding `""` when there is no time. "Absent" and "present but empty"
    /// are different inputs, and the document clock used to treat them
    /// differently from the entity.
    fn grouped_point_with_empty_attr(time: &str, group: &str) -> Feature {
        grouped_point_inner(time, group, true)
    }

    fn grouped_point_inner(time: &str, group: &str, keep_empty: bool) -> Feature {
        let mut f = fixture::point_feature_in(4326);
        if !time.is_empty() || keep_empty {
            f.insert(
                Attribute::new("timestamp"),
                AttributeValue::String(time.to_string()),
            );
        }
        f.insert(
            Attribute::new("vehicleId"),
            AttributeValue::String(group.to_string()),
        );
        f
    }

    #[test]
    fn a_group_mixing_timed_and_untimed_features_keeps_its_samples_aligned() {
        // `cartographicDegrees` is read in groups of four on a time-dynamic
        // entity (time, lon, lat, height) and in threes on a static one. The
        // untimed branch pushed a bare triple into the same vector the timed
        // branch filled with quads, so one untimed feature in a group knocked
        // every later sample out of alignment: the entity flew to garbage
        // coordinates at garbage times, and nothing was reported.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("0", "v1"),
                grouped_point("", "v1"),
                grouped_point("60", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));

        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity");
        let samples = entity["position"]["cartographicDegrees"]
            .as_array()
            .expect("a sample list");

        assert_eq!(
            samples.len() % 4,
            0,
            "a time-dynamic sample list must be whole quads: {samples:?}"
        );
        assert_eq!(
            samples.len(),
            8,
            "the two timed features contribute one sample each: {samples:?}"
        );
        assert_eq!(
            [samples[0].clone(), samples[4].clone()],
            [serde_json::json!(0.0), serde_json::json!(60.0)],
            "each quad must begin with its own time: {samples:?}"
        );
    }

    #[test]
    fn a_group_where_no_feature_is_timed_writes_exactly_one_static_position() {
        // The guard that keeps a bare triple out of a time-dynamic list only
        // fires when some sibling carries a time. When *every* feature in the
        // group is untimed — a misspelled `timeField`, or an attribute absent
        // on this group alone — the entity is static, and each feature pushed
        // its own triple into one `cartographicDegrees`. Cesium chooses
        // between the static and time-tagged readings on length alone
        // (`> 3` means time-tagged), so three features produced a 9-element
        // list read as two-and-a-bit quads: the first longitude became a time.
        // A static entity holds exactly one position.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("", "v1"),
                grouped_point("", "v1"),
                grouped_point("", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));

        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity");
        let position = &entity["position"];
        let samples = position["cartographicDegrees"]
            .as_array()
            .expect("a position list");

        assert_eq!(
            samples.len(),
            3,
            "a static entity holds one [lon, lat, height], not one per feature: {samples:?}"
        );
        assert!(
            samples.iter().all(|v| v.is_f64()),
            "every element of a static position is a coordinate: {samples:?}"
        );
        assert!(
            position.get("interpolationAlgorithm").is_none(),
            "a static entity carries no interpolation hint: {position}"
        );
        assert!(
            position.get("epoch").is_none(),
            "a static entity carries no epoch: {position}"
        );
    }

    #[test]
    fn a_single_untimed_feature_still_writes_a_plain_triple() {
        // The companion to the test above: collapsing to one position must not
        // turn the ordinary one-feature group into something else.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![grouped_point("", "v1")],
            "timestamp",
            "vehicleId",
        ));

        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity");
        let samples = entity["position"]["cartographicDegrees"]
            .as_array()
            .expect("a position list");

        assert_eq!(samples.len(), 3, "one feature, one triple: {samples:?}");
    }

    #[test]
    fn an_explicitly_empty_time_value_is_skipped_by_the_document_clock() {
        // An attribute that exists and holds `""` is not a time. The clock
        // collected it anyway, which made its times non-numeric and sorted the
        // empty string to the front, so it wrote `currentTime: ""` and
        // `interval: "/60"` — neither of them the ISO 8601 CZML requires.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point_with_empty_attr("0", "v1"),
                grouped_point_with_empty_attr("", "v1"),
                grouped_point_with_empty_attr("60", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));

        let clock = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("document".to_string())))
            .and_then(|p| p.get("clock"))
            .expect("the document packet's clock");

        assert_eq!(
            clock["interval"],
            serde_json::json!("1970-01-01T00:00:00Z/1970-01-01T00:01:00Z"),
            "the empty value must not become the interval's start: {clock:?}"
        );
        assert_eq!(
            clock["currentTime"],
            serde_json::json!("1970-01-01T00:00:00Z"),
            "currentTime must be an ISO 8601 instant: {clock:?}"
        );
    }

    #[test]
    fn a_dropped_untimed_feature_does_not_choose_the_entitys_graphic() {
        // Untimed features sort to the front and contribute no sample, so
        // taking the sorted group's first feature as the representative let a
        // feature with no place on the timeline pick the graphic. When that
        // feature is areal, the polygon branch emits `polygon` and removes
        // `position` — discarding every timed sample the loop just collected.
        let untimed_polygon = {
            let mut f = fixture::polygon_timeseries_feature_in(4326, "2024-01-01T00:00:00Z", "v1");
            f.remove(Attribute::new("timestamp"));
            f
        };
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                untimed_polygon,
                grouped_point("0", "v1"),
                grouped_point("60", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));

        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity");

        let samples = entity
            .get("position")
            .and_then(|p| p.get("cartographicDegrees"))
            .and_then(|d| d.as_array())
            .expect("the timed samples must survive");
        assert_eq!(
            samples.len(),
            8,
            "both timed samples must be kept: {samples:?}"
        );
        assert!(
            entity.get("polygon").is_none(),
            "a dropped feature must not choose the graphic: {entity:?}"
        );
    }

    #[test]
    fn a_timed_feature_that_emits_no_sample_does_not_choose_the_entitys_graphic() {
        // Having a time value is not enough to represent the entity. A feature
        // is placed from its *first vertex*, while its polygon is built face
        // by face in each face's own frame, so a collection whose first leaf
        // sits in an unplaceable frame fails placement and still yields a
        // drawable polygon. Picked as the representative, it emitted `polygon`
        // and removed `position` — throwing away the samples its siblings had
        // contributed, which is the same erasure as the untimed case by a
        // different route.
        let mut sample_less = fixture::unplaceable_first_leaf_with_drawable_polygon(4326);
        sample_less.insert(
            Attribute::new("timestamp"),
            AttributeValue::String("0".to_string()),
        );
        sample_less.insert(
            Attribute::new("vehicleId"),
            AttributeValue::String("v1".to_string()),
        );

        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                sample_less,
                grouped_point("30", "v1"),
                grouped_point("60", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));

        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity");

        let samples = entity
            .get("position")
            .and_then(|p| p.get("cartographicDegrees"))
            .and_then(|d| d.as_array())
            .expect("the placeable samples must survive");
        assert_eq!(
            samples.len(),
            8,
            "both placeable samples must be kept: {samples:?}"
        );
        assert!(
            entity.get("polygon").is_none(),
            "a feature that contributed no sample must not choose the graphic: {entity:?}"
        );
    }

    #[test]
    fn a_groups_numeric_times_are_read_the_same_way_as_the_document_clock() {
        // The entity decided `all_numeric` over *every* feature in the group
        // while the document clock skipped the untimed ones, so a single
        // untimed feature made the two disagree: the clock wrote an ISO
        // interval against the Unix epoch while the entity wrote its times as
        // JSON strings, which CZML does not accept as sample times.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("0", "v1"),
                grouped_point("", "v1"),
                grouped_point("60", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));

        let entity = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("v1".to_string())))
            .expect("the vehicleId=v1 entity");
        let position = &entity["position"];

        assert_eq!(
            position["epoch"],
            serde_json::json!(UNIX_EPOCH_ISO),
            "numeric times must resolve against the same epoch the clock uses: {position:?}"
        );
        let samples = position["cartographicDegrees"].as_array().unwrap();
        assert!(
            samples[0].is_number(),
            "a numeric time must stay a number, not become a string: {samples:?}"
        );

        let clock = doc
            .iter()
            .find(|p| p.get("id") == Some(&Value::String("document".to_string())))
            .and_then(|p| p.get("clock"))
            .expect("the document packet's clock");
        assert_eq!(
            clock["interval"],
            serde_json::json!("1970-01-01T00:00:00Z/1970-01-01T00:01:00Z"),
            "the clock reads the same times the entity does: {clock:?}"
        );
    }

    /// Test-only helpers: build features directly against the new geometry
    /// model, then drive a real `CzmlWriter` end to end (factory → `process`
    /// → `finish` → file) and hand back what it wrote.
    /// The `v1` entity packet from a written timeseries document.
    fn entity_of(doc: &[Value], id: &str) -> Value {
        doc.iter()
            .find(|p| p.get("id") == Some(&Value::String(id.to_string())))
            .unwrap_or_else(|| panic!("no entity packet {id:?} in {doc:?}"))
            .clone()
    }

    #[test]
    fn a_minute_precision_timestamp_is_written_not_dropped() {
        // ISO 8601 allows `hh:mm` with no seconds and Cesium's
        // `JulianDate.fromIso8601` reads it, but RFC 3339 requires seconds and
        // `parse_from_rfc3339` is what the validation was built on. Every
        // branch refused this spelling, so a group that had been written and
        // read back fine lost both of its positions and came out as an entity
        // with an empty `cartographicDegrees`.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("2024-01-01T00:00Z", "v1"),
                grouped_point("2024-01-01T00:01Z", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));
        let entity = entity_of(&doc, "v1");
        let samples = entity["position"]["cartographicDegrees"]
            .as_array()
            .expect("cartographicDegrees")
            .clone();
        assert_eq!(
            samples.len(),
            8,
            "both minute-precision samples belong in the entity: {entity:?}"
        );
        // The bounds are normalized even though the samples keep the spelling
        // the feature used, because availability is built from the instant.
        assert_eq!(
            entity["availability"],
            serde_json::json!("2024-01-01T00:00:00Z/2024-01-01T00:01:00Z")
        );
    }

    #[test]
    fn a_groups_availability_runs_forward_when_its_times_mix_spellings() {
        // Against an epoch of midnight, `3600` is 01:00 and the timestamp is
        // 02:00. The bounds were taken in the order the raw strings sorted in,
        // and `'2' < '3'` put the later sample first, so the window ran
        // 02:00 -> 01:00 and covered neither sample.
        let mut with = HashMap::new();
        with.insert("timeField".to_string(), serde_json::json!("timestamp"));
        with.insert(
            "groupTimeseriesBy".to_string(),
            serde_json::json!("vehicleId"),
        );
        with.insert(
            "epoch".to_string(),
            serde_json::json!("2024-01-01T00:00:00Z"),
        );
        let doc = parse(&fixture::write_batch_with_params(
            vec![
                grouped_point("2024-01-01T02:00:00Z", "v1"),
                grouped_point("3600", "v1"),
            ],
            with,
        ));
        let entity = entity_of(&doc, "v1");
        assert_eq!(
            entity["availability"],
            serde_json::json!("2024-01-01T01:00:00Z/2024-01-01T02:00:00Z"),
            "the window must run from the earlier instant to the later one: {entity:?}"
        );
        // And the samples themselves are in that order, earlier one first.
        let samples = entity["position"]["cartographicDegrees"]
            .as_array()
            .expect("cartographicDegrees")
            .clone();
        assert_eq!(samples[0], serde_json::json!("2024-01-01T01:00:00Z"));
    }

    #[test]
    fn the_document_clock_is_a_timestamp_when_one_group_is_numeric_and_another_is_not() {
        // Whether times are numeric is decided per entity, but the clock asked
        // the question of the whole document: one ISO group made the answer
        // `false`, so the numeric group's `0` was sorted as a string and
        // written into `currentTime` verbatim. CZML requires an ISO 8601
        // instant there, so the viewer failed to load the document.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("0", "numeric"),
                grouped_point("60", "numeric"),
                grouped_point("2024-06-01T00:00:00Z", "iso"),
                grouped_point("2024-06-01T00:05:00Z", "iso"),
            ],
            "timestamp",
            "vehicleId",
        ));
        let current = doc[0]["clock"]["currentTime"]
            .as_str()
            .expect("the document clock's currentTime")
            .to_string();
        assert!(
            chrono::DateTime::parse_from_rfc3339(&current).is_ok(),
            "clock currentTime must be a timestamp, got {current:?}"
        );
        // The numeric group counts from the Unix epoch, exactly as its own
        // entity does, so it is the earlier of the two groups.
        assert_eq!(current, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn a_time_the_entity_refused_is_absent_from_the_clock_too() {
        // `n/a` sorted to the end of the strings and became the clock's upper
        // bound after the entity had already reported and dropped it, leaving
        // a document whose clock named a time nothing was written at.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("2024-01-01T00:00:00Z", "v1"),
                grouped_point("2024-01-01T00:01:00Z", "v1"),
                grouped_point("n/a", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));
        let text = serde_json::to_string(&doc).expect("serializable");
        assert!(
            !text.contains("n/a"),
            "a time the entity refused must not reach the clock: {text}"
        );
        assert_eq!(
            doc[0]["clock"]["interval"],
            serde_json::json!("2024-01-01T00:00:00Z/2024-01-01T00:01:00Z")
        );
    }

    #[test]
    fn an_unrepresentable_numeric_time_reaches_neither_the_bounds_nor_the_clock() {
        // `1e30` is finite, so the finite check passed it. The conversion then
        // handed back the raw string rather than panicking, and that string
        // went into the entity's availability and the document clock, giving a
        // file whose time bounds are not valid CZML.
        let doc = parse(&fixture::write_timeseries_batch(
            vec![
                grouped_point("0", "v1"),
                grouped_point("60", "v1"),
                grouped_point("1e30", "v1"),
            ],
            "timestamp",
            "vehicleId",
        ));
        let text = serde_json::to_string(&doc).expect("serializable");
        assert!(
            !text.contains("1e30"),
            "an unwritable time must reach no part of the file: {text}"
        );
        // Pinned separately from the clock: dropping the sample is the
        // entity-side half of the same fix, and four elements per sample means
        // the two writable features are all that is left.
        let entity = entity_of(&doc, "v1");
        assert_eq!(
            entity["position"]["cartographicDegrees"]
                .as_array()
                .expect("cartographicDegrees")
                .len(),
            8,
            "only the two writable samples belong in the entity: {entity:?}"
        );
    }

    #[test]
    fn an_epoch_offset_time_is_accepted_by_the_entity_and_the_clock_alike() {
        // `<iso>+<N>s` resolved to an instant for the clock but was refused
        // for the entity, because acceptance was decided twice and the
        // entity's test was `parse::<f64>()`, which this spelling fails. The
        // sample was dropped while the clock interval still covered it.
        let mut with = HashMap::new();
        with.insert("timeField".to_string(), serde_json::json!("timestamp"));
        with.insert(
            "groupTimeseriesBy".to_string(),
            serde_json::json!("vehicleId"),
        );
        with.insert(
            "epoch".to_string(),
            serde_json::json!("2024-01-01T00:00:00Z"),
        );
        let doc = parse(&fixture::write_batch_with_params(
            vec![
                grouped_point("2024-01-01T00:00:00Z+120s", "v1"),
                grouped_point("2024-01-01T00:00:00Z+240s", "v1"),
            ],
            with,
        ));
        let entity = entity_of(&doc, "v1");
        let samples = entity["position"]["cartographicDegrees"]
            .as_array()
            .expect("cartographicDegrees")
            .clone();
        assert_eq!(
            samples.len(),
            8,
            "both epoch-offset samples belong in the entity: {entity:?}"
        );
        // Written as the instant it names, not as the bare offset number.
        assert_eq!(samples[0], serde_json::json!("2024-01-01T00:02:00Z"));
        // And the entity's window and the document clock agree about it.
        assert_eq!(
            entity["availability"],
            serde_json::json!("2024-01-01T00:02:00Z/2024-01-01T00:04:00Z")
        );
        assert_eq!(
            doc[0]["clock"]["interval"],
            serde_json::json!("2024-01-01T00:02:00Z/2024-01-01T00:04:00Z")
        );
    }

    #[test]
    fn a_fractional_epoch_keeps_its_fraction_for_a_whole_second_offset() {
        // The written precision was chosen from `offset.fract()`, so time `0`
        // against an epoch carrying half a second was written as the whole
        // second: a sample 500 ms early, and one that can collide with a
        // sample genuinely written at that whole second.
        assert_eq!(
            strip_epoch_offset_for_availability("0", Some("2024-01-01T00:00:00.500Z")),
            "2024-01-01T00:00:00.500Z"
        );
    }

    #[test]
    fn a_collection_keeps_its_polygon_when_a_sibling_line_cannot_be_placed() {
        // Every face and every line of a collection carries its own frame, so
        // a placeable polygon can sit beside a line the writer cannot resolve.
        // Returning early on the line discarded the polygon packet already
        // built, losing geometry the writer had emitted before lines were
        // drawn at all.
        let doc = parse(&fixture::write_batch(vec![
            fixture::polygon_with_unplaceable_line_in(4326),
        ]));
        assert!(
            doc.iter().any(|p| p.get("polygon").is_some()),
            "the placeable polygon must still be written: {doc:?}"
        );
        assert!(
            doc.iter().all(|p| p.get("polyline").is_none()),
            "the unplaceable line must not be written: {doc:?}"
        );
    }

    mod fixture {
        use std::collections::HashMap;
        use std::str::FromStr;

        use reearth_flow_common::uri::Uri;
        use reearth_flow_geometry::collection::Collection3D;
        use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
        use reearth_flow_geometry::line_string::LineString3D;
        use reearth_flow_geometry::point::Point3D;
        use reearth_flow_geometry::polygon::Polygon3D;
        use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;
        use reearth_flow_geometry::{Euclidean3DGeometry, Geometry as NewGeometry};
        use reearth_flow_runtime::event::EventHub;
        use reearth_flow_runtime::executor_operation::NodeContext;
        use reearth_flow_runtime::node::{FEATURES_PORT, Sink, SinkFactory};
        use reearth_flow_types::{Attribute, AttributeValue, Feature};

        use super::super::CzmlWriterFactory;

        fn frame(epsg: u16) -> CoordinateFrame {
            CoordinateFrame::Crs(EpsgCode::new(epsg))
        }

        fn feature_with(geometry: NewGeometry) -> Feature {
            Feature::new_with_attributes_and_geometry(indexmap::IndexMap::new(), geometry)
        }

        /// A single-face square, `frame`d as requested; one exterior ring, no
        /// holes, closed.
        fn single_face_mesh(frame: CoordinateFrame) -> NewGeometry {
            NewGeometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    frame,
                    [
                        [0.0, 0.0, 0.0],
                        [100.0, 0.0, 0.0],
                        [100.0, 100.0, 0.0],
                        [0.0, 100.0, 0.0],
                        [0.0, 0.0, 0.0],
                    ],
                    std::iter::empty::<Vec<[f64; 3]>>(),
                ),
            )))
        }

        /// A collection whose *first* leaf is an unplaceable point and whose
        /// second is a placeable polygon. `cartographic_position` reads the
        /// first vertex, so it fails; `faces_of` walks every leaf in its own
        /// frame, so the polygon still converts. The two disagree, which is
        /// what makes this feature a sample-less polygon source.
        pub(super) fn unplaceable_first_leaf_with_drawable_polygon(epsg: u16) -> Feature {
            let collection = Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(
                    CoordinateFrame::default(),
                    [1.0, 2.0, 3.0],
                )),
                match single_face_mesh(frame(epsg)) {
                    NewGeometry::Euclidean3D(g) => g,
                    _ => unreachable!("single_face_mesh is Euclidean3D"),
                },
            ]);
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Collection(
                collection,
            )))
        }

        /// A collection whose polygon can be placed and whose line cannot.
        /// Each member carries its own frame, so this is an ordinary shape
        /// rather than a contrived one.
        pub(super) fn polygon_with_unplaceable_line_in(epsg: u16) -> Feature {
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Collection(
                Collection3D::new([
                    Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                        frame(epsg),
                        [
                            [0.0, 0.0, 0.0],
                            [100.0, 0.0, 0.0],
                            [100.0, 100.0, 0.0],
                            [0.0, 0.0, 0.0],
                        ],
                        std::iter::empty::<Vec<[f64; 3]>>(),
                    ))),
                    Euclidean3DGeometry::LineString(LineString3D::from_coords(
                        CoordinateFrame::default(),
                        vec![[0.0, 300.0, 0.0], [100.0, 300.0, 0.0]],
                    )),
                ]),
            )))
        }

        pub(super) fn mesh_feature_in(epsg: u16) -> Feature {
            feature_with(single_face_mesh(frame(epsg)))
        }

        /// Same mesh as [`mesh_feature_in`], but in the default (Euclidean,
        /// not a concrete CRS) frame — unplaceable on the WGS84 globe.
        pub(super) fn mesh_feature_euclidean() -> Feature {
            feature_with(single_face_mesh(CoordinateFrame::default()))
        }

        pub(super) fn point_feature_in(epsg: u16) -> Feature {
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Point(
                Point3D::new(frame(epsg), [10.0, 20.0, 0.0]),
            )))
        }

        /// A degenerate (but validly indexed) triangle repeated three times as
        /// three distinct mesh faces — enough to exercise "every face writes",
        /// without needing a real three-differently-shaped-faces surface.
        pub(super) fn three_face_mesh_in(epsg: u16) -> Feature {
            let vertices = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [0.0, 100.0, 0.0]];
            let faces: Vec<Vec<u32>> = std::iter::repeat_n(vec![0u32, 1, 2], 3).collect();
            let mesh =
                PolygonMesh3D::from_parts(frame(epsg), vertices, faces).expect("three valid faces");
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(
                Box::new(mesh),
            )))
        }

        pub(super) fn line_feature_in(epsg: u16) -> Feature {
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::LineString(
                LineString3D::from_coords(
                    frame(epsg),
                    vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [100.0, 100.0, 0.0]],
                ),
            )))
        }

        /// Three distinct `LineString`s under one `Collection` — the shape a
        /// CityGML `MultiCurve` takes in the new model.
        pub(super) fn three_line_feature_in(epsg: u16) -> Feature {
            let members = (0..3).map(|i| {
                let offset = f64::from(i) * 200.0;
                Euclidean3DGeometry::LineString(LineString3D::from_coords(
                    frame(epsg),
                    vec![[0.0, offset, 0.0], [100.0, offset, 0.0]],
                ))
            });
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Collection(
                Collection3D::new(members),
            )))
        }

        /// One `Polygon` and one `LineString` under a single `Collection` —
        /// a feature whose geometry is both areal and linear.
        pub(super) fn polygon_and_line_feature_in(epsg: u16) -> Feature {
            feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Collection(
                Collection3D::new([
                    Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                        frame(epsg),
                        [
                            [0.0, 0.0, 0.0],
                            [100.0, 0.0, 0.0],
                            [100.0, 100.0, 0.0],
                            [0.0, 0.0, 0.0],
                        ],
                        std::iter::empty::<Vec<[f64; 3]>>(),
                    ))),
                    Euclidean3DGeometry::LineString(LineString3D::from_coords(
                        frame(epsg),
                        vec![[0.0, 300.0, 0.0], [100.0, 300.0, 0.0]],
                    )),
                ]),
            )))
        }

        /// A `preserveRaw` round-trip feature: a whole CZML packet carried as
        /// `czml.*` attributes, plus the standard `id`/`name`/`availability`
        /// keys and one ordinary attribute for the properties bag.
        pub(super) fn embedded_czml_feature() -> Feature {
            let mut f = feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Point(
                Point3D::new(frame(4979), [35.68, 139.76, 10.0]),
            )));
            let mut s = |k: &str, v: &str| {
                f.insert(Attribute::new(k), AttributeValue::String(v.to_string()));
            };
            s("id", "vehicle-1");
            s("name", "Vehicle 1");
            s("availability", "2024-01-01T00:00:00Z/2024-01-02T00:00:00Z");
            s("czml.epoch", "2024-01-01T00:00:00Z");
            s("czml.interpolationAlgorithm", "LAGRANGE");
            s(
                "czml.timeseries",
                r#"[{"time":"0","timeOffset":0,"lon":139.69,"lat":35.69,"height":50},
                    {"time":"120","timeOffset":120,"lon":139.70,"lat":35.69,"height":52}]"#,
            );
            s("czml.point", r#"{"pixelSize":12}"#);
            s("czml.label", r#"{"text":"V1"}"#);
            s("czml.path", r#"{"width":3}"#);
            s("fleet", "north");
            f.insert(
                Attribute::new("czml.interpolationDegree"),
                AttributeValue::Number(serde_json::Number::from(5)),
            );
            f
        }

        /// A placeable point carrying only a plain time attribute — no
        /// `czml.*` at all, so only `timeField` can route it to Embedded Mode.
        pub(super) fn timed_point_feature_in(epsg: u16, time: &str) -> Feature {
            let mut f = feature_with(NewGeometry::Euclidean3D(Euclidean3DGeometry::Point(
                Point3D::new(frame(epsg), [35.68, 139.76, 10.0]),
            )));
            f.insert(
                Attribute::new("timestamp"),
                AttributeValue::String(time.to_string()),
            );
            f
        }

        /// A single-face mesh feature carrying the two attributes a
        /// grouped-timeseries workflow needs: a time value and a group key.
        pub(super) fn polygon_timeseries_feature_in(epsg: u16, time: &str, group: &str) -> Feature {
            let mut f = feature_with(single_face_mesh(frame(epsg)));
            f.insert(
                Attribute::new("timestamp"),
                AttributeValue::String(time.to_string()),
            );
            f.insert(
                Attribute::new("vehicleId"),
                AttributeValue::String(group.to_string()),
            );
            f
        }

        /// Build a `CzmlWriter` via its factory, feed `features` through
        /// `process`, call `finish` against a real temp-dir `SinkOutput`, and
        /// return what it wrote.
        pub(super) fn write_batch(features: Vec<Feature>) -> String {
            write_batch_with(features, HashMap::new())
        }

        /// As [`write_batch`], but with `timeField` set and
        /// `groupTimeseriesBy` deliberately left unset — the old
        /// `per_entity_mode` trigger for Embedded Mode.
        pub(super) fn write_batch_with_time_field(
            features: Vec<Feature>,
            time_field: &str,
        ) -> String {
            let mut extra = HashMap::new();
            extra.insert("timeField".to_string(), serde_json::json!(time_field));
            write_batch_with(features, extra)
        }

        /// As [`write_batch`], but wired for grouped-timeseries mode
        /// (`timeField` + `groupTimeseriesBy` both set).
        pub(super) fn write_timeseries_batch(
            features: Vec<Feature>,
            time_field: &str,
            group_timeseries_by: &str,
        ) -> String {
            // Unlike `output` (a `Code`, which deserializes from a
            // `{"type", "value"}` struct), `timeField`/`groupTimeseriesBy` are
            // plain `Attribute`s and deserialize straight from a bare string.
            let mut extra = HashMap::new();
            extra.insert("timeField".to_string(), serde_json::json!(time_field));
            extra.insert(
                "groupTimeseriesBy".to_string(),
                serde_json::json!(group_timeseries_by),
            );
            write_batch_with(features, extra)
        }

        /// As [`write_timeseries_batch`], but with a `polygonStyle` block, so a
        /// test can see whether the styling parameters reach the entity's
        /// polygon at all.
        pub(super) fn write_styled_timeseries_batch(
            features: Vec<Feature>,
            time_field: &str,
            group_timeseries_by: &str,
            color_attribute: &str,
            height_attribute: &str,
        ) -> String {
            let mut extra = HashMap::new();
            extra.insert("timeField".to_string(), serde_json::json!(time_field));
            extra.insert(
                "groupTimeseriesBy".to_string(),
                serde_json::json!(group_timeseries_by),
            );
            extra.insert(
                "polygonStyle".to_string(),
                serde_json::json!({
                    "colorAttribute": color_attribute,
                    "heightAttribute": height_attribute,
                }),
            );
            write_batch_with(features, extra)
        }

        /// As [`write_styled_timeseries_batch`], but passing the three styling
        /// parameters at the top level, where they lived before they were
        /// grouped under `polygonStyle`.
        pub(super) fn write_legacy_styled_timeseries_batch(
            features: Vec<Feature>,
            time_field: &str,
            group_timeseries_by: &str,
            color_attribute: &str,
            height_attribute: &str,
        ) -> String {
            let mut extra = HashMap::new();
            extra.insert("timeField".to_string(), serde_json::json!(time_field));
            extra.insert(
                "groupTimeseriesBy".to_string(),
                serde_json::json!(group_timeseries_by),
            );
            extra.insert(
                "colorAttribute".to_string(),
                serde_json::json!(color_attribute),
            );
            extra.insert(
                "heightAttribute".to_string(),
                serde_json::json!(height_attribute),
            );
            extra.insert("opacity".to_string(), serde_json::json!(120));
            write_batch_with(features, extra)
        }

        /// [`write_batch_with`] under a name a test can reach, for the cases
        /// that need a parameter combination no named helper covers.
        pub(super) fn write_batch_with_params(
            features: Vec<Feature>,
            with: HashMap<String, serde_json::Value>,
        ) -> String {
            write_batch_with(features, with)
        }

        fn write_batch_with(
            features: Vec<Feature>,
            mut with: HashMap<String, serde_json::Value>,
        ) -> String {
            let tmp = tempfile::tempdir().expect("tempdir");
            let root =
                Uri::from_str(&format!("file://{}", tmp.path().display())).expect("valid file uri");
            let node_ctx = NodeContext {
                sandbox_root: root,
                ..NodeContext::default()
            };

            with.insert(
                "output".to_string(),
                serde_json::json!({"type": "string", "value": "out.czml"}),
            );

            let mut sink = CzmlWriterFactory
                .build(
                    node_ctx.clone(),
                    EventHub::new(30),
                    "CZML Writer".to_string(),
                    Some(with),
                )
                .expect("factory build");

            let base_ctx = node_ctx.as_context();
            for feature in features {
                let ectx = base_ctx.as_executor_context(feature, FEATURES_PORT.clone());
                sink.process(ectx).expect("process");
            }

            sink.finish(node_ctx).expect("finish");

            std::fs::read_to_string(tmp.path().join("out.czml")).expect("read output")
        }
    }
}

/// The new-geometry counterpart of `diagnostics_tests`: pins that a skipped
/// feature actually reaches the diagnostics handle, with the right code *and
/// the right kind*.
///
/// The kind is the load-bearing half. `ctx.warn()` records
/// `DiagnosticKind::WarnContinue` and never calls `handle.resolve(code)`, so
/// the summary comes back with `effective_disposition: None` — a dropped
/// feature reported as "warned about ... continue", and a `fatal` disposition
/// override on the node silently ignored. `ctx.report()` resolves and records
/// `WarnDrop`. Asserting only the code would pass either way, which is how
/// the mis-routing survived review in the first place.
#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_diagnostics_tests {
    // Readable diffs on a failing packet or sample array.
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    use reearth_flow_diagnostics::Disposition;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry as NewGeometry};
    use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
    use reearth_flow_runtime::node::NodeHandle;

    use super::*;

    fn make_handle() -> Arc<NodeDiagnosticsHandle> {
        Arc::new(NodeDiagnosticsHandle::new(
            "n1".to_string(),
            NodeHandle::for_test("n1"),
            "writer".into(),
            "CzmlWriter".into(),
            Arc::default(),
            Arc::new(reearth_flow_diagnostics::DispositionPolicy::default()),
            true,
        ))
    }

    fn context_with_handle(handle: &Arc<NodeDiagnosticsHandle>) -> Context {
        let node_ctx = NodeContext::default();
        let mut ctx: Context = node_ctx.into();
        ctx.diagnostics = Some(handle.clone());
        ctx
    }

    #[test]
    fn an_unplaceable_frame_is_reported_as_a_drop_not_a_warn_continue() {
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        // A point in the default (Euclidean, not a concrete CRS) frame: there
        // is no way to know where on the globe it is, so it is dropped.
        let feature = Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            NewGeometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::default(),
                [1.0, 2.0, 3.0],
            ))),
        );
        let mut cache = ReprojectionCache::default();

        let packets = feature_to_packets_next(&ctx, &feature, &mut cache);

        assert!(packets.is_empty(), "an unplaceable feature writes nothing");
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        assert!(
            summaries[0].message.contains("czml.unplaceable_frame"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop),
            "a dropped feature must be reported (WarnDrop), not warn()ed \
             (which leaves effective_disposition None and skips resolve())"
        );
        assert!(
            summaries[0].message.contains("dropped"),
            "the summary must read as a drop, not \"warned about\": {}",
            summaries[0].message
        );
    }

    #[test]
    fn a_feature_with_no_drawable_geometry_is_reported_as_a_drop() {
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let feature =
            Feature::new_with_attributes_and_geometry(indexmap::IndexMap::new(), NewGeometry::None);
        let mut cache = ReprojectionCache::default();

        let packets = feature_to_packets_next(&ctx, &feature, &mut cache);

        assert!(packets.is_empty());
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert!(
            summaries[0].message.contains("czml.empty_geometry"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
    }

    /// `CzmlWriterCompiledParam` with only `time_field` set — the Embedded Mode
    /// per-entity trigger.
    fn per_entity_params() -> CzmlWriterCompiledParam {
        CzmlWriterCompiledParam {
            output: "out.czml".to_string(),
            group_by: None,
            time_field: Some(Attribute::new("timestamp")),
            epoch: None,
            interpolation_algorithm: InterpolationAlgorithm::default(),
            interpolation_degree: 1,
            group_timeseries_by: None,
            color_attribute: None,
            opacity: 180,
            height_attribute: None,
            polygon_style_was_set: false,
        }
    }

    #[test]
    fn an_unplaceable_feature_in_embedded_mode_is_reported_not_silently_unpositioned() {
        // Embedded Mode and the grouped-timeseries path both took an `Option`
        // from `cartographic_position` and wrote the entity with no position,
        // saying nothing — a batch could lose every position with the run
        // still green. The entity is still written, because its `czml.*`
        // attributes carry graphics worth keeping; it is the silence that was
        // the defect.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let feature = Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            NewGeometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::default(),
                [1.0, 2.0, 3.0],
            ))),
        );
        let mut cache = ReprojectionCache::default();

        let out =
            build_embedded_czml_next(&ctx, &[feature], &per_entity_params(), &mut cache).unwrap();
        let doc: Vec<Value> =
            serde_json::from_slice(&out).expect("written CZML must be valid JSON");

        assert!(
            doc.iter().all(|p| p.get("position").is_none()),
            "an unplaceable feature must not be given a position: {doc:?}"
        );
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert!(
            summaries[0].message.contains("czml.unplaceable_frame"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
    }

    #[test]
    fn an_embedded_graphic_places_the_entity_so_absent_geometry_is_not_reported() {
        // The counterpart to the test above, and the line between them is the
        // whole point. A feature read with `preserveRaw` can carry no geometry
        // at all and a `czml.polygon` whose own `positions` place it. Reporting
        // `czml.empty_geometry` the moment geometry fails to place it would
        // call that a failure while the packet goes out fully drawn — a false
        // positive, and the fastest way to teach users to ignore the code.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut feature =
            Feature::new_with_attributes_and_geometry(indexmap::IndexMap::new(), NewGeometry::None);
        feature.insert(
            Attribute::new("czml.polygon"),
            AttributeValue::String(
                r#"{"positions":{"cartographicDegrees":[139.0,36.0,0,139.1,36.0,0,139.1,36.1,0]}}"#
                    .to_string(),
            ),
        );
        let mut cache = ReprojectionCache::default();

        let out =
            build_embedded_czml_next(&ctx, &[feature], &per_entity_params(), &mut cache).unwrap();
        let doc: Vec<Value> =
            serde_json::from_slice(&out).expect("written CZML must be valid JSON");

        assert!(
            doc.iter().any(|p| p.get("polygon").is_some()),
            "the embedded polygon must survive: {doc:?}"
        );
        assert!(
            handle.inner.drain_summaries().is_empty(),
            "a packet that carries its own graphic is not a placement failure"
        );
    }

    /// Build one `preserveRaw`-shaped feature: no geometry at all, and a single
    /// `czml.*` attribute holding `body` verbatim.
    fn embedded_feature(key: &str, body: &str) -> Feature {
        let mut feature =
            Feature::new_with_attributes_and_geometry(indexmap::IndexMap::new(), NewGeometry::None);
        feature.insert(
            Attribute::new(key),
            AttributeValue::String(body.to_string()),
        );
        feature
    }

    /// Run the embedded path over one feature and hand back the packets and
    /// whatever was reported, so the placement cases read as one line each.
    fn embedded_run(feature: Feature) -> (Vec<Value>, Vec<String>, Vec<Option<Disposition>>) {
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut cache = ReprojectionCache::default();
        let out =
            build_embedded_czml_next(&ctx, &[feature], &per_entity_params(), &mut cache).unwrap();
        let doc: Vec<Value> =
            serde_json::from_slice(&out).expect("written CZML must be valid JSON");
        let summaries = handle.inner.drain_summaries();
        let messages = summaries.iter().map(|s| s.message.clone()).collect();
        let dispositions = summaries.iter().map(|s| s.effective_disposition).collect();
        (doc, messages, dispositions)
    }

    #[test]
    fn an_embedded_point_graphic_does_not_place_the_entity_so_absent_geometry_is_reported() {
        // The control for the test above, and the line between them is which
        // graphics place themselves. A `polygon` carries its own `positions`;
        // a `point` is drawn at the containing entity's `position`, so a packet
        // with `point` and no `position` is not drawn anywhere. Treating any
        // recognised graphic as proof of placement suppressed the diagnostic
        // for exactly that case and wrote an invisible entity in silence.
        let (doc, messages, dispositions) = embedded_run(embedded_feature(
            "czml.point",
            r#"{"pixelSize":10,"color":{"rgba":[255,0,0,255]}}"#,
        ));

        assert!(
            doc.iter().any(|p| p.get("point").is_some()),
            "the embedded point must survive: {doc:?}"
        );
        assert!(
            doc.iter().all(|p| p.get("position").is_none()),
            "nothing could place the entity, so it must have no position: {doc:?}"
        );
        assert_eq!(messages.len(), 1, "got {messages:?}");
        assert!(
            messages[0].contains("czml.empty_geometry"),
            "got {}",
            messages[0]
        );
        assert_eq!(dispositions[0], Some(Disposition::WarnDrop));
    }

    #[test]
    fn an_embedded_polygon_without_positions_is_reported() {
        // The pair in `SELF_PLACING_GRAPHICS` is a key *and* the property that
        // does the placing, because a `polygon` is only self-placing when it
        // actually carries `positions`. A styling-only polygon — the shape a
        // `preserveRaw` read produces when the source referenced its positions
        // from elsewhere — places nothing, and must not buy silence.
        let (doc, messages, _) = embedded_run(embedded_feature(
            "czml.polygon",
            r#"{"material":{"solidColor":{"color":{"rgba":[0,255,0,128]}}}}"#,
        ));

        assert!(
            doc.iter().all(|p| p.get("position").is_none()),
            "nothing could place the entity, so it must have no position: {doc:?}"
        );
        assert_eq!(messages.len(), 1, "got {messages:?}");
        assert!(
            messages[0].contains("czml.empty_geometry"),
            "got {}",
            messages[0]
        );
    }

    /// A grouped-timeseries point feature, as `new_geometry_tests` builds one,
    /// repeated here because the two test modules do not share helpers.
    fn diag_grouped_point(time: &str, group: &str) -> Feature {
        let mut f = Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            NewGeometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Crs(reearth_flow_geometry::coordinate::EpsgCode::new(4326)),
                [10.0, 20.0, 0.0],
            ))),
        );
        if !time.is_empty() {
            f.insert(
                Attribute::new("timestamp"),
                AttributeValue::String(time.to_string()),
            );
        }
        f.insert(
            Attribute::new("vehicleId"),
            AttributeValue::String(group.to_string()),
        );
        f
    }

    /// The areal analogue of [`diag_grouped_point`]: a one-face polygon with
    /// the same timestamp and group attributes.
    fn diag_grouped_polygon(time: &str, group: &str) -> Feature {
        use reearth_flow_geometry::polygon::Polygon3D;
        let mut f = Feature::new_with_attributes_and_geometry(
            indexmap::IndexMap::new(),
            NewGeometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    CoordinateFrame::Crs(reearth_flow_geometry::coordinate::EpsgCode::new(4326)),
                    [
                        [10.0, 20.0, 0.0],
                        [10.001, 20.0, 0.0],
                        [10.001, 20.001, 0.0],
                        [10.0, 20.0, 0.0],
                    ],
                    std::iter::empty::<Vec<[f64; 3]>>(),
                ),
            ))),
        );
        f.insert(
            Attribute::new("timestamp"),
            AttributeValue::String(time.to_string()),
        );
        f.insert(
            Attribute::new("vehicleId"),
            AttributeValue::String(group.to_string()),
        );
        f
    }

    #[test]
    fn a_groups_untimed_feature_is_reported_not_silently_dropped() {
        // The sample above cannot be written, so it is dropped — and a drop
        // this writer makes on its own is exactly what the audit set out to
        // stop doing in silence.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut cache = ReprojectionCache::default();
        let timed = diag_grouped_point("0", "v1");
        let untimed = diag_grouped_point("", "v1");
        let features: Vec<&Feature> = vec![&timed, &untimed];

        let mut params = per_entity_params();
        params.group_timeseries_by = Some(Attribute::new("vehicleId"));

        let _ = build_entity_packet_next(&ctx, "v1", &features, &params, &mut cache).unwrap();

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1, "got {summaries:?}");
        assert!(
            summaries[0].message.contains("czml.untimed_sample"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
    }

    #[test]
    fn a_time_value_that_names_no_time_is_dropped_and_reported() {
        // With a non-numeric time in the group, `time_value` was the raw
        // string, so `"n/a"` went into `cartographicDegrees`, `availability`
        // and `clock.interval`. `JulianDate.fromIso8601` throws on it, so the
        // whole document failed to load — and nothing was reported.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut cache = ReprojectionCache::default();
        let good = diag_grouped_point("2024-01-01T00:00:00Z", "v1");
        let bad = diag_grouped_point("n/a", "v1");
        let features: Vec<&Feature> = vec![&good, &bad];

        let mut params = per_entity_params();
        params.group_timeseries_by = Some(Attribute::new("vehicleId"));

        let packet = build_entity_packet_next(&ctx, "v1", &features, &params, &mut cache).unwrap();

        let samples = packet["position"]["cartographicDegrees"]
            .as_array()
            .expect("a sample list");
        assert_eq!(samples.len(), 4, "only the real time survives: {samples:?}");
        assert!(
            !packet.to_string().contains("n/a"),
            "the unwritable value must not reach the packet at all: {packet}"
        );

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1, "got {summaries:?}");
        assert!(
            summaries[0].message.contains("czml.unparsable_time"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
    }

    #[test]
    fn a_time_value_is_writable_only_when_czml_can_read_it() {
        // Wider than RFC 3339 on purpose: CZML follows ISO 8601, and a viewer
        // reads a zone-less date-time as UTC and a bare date as its midnight.
        // Being stricter would drop values that reach the file and work.
        for good in [
            "2024-01-01T00:00:00Z",
            "2024-01-01T00:00:00+09:00",
            "2024-01-01T00:00:00",
            "2024-01-01T00:00:00.250",
            "2024-01-01",
        ] {
            assert!(is_writable_timestamp(good), "{good} reaches the file");
        }
        for bad in ["n/a", "", "60", "01/01/2024", "yesterday"] {
            assert!(!is_writable_timestamp(bad), "{bad} names no time");
        }

        // A number names a time only when an epoch says what it counts from.
        assert_eq!(
            writable_time_value("60", Some("2024-01-01T00:00:00Z")),
            Some(serde_json::json!("2024-01-01T00:01:00Z"))
        );
        assert_eq!(writable_time_value("60", None), None);
        assert_eq!(
            writable_time_value("n/a", Some("2024-01-01T00:00:00Z")),
            None
        );
    }

    #[test]
    fn a_numeric_time_among_timestamps_is_converted_not_dropped() {
        // The non-numeric branch catches a group that mixes the two spellings
        // too. A number still names a time when an epoch says what it counts
        // from, so converting it keeps the group whole rather than dropping
        // half of it.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut cache = ReprojectionCache::default();
        let iso = diag_grouped_point("2024-01-01T00:01:00Z", "v1");
        let numeric = diag_grouped_point("60", "v1");
        let features: Vec<&Feature> = vec![&iso, &numeric];

        let mut params = per_entity_params();
        params.group_timeseries_by = Some(Attribute::new("vehicleId"));
        params.epoch = Some("2024-01-01T00:00:00Z".to_string());

        let packet = build_entity_packet_next(&ctx, "v1", &features, &params, &mut cache).unwrap();

        let samples = packet["position"]["cartographicDegrees"]
            .as_array()
            .expect("a sample list");
        assert_eq!(samples.len(), 8, "both samples survive: {samples:?}");
        assert!(
            samples.iter().all(|v| v != "60"),
            "the numeric time must be converted, not written as itself: {samples:?}"
        );
        assert!(
            handle.inner.drain_summaries().is_empty(),
            "nothing was dropped, so nothing is reported"
        );
    }

    #[test]
    fn an_areal_entitys_dropped_time_samples_are_reported() {
        // A CZML polygon carries its own coordinates, so the entity's
        // `position` cannot move it and the collected samples are discarded.
        // That is every feature but the first, and it used to happen in
        // silence — the single-feature test that pins the polygon could not
        // show the loss.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut cache = ReprojectionCache::default();
        let first = diag_grouped_polygon("2024-01-01T00:00:00Z", "v1");
        let second = diag_grouped_polygon("2024-01-01T00:01:00Z", "v1");
        let features: Vec<&Feature> = vec![&first, &second];

        let mut params = per_entity_params();
        params.group_timeseries_by = Some(Attribute::new("vehicleId"));

        let packet = build_entity_packet_next(&ctx, "v1", &features, &params, &mut cache).unwrap();

        assert!(packet.get("polygon").is_some(), "still a polygon: {packet}");
        assert!(
            packet.get("position").is_none(),
            "the polygon places itself: {packet}"
        );

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1, "got {summaries:?}");
        assert!(
            summaries[0].message.contains("czml.areal_samples_dropped"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
    }

    #[test]
    fn an_all_untimed_groups_extra_positions_are_reported_not_silently_dropped() {
        // The static side of the same drop. A group where nothing carries a
        // time collapses to one position, so every feature after the first
        // loses one — and losing it quietly is what the time-dynamic branch
        // already refuses to do.
        let handle = make_handle();
        let ctx = context_with_handle(&handle);
        let mut cache = ReprojectionCache::default();
        let first = diag_grouped_point("", "v1");
        let second = diag_grouped_point("", "v1");
        let third = diag_grouped_point("", "v1");
        let features: Vec<&Feature> = vec![&first, &second, &third];

        let mut params = per_entity_params();
        params.group_timeseries_by = Some(Attribute::new("vehicleId"));

        let packet = build_entity_packet_next(&ctx, "v1", &features, &params, &mut cache).unwrap();

        let samples = packet["position"]["cartographicDegrees"]
            .as_array()
            .expect("a position list");
        assert_eq!(samples.len(), 3, "one position survives: {samples:?}");

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1, "got {summaries:?}");
        assert_eq!(
            summaries[0].aggregated.as_ref().unwrap().count,
            2,
            "the two features after the first are each reported: {summaries:?}"
        );
        assert!(
            summaries[0].message.contains("czml.untimed_sample"),
            "got {}",
            summaries[0].message
        );
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop),
            "a dropped position must be reported (WarnDrop), not warn()ed"
        );
    }

    #[test]
    fn an_out_of_range_numeric_time_degrades_instead_of_panicking() {
        // `timeField` is a user-chosen attribute, so its values are arbitrary.
        // A nanosecond-since-epoch timestamp, or any value past what a
        // `DateTime` can hold, used to saturate through `as i64` and panic
        // inside chrono's `DateTime + TimeDelta` — in a sink's `finish()`,
        // which takes the whole job down rather than reporting anything.
        for ts in ["1e30", "-1e30", "1731000000000000000"] {
            let out = strip_epoch_offset_for_availability(ts, Some("1970-01-01T00:00:00Z"));
            assert_eq!(out, ts, "an unrepresentable time must come back as itself");
        }

        // The representable case still converts, so the guard above cannot be
        // satisfied by giving up on every input.
        assert_eq!(
            strip_epoch_offset_for_availability("60", Some("1970-01-01T00:00:00Z")),
            "1970-01-01T00:01:00Z"
        );
    }

    #[test]
    fn an_embedded_wall_places_the_entity_through_its_own_positions() {
        // `polygon` and `polyline` are not the whole self-placing family. A
        // `preserveRaw` read of a real CZML document can carry a `wall`,
        // `corridor`, `polylineVolume` or `rectangle`, each of which carries
        // its own coordinates exactly as a polygon does, so none of them is a
        // placement failure either.
        let (doc, messages, _) = embedded_run(embedded_feature(
            "czml.wall",
            r#"{"positions":{"cartographicDegrees":[139.0,36.0,0,139.1,36.0,0]}}"#,
        ));

        assert!(
            doc.iter().any(|p| p.get("wall").is_some()),
            "the embedded wall must survive: {doc:?}"
        );
        assert!(
            messages.is_empty(),
            "a wall carries its own positions, so it is placed: {messages:?}"
        );
    }
}

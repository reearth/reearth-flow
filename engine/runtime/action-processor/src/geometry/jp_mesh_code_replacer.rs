//! Replaces a feature's geometry with the Japanese standard regional mesh cell its code names.

use std::collections::HashMap;
use std::sync::Arc;

use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
use reearth_flow_geometry::{
    coordinate::{CoordinateFrame, EpsgCode},
    point::Point2D,
    Euclidean2DGeometry, Geometry,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::jpmesh::{
    overlap::{cell_centre, cell_face},
    JPMeshCode, TenDigitReading,
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;
use super::jp_standard_grid_accumulator::CellOutput;

#[derive(Debug, Clone, Default)]
pub struct JPMeshCodeReplacerFactory;

impl ProcessorFactory for JPMeshCodeReplacerFactory {
    fn name(&self) -> &str {
        "JP Mesh Code Replacer"
    }

    fn description(&self) -> &str {
        "Replaces each feature's geometry with the Japanese standard regional mesh cell (JIS X 0410) named by an attribute, whatever geometry and coordinate system the feature had before."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JPMeshCodeReplacerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: JPMeshCodeReplacerParam = match with {
            Some(with) => serde_json::from_value(Value::Object(with.into_iter().collect()))
                .map_err(|e| {
                    GeometryProcessorError::JPMeshCodeReplacerFactory(format!(
                        "Failed to deserialize parameters: {e}"
                    ))
                })?,
            None => {
                return Err(GeometryProcessorError::JPMeshCodeReplacerFactory(
                    "Missing required parameter `meshCodeAttribute`".to_string(),
                )
                .into())
            }
        };
        let epsg = two_dimensional(params.epsg_code)
            .map_err(GeometryProcessorError::JPMeshCodeReplacerFactory)?;
        Ok(Box::new(JPMeshCodeReplacer { params, epsg }))
    }
}

/// The 2D counterpart of the coordinate system the output is asked for, since
/// the cells are flat: a 3D code comes back as its horizontal component and a
/// 2D one comes back unchanged, with the coordinate values unaffected either
/// way. A code with no 2D form is refused here rather than used to label flat
/// geometry with a vertical axis.
fn two_dimensional(code: u16) -> Result<EpsgCode, String> {
    match CoordinateFrame::Crs(EpsgCode::new(code)).demote_to_2d() {
        Ok(CoordinateFrame::Crs(epsg)) => Ok(epsg),
        // A `Crs` frame demotes to a `Crs` frame; another frame would mean that
        // contract had changed, so say so rather than guess at a code.
        Ok(other) => Err(format!("`epsgCode` {code} resolved to {other:?}")),
        Err(e) => Err(format!(
            "`epsgCode` {code} has no two-dimensional form: {e}"
        )),
    }
}

/// # JP Mesh Code Replacer Parameters
///
/// Which attribute holds the mesh code, and the geometry that replaces each feature's own.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JPMeshCodeReplacerParam {
    /// # Mesh Code Attribute
    /// Attribute holding the mesh code. Text is recommended, since a number loses leading zeros;
    /// a number is accepted only when it is a whole number.
    mesh_code_attribute: Attribute,
    /// # Output Geometry
    /// The geometry each feature receives in place of its own.
    #[serde(default)]
    output: CellOutput,
    /// # Ten-Digit Code
    /// How a 10-digit code is read, since the digits alone cannot say which mesh it names.
    #[serde(default)]
    ten_digit_code: TenDigitCode,
    /// # EPSG Code
    /// The coordinate system of the output geometry, such as 6668 (latitude and longitude) or a
    /// plane rectangular system. A 3D code is replaced by its 2D counterpart, since the output is
    /// flat: 6697 becomes 6668, and 4979 becomes 4326. The coordinate values are unaffected.
    #[serde(default = "default_epsg_code")]
    epsg_code: u16,
}

fn default_epsg_code() -> u16 {
    6668
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
enum TenDigitCode {
    /// # 1/4 Mesh
    /// A 10-digit code names a 250 m cell, ending in two digits that each run 1 to 4.
    #[default]
    QuarterMesh,
    /// # 1/10 Mesh
    /// A 10-digit code names a 100 m cell, ending in two digits that each run 0 to 9.
    TenthMesh,
}

impl From<TenDigitCode> for TenDigitReading {
    fn from(t: TenDigitCode) -> Self {
        match t {
            TenDigitCode::QuarterMesh => TenDigitReading::QuarterMesh,
            TenDigitCode::TenthMesh => TenDigitReading::TenthMesh,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JPMeshCodeReplacer {
    params: JPMeshCodeReplacerParam,
    /// `params.epsg_code` demoted to two dimensions, resolved once at build time.
    epsg: EpsgCode,
}

impl Processor for JPMeshCodeReplacer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        match self.cell(feature.attributes.get(&self.params.mesh_code_attribute)) {
            Ok(geometry) => {
                let mut out = feature.clone();
                out.geometry = Arc::new(geometry);
                fw.send(ctx.new_with_feature_and_port(out, FEATURES_PORT.clone()));
            }
            Err(code) => {
                // No `with_message`: a real run's diagnostics aggregator keeps
                // only the code, so the registry text is what users see.
                ctx.warn(DiagnosticDraft::new(code));
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "JP Mesh Code Replacer"
    }
}

impl JPMeshCodeReplacer {
    /// The cell geometry the attribute names, or the registry code saying why not.
    fn cell(&self, value: Option<&AttributeValue>) -> Result<Geometry, ErrorCode> {
        let reading: TenDigitReading = self.params.ten_digit_code.into();
        let code = match value {
            Some(AttributeValue::String(text)) => JPMeshCode::parse(text, reading),
            Some(AttributeValue::Number(n)) => {
                whole_number(n).and_then(|n| JPMeshCode::parse(&n.to_string(), reading))
            }
            _ => None,
        }
        .ok_or(ErrorCode::MeshInvalidCode)?;

        let epsg = self.epsg;
        let failed = |_| ErrorCode::MeshAssignmentFailed;
        let geometry = match self.params.output {
            CellOutput::Polygon => {
                Euclidean2DGeometry::Polygon(Box::new(cell_face(&code, epsg).map_err(failed)?))
            }
            CellOutput::Point => Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(epsg),
                cell_centre(&code, epsg).map_err(failed)?,
            )),
        };
        Ok(Geometry::Euclidean2D(geometry))
    }
}

/// A number's value when it is a non-negative whole number that a double holds
/// exactly (below 2^53), so `53394611.0` reads as `53394611`.
fn whole_number(n: &serde_json::Number) -> Option<u64> {
    if let Some(u) = n.as_u64() {
        return Some(u);
    }
    let f = n.as_f64()?;
    (f >= 0.0 && f.fract() == 0.0 && f < 9_007_199_254_740_992.0).then_some(f as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use reearth_flow_common::uri::Uri;
    use reearth_flow_geometry::ops::reproject::{transform_coords_3d, ReprojectionCache};
    use reearth_flow_geometry::polygon::Polygon2D;
    use reearth_flow_runtime::event::Event;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_runtime::kvs;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::jpmesh::overlap::containing;
    use reearth_flow_types::jpmesh::JPMeshType;
    use reearth_flow_types::{Attributes, Feature};
    use serde_json::json;

    const GEO: EpsgCode = EpsgCode::new(6668);
    const JGD9: EpsgCode = EpsgCode::new(6677);
    const PRCS: EpsgCode = EpsgCode::new(6675);

    fn feature_with(code: Option<AttributeValue>) -> Feature {
        let mut attributes = Attributes::new();
        if let Some(code) = code {
            attributes.insert(Attribute::new("code".to_string()), code);
        }
        Feature::new_with_attributes_and_geometry(attributes, Geometry::None)
    }

    fn text(s: &str) -> Option<AttributeValue> {
        Some(AttributeValue::String(s.to_string()))
    }

    fn number(n: f64) -> Option<AttributeValue> {
        Some(AttributeValue::Number(
            serde_json::Number::from_f64(n).unwrap(),
        ))
    }

    /// Run the features through one processor and return what it sent, the
    /// port of each, and the code of every diagnostic raised.
    fn replace(with: Value, features: Vec<Feature>) -> (Vec<Feature>, Vec<Port>, Vec<ErrorCode>) {
        let hub = EventHub::new(64);
        let mut rx = hub.receiver.resubscribe();
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let with: HashMap<String, Value> = serde_json::from_value(with).unwrap();
        let mut processor = JPMeshCodeReplacerFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "JP Mesh Code Replacer".to_string(),
                Some(with),
            )
            .unwrap();
        for f in features {
            let ctx = ExecutorContext::new(
                f,
                FEATURES_PORT.clone(),
                Arc::new(serde_json::Map::new()),
                Arc::new(StorageResolver::new()),
                Arc::new(kvs::create_kv_store()),
                hub.clone(),
                Uri::for_test("file:///"),
            );
            processor.process(ctx, &fw).unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();
        let ProcessorChannelForwarder::Noop(noop) = &fw else {
            unreachable!("built as a noop forwarder");
        };
        let features = noop.send_features.lock().unwrap().clone();
        let ports = noop.send_ports.lock().unwrap().clone();
        let mut warnings = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let Event::Diagnostic(diagnostic) = event {
                warnings.push(diagnostic.code);
            }
        }
        (features, ports, warnings)
    }

    fn one(with: Value, code: Option<AttributeValue>) -> (Feature, Port, Vec<ErrorCode>) {
        let (mut out, mut ports, warnings) = replace(with, vec![feature_with(code)]);
        (out.remove(0), ports.remove(0), warnings)
    }

    fn polygon(f: &Feature) -> &Polygon2D {
        let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) = &*f.geometry else {
            panic!("expected a polygon, got {:?}", f.geometry);
        };
        p
    }

    fn point(f: &Feature) -> &Point2D {
        let Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) = &*f.geometry else {
            panic!("expected a point, got {:?}", f.geometry);
        };
        p
    }

    fn code_at(position: [f64; 2], epsg: EpsgCode, t: JPMeshType) -> u64 {
        containing([position[0], position[1], 0.0], epsg, t)
            .unwrap()
            .unwrap()
            .to_number()
    }

    fn cell_number(f: &Feature, epsg: EpsgCode) -> u64 {
        let p = polygon(f).exterior();
        let n = p.len() - 1;
        let mid = [
            p[..n].iter().map(|c| c[0]).sum::<f64>() / n as f64,
            p[..n].iter().map(|c| c[1]).sum::<f64>() / n as f64,
        ];
        code_at(mid, epsg, JPMeshType::Mesh1km)
    }

    #[test]
    fn a_code_becomes_its_cell_polygon() {
        let (out, port, warnings) = one(json!({"meshCodeAttribute": "code"}), text("53394611"));
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(port, FEATURES_PORT.clone());
        let p = polygon(&out);
        assert_eq!(p.frame(), &CoordinateFrame::Crs(GEO));
        let code = JPMeshCode::parse("53394611", TenDigitReading::QuarterMesh).unwrap();
        let centre = cell_centre(&code, GEO).unwrap();
        let ring = p.exterior();
        let (lo, hi) = ring
            .iter()
            .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), c| {
                (
                    [lo[0].min(c[0]), lo[1].min(c[1])],
                    [hi[0].max(c[0]), hi[1].max(c[1])],
                )
            });
        assert!((0..2).all(|i| lo[i] < centre[i] && centre[i] < hi[i]));
        assert_eq!(cell_number(&out, GEO), 53394611);
    }

    #[test]
    fn point_output_in_6668_is_lat_lng() {
        let (out, _, warnings) = one(
            json!({"meshCodeAttribute": "code", "output": "point"}),
            text("53394611"),
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        let p = point(&out);
        assert_eq!(p.frame(), &CoordinateFrame::Crs(GEO));
        let b = JPMeshCode::parse("53394611", TenDigitReading::QuarterMesh)
            .unwrap()
            .bounds();
        let [lat, lng] = p.position();
        assert!((lat - (b.min().y + b.max().y) / 2.0).abs() < 1e-9);
        assert!((lng - (b.min().x + b.max().x) / 2.0).abs() < 1e-9);
    }

    #[test]
    fn output_in_a_projected_crs() {
        let (out, _, warnings) = one(
            json!({"meshCodeAttribute": "code", "epsgCode": 6677}),
            text("53394611"),
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(polygon(&out).frame(), &CoordinateFrame::Crs(JGD9));
        assert_eq!(cell_number(&out, JGD9), 53394611);
    }

    #[test]
    fn an_integral_float_code_is_read_as_its_integer() {
        let (out, port, warnings) = one(json!({"meshCodeAttribute": "code"}), number(53394611.0));
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(port, FEATURES_PORT.clone());
        assert_eq!(cell_number(&out, GEO), 53394611);
    }

    #[test]
    fn an_integer_number_code_is_read() {
        let (out, _, _) = one(
            json!({"meshCodeAttribute": "code"}),
            Some(AttributeValue::Number(53394611u64.into())),
        );
        assert_eq!(cell_number(&out, GEO), 53394611);
    }

    #[test]
    fn a_fractional_number_is_rejected() {
        let (out, port, warnings) = one(json!({"meshCodeAttribute": "code"}), number(5339461.5));
        assert_eq!(port, REJECTED_PORT.clone());
        assert_eq!(warnings, vec![ErrorCode::MeshInvalidCode]);
        assert!(matches!(*out.geometry, Geometry::None));
    }

    #[test]
    fn a_negative_number_is_rejected() {
        let (_, port, warnings) = one(
            json!({"meshCodeAttribute": "code"}),
            Some(AttributeValue::Number((-53394611i64).into())),
        );
        assert_eq!(port, REJECTED_PORT.clone());
        assert_eq!(warnings, vec![ErrorCode::MeshInvalidCode]);
    }

    #[test]
    fn bad_codes_are_rejected() {
        let bad = [
            None,
            text(""),
            text("5339.4611"),
            text("533986"),
            Some(AttributeValue::Bool(true)),
            Some(AttributeValue::Null),
        ];
        for code in bad {
            let label = format!("{code:?}");
            let (_, port, warnings) = one(json!({"meshCodeAttribute": "code"}), code);
            assert_eq!(port, REJECTED_PORT.clone(), "{label}");
            assert_eq!(warnings, vec![ErrorCode::MeshInvalidCode], "{label}");
        }
    }

    #[test]
    fn ten_digit_reading_is_honoured() {
        let (_, port, warnings) = one(json!({"meshCodeAttribute": "code"}), text("5339461127"));
        assert_eq!(port, REJECTED_PORT.clone());
        assert_eq!(warnings, vec![ErrorCode::MeshInvalidCode]);

        let (out, port, warnings) = one(
            json!({"meshCodeAttribute": "code", "tenDigitCode": "tenthMesh"}),
            text("5339461127"),
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(port, FEATURES_PORT.clone());
        let expected = JPMeshCode::parse("5339461127", TenDigitReading::TenthMesh).unwrap();
        let ring = polygon(&out).exterior();
        let b = expected.bounds();
        for c in ring {
            assert!(c[0] >= b.min().y - 1e-9 && c[0] <= b.max().y + 1e-9);
            assert!(c[1] >= b.min().x - 1e-9 && c[1] <= b.max().x + 1e-9);
        }
    }

    /// The error a build refuses with, or a panic naming what it built instead.
    fn build_error(with: Value) -> String {
        let with: HashMap<String, Value> = serde_json::from_value(with).unwrap();
        match JPMeshCodeReplacerFactory.build(
            NodeContext::default(),
            EventHub::new(1),
            "JP Mesh Code Replacer".to_string(),
            Some(with),
        ) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected the build to be refused"),
        }
    }

    #[test]
    fn an_unknown_epsg_code_is_refused_at_build_time() {
        let message = build_error(json!({"meshCodeAttribute": "code", "epsgCode": 1}));
        assert!(message.contains("`epsgCode` 1"), "{message}");
    }

    #[test]
    fn a_three_dimensional_epsg_code_becomes_its_two_dimensional_counterpart() {
        // EPSG:6697 is EPSG:6668 plus a height axis, so the cell is the same
        // one and only the frame on it differs.
        let (out, _, warnings) = one(
            json!({"meshCodeAttribute": "code", "epsgCode": 6697}),
            text("53394611"),
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(polygon(&out).frame(), &CoordinateFrame::Crs(GEO));
        let flat = one(
            json!({"meshCodeAttribute": "code", "epsgCode": 6668}),
            text("53394611"),
        )
        .0;
        assert_eq!(polygon(&out).exterior(), polygon(&flat).exterior());
    }

    #[test]
    fn extractor_codes_round_trip() {
        // A point in 6675 (northing, easting), taken through the lat/lng
        // mesh lookup the extractor's first-vertex mode uses.
        let mut c = [[36.654247021162256, 137.0701714804155, 0.0]];
        transform_coords_3d(&mut ReprojectionCache::new(), GEO, PRCS, &mut c).unwrap();
        let p = [c[0][0], c[0][1]];
        let code = containing([p[0], p[1], 0.0], PRCS, JPMeshType::Mesh1km)
            .unwrap()
            .unwrap();
        let (out, _, warnings) = one(
            json!({"meshCodeAttribute": "code", "epsgCode": 6675}),
            text(&code.to_number().to_string()),
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        let face = cell_face(&code, PRCS).unwrap();
        assert_eq!(polygon(&out).exterior(), face.exterior());
        let ring = face.exterior();
        let (lo, hi) = ring
            .iter()
            .fold(([f64::MAX; 2], [f64::MIN; 2]), |(lo, hi), c| {
                (
                    [lo[0].min(c[0]), lo[1].min(c[1])],
                    [hi[0].max(c[0]), hi[1].max(c[1])],
                )
            });
        assert!((0..2).all(|i| lo[i] <= p[i] && p[i] <= hi[i]));
        assert_eq!(cell_number(&out, PRCS), code.to_number());
    }

    #[test]
    fn attributes_are_kept() {
        let mut f = feature_with(text("53394611"));
        let mut attrs = (*f.attributes).clone();
        attrs.insert(
            Attribute::new("name".to_string()),
            AttributeValue::String("kept".into()),
        );
        f.attributes = Arc::new(attrs);
        let (out, _, _) = replace(json!({"meshCodeAttribute": "code"}), vec![f]);
        assert_eq!(
            out[0].attributes.get(&Attribute::new("name".to_string())),
            Some(&AttributeValue::String("kept".into()))
        );
        assert_eq!(
            out[0].attributes.get(&Attribute::new("code".to_string())),
            Some(&AttributeValue::String("53394611".into()))
        );
    }

    #[test]
    fn the_mesh_code_attribute_is_required() {
        let with: HashMap<String, Value> = serde_json::from_value(json!({})).unwrap();
        assert!(JPMeshCodeReplacerFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "JP Mesh Code Replacer".to_string(),
                Some(with),
            )
            .is_err());
    }
}

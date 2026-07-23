use std::cell::RefCell;
use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::{ConvertFrame, ReprojectionCache};
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Code, CodeType, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

/// Input port carrying base-point features in the `fromPort` base-point mode.
static BASE_POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("base-point"));
/// Output port for features that could not be reprojected.
static REJECTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("rejected"));

thread_local! {
    /// The live PROJ transform holder. Kept off the `Processor` (which must be
    /// `Send + Sync + Clone`) because it wraps a non-shareable PROJ pointer; one
    /// per worker thread, reused across features so a stable CRS pair stays warm.
    static REPROJECTION_CACHE: RefCell<ReprojectionCache> = RefCell::new(ReprojectionCache::new());
}

/// The destination coordinate frame kind.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum DestinationFrame {
    /// # CRS
    /// Reproject to a coordinate reference system identified by an EPSG code.
    Crs,
    /// # Euclidean
    /// Convert to a non-georeferenced Euclidean frame.
    Euclidean,
}

/// How coordinates bridge the Euclidean/CRS boundary. Ignored for a pure
/// CRS-to-CRS reprojection.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum BasePointMode {
    /// # As Is
    /// Reinterpret coordinate values unchanged across the boundary.
    #[default]
    AsIs,
    /// # Value
    /// Offset by a base point given as an expression evaluating to `[x, y, z]`.
    Value,
    /// # From Port
    /// Offset by a base point taken from the base-point input port, matched to
    /// each feature by a key. The base-point geometry is reprojected into the
    /// destination frame before it is applied.
    FromPort,
}

/// # Coordinate Frame Reprojector Parameters
/// Reproject geometry across coordinate reference systems and convert between a
/// CRS and a Euclidean frame. Converting across the Euclidean/CRS boundary
/// reinterprets coordinates as-is: values and ring winding are left unchanged,
/// so orientation follows the destination frame's axis order.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CoordinateFrameReprojectorParam {
    /// # Destination Frame
    /// Coordinate frame to convert geometry into.
    destination_frame: DestinationFrame,
    /// # EPSG Code
    /// EPSG code of the destination CRS. Required when the destination frame is
    /// a CRS.
    #[serde(default)]
    epsg_code: Option<u16>,
    /// # Base Point Mode
    /// How coordinates bridge the Euclidean/CRS boundary.
    #[serde(default)]
    base_point_mode: BasePointMode,
    /// # Base Point
    /// Expression evaluating to an `[x, y, z]` origin in CRS space, in the CRS's
    /// declared axis order. Used when the base point mode is `value`.
    #[serde(default)]
    base_point: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    /// # Match Key
    /// Expression identifying which base-point feature applies to a given
    /// feature. Evaluated against both streams. Used when the base point mode is
    /// `fromPort`.
    #[serde(default)]
    match_key: Option<Code<{ CodeType::FlowExpr as u32 }>>,
}

#[derive(Debug, Clone, Default)]
pub struct CoordinateFrameReprojectorFactory;

impl ProcessorFactory for CoordinateFrameReprojectorFactory {
    fn name(&self) -> &str {
        "Coordinate Frame Reprojector"
    }

    fn description(&self) -> &str {
        "Reprojects geometry between coordinate reference systems and converts between a CRS and a Euclidean frame."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CoordinateFrameReprojectorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["coordinate-system", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), BASE_POINT_PORT.clone()]
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
        let params: CoordinateFrameReprojectorParam = {
            let with = with.ok_or_else(|| {
                GeometryProcessorError::CoordinateFrameReprojectorFactory(
                    "Missing required parameter `with`".to_string(),
                )
            })?;
            let value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CoordinateFrameReprojectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CoordinateFrameReprojectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        };

        let target = match params.destination_frame {
            DestinationFrame::Crs => {
                let epsg = params.epsg_code.ok_or_else(|| {
                    GeometryProcessorError::CoordinateFrameReprojectorFactory(
                        "`epsgCode` is required when the destination frame is a CRS".to_string(),
                    )
                })?;
                CoordinateFrame::Crs(EpsgCode::new(epsg))
            }
            DestinationFrame::Euclidean => CoordinateFrame::Euclidean,
        };

        let base_point = match params.base_point_mode {
            BasePointMode::AsIs => BasePointSource::AsIs,
            BasePointMode::Value => {
                let code = params.base_point.ok_or_else(|| {
                    GeometryProcessorError::CoordinateFrameReprojectorFactory(
                        "`basePoint` is required when the base point mode is `value`".to_string(),
                    )
                })?;
                let compiled = code.compile().map_err(|e| {
                    GeometryProcessorError::CoordinateFrameReprojectorFactory(format!(
                        "Failed to compile `basePoint` expression: {e:?}"
                    ))
                })?;
                BasePointSource::Value(compiled)
            }
            BasePointMode::FromPort => {
                let key = params.match_key.ok_or_else(|| {
                    GeometryProcessorError::CoordinateFrameReprojectorFactory(
                        "`matchKey` is required when the base point mode is `fromPort`".to_string(),
                    )
                })?;
                let compiled = key.compile().map_err(|e| {
                    GeometryProcessorError::CoordinateFrameReprojectorFactory(format!(
                        "Failed to compile `matchKey` expression: {e:?}"
                    ))
                })?;
                BasePointSource::FromPort { key: compiled }
            }
        };

        Ok(Box::new(CoordinateFrameReprojector {
            target,
            base_point,
            pending: Vec::new(),
            base_points: HashMap::new(),
        }))
    }
}

/// Resolved base-point sourcing for a built processor.
#[derive(Debug, Clone)]
enum BasePointSource {
    /// No offset: coordinate values reinterpreted unchanged.
    AsIs,
    /// Offset by an expression evaluated per feature.
    Value(CompiledCode),
    /// Offset by a base point from the base-point port, matched by key.
    FromPort { key: CompiledCode },
}

/// Reprojects a feature's geometry to a target coordinate frame, bridging the
/// Euclidean/CRS boundary by a base point when configured.
#[derive(Debug, Clone)]
pub struct CoordinateFrameReprojector {
    target: CoordinateFrame,
    base_point: BasePointSource,
    /// Buffered `(key, feature)` pairs from the features port, in `fromPort` mode.
    pending: Vec<(String, Feature)>,
    /// Base points keyed by match key, collected from the base-point port.
    base_points: HashMap<String, [f64; 3]>,
}

impl CoordinateFrameReprojector {
    /// Convert a feature's geometry to `self.target`, offsetting by `base_point`
    /// across the Euclidean/CRS boundary.
    fn convert(
        &self,
        feature: &Feature,
        base_point: Option<[f64; 3]>,
    ) -> Result<Feature, BoxedError> {
        let mut feature = feature.clone();
        REPROJECTION_CACHE
            .with(|cache| {
                feature.geometry_mut().convert_frame(
                    &self.target,
                    base_point,
                    &mut cache.borrow_mut(),
                )
            })
            .map_err(|e| {
                Box::new(GeometryProcessorError::CoordinateFrameReprojector(
                    e.to_string(),
                )) as BoxedError
            })?;
        Ok(feature)
    }

    /// Reproject a base-point feature's geometry into the destination frame and
    /// return its representative `[x, y, z]`, or `None` when it is not a single
    /// point or cannot be converted. Reprojecting first removes the ambiguity of
    /// which frame the base point was authored in, so the offset is always
    /// applied in the destination frame's coordinate space.
    fn base_point_in_target(&self, geometry: &Geometry) -> Option<[f64; 3]> {
        let mut geometry = geometry.clone();
        REPROJECTION_CACHE
            .with(|cache| geometry.convert_frame(&self.target, None, &mut cache.borrow_mut()))
            .ok()?;
        representative_point(&geometry)
    }

    /// Convert `feature` and forward it to the features port, or forward the
    /// original to the rejected port on failure.
    fn convert_and_forward(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        feature: &Feature,
        base_point: Option<[f64; 3]>,
    ) {
        match self.convert(feature, base_point) {
            Ok(converted) => {
                fw.send(ctx.new_with_feature_and_port(converted, FEATURES_PORT.clone()))
            }
            Err(e) => {
                ctx.event_hub
                    .warn_log(Some(ctx.error_span()), format!("reproject failed: {e}"));
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()))
            }
        }
    }
}

impl Processor for CoordinateFrameReprojector {
    fn num_threads(&self) -> usize {
        // The `fromPort` mode correlates two streams in a single buffer, so it
        // must run on one thread; the other modes are stateless per feature.
        match self.base_point {
            BasePointSource::FromPort { .. } => 1,
            _ => 2,
        }
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match &self.base_point {
            BasePointSource::AsIs => {
                if ctx.port != *BASE_POINT_PORT {
                    let feature = ctx.feature.clone();
                    self.convert_and_forward(&ctx, fw, &feature, None);
                }
            }
            BasePointSource::Value(code) => {
                if ctx.port != *BASE_POINT_PORT {
                    let base_point = code
                        .eval(&ctx.feature, ctx.env_vars.clone())
                        .ok()
                        .as_ref()
                        .and_then(attribute_value_to_xyz);
                    let feature = ctx.feature.clone();
                    match base_point {
                        Some(bp) => self.convert_and_forward(&ctx, fw, &feature, Some(bp)),
                        None => {
                            ctx.event_hub.warn_log(
                                Some(ctx.error_span()),
                                "base point expression did not evaluate to [x, y, z]".to_string(),
                            );
                            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                        }
                    }
                }
            }
            BasePointSource::FromPort { key } => {
                let feature = &ctx.feature;
                let matched_key = key
                    .eval(feature, ctx.env_vars.clone())
                    .ok()
                    .map(|v| v.to_string());
                if ctx.port == *BASE_POINT_PORT {
                    if let (Some(matched_key), Some(point)) =
                        (matched_key, self.base_point_in_target(&feature.geometry))
                    {
                        self.base_points.insert(matched_key, point);
                    } else {
                        ctx.event_hub.warn_log(
                            Some(ctx.error_span()),
                            "base-point feature lacks a key or a point geometry; skipping"
                                .to_string(),
                        );
                    }
                } else {
                    match matched_key {
                        Some(matched_key) => self.pending.push((matched_key, feature.clone())),
                        None => fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()),
                        ),
                    }
                }
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        if !matches!(self.base_point, BasePointSource::FromPort { .. }) {
            return Ok(());
        }
        let pending = std::mem::take(&mut self.pending);
        for (matched_key, feature) in pending {
            let base_point = self.base_points.get(&matched_key).copied();
            let (port, out) = match base_point {
                Some(bp) => match self.convert(&feature, Some(bp)) {
                    Ok(converted) => (FEATURES_PORT.clone(), converted),
                    Err(e) => {
                        ctx.event_hub
                            .warn_log(None, format!("reproject failed: {e}"));
                        (REJECTED_PORT.clone(), feature)
                    }
                },
                None => (REJECTED_PORT.clone(), feature),
            };
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx, out, port,
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Coordinate Frame Reprojector"
    }
}

/// A representative `[x, y, z]` from a point geometry, or `None` when the
/// geometry is not a single point.
fn representative_point(geometry: &Geometry) -> Option<[f64; 3]> {
    match geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => Some(p.position()),
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
            let [x, y] = p.position();
            Some([x, y, 0.0])
        }
        _ => None,
    }
}

/// Parse an attribute value into an `[x, y, z]` triple. Accepts a two- or
/// three-element numeric array; a two-element array is placed at `z = 0`.
fn attribute_value_to_xyz(
    value: &reearth_flow_common::attribute::AttributeValue,
) -> Option<[f64; 3]> {
    use reearth_flow_common::attribute::AttributeValue;
    let AttributeValue::Array(items) = value else {
        return None;
    };
    if items.len() != 2 && items.len() != 3 {
        return None;
    }
    let mut out = [0.0f64; 3];
    for (slot, item) in out.iter_mut().zip(items) {
        match item {
            AttributeValue::Number(n) => *slot = n.as_f64()?,
            _ => return None,
        }
    }
    Some(out)
}

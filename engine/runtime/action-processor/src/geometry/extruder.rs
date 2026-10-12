use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{
    extrude, BaseMode, ExtrudeDirection, ExtrudeError, ExtrudeOutput, ExtrudeSpec, TopMode,
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::AttributeValue;
use reearth_flow_types::{Code, CodeType, CompiledCode};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "new-geometry")]
use super::coordinate_frame_reprojector::attribute_value_to_xyz;
use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct ExtruderFactory;

impl ProcessorFactory for ExtruderFactory {
    fn name(&self) -> &str {
        "Extruder"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn description(&self) -> &str {
        "Extrudes a polygon geometry vertically by a given distance to produce a solid geometry."
    }

    #[cfg(feature = "new-geometry")]
    fn description(&self) -> &str {
        "Sweeps each geometry along a direction: points become line segments, lines become walls, and areas become solids or open shells."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ExtruderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d"]
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
        let params: ExtruderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::ExtruderFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::ExtruderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::ExtruderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        build_extruder(params)
    }
}

#[cfg(not(feature = "new-geometry"))]
fn build_extruder(params: ExtruderParam) -> Result<Box<dyn Processor>, BoxedError> {
    let distance = params
        .distance
        .compile()
        .map_err(|e| GeometryProcessorError::ExtruderFactory(format!("{e:?}")))?;
    Ok(Box::new(Extruder { distance }))
}

#[cfg(not(feature = "new-geometry"))]
#[derive(Debug, Clone)]
pub struct Extruder {
    distance: CompiledCode,
}

/// # Extruder Parameters
/// Configure how far each polygon is extruded.
#[cfg(not(feature = "new-geometry"))]
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExtruderParam {
    /// # Distance
    /// Height to extrude the polygon by, as a constant or an expression
    /// evaluated per feature.
    distance: Code<{ CodeType::FlowExpr as u32 }>,
}

#[cfg(feature = "new-geometry")]
type Expr = Code<{ CodeType::FlowExpr as u32 }>;

/// # Extruder Parameters
/// Configure the direction each geometry is swept along and what an area becomes.
#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExtruderParam {
    /// # Direction
    /// How each geometry is swept: up the vertical axis, along a vector, or along a planar surface's normal.
    direction: DirectionParam,
    /// # Output
    /// What an area becomes: a closed solid, a shell open at the bottom, or the side walls only. Points and lines always become segments and walls.
    #[serde(default)]
    output: OutputParam,
}

#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DirectionParam {
    /// # Vertical
    /// Sweeps along the vertical axis, with a choice of where the base sits and where the top ends.
    #[serde(rename_all = "camelCase")]
    Vertical {
        /// # Base
        /// Where the bottom sits. Defaults to each vertex's own elevation.
        #[serde(default)]
        base: BaseParam,
        /// # Top
        /// Where the top ends.
        top: TopParam,
    },
    /// # Vector
    /// Sweeps every vertex by the same vector. Needs a coordinate system in linear units.
    #[serde(rename_all = "camelCase")]
    Vector {
        /// # Vector
        /// Expression evaluating to the direction `[x, y, z]` in the geometry's coordinate system.
        vector: Expr,
        /// # Length
        /// Expression evaluating to the sweep length; the vector is normalized and scaled to it, and a negative length reverses it. When omitted, the vector's own length is used.
        #[serde(default)]
        length: Option<Expr>,
    },
    /// # Surface Normal
    /// Sweeps a planar surface along its front normal, for example to thicken a wall. Needs a coordinate system in linear units.
    #[serde(rename_all = "camelCase")]
    SurfaceNormal {
        /// # Distance
        /// Expression evaluating to the distance to sweep; negative sweeps toward the back.
        distance: Expr,
    },
}

#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BaseParam {
    /// # Keep Elevation
    /// Each vertex keeps its own elevation, so a draped footprint keeps its slope.
    #[default]
    KeepElevation,
    /// # Flatten To Lowest
    /// Each connected part of the geometry flattens its base to that part's own lowest elevation.
    FlattenToLowest,
    /// # Fixed Elevation
    /// Every base vertex moves to the given elevation.
    #[serde(rename_all = "camelCase")]
    FixedElevation {
        /// # Elevation
        /// Expression evaluating to the base elevation.
        elevation: Expr,
    },
}

#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TopParam {
    /// # By Height
    /// Each vertex rises by the height from its base, so the top follows a sloped base. A negative height sweeps down.
    #[serde(rename_all = "camelCase")]
    ByHeight {
        /// # Height
        /// Expression evaluating to the height.
        height: Expr,
    },
    /// # To Elevation
    /// A flat top at the given absolute elevation.
    #[serde(rename_all = "camelCase")]
    ToElevation {
        /// # Elevation
        /// Expression evaluating to the top elevation.
        elevation: Expr,
    },
    /// # Above Lowest
    /// A flat top the given height above the lowest base vertex of each connected part.
    #[serde(rename_all = "camelCase")]
    AboveLowest {
        /// # Height
        /// Expression evaluating to the height above the lowest base vertex of each connected part.
        height: Expr,
    },
    /// # Above Highest
    /// A flat top the given height above the highest base vertex of each connected part.
    #[serde(rename_all = "camelCase")]
    AboveHighest {
        /// # Height
        /// Expression evaluating to the height above the highest base vertex of each connected part.
        height: Expr,
    },
}

#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OutputParam {
    /// # Solid
    /// A closed solid with bottom, top and walls.
    #[default]
    Solid,
    /// # Open Bottom
    /// The top and walls, open at the bottom.
    OpenBottom,
    /// # Walls Only
    /// The side walls only.
    WallsOnly,
}

#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
enum Base {
    Keep,
    FlattenToLowest,
    Fixed(CompiledCode),
}

#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
enum Top {
    ByHeight(CompiledCode),
    ToElevation(CompiledCode),
    AboveLowest(CompiledCode),
    AboveHighest(CompiledCode),
}

#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
enum Direction {
    Vertical {
        base: Base,
        top: Top,
    },
    Vector {
        vector: CompiledCode,
        length: Option<CompiledCode>,
    },
    SurfaceNormal {
        distance: CompiledCode,
    },
}

#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
pub struct Extruder {
    direction: Direction,
    output: ExtrudeOutput,
}

#[cfg(feature = "new-geometry")]
fn build_extruder(params: ExtruderParam) -> Result<Box<dyn Processor>, BoxedError> {
    let compile = |name: &str, code: &Expr| {
        code.compile().map_err(|e| {
            GeometryProcessorError::ExtruderFactory(format!(
                "`{name}` expression does not compile: {e:?}"
            ))
        })
    };
    let direction = match &params.direction {
        DirectionParam::Vertical { base, top } => Direction::Vertical {
            base: match base {
                BaseParam::KeepElevation => Base::Keep,
                BaseParam::FlattenToLowest => Base::FlattenToLowest,
                BaseParam::FixedElevation { elevation } => {
                    Base::Fixed(compile("elevation", elevation)?)
                }
            },
            top: match top {
                TopParam::ByHeight { height } => Top::ByHeight(compile("height", height)?),
                TopParam::ToElevation { elevation } => {
                    Top::ToElevation(compile("elevation", elevation)?)
                }
                TopParam::AboveLowest { height } => Top::AboveLowest(compile("height", height)?),
                TopParam::AboveHighest { height } => Top::AboveHighest(compile("height", height)?),
            },
        },
        DirectionParam::Vector { vector, length } => Direction::Vector {
            vector: compile("vector", vector)?,
            length: length.as_ref().map(|l| compile("length", l)).transpose()?,
        },
        DirectionParam::SurfaceNormal { distance } => Direction::SurfaceNormal {
            distance: compile("distance", distance)?,
        },
    };
    let output = match params.output {
        OutputParam::Solid => ExtrudeOutput::Solid,
        OutputParam::OpenBottom => ExtrudeOutput::OpenBottom,
        OutputParam::WallsOnly => ExtrudeOutput::WallsOnly,
    };
    Ok(Box::new(Extruder { direction, output }))
}

/// Why a parameter did not yield a usable number.
#[cfg(feature = "new-geometry")]
enum ValueError {
    /// The expression itself failed: `expr.evaluation_failed`.
    Evaluation(String),
    /// It evaluated, but to null, NaN, Inf or a non-number.
    Invalid,
}

#[cfg(feature = "new-geometry")]
impl Extruder {
    fn number(code: &CompiledCode, ctx: &ExecutorContext) -> Result<f64, ValueError> {
        let value = code
            .eval(&ctx.feature, ctx.variables.clone())
            .map_err(|e| ValueError::Evaluation(e.to_string()))?;
        match &value {
            AttributeValue::Number(n) => n.as_f64(),
            _ => None,
        }
        .filter(|x| x.is_finite())
        .ok_or(ValueError::Invalid)
    }

    fn spec_for(&self, ctx: &ExecutorContext) -> Result<ExtrudeSpec, ValueError> {
        let direction = match &self.direction {
            Direction::Vertical { base, top } => ExtrudeDirection::Vertical {
                base: match base {
                    Base::Keep => BaseMode::KeepElevation,
                    Base::FlattenToLowest => BaseMode::FlattenToLowest,
                    Base::Fixed(c) => BaseMode::FixedElevation(Self::number(c, ctx)?),
                },
                top: match top {
                    Top::ByHeight(c) => TopMode::ByHeight(Self::number(c, ctx)?),
                    Top::ToElevation(c) => TopMode::ToElevation(Self::number(c, ctx)?),
                    Top::AboveLowest(c) => TopMode::AboveLowest(Self::number(c, ctx)?),
                    Top::AboveHighest(c) => TopMode::AboveHighest(Self::number(c, ctx)?),
                },
            },
            Direction::Vector { vector, length } => {
                let value = vector
                    .eval(&ctx.feature, ctx.variables.clone())
                    .map_err(|e| ValueError::Evaluation(e.to_string()))?;
                let vector = Some(&value)
                    .filter(|v| matches!(v, AttributeValue::Array(a) if a.len() == 3))
                    .and_then(attribute_value_to_xyz)
                    .filter(|v| v.iter().all(|c| c.is_finite()))
                    .ok_or(ValueError::Invalid)?;
                let length = length.as_ref().map(|c| Self::number(c, ctx)).transpose()?;
                ExtrudeDirection::Vector { vector, length }
            }
            Direction::SurfaceNormal { distance } => ExtrudeDirection::SurfaceNormal {
                distance: Self::number(distance, ctx)?,
            },
        };
        Ok(ExtrudeSpec {
            direction,
            output: self.output,
        })
    }

    fn reject(ctx: &ExecutorContext, fw: &ProcessorChannelForwarder, why: &str) {
        ctx.event_hub
            .debug_log(Some(ctx.error_span()), format!("extrude rejected: {why}"));
        fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
    }
}

#[cfg(feature = "new-geometry")]
fn code_for(e: &ExtrudeError) -> ErrorCode {
    match e {
        ExtrudeError::UnsupportedType(_) => ErrorCode::GeometryExtrudeUnsupportedType,
        ExtrudeError::InvalidSurface(_) => ErrorCode::GeometryExtrudeInvalidSurface,
        ExtrudeError::Degenerate(_) => ErrorCode::GeometryExtrudeDegenerate,
        ExtrudeError::UnsupportedFrame(_) => ErrorCode::GeometryExtrudeUnsupportedFrame,
    }
}

impl Processor for Extruder {
    fn num_threads(&self) -> usize {
        2
    }

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let spec = match self.spec_for(&ctx) {
            Ok(spec) => spec,
            Err(ValueError::Evaluation(why)) => {
                // Fatal by default; a relaxed policy still accounts for the feature.
                ctx.report(
                    DiagnosticDraft::new(ErrorCode::ExprEvaluationFailed).with_message(why.clone()),
                )?;
                Self::reject(&ctx, fw, &why);
                return Ok(());
            }
            Err(ValueError::Invalid) => {
                ctx.warn(DiagnosticDraft::new(ErrorCode::GeometryExtrudeInvalidValue));
                Self::reject(&ctx, fw, "a value is missing or not a number");
                return Ok(());
            }
        };
        match extrude(&ctx.feature.geometry, &spec) {
            Ok(geometry) => {
                let mut feature = ctx.feature.clone();
                feature.set_geometry(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Err(e) => {
                ctx.warn(DiagnosticDraft::new(code_for(&e)));
                Self::reject(&ctx, fw, &e.to_string());
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

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let height = self
            .distance
            .eval_float(feature, ctx.variables.clone())
            .map_err(|e| {
                GeometryProcessorError::Extruder(format!("Failed to evaluate distance: {e:?}"))
            })?;

        // A feature this action cannot extrude is routed to `rejected` rather
        // than failing the job: one stray geometry should not stop the run.
        // A polygon with no elevation is rejected too — extruding it would have
        // to invent a base elevation. Use a Three Dimension Forcer upstream to
        // set that base deliberately.
        let reject = |reason: &str| {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                format!("extrude rejected: {reason}"),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
        };

        let geom_inner = (*feature.geometry).clone();
        let GeometryValue::FlowGeometry3D(flow_geometry) = &geom_inner.value else {
            reject("geometry is absent or carries no elevation");
            return Ok(());
        };
        let FlowGeometry3D::Polygon(polygon) = flow_geometry else {
            reject("geometry is not a polygon");
            return Ok(());
        };
        let solid = polygon.extrude(height);
        let geometry = Geometry {
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::Solid(solid)),
            ..geom_inner
        };
        let mut feature = feature.clone();
        feature.geometry = Arc::new(geometry);
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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

    fn name(&self) -> &str {
        "Extruder"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_tests {
    use std::sync::Arc;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_diagnostics::ErrorCode;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::point_cloud::PointCloud;
    use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::event::Event;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_runtime::kvs;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::{AttributeValue, Feature};
    use serde_json::{json, Value};

    use super::*;

    fn footprint() -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                [
                    [0.0, 0.0],
                    [10.0, 0.0],
                    [10.0, 10.0],
                    [0.0, 10.0],
                    [0.0, 0.0],
                ],
                std::iter::empty::<Vec<[f64; 2]>>(),
            ),
        )))
    }

    fn by_height_attr() -> Value {
        json!({"direction": {"type": "vertical",
            "top": {"type": "byHeight", "height": {"type": "flowExpr", "value": "attributes.get(\"h\")"}}}})
    }

    /// Run `with` over `features`; return (port, feature) pairs and warned codes.
    fn run(with: Value, features: Vec<Feature>) -> (Vec<(Port, Feature)>, Vec<ErrorCode>) {
        let hub = EventHub::new(64);
        let mut rx = hub.receiver.resubscribe();
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let with = serde_json::from_value(with).unwrap();
        let mut processor: Box<dyn Processor> = ExtruderFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "Extruder".to_string(),
                Some(with),
            )
            .unwrap();
        for feature in features {
            let ctx = ExecutorContext::new(
                feature,
                FEATURES_PORT.clone(),
                Arc::new(serde_json::Map::new()),
                Arc::new(StorageResolver::new()),
                Arc::new(kvs::create_kv_store()),
                hub.clone(),
                Uri::for_test("file:///"),
            );
            processor.process(ctx, &fw).unwrap();
        }
        let ProcessorChannelForwarder::Noop(noop) = &fw else {
            unreachable!("built as a noop forwarder");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let sent = noop.send_features.lock().unwrap().clone();
        let mut codes = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let Event::Diagnostic(d) = event {
                codes.push(d.code);
            }
        }
        (ports.into_iter().zip(sent).collect(), codes)
    }

    fn with_height(h: AttributeValue) -> Feature {
        let mut f = Feature::from(footprint());
        f.insert("h", h);
        f.insert("buildingId", AttributeValue::String("b1".into()));
        f
    }

    #[test]
    fn footprint_with_height_attribute_becomes_a_solid_and_keeps_attributes() {
        let (out, codes) = run(
            by_height_attr(),
            vec![with_height(AttributeValue::Number(12.into()))],
        );
        assert!(codes.is_empty(), "{codes:?}");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, FEATURES_PORT.clone());
        assert!(matches!(
            &*out[0].1.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Solid(_))
        ));
        assert_eq!(
            out[0].1.get("buildingId"),
            Some(&AttributeValue::String("b1".into()))
        );
    }

    #[test]
    fn a_null_height_is_rejected_with_invalid_value_not_fatal() {
        let (out, codes) = run(by_height_attr(), vec![with_height(AttributeValue::Null)]);
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeInvalidValue]);
    }

    #[test]
    fn a_zero_height_is_degenerate() {
        let (out, codes) = run(
            by_height_attr(),
            vec![with_height(AttributeValue::Number(0.into()))],
        );
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeDegenerate]);
    }

    #[test]
    fn a_point_cloud_is_an_unsupported_type() {
        let cloud = PointCloud::from_positions(CoordinateFrame::Euclidean, vec![[0.0, 0.0, 0.0]]);
        let f = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(
            Box::new(cloud),
        )));
        let (out, codes) = run(
            json!({"direction": {"type": "vertical",
                "top": {"type": "byHeight", "height": {"type": "flowExpr", "value": "3"}}}}),
            vec![f],
        );
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeUnsupportedType]);
    }

    #[test]
    fn a_vector_on_degrees_is_an_unsupported_frame() {
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Crs(EpsgCode::new(4326)),
                [
                    [139.0, 35.0],
                    [139.001, 35.0],
                    [139.001, 35.001],
                    [139.0, 35.0],
                ],
                std::iter::empty::<Vec<[f64; 2]>>(),
            ),
        )));
        let (out, codes) = run(
            json!({"direction": {"type": "vector", "vector": {"type": "flowExpr", "value": "[0, 0, 1]"}}}),
            vec![Feature::from(g)],
        );
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeUnsupportedFrame]);
    }

    #[test]
    fn every_mode_deserializes_and_runs() {
        let params = [
            json!({"direction": {"type": "vertical", "base": {"type": "flattenToLowest"},
                "top": {"type": "toElevation", "elevation": {"type": "flowExpr", "value": "20"}}}}),
            json!({"direction": {"type": "vertical",
                "base": {"type": "fixedElevation", "elevation": {"type": "flowExpr", "value": "1"}},
                "top": {"type": "aboveLowest", "height": {"type": "flowExpr", "value": "5"}}}, "output": "openBottom"}),
            json!({"direction": {"type": "vertical",
                "top": {"type": "aboveHighest", "height": {"type": "flowExpr", "value": "5"}}}, "output": "wallsOnly"}),
            json!({"direction": {"type": "vector", "vector": {"type": "flowExpr", "value": "[1, 0, 1]"}, "length": {"type": "flowExpr", "value": "2"}}}),
            json!({"direction": {"type": "surfaceNormal", "distance": {"type": "flowExpr", "value": "0.5"}}}),
        ];
        let expected_solid = [true, false, false, true, true];
        let outputs = ["solid", "openBottom", "wallsOnly", "solid", "solid"];
        for ((with, solid), output) in params.into_iter().zip(expected_solid).zip(outputs) {
            let (out, codes) = run(with.clone(), vec![Feature::from(footprint())]);
            assert!(codes.is_empty(), "{with}: {codes:?}");
            assert_eq!(out[0].0, FEATURES_PORT.clone(), "{with}");
            match &*out[0].1.geometry {
                Geometry::Euclidean3D(Euclidean3DGeometry::Solid(_)) => {
                    assert!(solid, "{output}: unexpected solid")
                }
                Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(_)) => {
                    assert!(!solid, "{output}: unexpected mesh")
                }
                other => panic!("{output}: unexpected geometry {other:?}"),
            }
        }
    }

    #[test]
    fn a_string_height_is_invalid_not_parsed() {
        let (out, codes) = run(
            by_height_attr(),
            vec![with_height(AttributeValue::String("12".into()))],
        );
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeInvalidValue]);
    }

    #[test]
    fn a_two_element_vector_is_invalid() {
        let (out, codes) = run(
            json!({"direction": {"type": "vector",
                "vector": {"type": "flowExpr", "value": "[0, 1]"}}}),
            vec![Feature::from(footprint())],
        );
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeInvalidValue]);
    }

    #[test]
    fn an_attribute_driven_vector_extrudes() {
        let mut f = Feature::from(footprint());
        f.insert(
            "dir",
            AttributeValue::Array(vec![
                AttributeValue::Number(0.into()),
                AttributeValue::Number(0.into()),
                AttributeValue::Number(1.into()),
            ]),
        );
        let (out, codes) = run(
            json!({"direction": {"type": "vector",
                "vector": {"type": "flowExpr", "value": "attributes.get(\"dir\")"}}}),
            vec![f],
        );
        assert!(codes.is_empty(), "{codes:?}");
        assert_eq!(out[0].0, FEATURES_PORT.clone());
        assert!(matches!(
            &*out[0].1.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Solid(_))
        ));
    }

    #[test]
    fn a_non_planar_surface_normal_is_an_invalid_surface() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                [
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                std::iter::empty::<Vec<[f64; 3]>>(),
            ),
        )));
        let (out, codes) = run(
            json!({"direction": {"type": "surfaceNormal",
                "distance": {"type": "flowExpr", "value": "1"}}}),
            vec![Feature::from(g)],
        );
        assert_eq!(out[0].0, REJECTED_PORT.clone());
        assert_eq!(codes, vec![ErrorCode::GeometryExtrudeInvalidSurface]);
    }

    #[test]
    fn an_expression_that_fails_at_runtime_is_fatal_by_default() {
        // No diagnostics handle is attached, so `ctx.report` falls back to the
        // code's default disposition for `expr.evaluation_failed`: fatal.
        let hub = EventHub::new(64);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let with = serde_json::from_value(json!({"direction": {"type": "vertical",
            "top": {"type": "byHeight",
                "height": {"type": "flowExpr", "value": "\"a\" * 2"}}}}))
        .unwrap();
        let mut processor = ExtruderFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "Extruder".to_string(),
                Some(with),
            )
            .unwrap();
        let ctx = ExecutorContext::new(
            Feature::from(footprint()),
            FEATURES_PORT.clone(),
            Arc::new(serde_json::Map::new()),
            Arc::new(StorageResolver::new()),
            Arc::new(kvs::create_kv_store()),
            hub,
            Uri::for_test("file:///"),
        );
        assert!(processor.process(ctx, &fw).is_err());
    }
}

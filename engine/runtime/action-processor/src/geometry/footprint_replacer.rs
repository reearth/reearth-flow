use std::collections::HashMap;

use once_cell::sync::Lazy;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::{
    algorithm::{area2d::Area2D, bool_ops::BooleanOps},
    types::{
        coordinate::Coordinate2D,
        geometry::{Geometry2D, Geometry3D},
        line_string::LineString2D,
        multi_polygon::MultiPolygon2D,
        polygon::{Polygon2D, Polygon3D},
    },
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{CityGmlGeometry, Feature, GeometryValue};
use serde_json::Value;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{FootprintError, FootprintPlane};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::{Code, CodeType, CompiledCode};
#[cfg(feature = "new-geometry")]
use schemars::JsonSchema;
#[cfg(feature = "new-geometry")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "new-geometry")]
use super::coordinate_frame_reprojector::attribute_value_to_xyz;
#[cfg(feature = "new-geometry")]
use super::errors::GeometryProcessorError;

/// The plane the footprint is projected onto.
#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ProjectionPlane {
    /// # Horizontal
    /// Projects along the vertical axis onto the horizontal plane, dropping
    /// height. The footprint stays in the geometry's horizontal coordinate
    /// system.
    #[default]
    Horizontal,
    /// # Custom Plane
    /// Projects along a plane's normal onto that plane. The footprint is
    /// expressed in in-plane coordinates anchored at the plane origin.
    #[serde(rename_all = "camelCase")]
    Custom {
        /// # Normal
        /// Expression evaluating to the plane normal `[x, y, z]` in the
        /// geometry's coordinate frame; any non-zero length.
        normal: Code<{ CodeType::FlowExpr as u32 }>,
        /// # Origin
        /// Expression evaluating to the plane origin `[x, y, z]` in the
        /// geometry's coordinate frame. Defaults to `[0, 0, 0]`.
        #[serde(default)]
        origin: Option<Code<{ CodeType::FlowExpr as u32 }>>,
        /// # X Axis
        /// Expression evaluating to a direction `[x, y, z]` whose in-plane
        /// component becomes the footprint's x axis. When omitted, the
        /// footprint's y axis is the in-plane direction closest to vertical.
        #[serde(default)]
        x_axis: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    },
}

/// # Footprint Replacer Parameters
/// Configure the plane the geometry is projected onto. The geometry must be in a
/// Euclidean frame or a coordinate reference system in linear units; faces
/// smaller than 1e-6 square units after projection are dropped.
#[cfg(feature = "new-geometry")]
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FootprintReplacerParam {
    /// # Projection Plane
    /// Plane the geometry is projected onto. Defaults to the horizontal plane.
    #[serde(default)]
    pub projection_plane: ProjectionPlane,
}

pub static FOOTPRINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("footprint"));

#[derive(Debug, Clone, Default)]
pub struct FootprintReplacerFactory;

impl ProcessorFactory for FootprintReplacerFactory {
    fn name(&self) -> &str {
        "Footprint Replacer"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn description(&self) -> &str {
        "Replaces a feature's 3D geometry with its 2D footprint projected onto the XY plane."
    }

    #[cfg(feature = "new-geometry")]
    fn description(&self) -> &str {
        "Replaces a feature's geometry with its footprint: the dissolved projection of its faces onto the horizontal plane or a custom plane."
    }

    #[cfg(not(feature = "new-geometry"))]
    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    #[cfg(feature = "new-geometry")]
    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FootprintReplacerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FOOTPRINT_PORT.clone(), REJECTED_PORT.clone()]
    }

    #[cfg(not(feature = "new-geometry"))]
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(FootprintReplacer))
    }

    #[cfg(feature = "new-geometry")]
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: FootprintReplacerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::FootprintReplacerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::FootprintReplacerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            FootprintReplacerParam::default()
        };
        let compile = |name: &str, code: Code<{ CodeType::FlowExpr as u32 }>| {
            code.compile().map_err(|e| {
                GeometryProcessorError::FootprintReplacerFactory(format!(
                    "Failed to compile `{name}` expression: {e:?}"
                ))
            })
        };
        let plane = match param.projection_plane {
            ProjectionPlane::Horizontal => PlaneSource::Horizontal,
            ProjectionPlane::Custom {
                normal,
                origin,
                x_axis,
            } => PlaneSource::Custom {
                normal: compile("normal", normal)?,
                origin: origin.map(|code| compile("origin", code)).transpose()?,
                x_axis: x_axis.map(|code| compile("xAxis", code)).transpose()?,
            },
        };
        Ok(Box::new(FootprintReplacer { plane }))
    }
}

/// The projection plane of a built processor.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
enum PlaneSource {
    /// The horizontal plane of the geometry's frame.
    Horizontal,
    /// A plane given per feature by expressions.
    Custom {
        normal: CompiledCode,
        origin: Option<CompiledCode>,
        x_axis: Option<CompiledCode>,
    },
}

/// Replaces a feature's geometry with its footprint on a plane.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
pub struct FootprintReplacer {
    plane: PlaneSource,
}

#[cfg(feature = "new-geometry")]
impl FootprintReplacer {
    /// The plane for `ctx`'s feature, or why its expressions did not yield one.
    fn plane_for(&self, ctx: &ExecutorContext) -> Result<FootprintPlane, String> {
        let PlaneSource::Custom {
            normal,
            origin,
            x_axis,
        } = &self.plane
        else {
            return Ok(FootprintPlane::Horizontal);
        };
        let eval = |name: &str, code: &CompiledCode| {
            let value = code
                .eval(&ctx.feature, ctx.variables.clone())
                .map_err(|e| format!("`{name}` expression failed to evaluate: {e}"))?;
            attribute_value_to_xyz(&value)
                .ok_or_else(|| format!("`{name}` expression did not evaluate to [x, y, z]"))
        };
        Ok(FootprintPlane::Normal {
            normal: eval("normal", normal)?,
            origin: origin
                .as_ref()
                .map(|code| eval("origin", code))
                .transpose()?
                .unwrap_or([0.0; 3]),
            x_axis: x_axis
                .as_ref()
                .map(|code| eval("xAxis", code))
                .transpose()?,
        })
    }
}

#[cfg(not(feature = "new-geometry"))]
#[derive(Debug, Clone)]
pub struct FootprintReplacer;

impl Processor for FootprintReplacer {
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let plane = match self.plane_for(&ctx) {
            Ok(plane) => plane,
            Err(why) => {
                ctx.event_hub
                    .warn_log(Some(ctx.error_span()), format!("footprint rejected: {why}"));
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };
        match ctx.feature.geometry.footprint_on(&plane) {
            Ok(geometry) => {
                let mut feature = ctx.feature.clone();
                feature.set_geometry(geometry);
                fw.send(ctx.new_with_feature_and_port(feature, FOOTPRINT_PORT.clone()));
            }
            Err(e) => {
                let message = format!("footprint rejected: {e}");
                match e {
                    FootprintError::Empty | FootprintError::Unsupported(_) => {
                        ctx.event_hub.debug_log(Some(ctx.error_span()), message)
                    }
                    _ => ctx.event_hub.warn_log(Some(ctx.error_span()), message),
                }
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(&mut self, _: NodeContext, _: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
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
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry3D(geom) => {
                if let Some(footprint) = create_footprint_from_geometry3d(feature, geom) {
                    fw.send(ctx.new_with_feature_and_port(footprint, FOOTPRINT_PORT.clone()));
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            GeometryValue::CityGmlGeometry(citygml) => {
                if let Some(footprint) = create_footprint_from_citygml(feature, citygml) {
                    fw.send(ctx.new_with_feature_and_port(footprint, FOOTPRINT_PORT.clone()));
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(&mut self, _: NodeContext, _: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Footprint Replacer"
    }
}

/// Extract 3D polygons from a Geometry3D, handling solids, surfaces, and other geometry types.
#[cfg(not(feature = "new-geometry"))]
fn extract_polygons_from_geometry3d(geom: &Geometry3D<f64>) -> Vec<Polygon3D<f64>> {
    match geom {
        Geometry3D::Polygon(poly) => vec![poly.clone()],
        Geometry3D::MultiPolygon(mpoly) => mpoly.0.clone(),
        Geometry3D::Solid(solid) => {
            // Extract all faces from the solid and convert to polygons
            solid
                .all_faces()
                .into_iter()
                .map(|face| {
                    let coords = face.0;
                    Polygon3D::new(
                        reearth_flow_geometry::types::line_string::LineString3D::new(coords),
                        vec![],
                    )
                })
                .collect()
        }
        Geometry3D::Triangle(triangle) => {
            // Convert triangle to polygon
            let arr = triangle.to_array();
            let coords = vec![arr[0], arr[1], arr[2], arr[0]];
            vec![Polygon3D::new(
                reearth_flow_geometry::types::line_string::LineString3D::new(coords),
                vec![],
            )]
        }
        Geometry3D::GeometryCollection(gc) => {
            // Recursively extract polygons from geometry collection
            gc.iter()
                .flat_map(extract_polygons_from_geometry3d)
                .collect()
        }
        _ => vec![],
    }
}

/// Create footprint from FlowGeometry3D
#[cfg(not(feature = "new-geometry"))]
fn create_footprint_from_geometry3d(feature: &Feature, geom: &Geometry3D<f64>) -> Option<Feature> {
    let polygons = extract_polygons_from_geometry3d(geom);

    if polygons.is_empty() {
        return None;
    }

    create_footprint_from_polygons(feature, &polygons)
}

/// Create footprint from CityGML geometry
#[cfg(not(feature = "new-geometry"))]
fn create_footprint_from_citygml(feature: &Feature, citygml: &CityGmlGeometry) -> Option<Feature> {
    // Collect all polygons from all GML geometries
    let polygons: Vec<Polygon3D<f64>> = citygml
        .gml_geometries
        .iter()
        .flat_map(|gml_geom| gml_geom.polygons.clone())
        .collect();

    if polygons.is_empty() {
        return None;
    }

    create_footprint_from_polygons(feature, &polygons)
}

/// Project a 3D polygon to the XY plane (drop Z coordinate)
#[cfg(not(feature = "new-geometry"))]
fn project_polygon_to_2d(polygon: &Polygon3D<f64>) -> Polygon2D<f64> {
    let exterior: Vec<Coordinate2D<f64>> = polygon
        .exterior()
        .coords()
        .map(|c| Coordinate2D::new_(c.x, c.y))
        .collect();

    let interiors: Vec<LineString2D<f64>> = polygon
        .interiors()
        .iter()
        .map(|interior| {
            let coords: Vec<Coordinate2D<f64>> = interior
                .coords()
                .map(|c| Coordinate2D::new_(c.x, c.y))
                .collect();
            LineString2D::new(coords)
        })
        .collect();

    Polygon2D::new(LineString2D::new(exterior), interiors)
}

/// Minimum area threshold for projected polygons.  Polygons whose XY
/// projection has an area smaller than this are considered degenerate
/// (e.g. vertical wall faces) and are excluded from the footprint.
#[cfg(not(feature = "new-geometry"))]
const MIN_PROJECTED_AREA: f64 = 1e-6;

/// Create footprint from a collection of 3D polygons
#[cfg(not(feature = "new-geometry"))]
fn create_footprint_from_polygons(
    feature: &Feature,
    polygons: &[Polygon3D<f64>],
) -> Option<Feature> {
    let mut projected_polygons = Vec::new();

    // Project each polygon to the XY plane
    for polygon in polygons {
        let projected = project_polygon_to_2d(polygon);

        // Skip degenerate polygons (less than 3 points in exterior)
        if projected.exterior().0.len() < 3 {
            continue;
        }

        // Skip near-zero-area projections (e.g. vertical wall faces)
        if projected.unsigned_area2d() < MIN_PROJECTED_AREA {
            continue;
        }

        projected_polygons.push(projected);
    }

    if projected_polygons.is_empty() {
        return None;
    }

    // Union all projected polygons in a single pass.
    // Splitting into two MultiPolygons and calling union() once lets i_overlay's
    // sweep-line dissolve all overlaps in one pass with the NonZero fill rule,
    // instead of doing N-1 sequential union operations that accumulate vertex noise.
    let combined_polygons = if projected_polygons.len() == 1 {
        MultiPolygon2D::new(projected_polygons)
    } else {
        let mid = projected_polygons.len() / 2;
        let group_b = MultiPolygon2D::new(projected_polygons.split_off(mid));
        let group_a = MultiPolygon2D::new(projected_polygons);
        group_a.union(&group_b)
    };

    let mut result_feature = feature.clone();
    result_feature.geometry_mut().value =
        GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(combined_polygons));

    Some(result_feature)
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::{BaseFrame, CoordinateFrame, EpsgCode, TangentPlane};
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::solid::{Shell, Solid};
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3DData;
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};
    use serde_json::json;

    /// A closed triangle shell over the axis-aligned box `[min, min + size]`.
    fn box_shell(min: [f64; 3], size: [f64; 3]) -> TriangularMesh3DData {
        let corners: Vec<[f64; 3]> = (0..8u32)
            .map(|i| {
                [
                    min[0] + if i & 1 != 0 { size[0] } else { 0.0 },
                    min[1] + if i & 2 != 0 { size[1] } else { 0.0 },
                    min[2] + if i & 4 != 0 { size[2] } else { 0.0 },
                ]
            })
            .collect();
        #[rustfmt::skip]
        const TRIS: [u32; 36] = [
            0, 1, 3,  0, 3, 2,
            4, 7, 5,  4, 6, 7,
            0, 4, 5,  0, 5, 1,
            2, 3, 7,  2, 7, 6,
            0, 2, 6,  0, 6, 4,
            1, 5, 7,  1, 7, 3,
        ];
        TriangularMesh3DData::from_parts(corners, TRIS).unwrap()
    }

    fn box_solid(frame: CoordinateFrame, min: [f64; 3], size: [f64; 3]) -> Geometry {
        let solid = Solid::from_exterior(frame, Shell::TriangularMesh(box_shell(min, size)));
        Geometry::Euclidean3D(Euclidean3DGeometry::Solid(Box::new(solid)))
    }

    fn wall() -> Geometry {
        let face = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [
                [0.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [3.0, 0.0, 5.0],
                [0.0, 0.0, 5.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(face)))
    }

    /// A feature carrying `geometry` and one attribute to trace through.
    fn feature(geometry: Geometry) -> Feature {
        let mut feature = Feature::from(geometry);
        feature.insert("buildingId", AttributeValue::Number(7.into()));
        feature
    }

    fn build(with: Option<Value>) -> Box<dyn Processor> {
        let with = with.map(|value| serde_json::from_value(value).unwrap());
        FootprintReplacerFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "Footprint Replacer".to_string(),
                with,
            )
            .unwrap()
    }

    /// Run `processor` over `feature`, returning what it sent, port by port.
    fn run(processor: &mut dyn Processor, feature: &Feature) -> Vec<(Port, Feature)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        processor
            .process(create_default_execute_context(feature), &fw)
            .unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    fn polygon(feature: &Feature) -> reearth_flow_geometry::polygon::Polygon2D {
        match &*feature.geometry {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => (**p).clone(),
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    #[test]
    fn defaults_to_the_horizontal_footprint() {
        let frame = CoordinateFrame::Crs(EpsgCode::new(6677));
        let input = feature(box_solid(frame, [10.0, 20.0, 30.0], [4.0, 3.0, 2.0]));
        let sent = run(&mut *build(None), &input);
        assert_eq!(sent.len(), 1);
        let (port, out) = &sent[0];
        assert_eq!(port.to_string(), "footprint");
        let footprint = polygon(out);
        assert!((footprint.area() - 12.0).abs() < 1e-9);
        assert_eq!(
            footprint.frame(),
            &CoordinateFrame::Crs(EpsgCode::new(6677))
        );
        assert_eq!(
            out.attributes.get(&Attribute::new("buildingId")),
            Some(&AttributeValue::Number(7.into()))
        );
        assert_eq!(out.id, input.id);
    }

    #[test]
    fn a_custom_plane_comes_from_per_feature_expressions() {
        let mut input = feature(box_solid(
            CoordinateFrame::Euclidean,
            [1.0, 2.0, 3.0],
            [4.0, 3.0, 2.0],
        ));
        input.insert(
            "facing",
            AttributeValue::Array(vec![
                AttributeValue::Number(1.into()),
                AttributeValue::Number(0.into()),
                AttributeValue::Number(0.into()),
            ]),
        );
        let mut processor = build(Some(json!({
            "projectionPlane": {
                "type": "custom",
                "normal": { "type": "flowExpr", "value": "attributes.get(\"facing\")" },
                "origin": { "type": "flowExpr", "value": "[0, 2, 3]" },
            }
        })));
        let sent = run(&mut *processor, &input);
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0.to_string(), "footprint");
        let footprint = polygon(&sent[0].1);
        let plane =
            TangentPlane::from_normal(BaseFrame::Euclidean, [0.0, 2.0, 3.0], [1.0, 0.0, 0.0], None)
                .unwrap();
        assert_eq!(
            footprint.frame(),
            &CoordinateFrame::Tangent(Box::new(plane))
        );
        // Seen along +x the box is its y-z extent, 3 by 2, with the origin at
        // its lower corner.
        assert!((footprint.area() - 6.0).abs() < 1e-9);
        for &[x, y] in footprint.exterior() {
            assert!((-1e-9..=3.0 + 1e-9).contains(&x) && (-1e-9..=2.0 + 1e-9).contains(&y));
        }
    }

    /// The reason `plane_for` gives when the `normal` expression is `expr`.
    fn plane_failure(expr: &str) -> String {
        let code: Code<{ CodeType::FlowExpr as u32 }> =
            serde_json::from_value(json!({ "type": "flowExpr", "value": expr })).unwrap();
        let replacer = FootprintReplacer {
            plane: PlaneSource::Custom {
                normal: code.compile().unwrap(),
                origin: None,
                x_axis: None,
            },
        };
        let input = feature(box_solid(CoordinateFrame::Euclidean, [0.0; 3], [1.0; 3]));
        replacer
            .plane_for(&create_default_execute_context(&input))
            .unwrap_err()
    }

    #[test]
    fn a_failed_expression_is_told_apart_from_a_wrongly_shaped_one() {
        let broken = plane_failure("no_such_function()");
        assert!(broken.contains("failed to evaluate"), "{broken}");
        assert!(
            broken.contains("'no_such_function' is not defined"),
            "{broken}"
        );
        assert!(plane_failure("\"not a vector\"").contains("did not evaluate to [x, y, z]"));
    }

    #[test]
    fn a_wall_has_no_horizontal_footprint_and_is_rejected() {
        let input = feature(wall());
        let sent = run(&mut *build(None), &input);
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0.to_string(), "rejected");
        assert_eq!(sent[0].1.id, input.id);
    }
}

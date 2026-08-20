#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{self, triangulation::Cache, Coerce};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::{
    geometry::{Geometry2D, Geometry3D},
    multi_line_string::{MultiLineString2D, MultiLineString3D},
    polygon::{Polygon2D, Polygon3D},
    triangular_mesh::TriangularMesh,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{CityGmlGeometry, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::errors::GeometryProcessorError;

#[cfg(feature = "new-geometry")]
thread_local! {
    /// Scratch reused across features so tessellating a stream pays earcut's
    /// allocation cost once. Kept off the `Processor` (which must be `Send +
    /// Sync + Clone`, and whose fields are the action's serialized parameters);
    /// one per worker thread.
    static TRIANGULATION_CACHE: std::cell::RefCell<Cache> =
        std::cell::RefCell::new(Cache::new());
}

#[derive(Debug, Clone, Default)]
pub(super) struct GeometryCoercerFactory;

impl ProcessorFactory for GeometryCoercerFactory {
    fn name(&self) -> &str {
        "Geometry Coercer"
    }

    fn description(&self) -> &str {
        "Coerces and converts feature geometries to specified target geometry types"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryCoercer))
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
        vec![FEATURES_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let coercer: GeometryCoercer = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryCoercerFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryCoercerFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryCoercerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(coercer))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum CoercionTarget {
    /// # Line String
    /// Replaces every face with the polylines of its boundary rings, holes
    /// included.
    LineString,
    /// # Polygon
    /// Rebuilds faces: a closed line string becomes the face it bounds, and a
    /// surface or a solid becomes the individual faces it is built from.
    Polygon,
    /// # Triangular Mesh
    /// Tessellates a face or a surface into triangles. A solid stays a solid,
    /// with its boundary triangulated.
    TriangularMesh,
}

#[cfg(feature = "new-geometry")]
impl From<&CoercionTarget> for ops::CoercionTarget {
    fn from(target: &CoercionTarget) -> Self {
        match target {
            CoercionTarget::LineString => ops::CoercionTarget::LineString,
            CoercionTarget::Polygon => ops::CoercionTarget::Polygon,
            CoercionTarget::TriangularMesh => ops::CoercionTarget::TriangularMesh,
        }
    }
}

/// # Geometry Coercer Parameters
///
/// Configuration for coercing geometries to specific target types.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GeometryCoercer {
    /// # Target Type
    /// Geometry type to re-represent each feature as. A feature the target does
    /// not apply to passes through unchanged.
    target_type: CoercionTarget,
}

impl Processor for GeometryCoercer {
    /// Geometry the target does not apply to leaves via `features` with the type
    /// it arrived with. A multi-part geometry is coerced member by member, so one
    /// feature in is always one feature out.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // The geometry sits behind a shared `Arc`, so it cannot be borrowed mutably
        // the way coercion needs. Work on a local copy instead.
        let mut geometry = (*ctx.feature.geometry).clone();
        let target: ops::CoercionTarget = (&self.target_type).into();
        let coerced =
            TRIANGULATION_CACHE.with(|cache| geometry.coerce(target, &mut cache.borrow_mut()));
        match coerced {
            Ok(coerced) => {
                let mut feature = ctx.feature.clone();
                feature.set_geometry(coerced);
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Err(e) => {
                // Leaving geometry as it is is normal business here, not a
                // failure, so it is not worth a warning per feature.
                ctx.event_hub.debug_log(
                    Some(ctx.error_span()),
                    format!("geometry left unchanged: {e}"),
                );
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), FEATURES_PORT.clone()));
            }
        }
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
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), FEATURES_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw)?;
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw)?;
            }
            GeometryValue::CityGmlGeometry(geos) => {
                self.handle_city_gml_geometry(geos, feature, geometry, &ctx, fw)?;
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
        "Geometry Coercer"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::collection::Collection3D;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::point::Point3D;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};

    /// A closed 4x4 square, as an exterior ring.
    const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    fn face() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            SQUARE,
            Vec::<Vec<[f64; 3]>>::new(),
        )))
    }

    fn point() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]))
    }

    /// The attribute traced from input to output; its value is arbitrary.
    const TRACED_ATTRIBUTE: &str = "surfaceId";
    const TRACED_VALUE: i64 = 7;

    /// A feature carrying `geometry` and one attribute to trace through.
    fn feature(geometry: Geometry) -> Feature {
        let mut feature = Feature::from(geometry);
        feature.insert(
            TRACED_ATTRIBUTE,
            AttributeValue::Number(TRACED_VALUE.into()),
        );
        feature
    }

    /// Run the processor over `feature`, returning what it sent, port by port.
    fn coerce(feature: &Feature, target_type: CoercionTarget) -> Vec<(Port, Feature)> {
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        GeometryCoercer { target_type }
            .process(create_default_execute_context(feature), &fw)
            .unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("built as a noop forwarder");
        };
        let ports = noop.send_ports.lock().unwrap().clone();
        let features = noop.send_features.lock().unwrap().clone();
        ports.into_iter().zip(features).collect()
    }

    /// The single feature the processor sent, and the port it left by.
    fn only(sent: Vec<(Port, Feature)>) -> (String, Feature) {
        let [(port, feature)] = <[_; 1]>::try_from(sent).expect("one feature in, one feature out");
        (port.to_string(), feature)
    }

    #[test]
    fn a_face_leaves_as_the_target_type_with_its_identity_and_attributes() {
        let input = feature(Geometry::Euclidean3D(face()));

        let (port, out) = only(coerce(&input, CoercionTarget::LineString));
        assert_eq!(port, "features");
        assert!(matches!(
            &*out.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
        assert_eq!(out.id, input.id);
        assert_eq!(
            out.attributes.get(&Attribute::new(TRACED_ATTRIBUTE)),
            Some(&AttributeValue::Number(TRACED_VALUE.into()))
        );

        let (_, out) = only(coerce(&input, CoercionTarget::TriangularMesh));
        assert!(matches!(
            &*out.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(_))
        ));
    }

    // The target may simply not apply — the geometry already is that type, or
    // has no such counterpart. Either way the feature carries on unchanged
    // rather than stopping the node.
    #[test]
    fn geometry_the_target_does_not_apply_to_passes_through_unchanged() {
        for geometry in [
            Geometry::Euclidean3D(face()),
            Geometry::Euclidean3D(point()),
            Geometry::None,
        ] {
            let input = feature(geometry);
            let (port, out) = only(coerce(&input, CoercionTarget::Polygon));
            assert_eq!(port, "features");
            assert_eq!(out.id, input.id);
            assert_eq!(&*out.geometry, &*input.geometry);
        }
    }

    // A multi-part geometry is coerced in place, so it does not fan out into one
    // feature per part the way the old CityGML handling did.
    #[test]
    fn a_multi_part_geometry_stays_one_feature() {
        let collection =
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                face(),
                face(),
            ])));
        let (_, out) = only(coerce(&feature(collection), CoercionTarget::LineString));
        let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) = &*out.geometry else {
            panic!("expected a 3D collection, got {:?}", out.geometry);
        };
        assert_eq!(c.members().len(), 2);
    }
}

impl GeometryCoercer {
    #[cfg(not(feature = "new-geometry"))]
    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), String> {
        match geos {
            Geometry2D::LineString(line_string) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoercionTarget::LineString => {
                        // Already a LineString, no conversion needed
                        // Keep as is
                    }
                    CoercionTarget::Polygon => {
                        // Check if the LineString is closed (first point equals last point)
                        if line_string.0.len() >= 4 && line_string.0.first() == line_string.0.last()
                        {
                            // It's closed, convert to a Polygon with this as the exterior ring
                            let polygon = Polygon2D::new(line_string.clone(), vec![]);
                            let mut geometry = geometry.clone();
                            geometry.value =
                                GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon));
                            feature.geometry = Arc::new(geometry);
                        } else {
                            return Err(
                                "Cannot convert to Polygon: LineString is not closed".to_string()
                            );
                        }
                    }
                    CoercionTarget::TriangularMesh => Err("Not supported".to_string())?,
                }
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Geometry2D::Polygon(polygon) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoercionTarget::LineString => {
                        let line_strings = polygon.rings().to_vec();
                        let geo = if let Some(first) = line_strings.first() {
                            if line_strings.len() == 1 {
                                Geometry2D::LineString(first.clone())
                            } else {
                                Geometry2D::MultiLineString(MultiLineString2D::new(line_strings))
                            }
                        } else {
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry2D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoercionTarget::Polygon => {
                        // Already a polygon, no conversion needed
                        // Keep as is
                    }
                    CoercionTarget::TriangularMesh => Err("Not supported".to_string())?,
                }
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Geometry2D::MultiPolygon(polygons) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoercionTarget::LineString => {
                        let mut geometries = Vec::<Geometry2D>::new();
                        for polygon in polygons.iter() {
                            let line_strings = polygon.rings().to_vec();
                            if let Some(first) = line_strings.first() {
                                let geometry = if line_strings.len() == 1 {
                                    Geometry2D::LineString(first.clone())
                                } else {
                                    Geometry2D::MultiLineString(MultiLineString2D::new(
                                        line_strings,
                                    ))
                                };
                                geometries.push(geometry);
                            }
                        }
                        let geo = if let Some(first) = geometries.first() {
                            if geometries.len() == 1 {
                                first.clone()
                            } else {
                                Geometry2D::GeometryCollection(geometries)
                            }
                        } else {
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry2D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoercionTarget::Polygon => {
                        // Already MultiPolygon, no direct conversion to single Polygon
                        // Keep as is or convert to GeometryCollection if there's one polygon
                    }
                    CoercionTarget::TriangularMesh => Err("Not supported".to_string())?,
                }
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            _ => return Err("Not supported".to_string()), // Not supported
        }
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), String> {
        match geos {
            Geometry3D::LineString(line_string) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoercionTarget::LineString => {
                        // Already a LineString, no conversion needed
                        // Keep as is
                    }
                    CoercionTarget::Polygon => {
                        // Check if the LineString is closed (first point equals last point)
                        if line_string.0.len() >= 4 && line_string.0.first() == line_string.0.last()
                        {
                            // It's closed, convert to a Polygon with this as the exterior ring
                            let polygon = Polygon3D::new(line_string.clone(), vec![]);
                            let mut geometry = geometry.clone();
                            geometry.value =
                                GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon));
                            feature.geometry = Arc::new(geometry);
                        } else {
                            return Err(
                                "Cannot convert to Polygon: LineString is not closed".to_string()
                            );
                        }
                    }
                    CoercionTarget::TriangularMesh => {
                        return Err("not supported".to_string())?;
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Geometry3D::Polygon(polygon) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoercionTarget::LineString => {
                        let line_strings = polygon.rings().to_vec();
                        let geo = if let Some(first) = line_strings.first() {
                            if line_strings.len() == 1 {
                                Geometry3D::LineString(first.clone())
                            } else {
                                Geometry3D::MultiLineString(MultiLineString3D::new(line_strings))
                            }
                        } else {
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoercionTarget::Polygon => {
                        // Already a polygon, no conversion needed
                        // Keep as is
                    }
                    CoercionTarget::TriangularMesh => {
                        let faces = polygon.rings();
                        let triangular_mesh = TriangularMesh::<f64, f64>::from_faces(&faces, None)?;
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::TriangularMesh(
                            triangular_mesh,
                        ));
                        feature.geometry = Arc::new(geometry);
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            Geometry3D::MultiPolygon(polygons) => {
                let mut feature = feature.clone();
                match self.target_type {
                    CoercionTarget::LineString => {
                        let mut geometries = Vec::<Geometry3D>::new();
                        for polygon in polygons.iter() {
                            let line_strings = polygon.rings().to_vec();
                            if let Some(first) = line_strings.first() {
                                let geometry = if line_strings.len() == 1 {
                                    Geometry3D::LineString(first.clone())
                                } else {
                                    Geometry3D::MultiLineString(MultiLineString3D::new(
                                        line_strings,
                                    ))
                                };
                                geometries.push(geometry);
                            }
                        }
                        let geo = if let Some(first) = geometries.first() {
                            if geometries.len() == 1 {
                                first.clone()
                            } else {
                                Geometry3D::GeometryCollection(geometries)
                            }
                        } else {
                            return Ok(());
                        };
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(geo);
                        feature.geometry = Arc::new(geometry);
                    }
                    CoercionTarget::Polygon => {
                        // Already MultiPolygon, no direct conversion to single Polygon
                    }
                    CoercionTarget::TriangularMesh => {
                        let faces: Vec<_> = polygons.iter().flat_map(|p| p.rings()).collect();
                        let triangular_mesh = TriangularMesh::<f64, f64>::from_faces(&faces, None)?;
                        let mut geometry = geometry.clone();
                        geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::TriangularMesh(
                            triangular_mesh,
                        ));
                        feature.geometry = Arc::new(geometry);
                    }
                }
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            _ => return Err("Not supported".to_string()), // Not supported
        };
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn handle_city_gml_geometry(
        &self,
        geos: &CityGmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), String> {
        for geo_feature in geos.gml_geometries.iter() {
            let mut geometries = Vec::<Geometry3D>::new();
            match &self.target_type {
                CoercionTarget::LineString => {
                    for polygon in geo_feature.polygons.iter() {
                        let line_strings = polygon.rings().to_vec();
                        if let Some(first) = line_strings.first() {
                            let geometry = if line_strings.len() == 1 {
                                Geometry3D::LineString(first.clone())
                            } else {
                                Geometry3D::MultiLineString(MultiLineString3D::new(line_strings))
                            };
                            geometries.push(geometry);
                        }
                    }
                    let geo = if let Some(first) = geometries.first() {
                        if geometries.len() == 1 {
                            first.clone()
                        } else {
                            Geometry3D::GeometryCollection(geometries)
                        }
                    } else {
                        return Ok(());
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                CoercionTarget::Polygon => {
                    // For CityGML, we already have polygons, so we just pass them through
                    for polygon in geo_feature.polygons.iter() {
                        geometries.push(Geometry3D::Polygon(polygon.clone()));
                    }
                    let geo = if let Some(first) = geometries.first() {
                        if geometries.len() == 1 {
                            first.clone()
                        } else {
                            Geometry3D::GeometryCollection(geometries)
                        }
                    } else {
                        return Ok(());
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                CoercionTarget::TriangularMesh => {
                    for polygon in geo_feature.polygons.iter() {
                        let triangular_mesh = TriangularMesh::<f64, f64>::try_from_polygons(
                            vec![polygon.clone()],
                            None,
                        )?;
                        geometries.push(Geometry3D::TriangularMesh(triangular_mesh));
                    }
                    let geo = if let Some(first) = geometries.first() {
                        if geometries.len() == 1 {
                            first.clone()
                        } else {
                            Geometry3D::GeometryCollection(geometries)
                        }
                    } else {
                        return Ok(());
                    };
                    let mut geometry = geometry.clone();
                    geometry.value = GeometryValue::FlowGeometry3D(geo);
                    let mut feature = feature.clone();
                    feature.refresh_id();
                    feature.geometry = Arc::new(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
            }
        }
        Ok(())
    }
}

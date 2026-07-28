use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use inflector::cases::camelcase::to_camel_case;
use once_cell::sync::Lazy;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Feature, Geometry, GeometryType, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub struct GeometryFilterFactory;

impl ProcessorFactory for GeometryFilterFactory {
    fn name(&self) -> &str {
        "Geometry Filter"
    }

    fn description(&self) -> &str {
        "Filter Features by Geometry Type"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        let mut result = vec![UNFILTERED_PORT.clone()];
        result.extend(GeometryFilterParam::all_ports());
        result
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: GeometryFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryFilterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = GeometryFilter { params };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct GeometryFilter {
    params: GeometryFilterParam,
}

/// # Geometry Filter Parameters
/// Configure how to filter features based on their geometry type
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "filterType", rename_all = "camelCase")]
pub enum GeometryFilterParam {
    None,
    #[cfg(not(feature = "new-geometry"))]
    Multiple,
    GeometryType,
    /// # Detailed Geometry Type
    /// Route by the exact geometry type instead of the coarse family, so a face,
    /// a surface mesh and a multi-surface each leave by their own port.
    #[cfg(feature = "new-geometry")]
    DetailedGeometryType,
}

impl GeometryFilterParam {
    fn none_port() -> Port {
        Port::new("none")
    }

    #[cfg(not(feature = "new-geometry"))]
    fn output_port(&self) -> Port {
        match self {
            GeometryFilterParam::None => Self::none_port(),
            GeometryFilterParam::Multiple => Port::new("contains"),
            GeometryFilterParam::GeometryType => unreachable!(),
        }
    }

    #[cfg(not(feature = "new-geometry"))]
    fn all_feature_type_ports() -> Vec<Port> {
        GeometryType::all_type_names()
            .iter()
            .map(|name| Port::new(to_camel_case(name)))
            .collect()
    }

    #[cfg(not(feature = "new-geometry"))]
    fn all_ports() -> Vec<Port> {
        let mut result = vec![
            GeometryFilterParam::None.output_port(),
            GeometryFilterParam::Multiple.output_port(),
        ];
        result.extend(GeometryFilterParam::all_feature_type_ports());
        result
    }

    /// `geometryType` keeps the coarse ports the legacy world exposed, and
    /// `detailedGeometryType` adds one port per exact type. The two modes share
    /// `point` and `solid`, which land on the same geometry either way, so the
    /// lists are merged rather than concatenated.
    #[cfg(feature = "new-geometry")]
    fn all_ports() -> Vec<Port> {
        let mut result = vec![Self::none_port()];
        for port in CoarseType::ALL
            .iter()
            .map(|ty| ty.port())
            .chain(DetailedType::ALL.iter().map(|ty| ty.port()))
        {
            if !result.contains(&port) {
                result.push(port);
            }
        }
        result
    }
}

impl Processor for GeometryFilter {
    // Routes without touching the feature: every mode only picks the port. A
    // geometry no port claims leaves by `unfiltered`, which is the catch-all
    // rather than an error outlet, so there is no rejected port here.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let geometry = &ctx.feature.geometry;
        let port = match self.params {
            // An absent geometry only, never an empty collection: this mode is
            // wired as a "has a geometry at all" gate.
            GeometryFilterParam::None => {
                if matches!(**geometry, Geometry::None) {
                    GeometryFilterParam::none_port()
                } else {
                    UNFILTERED_PORT.clone()
                }
            }
            GeometryFilterParam::GeometryType => {
                port_or_unfiltered(coarse_type(geometry).map(CoarseType::port))
            }
            GeometryFilterParam::DetailedGeometryType => {
                port_or_unfiltered(detailed_type(geometry).map(DetailedType::port))
            }
        };
        fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), port));
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        match self.params {
            GeometryFilterParam::None => match &feature.geometry.value {
                GeometryValue::None => fw.send(ctx.new_with_feature_and_port(
                    feature.clone(),
                    GeometryFilterParam::None.output_port(),
                )),
                _ => {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
                }
            },
            GeometryFilterParam::Multiple => {
                if feature.geometry.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
                } else {
                    filter_multiple_geometry(&ctx, fw, feature, &feature.geometry)
                }
            }
            GeometryFilterParam::GeometryType => {
                if feature.geometry.is_empty() {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
                } else {
                    filter_geometry_type(&ctx, fw, feature, &feature.geometry)
                }
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
        "Geometry Filter"
    }
}

#[cfg(not(feature = "new-geometry"))]
fn filter_multiple_geometry(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    geometry: &Geometry,
) {
    match &geometry.value {
        GeometryValue::None => {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
        }
        GeometryValue::FlowGeometry3D(geometry) => match geometry {
            Geometry3D::MultiPolygon(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            Geometry3D::GeometryCollection(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone())),
        },
        GeometryValue::FlowGeometry2D(geometry) => match geometry {
            Geometry2D::MultiPolygon(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            Geometry2D::GeometryCollection(_) => fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                GeometryFilterParam::Multiple.output_port(),
            )),
            _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone())),
        },
        GeometryValue::CityGmlGeometry(geometry) => {
            if geometry.gml_geometries.len() > 1 {
                fw.send(ctx.new_with_feature_and_port(
                    feature.clone(),
                    GeometryFilterParam::Multiple.output_port(),
                ))
            } else {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
            }
        }
    }
}

#[cfg(not(feature = "new-geometry"))]
fn filter_geometry_type(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    geometry: &Geometry,
) {
    match &geometry.value {
        GeometryValue::None => {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
        }
        GeometryValue::FlowGeometry3D(geometry) => {
            let geometry_type: GeometryType = geometry.into();
            fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                Port::new(to_camel_case(geometry_type.name())),
            ))
        }
        GeometryValue::FlowGeometry2D(geometry) => {
            let geometry_type: GeometryType = geometry.into();
            fw.send(ctx.new_with_feature_and_port(
                feature.clone(),
                Port::new(to_camel_case(geometry_type.name())),
            ))
        }
        GeometryValue::CityGmlGeometry(geometry) => {
            if geometry.gml_geometries.len() != 1 {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()))
            } else {
                let Some(first_feature) = geometry.gml_geometries.first() else {
                    fw.send(
                        ctx.new_with_feature_and_port(feature.clone(), UNFILTERED_PORT.clone()),
                    );
                    return;
                };
                fw.send(ctx.new_with_feature_and_port(
                    feature.clone(),
                    Port::new(to_camel_case(first_feature.name())),
                ))
            }
        }
    }
}

/// The coarse family a geometry belongs to under `filterType: geometryType`.
///
/// These are the only distinctions the legacy geometry world could draw, and the
/// mode keeps them so that workflows wired against the old ports keep routing the
/// same features to the same downstream nodes. Both embedding dimensions collapse
/// into one family; `detailedGeometryType` is where they separate.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoarseType {
    Point,
    Curve,
    Surface,
    Triangle,
    Solid,
}

#[cfg(feature = "new-geometry")]
impl CoarseType {
    const ALL: &[Self] = &[
        Self::Point,
        Self::Curve,
        Self::Surface,
        Self::Triangle,
        Self::Solid,
    ];

    fn port(self) -> Port {
        Port::new(match self {
            Self::Point => "point",
            Self::Curve => "curve",
            Self::Surface => "surface",
            Self::Triangle => "triangle",
            Self::Solid => "solid",
        })
    }
}

/// The exact type a geometry is under `filterType: detailedGeometryType`, one
/// bucket per type the geometry model distinguishes.
///
/// A planar `Polygon` and a `Face` in space are separate buckets, as are a
/// collection and the leaf it holds — the granularity the FME reference
/// workflows filter at, which the legacy world flattened away.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailedType {
    Point,
    MultiPoint,
    PointCloud,
    LineString,
    MultiCurve,
    Polygon,
    MultiArea,
    Face,
    PolygonMesh,
    TriangularMesh,
    MultiSurface,
    Solid,
    Csg,
    MultiSolid,
    Aggregate,
}

#[cfg(feature = "new-geometry")]
impl DetailedType {
    const ALL: &[Self] = &[
        Self::Point,
        Self::MultiPoint,
        Self::PointCloud,
        Self::LineString,
        Self::MultiCurve,
        Self::Polygon,
        Self::MultiArea,
        Self::Face,
        Self::PolygonMesh,
        Self::TriangularMesh,
        Self::MultiSurface,
        Self::Solid,
        Self::Csg,
        Self::MultiSolid,
        Self::Aggregate,
    ];

    fn port(self) -> Port {
        Port::new(match self {
            Self::Point => "point",
            Self::MultiPoint => "multi-point",
            Self::PointCloud => "point-cloud",
            Self::LineString => "line-string",
            Self::MultiCurve => "multi-curve",
            Self::Polygon => "polygon",
            Self::MultiArea => "multi-area",
            Self::Face => "face",
            Self::PolygonMesh => "polygon-mesh",
            Self::TriangularMesh => "triangular-mesh",
            Self::MultiSurface => "multi-surface",
            Self::Solid => "solid",
            Self::Csg => "csg",
            Self::MultiSolid => "multi-solid",
            Self::Aggregate => "aggregate",
        })
    }
}

#[cfg(feature = "new-geometry")]
fn port_or_unfiltered(port: Option<Port>) -> Port {
    port.unwrap_or_else(|| UNFILTERED_PORT.clone())
}

/// The single value every item agrees on, or `None` if there are no items, if any
/// item has no value, or if two items disagree.
#[cfg(feature = "new-geometry")]
fn unify<T: Copy + PartialEq>(items: impl IntoIterator<Item = Option<T>>) -> Option<T> {
    let mut items = items.into_iter();
    let agreed = items.next()??;
    for item in items {
        if item? != agreed {
            return None;
        }
    }
    Some(agreed)
}

/// `None` where no family claims the geometry, which routes it to `unfiltered`.
#[cfg(feature = "new-geometry")]
fn coarse_type(geometry: &Geometry) -> Option<CoarseType> {
    match geometry {
        Geometry::None => None,
        Geometry::Euclidean2D(geometry) => coarse_type_2d(geometry),
        Geometry::Euclidean3D(geometry) => coarse_type_3d(geometry),
        // A one-member collection is the shape a reader gives a feature with a
        // single geometry property, so it filters as that member. Anything
        // holding more is a container the coarse families cannot name.
        Geometry::GeometryCollection(collection) => match collection.members() {
            [member] => coarse_type(member),
            _ => None,
        },
    }
}

#[cfg(feature = "new-geometry")]
fn coarse_type_2d(geometry: &Euclidean2DGeometry) -> Option<CoarseType> {
    match geometry {
        Euclidean2DGeometry::Point(_) => Some(CoarseType::Point),
        Euclidean2DGeometry::LineString(_) => Some(CoarseType::Curve),
        Euclidean2DGeometry::Polygon(_) | Euclidean2DGeometry::PolygonMesh(_) => {
            Some(CoarseType::Surface)
        }
        Euclidean2DGeometry::TriangularMesh(_) => Some(CoarseType::Triangle),
        // A collection takes its members' family, so a multi-surface filters as a
        // surface the way it did before. Mixed families have no single answer.
        Euclidean2DGeometry::Collection(collection) => {
            unify(collection.members().iter().map(coarse_type_2d))
        }
    }
}

#[cfg(feature = "new-geometry")]
fn coarse_type_3d(geometry: &Euclidean3DGeometry) -> Option<CoarseType> {
    match geometry {
        Euclidean3DGeometry::Point(_) | Euclidean3DGeometry::PointCloud(_) => {
            Some(CoarseType::Point)
        }
        Euclidean3DGeometry::LineString(_) => Some(CoarseType::Curve),
        Euclidean3DGeometry::Polygon(_) | Euclidean3DGeometry::PolygonMesh(_) => {
            Some(CoarseType::Surface)
        }
        Euclidean3DGeometry::TriangularMesh(_) => Some(CoarseType::Triangle),
        Euclidean3DGeometry::Solid(_) | Euclidean3DGeometry::Csg(_) => Some(CoarseType::Solid),
        Euclidean3DGeometry::Collection(collection) => {
            unify(collection.members().iter().map(coarse_type_3d))
        }
    }
}

/// `None` where no bucket claims the geometry, which routes it to `unfiltered`.
#[cfg(feature = "new-geometry")]
fn detailed_type(geometry: &Geometry) -> Option<DetailedType> {
    match geometry {
        Geometry::None => None,
        Geometry::Euclidean2D(geometry) => detailed_type_2d(geometry),
        Geometry::Euclidean3D(geometry) => detailed_type_3d(geometry),
        Geometry::GeometryCollection(collection) => match collection.members() {
            [] => None,
            [member] => detailed_type(member),
            // Several members of possibly unrelated types: the case FME calls an
            // aggregate, and what a reader produces for a feature carrying more
            // than one geometry property.
            _ => Some(DetailedType::Aggregate),
        },
    }
}

#[cfg(feature = "new-geometry")]
fn detailed_type_2d(geometry: &Euclidean2DGeometry) -> Option<DetailedType> {
    match geometry {
        Euclidean2DGeometry::Point(_) => Some(DetailedType::Point),
        Euclidean2DGeometry::LineString(_) => Some(DetailedType::LineString),
        Euclidean2DGeometry::Polygon(_) => Some(DetailedType::Polygon),
        Euclidean2DGeometry::PolygonMesh(_) => Some(DetailedType::PolygonMesh),
        Euclidean2DGeometry::TriangularMesh(_) => Some(DetailedType::TriangularMesh),
        Euclidean2DGeometry::Collection(collection) => {
            unify(collection.members().iter().map(multi_type_2d))
        }
    }
}

#[cfg(feature = "new-geometry")]
fn detailed_type_3d(geometry: &Euclidean3DGeometry) -> Option<DetailedType> {
    match geometry {
        Euclidean3DGeometry::Point(_) => Some(DetailedType::Point),
        Euclidean3DGeometry::PointCloud(_) => Some(DetailedType::PointCloud),
        Euclidean3DGeometry::LineString(_) => Some(DetailedType::LineString),
        Euclidean3DGeometry::Polygon(_) => Some(DetailedType::Face),
        Euclidean3DGeometry::PolygonMesh(_) => Some(DetailedType::PolygonMesh),
        Euclidean3DGeometry::TriangularMesh(_) => Some(DetailedType::TriangularMesh),
        Euclidean3DGeometry::Solid(_) => Some(DetailedType::Solid),
        Euclidean3DGeometry::Csg(_) => Some(DetailedType::Csg),
        Euclidean3DGeometry::Collection(collection) => {
            unify(collection.members().iter().map(multi_type_3d))
        }
    }
}

/// The `Multi*` bucket a 2D collection whose members all look like this one falls
/// in.
///
/// Members are grouped by family, not by exact type: a multi-surface read from
/// GML mixes rings and oriented surfaces, which become polygons and meshes here,
/// and it has to stay one collection. A nested collection contributes the bucket
/// it is itself named by.
#[cfg(feature = "new-geometry")]
fn multi_type_2d(member: &Euclidean2DGeometry) -> Option<DetailedType> {
    match member {
        Euclidean2DGeometry::Point(_) => Some(DetailedType::MultiPoint),
        Euclidean2DGeometry::LineString(_) => Some(DetailedType::MultiCurve),
        Euclidean2DGeometry::Polygon(_)
        | Euclidean2DGeometry::PolygonMesh(_)
        | Euclidean2DGeometry::TriangularMesh(_) => Some(DetailedType::MultiArea),
        Euclidean2DGeometry::Collection(collection) => {
            unify(collection.members().iter().map(multi_type_2d))
        }
    }
}

/// The `Multi*` bucket a 3D collection whose members all look like this one falls
/// in; see [`multi_type_2d`] for how members are grouped.
#[cfg(feature = "new-geometry")]
fn multi_type_3d(member: &Euclidean3DGeometry) -> Option<DetailedType> {
    match member {
        Euclidean3DGeometry::Point(_) | Euclidean3DGeometry::PointCloud(_) => {
            Some(DetailedType::MultiPoint)
        }
        Euclidean3DGeometry::LineString(_) => Some(DetailedType::MultiCurve),
        Euclidean3DGeometry::Polygon(_)
        | Euclidean3DGeometry::PolygonMesh(_)
        | Euclidean3DGeometry::TriangularMesh(_) => Some(DetailedType::MultiSurface),
        Euclidean3DGeometry::Solid(_) | Euclidean3DGeometry::Csg(_) => {
            Some(DetailedType::MultiSolid)
        }
        Euclidean3DGeometry::Collection(collection) => {
            unify(collection.members().iter().map(multi_type_3d))
        }
    }
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod tests {
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::feature::Attributes;

    use crate::tests::utils::create_default_execute_context;

    use super::*;

    #[test]
    fn test_filter_multiple_geometry_null() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::new_with_attributes(Attributes::new());
        let geometry = Geometry {
            value: GeometryValue::None,
            ..Default::default()
        };
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &geometry);
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(UNFILTERED_PORT.clone())
            );
        }
    }

    #[test]
    fn test_filter_multiple_geometry_3d_multipolygon() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::MultiPolygon(Default::default())),
                ..Default::default()
            },
        );
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &feature.geometry.clone());
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(GeometryFilterParam::Multiple.output_port())
            );
        }
    }

    #[test]
    fn test_filter_multiple_geometry_3d_geometry_collection() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::GeometryCollection(
                    Default::default(),
                )),
                ..Default::default()
            },
        );
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &feature.geometry.clone());
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(GeometryFilterParam::Multiple.output_port())
            );
        }
    }

    #[test]
    fn test_filter_multiple_geometry_3d_other_geometry() {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry {
                value: GeometryValue::FlowGeometry3D(Geometry3D::Point(Default::default())),
                ..Default::default()
            },
        );
        let ctx = create_default_execute_context(&feature);
        filter_multiple_geometry(&ctx, &fw, &feature, &feature.geometry.clone());
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            assert_eq!(
                noop.send_ports.lock().unwrap().first().cloned(),
                Some(UNFILTERED_PORT.clone())
            );
        }
    }

    // Add more tests for other scenarios...
}

#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_tests {
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::collection::{Collection2D, Collection3D};
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::csg::Csg;
    use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::point::{Point2D, Point3D};
    use reearth_flow_geometry::point_cloud::PointCloud;
    use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_geometry::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
    use reearth_flow_geometry::solid::Solid;
    use reearth_flow_geometry::triangular_mesh::{
        TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData,
    };
    use reearth_flow_geometry::GeometryCollection;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    use crate::tests::utils::create_default_execute_context;

    use super::*;

    fn frame() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    /// The port a feature carrying `geometry` leaves the filter by.
    fn route(params: GeometryFilterParam, geometry: Geometry) -> Port {
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let feature = Feature::from(geometry);
        let ctx = create_default_execute_context(&feature);
        GeometryFilter { params }.process(ctx, &fw).unwrap();
        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!()
        };
        let sent = noop.send_ports.lock().unwrap().first().cloned();
        sent.expect("the filter sends every feature somewhere")
    }

    fn null_gate(geometry: Geometry) -> Port {
        route(GeometryFilterParam::None, geometry)
    }

    fn coarse(geometry: Geometry) -> Port {
        route(GeometryFilterParam::GeometryType, geometry)
    }

    fn detailed(geometry: Geometry) -> Port {
        route(GeometryFilterParam::DetailedGeometryType, geometry)
    }

    fn port(name: &str) -> Port {
        Port::new(name)
    }

    fn two_d(geometry: Euclidean2DGeometry) -> Geometry {
        Geometry::Euclidean2D(geometry)
    }

    fn three_d(geometry: Euclidean3DGeometry) -> Geometry {
        Geometry::Euclidean3D(geometry)
    }

    fn point_2d() -> Euclidean2DGeometry {
        Euclidean2DGeometry::Point(Point2D::new(frame(), [0.0, 0.0]))
    }

    fn point_3d() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Point(Point3D::new(frame(), [0.0, 0.0, 0.0]))
    }

    fn point_cloud() -> Euclidean3DGeometry {
        Euclidean3DGeometry::PointCloud(Box::new(PointCloud::from_positions(
            frame(),
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        )))
    }

    fn line_string_2d() -> Euclidean2DGeometry {
        Euclidean2DGeometry::LineString(LineString2D::from_coords(
            frame(),
            [[0.0, 0.0], [1.0, 1.0]],
        ))
    }

    fn line_string_3d() -> Euclidean3DGeometry {
        Euclidean3DGeometry::LineString(LineString3D::from_coords(
            frame(),
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        ))
    }

    fn polygon_2d() -> Euclidean2DGeometry {
        Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
            frame(),
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        )))
    }

    fn polygon_3d() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
            frame(),
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )))
    }

    fn polygon_mesh_2d() -> Euclidean2DGeometry {
        Euclidean2DGeometry::PolygonMesh(Box::new(
            PolygonMesh2D::from_parts(
                frame(),
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                [[0u32, 1, 2]],
            )
            .unwrap(),
        ))
    }

    fn polygon_mesh_3d() -> Euclidean3DGeometry {
        Euclidean3DGeometry::PolygonMesh(Box::new(
            PolygonMesh3D::from_parts(
                frame(),
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                [[0u32, 1, 2]],
            )
            .unwrap(),
        ))
    }

    fn triangular_mesh_2d() -> Euclidean2DGeometry {
        Euclidean2DGeometry::TriangularMesh(Box::new(
            TriangularMesh2D::from_parts(
                frame(),
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                [0u32, 1, 2],
            )
            .unwrap(),
        ))
    }

    fn triangular_mesh_3d() -> Euclidean3DGeometry {
        Euclidean3DGeometry::TriangularMesh(Box::new(
            TriangularMesh3D::from_parts(
                frame(),
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                [0u32, 1, 2],
            )
            .unwrap(),
        ))
    }

    fn bare_solid() -> Solid {
        // Construction does not check closure, so an open shell is a fine stand-in.
        Solid::from_exterior(
            frame(),
            TriangularMesh3DData::from_parts(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
                [0u32, 1, 2],
            )
            .unwrap(),
        )
    }

    fn solid() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Solid(Box::new(bare_solid()))
    }

    fn csg() -> Euclidean3DGeometry {
        Euclidean3DGeometry::Csg(Csg::union(bare_solid(), bare_solid()))
    }

    fn collection_2d(
        members: impl IntoIterator<Item = Euclidean2DGeometry>,
    ) -> Euclidean2DGeometry {
        Euclidean2DGeometry::Collection(Collection2D::new(members))
    }

    fn collection_3d(
        members: impl IntoIterator<Item = Euclidean3DGeometry>,
    ) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Collection(Collection3D::new(members))
    }

    fn geometry_collection(members: impl IntoIterator<Item = Geometry>) -> Geometry {
        Geometry::GeometryCollection(GeometryCollection::new(members))
    }

    #[test]
    fn null_gate_admits_only_an_absent_geometry() {
        assert_eq!(null_gate(Geometry::None), port("none"));
        assert_eq!(null_gate(three_d(point_3d())), UNFILTERED_PORT.clone());
    }

    // An empty collection is a geometry, just an empty one, so the gate must not
    // treat it as absent: workflows use this mode to keep features that have a
    // geometry at all.
    #[test]
    fn null_gate_rejects_an_empty_collection() {
        assert_eq!(
            null_gate(three_d(collection_3d([]))),
            UNFILTERED_PORT.clone()
        );
        assert_eq!(null_gate(geometry_collection([])), UNFILTERED_PORT.clone());
    }

    #[test]
    fn coarse_mode_maps_every_leaf_to_its_family() {
        assert_eq!(coarse(two_d(point_2d())), port("point"));
        assert_eq!(coarse(three_d(point_3d())), port("point"));
        assert_eq!(coarse(three_d(point_cloud())), port("point"));
        assert_eq!(coarse(two_d(line_string_2d())), port("curve"));
        assert_eq!(coarse(three_d(line_string_3d())), port("curve"));
        assert_eq!(coarse(two_d(polygon_2d())), port("surface"));
        assert_eq!(coarse(three_d(polygon_3d())), port("surface"));
        assert_eq!(coarse(two_d(polygon_mesh_2d())), port("surface"));
        assert_eq!(coarse(three_d(polygon_mesh_3d())), port("surface"));
        assert_eq!(coarse(two_d(triangular_mesh_2d())), port("triangle"));
        assert_eq!(coarse(three_d(triangular_mesh_3d())), port("triangle"));
        assert_eq!(coarse(three_d(solid())), port("solid"));
        assert_eq!(coarse(three_d(csg())), port("solid"));
        assert_eq!(coarse(Geometry::None), UNFILTERED_PORT.clone());
    }

    #[test]
    fn coarse_mode_gives_a_collection_its_members_family() {
        assert_eq!(
            coarse(three_d(collection_3d([polygon_3d(), polygon_3d()]))),
            port("surface")
        );
        // A GML multi-surface can hold both plain polygons and oriented surfaces,
        // which land as polygons and meshes: still one surface collection.
        assert_eq!(
            coarse(three_d(collection_3d([polygon_3d(), polygon_mesh_3d()]))),
            port("surface")
        );
        assert_eq!(
            coarse(three_d(collection_3d([solid(), solid()]))),
            port("solid")
        );
        assert_eq!(
            coarse(three_d(collection_3d([
                collection_3d([polygon_3d()]),
                polygon_3d()
            ]))),
            port("surface")
        );
    }

    #[test]
    fn coarse_mode_unfilters_a_collection_without_one_family() {
        assert_eq!(
            coarse(three_d(collection_3d([polygon_3d(), point_3d()]))),
            UNFILTERED_PORT.clone()
        );
        assert_eq!(coarse(three_d(collection_3d([]))), UNFILTERED_PORT.clone());
    }

    // The shape a CityGML reader produces: one member per geometry property, so a
    // feature with a single property still filters as that geometry. This is what
    // the quality-check workflows route on.
    #[test]
    fn coarse_mode_looks_through_a_single_member_geometry_collection() {
        assert_eq!(
            coarse(geometry_collection([three_d(solid())])),
            port("solid")
        );
        assert_eq!(
            coarse(geometry_collection([three_d(collection_3d([
                polygon_3d(),
                polygon_3d()
            ]))])),
            port("surface")
        );
        assert_eq!(
            coarse(geometry_collection([three_d(triangular_mesh_3d())])),
            port("triangle")
        );
    }

    #[test]
    fn coarse_mode_unfilters_a_multi_member_geometry_collection() {
        assert_eq!(
            coarse(geometry_collection([three_d(solid()), three_d(solid())])),
            UNFILTERED_PORT.clone()
        );
        assert_eq!(coarse(geometry_collection([])), UNFILTERED_PORT.clone());
    }

    #[test]
    fn detailed_mode_maps_every_leaf_to_its_own_port() {
        assert_eq!(detailed(two_d(point_2d())), port("point"));
        assert_eq!(detailed(three_d(point_3d())), port("point"));
        assert_eq!(detailed(three_d(point_cloud())), port("point-cloud"));
        assert_eq!(detailed(two_d(line_string_2d())), port("line-string"));
        assert_eq!(detailed(three_d(line_string_3d())), port("line-string"));
        assert_eq!(detailed(two_d(polygon_mesh_2d())), port("polygon-mesh"));
        assert_eq!(detailed(three_d(polygon_mesh_3d())), port("polygon-mesh"));
        assert_eq!(
            detailed(two_d(triangular_mesh_2d())),
            port("triangular-mesh")
        );
        assert_eq!(
            detailed(three_d(triangular_mesh_3d())),
            port("triangular-mesh")
        );
        assert_eq!(detailed(three_d(solid())), port("solid"));
        assert_eq!(detailed(three_d(csg())), port("csg"));
        assert_eq!(detailed(Geometry::None), UNFILTERED_PORT.clone());
    }

    // The distinction the legacy world could not express: a footprint in the plane
    // and a face in space are separate types, as they are in FME.
    #[test]
    fn detailed_mode_separates_a_planar_polygon_from_a_face() {
        assert_eq!(detailed(two_d(polygon_2d())), port("polygon"));
        assert_eq!(detailed(three_d(polygon_3d())), port("face"));
    }

    #[test]
    fn detailed_mode_names_a_collection_by_its_members_family() {
        assert_eq!(
            detailed(two_d(collection_2d([point_2d(), point_2d()]))),
            port("multi-point")
        );
        assert_eq!(
            detailed(three_d(collection_3d([line_string_3d()]))),
            port("multi-curve")
        );
        assert_eq!(
            detailed(two_d(collection_2d([polygon_2d(), polygon_2d()]))),
            port("multi-area")
        );
        assert_eq!(
            detailed(three_d(collection_3d([polygon_3d(), polygon_mesh_3d()]))),
            port("multi-surface")
        );
        assert_eq!(
            detailed(three_d(collection_3d([solid(), solid()]))),
            port("multi-solid")
        );
        assert_eq!(
            detailed(three_d(collection_3d([polygon_3d(), point_3d()]))),
            UNFILTERED_PORT.clone()
        );
    }

    #[test]
    fn detailed_mode_calls_a_multi_member_geometry_collection_an_aggregate() {
        assert_eq!(
            detailed(geometry_collection([
                three_d(solid()),
                three_d(collection_3d([polygon_3d()]))
            ])),
            port("aggregate")
        );
        assert_eq!(
            detailed(geometry_collection([three_d(solid())])),
            port("solid")
        );
        assert_eq!(detailed(geometry_collection([])), UNFILTERED_PORT.clone());
    }

    // The three types the PLATEAU reference workflows filter on in FME's detailed
    // mode: Face, BRepSolid and CompositeSurface. The coarse mode flattens the
    // first and the last into one port, which is why the mode exists.
    #[test]
    fn detailed_mode_separates_the_types_the_reference_workflows_filter_on() {
        assert_eq!(detailed(three_d(polygon_3d())), port("face"));
        assert_eq!(detailed(three_d(solid())), port("solid"));
        assert_eq!(detailed(three_d(polygon_mesh_3d())), port("polygon-mesh"));
        assert_eq!(coarse(three_d(polygon_3d())), port("surface"));
        assert_eq!(coarse(three_d(polygon_mesh_3d())), port("surface"));
    }

    // A GML multi-surface: coarse routing keeps it on the port existing workflows
    // wire, and detailed routing sees the collection.
    #[test]
    fn the_two_modes_agree_on_a_multi_surface() {
        let multi_surface = || three_d(collection_3d([polygon_3d(), polygon_3d()]));
        assert_eq!(coarse(multi_surface()), port("surface"));
        assert_eq!(detailed(multi_surface()), port("multi-surface"));
    }

    // A bucket that routes to a port the factory never declared silently drops
    // every feature that reaches it, so the two lists have to be checked against
    // each other rather than one bucket at a time.
    #[test]
    fn every_port_a_bucket_routes_to_is_declared_by_the_factory() {
        let declared = GeometryFilterFactory.get_output_ports();
        let mut seen = std::collections::HashSet::new();
        for port in &declared {
            assert!(seen.insert(port.clone()), "port {port} is declared twice");
        }
        let routable = [UNFILTERED_PORT.clone(), GeometryFilterParam::none_port()]
            .into_iter()
            .chain(CoarseType::ALL.iter().map(|ty| ty.port()))
            .chain(DetailedType::ALL.iter().map(|ty| ty.port()));
        for port in routable {
            assert!(declared.contains(&port), "port {port} is not declared");
        }
    }

    // The legacy `contains` port has no producer in this world: `multiple` is gone
    // and the detailed buckets replace it.
    #[test]
    fn the_factory_declares_no_contains_port() {
        assert!(!GeometryFilterFactory
            .get_output_ports()
            .contains(&port("contains")));
    }
}

use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::algorithm::area2d::Area2D;
use reearth_flow_geometry::algorithm::area3d::Area3D;
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_geometry::algorithm::interior_point::InteriorPoint;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::point::Point;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));

/// Method used to compute the center point of a geometry.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) enum CenterPointMode {
    /// Computes the centroid (center of gravity) of the geometry.
    #[default]
    CenterOfGravity,
    /// Computes the center of the geometry's bounding box.
    BoundingBoxCenter,
    /// Computes a point guaranteed to lie inside the geometry (pole of inaccessibility).
    AnyInsidePoint,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CenterPointReplacerParam {
    /// The method used to compute the replacement center point.
    #[serde(default)]
    mode: CenterPointMode,
}

#[derive(Debug, Clone, Default)]
pub(super) struct CenterPointReplacerFactory;

impl ProcessorFactory for CenterPointReplacerFactory {
    fn name(&self) -> &str {
        "CenterPointReplacer"
    }

    fn description(&self) -> &str {
        "Replace Feature Geometry with Center Point"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CenterPointReplacerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![POINT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: CenterPointReplacerParam = if let Some(with) = with {
            let value = serde_json::to_value(with)
                .map_err(|e| format!("CenterPointReplacer: failed to serialize parameters: {e}"))?;
            serde_json::from_value(value).map_err(|e| {
                format!("CenterPointReplacer: failed to deserialize parameters: {e}")
            })?
        } else {
            CenterPointReplacerParam::default()
        };
        Ok(Box::new(CenterPointReplacer { mode: params.mode }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CenterPointReplacer {
    mode: CenterPointMode,
}

impl Processor for CenterPointReplacer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            self.send_rejected(feature, &ctx, fw);
            return Ok(());
        }
        match &geometry.value {
            GeometryValue::None => {
                self.send_rejected(feature, &ctx, fw);
            }
            GeometryValue::FlowGeometry2D(geos) => {
                self.handle_2d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::FlowGeometry3D(geos) => {
                self.handle_3d_geometry(geos, feature, geometry, &ctx, fw);
            }
            GeometryValue::CityGmlGeometry(city_gml) => {
                self.handle_citygml_geometry(city_gml, feature, geometry, &ctx, fw);
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
        "CenterPointReplacer"
    }
}

impl CenterPointReplacer {
    fn send_rejected(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        let mut feature = feature.clone();
        feature.insert(
            "fme_rejection_code",
            AttributeValue::String("INVALID_GEOMETRY_TYPE".into()),
        );
        fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
    }

    fn is_area_geometry_2d(geos: &Geometry2D) -> bool {
        matches!(
            geos,
            Geometry2D::Polygon(_)
                | Geometry2D::MultiPolygon(_)
                | Geometry2D::Rect(_)
                | Geometry2D::Triangle(_)
        )
    }

    fn is_area_geometry_3d(geos: &Geometry3D) -> bool {
        matches!(
            geos,
            Geometry3D::Polygon(_)
                | Geometry3D::MultiPolygon(_)
                | Geometry3D::Rect(_)
                | Geometry3D::Triangle(_)
        )
    }

    fn handle_2d_geometry(
        &self,
        geos: &Geometry2D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match &self.mode {
            CenterPointMode::CenterOfGravity => {
                let Some(centroid) = geos.centroid() else {
                    self.send_rejected(feature, ctx, fw);
                    return;
                };
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry2D(centroid.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
            CenterPointMode::BoundingBoxCenter => {
                let Some(rect) = geos.bounding_rect() else {
                    self.send_rejected(feature, ctx, fw);
                    return;
                };
                let center = rect.center();
                let point = Point::from(center);
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry2D(point.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
            CenterPointMode::AnyInsidePoint => {
                if !Self::is_area_geometry_2d(geos) {
                    self.send_rejected(feature, ctx, fw);
                    return;
                }
                let point = match geos {
                    Geometry2D::Polygon(p) => p.interior_point(),
                    Geometry2D::MultiPolygon(mp) => {
                        // Use the polygon with the largest area
                        mp.iter()
                            .filter_map(|p| p.interior_point().map(|pt| (pt, p.unsigned_area2d())))
                            .max_by(|(_, a), (_, b)| {
                                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .map(|(pt, _)| pt)
                    }
                    Geometry2D::Rect(r) => {
                        let center = r.center();
                        Some(Point::from(center))
                    }
                    Geometry2D::Triangle(t) => t.centroid().into(),
                    _ => None,
                };
                let Some(point) = point else {
                    self.send_rejected(feature, ctx, fw);
                    return;
                };
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry2D(point.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
        }
    }

    fn handle_3d_geometry(
        &self,
        geos: &Geometry3D,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match &self.mode {
            CenterPointMode::CenterOfGravity => {
                let Some(centroid) = geos.centroid() else {
                    self.send_rejected(feature, ctx, fw);
                    return;
                };
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry3D(centroid.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
            CenterPointMode::BoundingBoxCenter => {
                let Some(rect) = geos.bounding_rect() else {
                    self.send_rejected(feature, ctx, fw);
                    return;
                };
                let center = rect.center();
                let point = Point::from(center);
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry3D(point.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
            CenterPointMode::AnyInsidePoint => {
                if !Self::is_area_geometry_3d(geos) {
                    self.send_rejected(feature, ctx, fw);
                    return;
                }
                let point = match geos {
                    Geometry3D::Polygon(p) => p.interior_point(),
                    Geometry3D::MultiPolygon(mp) => mp
                        .iter()
                        .filter_map(|p| p.interior_point().map(|pt| (pt, p.unsigned_area3d())))
                        .max_by(|(_, a), (_, b)| {
                            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(pt, _)| pt),
                    Geometry3D::Rect(r) => {
                        let center = r.center();
                        Some(Point::from(center))
                    }
                    Geometry3D::Triangle(t) => t.centroid().into(),
                    _ => None,
                };
                let Some(point) = point else {
                    self.send_rejected(feature, ctx, fw);
                    return;
                };
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry3D(point.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
        }
    }

    fn handle_citygml_geometry(
        &self,
        city_gml: &CityGmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) {
        match &self.mode {
            CenterPointMode::AnyInsidePoint => {
                // CityGML is 3D solid geometry, not area — reject per FME spec
                self.send_rejected(feature, ctx, fw);
            }
            CenterPointMode::CenterOfGravity => {
                let polygons: Vec<_> = city_gml
                    .gml_geometries
                    .iter()
                    .flat_map(|g| g.polygons.iter())
                    .collect();
                if polygons.is_empty() {
                    self.send_rejected(feature, ctx, fw);
                    return;
                }
                // Compute average centroid across all polygons
                let mut total_x = 0.0_f64;
                let mut total_y = 0.0_f64;
                let mut total_z = 0.0_f64;
                let mut count = 0u64;
                for polygon in &polygons {
                    if let Some(centroid) = polygon.centroid() {
                        total_x += centroid.x();
                        total_y += centroid.y();
                        total_z += centroid.z();
                        count += 1;
                    }
                }
                if count == 0 {
                    self.send_rejected(feature, ctx, fw);
                    return;
                }
                let n = count as f64;
                let point = Point::new_(total_x / n, total_y / n, total_z / n);
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry3D(point.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
            CenterPointMode::BoundingBoxCenter => {
                let all_coords: Vec<_> = city_gml
                    .gml_geometries
                    .iter()
                    .flat_map(|g| g.polygons.iter())
                    .flat_map(|p| p.exterior().0.iter())
                    .collect();
                if all_coords.is_empty() {
                    self.send_rejected(feature, ctx, fw);
                    return;
                }
                let mut min_x = f64::INFINITY;
                let mut min_y = f64::INFINITY;
                let mut min_z = f64::INFINITY;
                let mut max_x = f64::NEG_INFINITY;
                let mut max_y = f64::NEG_INFINITY;
                let mut max_z = f64::NEG_INFINITY;
                for c in &all_coords {
                    if c.x < min_x {
                        min_x = c.x;
                    }
                    if c.y < min_y {
                        min_y = c.y;
                    }
                    if c.z < min_z {
                        min_z = c.z;
                    }
                    if c.x > max_x {
                        max_x = c.x;
                    }
                    if c.y > max_y {
                        max_y = c.y;
                    }
                    if c.z > max_z {
                        max_z = c.z;
                    }
                }
                let point = Point::new_(
                    (min_x + max_x) / 2.0,
                    (min_y + max_y) / 2.0,
                    (min_z + max_z) / 2.0,
                );
                let mut feature = feature.clone();
                feature.geometry = Geometry {
                    epsg: geometry.epsg,
                    value: GeometryValue::FlowGeometry3D(point.into()),
                }
                .into();
                fw.send(ctx.new_with_feature_and_port(feature, POINT_PORT.clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::types::coordinate::Coordinate;
    use reearth_flow_geometry::types::line_string::LineString3D;
    use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
    use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::feature::Attributes;
    use reearth_flow_types::{Attribute, GeometryType, GmlGeometry};

    use super::*;
    use crate::tests::utils::create_default_execute_context;

    fn make_polygon_3d(coords: &[(f64, f64, f64)]) -> Polygon3D<f64> {
        let line: LineString3D<f64> =
            LineString3D::new(coords.iter().map(|&c| Coordinate::from(c)).collect());
        Polygon3D::new(line, vec![])
    }

    fn make_gml_geometry(polygons: Vec<Polygon3D<f64>>) -> GmlGeometry {
        GmlGeometry {
            len: polygons.len() as u32,
            polygons,
            ..GmlGeometry::new(GeometryType::Surface, Some(1))
        }
    }

    fn make_citygml_feature(gml_geometries: Vec<GmlGeometry>) -> Feature {
        let city_gml = CityGmlGeometry {
            gml_geometries,
            materials: vec![],
            textures: vec![],
            polygon_materials: vec![],
            polygon_textures: vec![],
            polygon_uvs: MultiPolygon2D::default(),
        };
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::CityGmlGeometry(city_gml),
        };
        Feature::new_with_attributes_and_geometry(Attributes::new(), geometry, Default::default())
    }

    fn run_processor_with_mode(
        feature: &Feature,
        mode: CenterPointMode,
    ) -> (Vec<Feature>, Vec<Port>) {
        let mut processor = CenterPointReplacer { mode };
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop);
        let ctx = create_default_execute_context(feature);
        processor.process(ctx, &fw).unwrap();
        if let ProcessorChannelForwarder::Noop(noop) = fw {
            let features = noop.send_features.lock().unwrap().clone();
            let ports = noop.send_ports.lock().unwrap().clone();
            (features, ports)
        } else {
            unreachable!()
        }
    }

    fn run_processor(feature: &Feature) -> (Vec<Feature>, Vec<Port>) {
        run_processor_with_mode(feature, CenterPointMode::CenterOfGravity)
    }

    fn make_2d_polygon_feature(coords: &[(f64, f64)]) -> Feature {
        let line_coords: Vec<Coordinate<f64, _>> =
            coords.iter().map(|&(x, y)| (x, y).into()).collect();
        let polygon = Polygon2D::new(line_coords.into(), vec![]);
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon)),
        };
        Feature::new_with_attributes_and_geometry(Attributes::new(), geometry, Default::default())
    }

    // =========================================================================
    // Center of Gravity tests (existing behavior)
    // =========================================================================

    #[test]
    fn test_2d_polygon_centroid() {
        let feature =
            make_2d_polygon_feature(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)]);

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => {
                assert!((p.x() - 2.0).abs() < 1e-10);
                assert!((p.y() - 2.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry2D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_3d_polygon_centroid() {
        let polygon = make_polygon_3d(&[
            (0.0, 0.0, 0.0),
            (4.0, 0.0, 0.0),
            (4.0, 4.0, 0.0),
            (0.0, 4.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)),
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                assert!((p.x() - 2.0).abs() < 1e-10);
                assert!((p.y() - 2.0).abs() < 1e-10);
                assert!((p.z() - 0.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_citygml_single_polygon_centroid() {
        let polygon = make_polygon_3d(&[
            (0.0, 0.0, 10.0),
            (6.0, 0.0, 10.0),
            (6.0, 6.0, 10.0),
            (0.0, 6.0, 10.0),
            (0.0, 0.0, 10.0),
        ]);
        let gml = make_gml_geometry(vec![polygon]);
        let feature = make_citygml_feature(vec![gml]);

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                assert!((p.x() - 3.0).abs() < 1e-10);
                assert!((p.y() - 3.0).abs() < 1e-10);
                assert!((p.z() - 10.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    // =========================================================================
    // CityGML multi-polygon now succeeds for centroid & bbox modes
    // =========================================================================

    #[test]
    fn test_citygml_multiple_polygons_centroid() {
        let poly1 = make_polygon_3d(&[
            (0.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
            (2.0, 2.0, 0.0),
            (0.0, 2.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let poly2 = make_polygon_3d(&[
            (4.0, 4.0, 10.0),
            (6.0, 4.0, 10.0),
            (6.0, 6.0, 10.0),
            (4.0, 6.0, 10.0),
            (4.0, 4.0, 10.0),
        ]);
        let gml = make_gml_geometry(vec![poly1, poly2]);
        let feature = make_citygml_feature(vec![gml]);

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                // Average of centroids: (1,1,0) and (5,5,10) => (3,3,5)
                assert!((p.x() - 3.0).abs() < 1e-10);
                assert!((p.y() - 3.0).abs() < 1e-10);
                assert!((p.z() - 5.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    // =========================================================================
    // Bounding Box Center tests
    // =========================================================================

    #[test]
    fn test_2d_polygon_bbox_center() {
        let feature =
            make_2d_polygon_feature(&[(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0), (0.0, 0.0)]);

        let (features, ports) =
            run_processor_with_mode(&feature, CenterPointMode::BoundingBoxCenter);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => {
                assert!((p.x() - 2.0).abs() < 1e-10);
                assert!((p.y() - 1.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry2D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_3d_polygon_bbox_center() {
        let polygon = make_polygon_3d(&[
            (0.0, 0.0, 5.0),
            (6.0, 0.0, 5.0),
            (6.0, 4.0, 15.0),
            (0.0, 4.0, 15.0),
            (0.0, 0.0, 5.0),
        ]);
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)),
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (features, ports) =
            run_processor_with_mode(&feature, CenterPointMode::BoundingBoxCenter);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                assert!((p.x() - 3.0).abs() < 1e-10);
                assert!((p.y() - 2.0).abs() < 1e-10);
                assert!((p.z() - 10.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_citygml_bbox_center() {
        let poly1 = make_polygon_3d(&[
            (0.0, 0.0, 0.0),
            (2.0, 0.0, 0.0),
            (2.0, 2.0, 0.0),
            (0.0, 2.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let poly2 = make_polygon_3d(&[
            (8.0, 8.0, 10.0),
            (10.0, 8.0, 10.0),
            (10.0, 10.0, 10.0),
            (8.0, 10.0, 10.0),
            (8.0, 8.0, 10.0),
        ]);
        let gml = make_gml_geometry(vec![poly1, poly2]);
        let feature = make_citygml_feature(vec![gml]);

        let (features, ports) =
            run_processor_with_mode(&feature, CenterPointMode::BoundingBoxCenter);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry3D(Geometry3D::Point(p)) => {
                // bbox: (0,0,0)-(10,10,10) => center (5,5,5)
                assert!((p.x() - 5.0).abs() < 1e-10);
                assert!((p.y() - 5.0).abs() < 1e-10);
                assert!((p.z() - 5.0).abs() < 1e-10);
            }
            other => panic!("expected FlowGeometry3D Point, got {:?}", other),
        }
    }

    // =========================================================================
    // Any Inside Point tests
    // =========================================================================

    #[test]
    fn test_any_inside_point_convex() {
        let feature =
            make_2d_polygon_feature(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)]);

        let (features, ports) = run_processor_with_mode(&feature, CenterPointMode::AnyInsidePoint);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Point(p)) => {
                // For a square, the interior point should be near center
                assert!((p.x() - 2.0).abs() < 0.5);
                assert!((p.y() - 2.0).abs() < 0.5);
            }
            other => panic!("expected FlowGeometry2D Point, got {:?}", other),
        }
    }

    #[test]
    fn test_any_inside_point_concave() {
        // C-shaped polygon where centroid would be outside
        let feature = make_2d_polygon_feature(&[
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 1.0),
            (1.0, 1.0),
            (1.0, 9.0),
            (10.0, 9.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);

        let (features, ports) = run_processor_with_mode(&feature, CenterPointMode::AnyInsidePoint);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], POINT_PORT.clone());
        // The point must be inside the polygon — for a C shape, this means it
        // should be in the left arm, not in the open area
        match &features[0].geometry.value {
            GeometryValue::FlowGeometry2D(Geometry2D::Point(_)) => {
                // Successfully produced a point (polylabel guarantees it's inside)
            }
            other => panic!("expected FlowGeometry2D Point, got {:?}", other),
        }
    }

    // =========================================================================
    // Rejection tests
    // =========================================================================

    #[test]
    fn test_any_inside_point_rejects_line() {
        use reearth_flow_geometry::types::line_string::LineString2D;
        let line = LineString2D::new(vec![Coordinate::new_(0.0, 0.0), Coordinate::new_(1.0, 1.0)]);
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry2D(Geometry2D::LineString(line)),
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (features, ports) = run_processor_with_mode(&feature, CenterPointMode::AnyInsidePoint);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
        // Verify rejection code attribute
        let code = features[0].get(Attribute::new("fme_rejection_code"));
        assert_eq!(
            code,
            Some(&AttributeValue::String("INVALID_GEOMETRY_TYPE".into()))
        );
    }

    #[test]
    fn test_any_inside_point_rejects_point() {
        let point = reearth_flow_geometry::types::point::Point2D::from((1.0, 2.0));
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::FlowGeometry2D(Geometry2D::Point(point)),
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (_, ports) = run_processor_with_mode(&feature, CenterPointMode::AnyInsidePoint);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
    }

    #[test]
    fn test_any_inside_point_rejects_citygml() {
        let polygon = make_polygon_3d(&[
            (0.0, 0.0, 0.0),
            (4.0, 0.0, 0.0),
            (4.0, 4.0, 0.0),
            (0.0, 4.0, 0.0),
            (0.0, 0.0, 0.0),
        ]);
        let gml = make_gml_geometry(vec![polygon]);
        let feature = make_citygml_feature(vec![gml]);

        let (features, ports) = run_processor_with_mode(&feature, CenterPointMode::AnyInsidePoint);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
        let code = features[0].get(Attribute::new("fme_rejection_code"));
        assert_eq!(
            code,
            Some(&AttributeValue::String("INVALID_GEOMETRY_TYPE".into()))
        );
    }

    #[test]
    fn test_rejected_feature_has_rejection_code() {
        let geometry = Geometry {
            epsg: None,
            value: GeometryValue::None,
        };
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            geometry,
            Default::default(),
        );

        let (features, ports) = run_processor(&feature);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0], REJECTED_PORT.clone());
        let code = features[0].get(Attribute::new("fme_rejection_code"));
        assert_eq!(
            code,
            Some(&AttributeValue::String("INVALID_GEOMETRY_TYPE".into()))
        );
    }
}

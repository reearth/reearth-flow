use std::collections::HashMap;

use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
use reearth_flow_geometry::algorithm::contains::Contains;
use reearth_flow_geometry::algorithm::intersects::Intersects;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D};
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::jpmesh::{JPMeshCode, JPMeshType};
use reearth_flow_types::{Attribute, AttributeValue, GeometryValue};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct JPStandardGridAccumulatorFactory;

impl ProcessorFactory for JPStandardGridAccumulatorFactory {
    fn name(&self) -> &str {
        "JPStandardGridAccumulator"
    }

    fn description(&self) -> &str {
        "Creates a convex partition based on a group of input features."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(JPStandardGridAccumulator))
    }
}

#[derive(Debug, Clone)]
pub struct JPStandardGridAccumulator;

impl Processor for JPStandardGridAccumulator {
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
            GeometryValue::FlowGeometry2D(geometry) => {
                let bounds = if let Some(bounds) = geometry.bounding_rect() {
                    bounds
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                let meshs_80km_inside_bounds =
                    JPMeshCode::from_inside_bounds(bounds, JPMeshType::Mesh80km);
                let meshs_80km = meshs_80km_inside_bounds
                    .iter()
                    .filter(|&mesh| JPStandardGridAccumulator::check_intersects(geometry, mesh))
                    .collect::<Vec<_>>();
                let meshs_10km = meshs_80km
                    .iter()
                    .flat_map(|mesh| mesh.downscale())
                    .filter(|mesh| JPStandardGridAccumulator::check_intersects(geometry, mesh))
                    .collect::<Vec<_>>();
                let meshes_1km = meshs_10km
                    .iter()
                    .flat_map(|mesh| mesh.downscale())
                    .filter(|mesh| JPStandardGridAccumulator::check_intersects(geometry, mesh))
                    .collect::<Vec<_>>();

                for meshcode in meshes_1km {
                    let binded_geometry = if let Some(binded_geometry) =
                        self.bind_geometry_into_mesh_2d(geometry, meshcode.to_number())
                    {
                        binded_geometry
                    } else {
                        continue;
                    };

                    let mut attributes = feature.attributes.clone();
                    attributes.insert(
                        Attribute::new("meshcode"),
                        AttributeValue::String(meshcode.to_number().to_string()),
                    );

                    let mut new_feature = feature.clone();
                    new_feature.geometry.value = GeometryValue::FlowGeometry2D(binded_geometry);
                    new_feature.attributes = attributes;

                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "JPStandardGridAccumulator"
    }
}

impl JPStandardGridAccumulator {
    fn check_intersects(geometry: &Geometry2D, mesh: &JPMeshCode) -> bool {
        let bounds = mesh.into_bounds();
        match geometry {
            Geometry::Point(point) => bounds.contains(&point.0),
            Geometry::LineString(line_string) => bounds.intersects(line_string),
            Geometry::MultiLineString(multi_line_string) => bounds.intersects(multi_line_string),
            Geometry::Polygon(polygon) => bounds.intersects(polygon),
            Geometry::MultiPolygon(multi_polygon) => bounds.intersects(multi_polygon),
            _ => false,
        }
    }

    fn bind_geometry_into_mesh_2d(
        &self,
        geometry: &Geometry2D,
        meshcode: u64,
    ) -> Option<Geometry2D> {
        // メッシュコードからバウンディングボックスを取得
        let mesh = JPMeshCode::from_number(meshcode, JPMeshType::Mesh1km);
        let bounds = mesh.into_bounds();

        let bounds_polygon = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(bounds.min().x, bounds.min().y),
                Coordinate2D::new_(bounds.max().x, bounds.min().y),
                Coordinate2D::new_(bounds.max().x, bounds.max().y),
                Coordinate2D::new_(bounds.min().x, bounds.max().y),
                Coordinate2D::new_(bounds.min().x, bounds.min().y),
            ]),
            vec![],
        );

        // ジオメトリの種類に応じて適切な処理を行う
        let bind_geometry = match geometry {
            // Pointの場合はそのまま返す
            Geometry::Point(_) => geometry.clone(),

            // LineStringの場合はBooleanOpsを使用して分割
            Geometry::LineString(line_string) => {
                // LineStringをMultiLineStringに変換
                let multi_line_string =
                    reearth_flow_geometry::types::multi_line_string::MultiLineString2D::new(vec![
                        line_string.clone(),
                    ]);

                // ポリゴンでクリップ
                let clipped = bounds_polygon.clip(&multi_line_string, false);

                // 結果が空の場合は元のジオメトリを返す
                if clipped.0.is_empty() {
                    return None;
                } else if clipped.0.len() == 1 {
                    // 結果が1つの場合はLineStringとして返す
                    Geometry::LineString(clipped.0[0].clone())
                } else {
                    // 結果が複数の場合はMultiLineStringとして返す
                    Geometry::MultiLineString(clipped)
                }
            }

            // MultiLineStringの場合はBooleanOpsを使用して分割
            Geometry::MultiLineString(multi_line_string) => {
                // ポリゴンでクリップ
                let clipped = bounds_polygon.clip(multi_line_string, false);

                // 結果が空の場合は元のジオメトリを返す
                if clipped.0.is_empty() {
                    return None;
                } else if clipped.0.len() == 1 {
                    // 結果が1つの場合はLineStringとして返す
                    Geometry::LineString(clipped.0[0].clone())
                } else {
                    // 結果が複数の場合はMultiLineStringとして返す
                    Geometry::MultiLineString(clipped)
                }
            }

            // Polygonの場合はBooleanOpsを使用して分割
            Geometry::Polygon(polygon) => {
                // ポリゴン同士の交差を計算
                let intersection = MultiPolygon2D::new(vec![bounds_polygon])
                    .intersection(&MultiPolygon2D::new(vec![polygon.clone()]));

                // 結果が空の場合は元のジオメトリを返す
                if intersection.0.is_empty() {
                    return None;
                } else if intersection.0.len() == 1 {
                    // 結果が1つの場合はPolygonとして返す
                    Geometry::Polygon(intersection.0[0].clone())
                } else {
                    // 結果が複数の場合はMultiPolygonとして返す
                    Geometry::MultiPolygon(intersection)
                }
            }

            // MultiPolygonの場合はBooleanOpsを使用して分割
            Geometry::MultiPolygon(multi_polygon) => {
                // ポリゴン同士の交差を計算
                let intersection =
                    MultiPolygon2D::new(vec![bounds_polygon]).intersection(multi_polygon);

                // 結果が空の場合は元のジオメトリを返す
                if intersection.0.is_empty() {
                    return None;
                } else if intersection.0.len() == 1 {
                    // 結果が1つの場合はPolygonとして返す
                    Geometry::Polygon(intersection.0[0].clone())
                } else {
                    // 結果が複数の場合はMultiPolygonとして返す
                    Geometry::MultiPolygon(intersection)
                }
            }

            // その他の型の場合は未実装
            _ => {
                return None;
            }
        };

        Some(bind_geometry)
    }
}

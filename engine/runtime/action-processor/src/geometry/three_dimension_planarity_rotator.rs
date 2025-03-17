use std::collections::HashMap;

use reearth_flow_geometry::types::coordinate::{Coordinate, Coordinate2D};
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::multi_line_string::{MultiLineString2D, MultiLineString3D};
use reearth_flow_geometry::types::multi_point::{MultiPoint2D, MultiPoint3D};
use reearth_flow_geometry::types::multi_polygon::{MultiPolygon2D, MultiPolygon3D};
use reearth_flow_geometry::types::no_value::NoValue;
use reearth_flow_geometry::types::point::{Point2D, Point3D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Feature, GeometryValue};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ThreeDimensionPlanarityRotatorFactory;

impl ProcessorFactory for ThreeDimensionPlanarityRotatorFactory {
    fn name(&self) -> &str {
        "ThreeDimensionPlanarityRotator"
    }

    fn description(&self) -> &str {
        "Divides the input geometry into Japanese standard (1km) mesh grid."
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
        Ok(Box::new(ThreeDimensionPlanarityRotator))
    }
}

#[derive(Debug, Clone)]
pub struct ThreeDimensionPlanarityRotator;

impl Processor for ThreeDimensionPlanarityRotator {
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
            GeometryValue::FlowGeometry3D(geometry) => {
                if let Some(rotated_feature) = self.rotate_to_horizontal(feature, geometry) {
                    println!("Rotated geometry to horizontal.");
                    fw.send(ctx.new_with_feature_and_port(rotated_feature, DEFAULT_PORT.clone()));
                } else {
                    println!("Failed to rotate geometry to horizontal.");
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
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
        "ThreeDimensionPlanarityRotator"
    }
}

impl ThreeDimensionPlanarityRotator {
    fn rotate_to_horizontal(
        &self,
        feature: &Feature,
        geometry: &Geometry3D<f64>,
    ) -> Option<Feature> {
        match geometry {
            Geometry3D::Point(point) => self.process_point(feature, point),
            Geometry3D::MultiPoint(multi_point) => self.process_multi_point(feature, multi_point),
            Geometry3D::LineString(line_string) => self.process_line_string(feature, line_string),
            Geometry3D::MultiLineString(multi_line_string) => {
                self.process_multi_line_string(feature, multi_line_string)
            }
            Geometry3D::Polygon(polygon) => self.process_polygon(feature, polygon),
            Geometry3D::MultiPolygon(multi_polygon) => {
                self.process_multi_polygon(feature, multi_polygon)
            }
            _ => None, // その他のジオメトリタイプはサポート外
        }
    }

    fn process_point(&self, feature: &Feature, point: &Point3D<f64>) -> Option<Feature> {
        // Pointの場合はZ座標を除去して2Dに変換
        let point_2d = Point2D::new(point.x(), point.y(), NoValue);
        let mut new_feature = feature.clone();
        new_feature.geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Point(point_2d));
        Some(new_feature)
    }

    fn process_multi_point(
        &self,
        feature: &Feature,
        multi_point: &MultiPoint3D<f64>,
    ) -> Option<Feature> {
        // MultiPointの場合は各ポイントのZ座標を除去して2Dに変換
        let points_2d = multi_point
            .0
            .iter()
            .map(|point| Point2D::new(point.x(), point.y(), NoValue))
            .collect();

        let multi_point_2d = MultiPoint2D::new(points_2d);
        let mut new_feature = feature.clone();
        new_feature.geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::MultiPoint(multi_point_2d));
        Some(new_feature)
    }

    fn process_line_string(
        &self,
        feature: &Feature,
        line_string: &LineString3D<f64>,
    ) -> Option<Feature> {
        if line_string.0.is_empty() {
            return None;
        }

        // 重心を計算
        let centroid = self.calculate_centroid_3d(&line_string.0);

        // 平面の法線ベクトルを計算
        let normal = self.calculate_normal_vector(&line_string.0)?;

        // 回転行列を計算
        let rotation_matrix = self.calculate_rotation_matrix(&normal)?;

        // 座標を回転
        let rotated_coords = line_string
            .0
            .iter()
            .map(|coord| {
                // 重心を原点に移動
                let translated = Coordinate::new__(
                    coord.x - centroid.x,
                    coord.y - centroid.y,
                    coord.z - centroid.z,
                );

                // 回転
                let rotated = self.apply_rotation(&translated, &rotation_matrix);

                // 重心を戻す
                Coordinate2D::new_(rotated.x + centroid.x, rotated.y + centroid.y)
            })
            .collect();

        let line_string_2d = LineString2D::new(rotated_coords);
        let mut new_feature = feature.clone();
        new_feature.geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::LineString(line_string_2d));
        Some(new_feature)
    }

    fn process_multi_line_string(
        &self,
        feature: &Feature,
        multi_line_string: &MultiLineString3D<f64>,
    ) -> Option<Feature> {
        if multi_line_string.0.is_empty() {
            return None;
        }

        // 各LineStringを処理
        let line_strings_2d = multi_line_string
            .0
            .iter()
            .filter_map(|line_string| {
                if line_string.0.is_empty() {
                    return None;
                }

                // 重心を計算
                let centroid = self.calculate_centroid_3d(&line_string.0);

                // 平面の法線ベクトルを計算
                let normal = self.calculate_normal_vector(&line_string.0)?;

                // 回転行列を計算
                let rotation_matrix = self.calculate_rotation_matrix(&normal)?;

                // 座標を回転
                let rotated_coords = line_string
                    .0
                    .iter()
                    .map(|coord| {
                        // 重心を原点に移動
                        let translated = Coordinate::new__(
                            coord.x - centroid.x,
                            coord.y - centroid.y,
                            coord.z - centroid.z,
                        );

                        // 回転
                        let rotated = self.apply_rotation(&translated, &rotation_matrix);

                        // 重心を戻す
                        Coordinate2D::new_(rotated.x + centroid.x, rotated.y + centroid.y)
                    })
                    .collect();

                Some(LineString2D::new(rotated_coords))
            })
            .collect::<Vec<_>>();

        if line_strings_2d.is_empty() {
            return None;
        }

        let multi_line_string_2d = MultiLineString2D::new(line_strings_2d);
        let mut new_feature = feature.clone();
        new_feature.geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::MultiLineString(multi_line_string_2d));
        Some(new_feature)
    }

    fn process_polygon(&self, feature: &Feature, polygon: &Polygon3D<f64>) -> Option<Feature> {
        let exterior_coords = polygon.exterior().coords().cloned().collect::<Vec<_>>();
        if exterior_coords.is_empty() {
            return None;
        }

        // 重心を計算
        let centroid = self.calculate_centroid_3d(&exterior_coords);

        // 平面の法線ベクトルを計算
        let normal = self.calculate_normal_vector(&exterior_coords)?;

        // 回転行列を計算
        let rotation_matrix = self.calculate_rotation_matrix(&normal)?;

        // 外部リングを回転
        let exterior_coords_2d = exterior_coords
            .iter()
            .map(|coord| {
                // 重心を原点に移動
                let translated = Coordinate::new__(
                    coord.x - centroid.x,
                    coord.y - centroid.y,
                    coord.z - centroid.z,
                );

                // 回転
                let rotated = self.apply_rotation(&translated, &rotation_matrix);

                // 重心を戻す
                Coordinate2D::new_(rotated.x + centroid.x, rotated.y + centroid.y)
            })
            .collect();

        // 内部リングを回転
        let interior_rings_2d = polygon
            .interiors()
            .iter()
            .map(|ring| {
                let interior_coords = ring.coords().collect::<Vec<_>>();
                let interior_coords_2d = interior_coords
                    .iter()
                    .map(|coord| {
                        // 重心を原点に移動
                        let translated = Coordinate::new__(
                            coord.x - centroid.x,
                            coord.y - centroid.y,
                            coord.z - centroid.z,
                        );

                        // 回転
                        let rotated = self.apply_rotation(&translated, &rotation_matrix);

                        // 重心を戻す
                        Coordinate2D::new_(rotated.x + centroid.x, rotated.y + centroid.y)
                    })
                    .collect();

                LineString2D::new(interior_coords_2d)
            })
            .collect();

        let polygon_2d = Polygon2D::new(LineString2D::new(exterior_coords_2d), interior_rings_2d);
        let mut new_feature = feature.clone();
        new_feature.geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon_2d));
        Some(new_feature)
    }

    fn process_multi_polygon(
        &self,
        feature: &Feature,
        multi_polygon: &MultiPolygon3D<f64>,
    ) -> Option<Feature> {
        if multi_polygon.0.is_empty() {
            return None;
        }

        // 各Polygonを処理
        let polygons_2d = multi_polygon
            .0
            .iter()
            .filter_map(|polygon| {
                let exterior_coords = polygon.exterior().coords().cloned().collect::<Vec<_>>();
                if exterior_coords.is_empty() {
                    return None;
                }

                // 重心を計算
                let centroid = self.calculate_centroid_3d(&exterior_coords);

                // 平面の法線ベクトルを計算
                let normal = self.calculate_normal_vector(&exterior_coords)?;

                // 回転行列を計算
                let rotation_matrix = self.calculate_rotation_matrix(&normal)?;

                // 外部リングを回転
                let exterior_coords_2d = exterior_coords
                    .iter()
                    .map(|coord| {
                        // 重心を原点に移動
                        let translated = Coordinate::new__(
                            coord.x - centroid.x,
                            coord.y - centroid.y,
                            coord.z - centroid.z,
                        );

                        // 回転
                        let rotated = self.apply_rotation(&translated, &rotation_matrix);

                        // 重心を戻す
                        Coordinate2D::new_(rotated.x + centroid.x, rotated.y + centroid.y)
                    })
                    .collect();

                // 内部リングを回転
                let interior_rings_2d = polygon
                    .interiors()
                    .iter()
                    .map(|ring| {
                        let interior_coords = ring.coords().collect::<Vec<_>>();
                        let interior_coords_2d = interior_coords
                            .iter()
                            .map(|coord| {
                                // 重心を原点に移動
                                let translated = Coordinate::new__(
                                    coord.x - centroid.x,
                                    coord.y - centroid.y,
                                    coord.z - centroid.z,
                                );

                                // 回転
                                let rotated = self.apply_rotation(&translated, &rotation_matrix);

                                // 重心を戻す
                                Coordinate2D::new_(rotated.x + centroid.x, rotated.y + centroid.y)
                            })
                            .collect();

                        LineString2D::new(interior_coords_2d)
                    })
                    .collect();

                Some(Polygon2D::new(
                    LineString2D::new(exterior_coords_2d),
                    interior_rings_2d,
                ))
            })
            .collect::<Vec<_>>();

        if polygons_2d.is_empty() {
            return None;
        }

        let multi_polygon_2d = MultiPolygon2D::new(polygons_2d);
        let mut new_feature = feature.clone();
        new_feature.geometry.value =
            GeometryValue::FlowGeometry2D(Geometry2D::MultiPolygon(multi_polygon_2d));
        Some(new_feature)
    }

    // 3D座標の重心を計算
    fn calculate_centroid_3d(&self, coords: &[Coordinate<f64, f64>]) -> Coordinate<f64, f64> {
        let count = coords.len() as f64;
        let sum_x = coords.iter().map(|c| c.x).sum::<f64>() / count;
        let sum_y = coords.iter().map(|c| c.y).sum::<f64>() / count;
        let sum_z = coords.iter().map(|c| c.z).sum::<f64>() / count;

        Coordinate::new__(sum_x, sum_y, sum_z)
    }

    // 平面の法線ベクトルを計算
    fn calculate_normal_vector(&self, coords: &[Coordinate<f64, f64>]) -> Option<[f64; 3]> {
        if coords.len() < 3 {
            // 少なくとも3点が必要
            return Some([0.0, 0.0, 1.0]); // デフォルトの法線ベクトル
        }

        // 最小二乗法で平面を推定
        let centroid = self.calculate_centroid_3d(coords);

        // 共分散行列を計算
        let mut cov_xx = 0.0;
        let mut cov_xy = 0.0;
        let mut cov_xz = 0.0;
        let mut cov_yy = 0.0;
        let mut cov_yz = 0.0;
        let mut cov_zz = 0.0;

        for coord in coords {
            let dx = coord.x - centroid.x;
            let dy = coord.y - centroid.y;
            let dz = coord.z - centroid.z;

            cov_xx += dx * dx;
            cov_xy += dx * dy;
            cov_xz += dx * dz;
            cov_yy += dy * dy;
            cov_yz += dy * dz;
            cov_zz += dz * dz;
        }

        // 最小固有値に対応する固有ベクトルが法線ベクトル
        // 簡略化のため、3x3行列の特性方程式を解く代わりに
        // 共分散行列の各成分の大きさを比較して近似的に法線ベクトルを求める

        // x方向が最も変化が小さい場合
        if cov_xx <= cov_yy && cov_xx <= cov_zz {
            let nx = 1.0;
            let ny = -cov_xy / cov_yy;
            let nz = -cov_xz / cov_zz;
            let norm = (nx * nx + ny * ny + nz * nz).sqrt();
            Some([nx / norm, ny / norm, nz / norm])
        }
        // y方向が最も変化が小さい場合
        else if cov_yy <= cov_xx && cov_yy <= cov_zz {
            let nx = -cov_xy / cov_xx;
            let ny = 1.0;
            let nz = -cov_yz / cov_zz;
            let norm = (nx * nx + ny * ny + nz * nz).sqrt();
            Some([nx / norm, ny / norm, nz / norm])
        }
        // z方向が最も変化が小さい場合
        else {
            let nx = -cov_xz / cov_xx;
            let ny = -cov_yz / cov_yy;
            let nz = 1.0;
            let norm = (nx * nx + ny * ny + nz * nz).sqrt();
            Some([nx / norm, ny / norm, nz / norm])
        }
    }

    // 回転行列を計算
    fn calculate_rotation_matrix(&self, normal: &[f64; 3]) -> Option<[[f64; 3]; 3]> {
        let [nx, ny, nz] = *normal;

        // 法線ベクトルがほぼ垂直な場合は回転不要
        if (nz - 1.0).abs() < 1e-6 {
            return Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        }

        // 法線ベクトルがほぼ逆向きの場合は180度回転
        if (nz + 1.0).abs() < 1e-6 {
            return Some([[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]);
        }

        // 回転軸を計算（法線ベクトルと[0,0,1]の外積）
        let axis_x = ny;
        let axis_y = -nx;
        let axis_z = 0.0;

        // 回転軸の長さを正規化
        let axis_length = (axis_x * axis_x + axis_y * axis_y).sqrt();
        if axis_length < 1e-6 {
            // 回転軸が定義できない場合（法線がほぼz軸に平行）
            return Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        }

        let axis_x = axis_x / axis_length;
        let axis_y = axis_y / axis_length;

        // 回転角度を計算（法線ベクトルと[0,0,1]のなす角）
        let cos_theta = nz;
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        // Rodriguesの回転公式で回転行列を計算
        let k_cross = [
            [0.0, -axis_z, axis_y],
            [axis_z, 0.0, -axis_x],
            [-axis_y, axis_x, 0.0],
        ];

        let k_k_t = [
            [axis_x * axis_x, axis_x * axis_y, axis_x * axis_z],
            [axis_y * axis_x, axis_y * axis_y, axis_y * axis_z],
            [axis_z * axis_x, axis_z * axis_y, axis_z * axis_z],
        ];

        let mut rotation_matrix = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let identity = if i == j { 1.0 } else { 0.0 };
                rotation_matrix[i][j] = identity * cos_theta
                    + k_cross[i][j] * sin_theta
                    + k_k_t[i][j] * (1.0 - cos_theta);
            }
        }

        Some(rotation_matrix)
    }

    // 回転を適用
    fn apply_rotation(
        &self,
        coord: &Coordinate<f64, f64>,
        rotation_matrix: &[[f64; 3]; 3],
    ) -> Coordinate<f64, f64> {
        let x = coord.x;
        let y = coord.y;
        let z = coord.z;

        let new_x =
            rotation_matrix[0][0] * x + rotation_matrix[0][1] * y + rotation_matrix[0][2] * z;
        let new_y =
            rotation_matrix[1][0] * x + rotation_matrix[1][1] * y + rotation_matrix[1][2] * z;
        let new_z =
            rotation_matrix[2][0] * x + rotation_matrix[2][1] * y + rotation_matrix[2][2] * z;

        Coordinate::new__(new_x, new_y, new_z)
    }
}

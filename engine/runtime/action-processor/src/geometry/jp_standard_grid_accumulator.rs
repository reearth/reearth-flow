use std::collections::{HashMap, HashSet};

use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D};
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::rect::{Rect, Rect2D};
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
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
                let meshcodes = geometry
                    .get_all_coordinates()
                    .iter()
                    .map(|coord| JPMeshCode::new(*coord, JPMeshType::Mesh1km).to_number())
                    .collect::<HashSet<u64>>();

                for meshcode in meshcodes {
                    let geometry = if let Some(geometry) =
                        self.bind_geometry_into_mesh_2d(geometry, meshcode)
                    {
                        geometry
                    } else {
                        continue;
                    };

                    let mut attributes = feature.attributes.clone();
                    attributes.insert(
                        Attribute::new("meshcode"),
                        AttributeValue::String(meshcode.to_string()),
                    );

                    let mut new_feature = feature.clone();
                    new_feature.geometry.value = GeometryValue::FlowGeometry2D(geometry);
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
    fn bind_geometry_into_mesh_2d(
        &self,
        geometry: &Geometry2D,
        meshcode: u64,
    ) -> Option<Geometry2D> {
        // メッシュコードからバウンディングボックスを取得
        let mesh = JPMeshCode::from_number(meshcode, JPMeshType::Mesh1km);
        let bounds = mesh.into_bounds();

        let bounds_polygon = bounds.to_polygon();

        // ジオメトリの種類に応じて適切な処理を行う
        let geometry = match geometry {
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
                    geometry.clone()
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
                    geometry.clone()
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
                let intersection = polygon.intersection(&bounds_polygon);

                // 結果が空の場合は元のジオメトリを返す
                if intersection.0.is_empty() {
                    geometry.clone()
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
                    multi_polygon.intersection(&MultiPolygon2D::new(vec![bounds_polygon]));

                // 結果が空の場合は元のジオメトリを返す
                if intersection.0.is_empty() {
                    geometry.clone()
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

        Some(geometry)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JPMeshType {
    /// 第1次地域区画
    Mesh80km,
    /// 第2次地域区画
    Mesh10km,
    /// 基準地域メッシュ
    Mesh1km,
    /// 2分の1地域メッシュ
    Mesh500m,
    /// 4分の1地域メッシュ
    Mesh250m,
    /// 8分の1地域メッシュ
    Mesh125m,
}

impl JPMeshType {
    const fn code_length(&self) -> usize {
        match self {
            JPMeshType::Mesh80km => 4,
            JPMeshType::Mesh10km => 6,
            JPMeshType::Mesh1km => 8,
            JPMeshType::Mesh500m => 9,
            JPMeshType::Mesh250m => 10,
            JPMeshType::Mesh125m => 11,
        }
    }

    const fn lat_interval_seconds(&self) -> f64 {
        match self {
            JPMeshType::Mesh80km => 2400.0,
            JPMeshType::Mesh10km => 300.0,
            JPMeshType::Mesh1km => 30.0,
            JPMeshType::Mesh500m => 15.0,
            JPMeshType::Mesh250m => 7.5,
            JPMeshType::Mesh125m => 3.75,
        }
    }

    const fn lng_interval_seconds(&self) -> f64 {
        match self {
            JPMeshType::Mesh80km => 3600.0,
            JPMeshType::Mesh10km => 450.0,
            JPMeshType::Mesh1km => 45.0,
            JPMeshType::Mesh500m => 22.5,
            JPMeshType::Mesh250m => 11.25,
            JPMeshType::Mesh125m => 5.625,
        }
    }

    const fn lat_interval(&self) -> f64 {
        self.lat_interval_seconds() / 3600.0
    }

    const fn lng_interval(&self) -> f64 {
        self.lng_interval_seconds() / 3600.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JPMeshCode {
    mesh_code_type: JPMeshType,
    seed: JPMeshCodeSeed,
}

impl JPMeshCode {
    fn new(coords: Coordinate2D<f64>, mesh_code_type: JPMeshType) -> Self {
        let seed = JPMeshCodeSeed::new(coords);
        JPMeshCode {
            mesh_code_type,
            seed,
        }
    }

    fn to_slice(&self) -> &[u8] {
        match self.mesh_code_type {
            JPMeshType::Mesh80km => &self.seed.code_2[..4],
            JPMeshType::Mesh10km => &self.seed.code_2[..6],
            JPMeshType::Mesh1km => &self.seed.code_2[..8],
            JPMeshType::Mesh500m => &self.seed.code_2[..9],
            JPMeshType::Mesh250m => &self.seed.code_2[..10],
            JPMeshType::Mesh125m => &self.seed.code_2[..11],
        }
    }

    fn from_number(mesh_code: u64, mesh_code_type: JPMeshType) -> Self {
        let mut code_2 = [0u8; 11];
        let mut mesh_code = mesh_code;
        let ifirst = 11 - mesh_code_type.code_length();
        for i in (0..11).rev() {
            let value = (mesh_code % 10) as u8;
            if i >= ifirst {
                code_2[i - ifirst] = value;
            }
            mesh_code /= 10;
        }

        JPMeshCode {
            mesh_code_type,
            seed: JPMeshCodeSeed { code_2 },
        }
    }

    fn to_number(&self) -> u64 {
        let mut result = 0;
        for &digit in self.to_slice() {
            result = result * 10 + digit as u64;
        }
        result
    }

    fn into_bounds(&self) -> Rect2D<f64> {
        self.seed.into_bounds(self.mesh_code_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JPMeshCodeSeed {
    // mesh code for 80km, 10km, 1km, 500m, 250m, 125m
    code_2: [u8; 11],
}

impl JPMeshCodeSeed {
    fn new(coords: Coordinate2D<f64>) -> Self {
        // latitude / interval (Mesh80km) = p % a
        let p = (coords.y / JPMeshType::Mesh80km.lat_interval()).floor() as u8;
        let a = coords.y % JPMeshType::Mesh80km.lat_interval();

        // a / lat_interval (Mesh10km) = q % b
        let q = (a / JPMeshType::Mesh10km.lat_interval()).floor() as u8;
        let b = a % JPMeshType::Mesh10km.lat_interval();

        // b / lat_interval (Mesh1km) = r % c
        let r = (b / JPMeshType::Mesh1km.lat_interval()).floor() as u8;
        let c = b % JPMeshType::Mesh1km.lat_interval();

        // c / lat_interval (Mesh500m) = s % d
        let s = (c / JPMeshType::Mesh500m.lat_interval()).floor() as u8;
        let d = c % JPMeshType::Mesh500m.lat_interval();

        // d / lat_interval (Mesh250m) = t % e
        let t = (d / JPMeshType::Mesh250m.lat_interval()).floor() as u8;

        // d / lat_interval (Mesh125m) = tt
        let tt = (d / JPMeshType::Mesh125m.lat_interval()).floor() as u8;

        // longitude - 100 degrees = u % f
        let u = (coords.x - 100.0).floor() as u8;
        let f = coords.x - 100.0 - u as f64;

        // f / lng_interval (Mesh10km) = v % g
        let v = (f / JPMeshType::Mesh10km.lng_interval()).floor() as u8;
        let g = f % JPMeshType::Mesh10km.lng_interval();

        // g / lng_interval (Mesh1km) = w % h
        let w = (g / JPMeshType::Mesh1km.lng_interval()).floor() as u8;
        let h = g % JPMeshType::Mesh1km.lng_interval();

        // h / lng_interval (Mesh500m) = x % i
        let x = (h / JPMeshType::Mesh500m.lng_interval()).floor() as u8;
        let i = h % JPMeshType::Mesh500m.lng_interval();

        // i / lng_interval (Mesh250m) = y % j
        let y = (i / JPMeshType::Mesh250m.lng_interval()).floor() as u8;

        // i / lng_interval (Mesh125m) = yy
        let yy = (i / JPMeshType::Mesh125m.lng_interval()).floor() as u8;

        // (s * 2)+(x + 1)= m
        let m = (s * 2) + (x + 1);

        // (t * 2)+(y + 1)= n
        let n = (t * 2) + (y + 1);

        // (tt * 2)+(yy + 1)= nn
        let nn = (tt * 2) + (yy + 1);

        // First 6 digits
        let head = {
            let p1 = (p / 10) % 10;
            let p2 = p % 10;
            let u1 = (u / 10) % 10;
            let u2 = u % 10;
            [p1, p2, u1, u2, q, v]
        };

        // Last 5 digits
        let tail_bin = { [r, w, m, n, nn] };

        let mut code_2 = [0u8; 11];
        code_2[..6].copy_from_slice(&head);
        code_2[6..11].copy_from_slice(&tail_bin);

        JPMeshCodeSeed { code_2 }
    }

    fn into_bounds(&self, mesh_code_type: JPMeshType) -> Rect2D<f64> {
        let p = (self.code_2[0] * 10 + self.code_2[1]) as f64;
        let u = (self.code_2[2] * 10 + self.code_2[3]) as f64;
        let q = self.code_2[4] as f64;
        let v = self.code_2[5] as f64;
        let r = self.code_2[6] as f64;
        let w = self.code_2[7] as f64;
        let m = self.code_2[8] as f64;
        let n = self.code_2[9] as f64;
        let nn = self.code_2[10] as f64;

        // Calculate latitude (southwest corner)
        let lat_base = p * JPMeshType::Mesh80km.lat_interval();
        let lat_q = q * JPMeshType::Mesh10km.lat_interval();
        let lat_r = r * JPMeshType::Mesh1km.lat_interval();
        let lat_m = ((m - 1.0) % 2.0) * JPMeshType::Mesh500m.lat_interval();
        let lat_n = ((n - 1.0) % 2.0) * JPMeshType::Mesh250m.lat_interval();
        let lat_nn = ((nn - 1.0) % 2.0) * JPMeshType::Mesh125m.lat_interval();

        // Calculate longitude (southwest corner)
        let lng_base = 100.0 + u;
        let lng_v = v * JPMeshType::Mesh10km.lng_interval();
        let lng_w = w * JPMeshType::Mesh1km.lng_interval();
        let lng_m = ((m - 1.0) / 2.0) * JPMeshType::Mesh500m.lng_interval();
        let lng_n = ((n - 1.0) / 2.0) * JPMeshType::Mesh250m.lng_interval();
        let lng_nn = ((nn - 1.0) / 2.0) * JPMeshType::Mesh125m.lng_interval();

        // Coordinates of southwest corner
        let min_lng = lng_base + lng_v + lng_w + lng_m + lng_n + lng_nn;
        let min_lat = lat_base + lat_q + lat_r + lat_m + lat_n + lat_nn;

        // Coordinates of northeast corner
        let max_lat = min_lat + mesh_code_type.lat_interval();
        let max_lng = min_lng + mesh_code_type.lng_interval();

        Rect::new(
            Coordinate2D::new_(min_lng, min_lat),
            Coordinate2D::new_(max_lng, max_lat),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    #[macro_export]
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert!(
                ($a - $b).abs() < EPSILON,
                "assertion failed: `(left ≈ right)`\n  left: `{}`\n right: `{}`\n",
                $a,
                $b
            );
        };
    }

    #[macro_export]
    macro_rules! assert_mesh_size_correct {
        ($bounds:expr, $lng_interval_seconds:expr, $lat_interval_seconds:expr) => {
            let min_coord = $bounds.min();
            let max_coord = $bounds.max();
            assert_approx_eq!(max_coord.x - min_coord.x, $lng_interval_seconds / 3600.0);
            assert_approx_eq!(max_coord.y - min_coord.y, $lat_interval_seconds / 3600.0);
        };
    }

    #[macro_export]
    macro_rules! assert_rect_includes {
        ($rect:expr, $point:expr) => {
            assert!(
                $rect.min().x <= $point.x
                    && $rect.min().y <= $point.y
                    && $rect.max().x > $point.x
                    && $rect.max().y > $point.y
            );
        };
    }

    #[macro_export]
    macro_rules! assert_rect_not_includes {
        ($rect:expr, $point:expr) => {
            assert!(
                $rect.min().x > $point.x
                    || $rect.min().y > $point.y
                    || $rect.max().x <= $point.x
                    || $rect.max().y <= $point.y
            );
        };
    }

    // small offset for checking coordinate inside the mesh
    const INNER_OFFSET: f64 = 0.000003;

    #[derive(Debug)]
    struct TestCase {
        mesh_code_number: u64,
        mesh_code_type: JPMeshType,
        left_bottom: Coordinate2D<f64>,
    }

    impl TestCase {
        fn inner_coord(&self) -> Coordinate2D<f64> {
            Coordinate2D::new_(
                self.left_bottom.x + INNER_OFFSET,
                self.left_bottom.y + INNER_OFFSET,
            )
        }
    }

    fn get_test_cases() -> Vec<TestCase> {
        return vec![
            TestCase {
                mesh_code_number: 64414277,
                mesh_code_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(141.3375, 43.058333),
            },
            TestCase {
                mesh_code_number: 61401589,
                mesh_code_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(140.7375, 40.816667),
            },
            TestCase {
                mesh_code_number: 59414142,
                mesh_code_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(141.15, 39.7),
            },
            TestCase {
                mesh_code_number: 57403629,
                mesh_code_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(140.8625, 38.266667),
            },
        ];
    }

    #[test]
    fn test_mesh_code_generation() {
        for test_case in get_test_cases() {
            let inner_coord = test_case.inner_coord();
            let mesh_code = JPMeshCode::new(inner_coord, test_case.mesh_code_type);

            let actual_number = mesh_code.to_number();
            assert_eq!(actual_number, test_case.mesh_code_number);
        }
    }

    #[test]
    fn test_mesh_code_into_bounds() {
        for test_case in get_test_cases() {
            let inner_coord = test_case.inner_coord();
            let mesh_code = JPMeshCode::new(inner_coord, test_case.mesh_code_type);

            let bounds = mesh_code.into_bounds();
            let min_coord = bounds.min();

            // check if the bottom left coordinate is correct
            assert_approx_eq!(min_coord.x, test_case.left_bottom.x);
            assert_approx_eq!(min_coord.y, test_case.left_bottom.y);

            // check if the size of the area is correct
            assert_mesh_size_correct!(bounds, 45.0, 30.0);
        }
    }

    #[test]
    fn test_mesh_code_from_number_to_number() {
        for test_case in get_test_cases() {
            let mesh_code =
                JPMeshCode::from_number(test_case.mesh_code_number, test_case.mesh_code_type);
            let number = mesh_code.to_number();
            assert_eq!(number, test_case.mesh_code_number);
        }
    }

    #[test]
    fn test_mesh_code_upscale() {
        // Create larger scale meshes by truncating digits from the dataset's mesh_code,
        // and verify that the dataset's inner coordinates are contained within these mesh boundaries
        for test_case in get_test_cases() {
            // 1km -> 10km
            let mesh_code_10km = test_case.mesh_code_number / 100;
            let mesh_code_10km_obj = JPMeshCode::from_number(mesh_code_10km, JPMeshType::Mesh10km);
            let bounds_10km = mesh_code_10km_obj.into_bounds();

            // verify that inner coordinates are contained within the mesh boundaries
            let inner_coord = test_case.inner_coord();
            assert_rect_includes!(bounds_10km, inner_coord);

            // verify that mesh size is correct
            assert_mesh_size_correct!(bounds_10km, 450.0, 300.0);

            // 1km -> 80km
            let mesh_code_80km = test_case.mesh_code_number / 10000;
            let mesh_code_80km_obj = JPMeshCode::from_number(mesh_code_80km, JPMeshType::Mesh80km);
            let bounds_80km = mesh_code_80km_obj.into_bounds();

            // check if the inner coordinate is included in the mesh
            assert_rect_includes!(bounds_80km, inner_coord);

            // check if the size of the area is correct
            assert_mesh_size_correct!(bounds_80km, 3600.0, 2400.0);
        }
    }

    #[test]
    fn test_mesh_code_downscale() {
        // Create smaller scale meshes by adding digits to the dataset's mesh_code,
        // and verify that the dataset's inner coordinates are contained within these mesh boundaries
        for test_case in get_test_cases() {
            // the mesh code will be (test_case.mesh_code_number * 1000 + 111)
            let inner_coord = test_case.inner_coord();

            // 1km -> 500m
            for i in 1..=4 {
                let mesh_code_500m = test_case.mesh_code_number * 10 + i;
                let mesh_code_500m_obj =
                    JPMeshCode::from_number(mesh_code_500m, JPMeshType::Mesh500m);
                let bounds_500m = mesh_code_500m_obj.into_bounds();

                assert_mesh_size_correct!(bounds_500m, 22.5, 15.0);

                if i == 1 {
                    assert_rect_includes!(bounds_500m, inner_coord);
                } else {
                    assert_rect_not_includes!(bounds_500m, inner_coord);
                }
            }

            // 1km -> 250m
            for j in 1..=4 {
                let mesh_code_250m = test_case.mesh_code_number * 100 + 10 + j;
                let mesh_code_250m_obj =
                    JPMeshCode::from_number(mesh_code_250m, JPMeshType::Mesh250m);
                let bounds_250m = mesh_code_250m_obj.into_bounds();

                assert_mesh_size_correct!(bounds_250m, 11.25, 7.5);

                if j == 1 {
                    assert_rect_includes!(bounds_250m, inner_coord);
                } else {
                    assert_rect_not_includes!(bounds_250m, inner_coord);
                }
            }

            // 1km -> 125m
            for k in 1..=4 {
                let mesh_code_125m = test_case.mesh_code_number * 1000 + 110 + k;
                let mesh_code_125m_obj =
                    JPMeshCode::from_number(mesh_code_125m, JPMeshType::Mesh125m);
                let bounds_125m = mesh_code_125m_obj.into_bounds();

                assert_mesh_size_correct!(bounds_125m, 5.625, 3.75);

                if k == 1 {
                    assert_rect_includes!(bounds_125m, inner_coord);
                } else {
                    assert_rect_not_includes!(bounds_125m, inner_coord);
                }
            }
        }
    }
}
